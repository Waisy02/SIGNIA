//! Inference helpers for SIGNIA pipeline.
//!
//! "Inference" in SIGNIA means deterministic, explainable derivations from a
//! partially-specified structure into a more complete IR/schema representation.
//!
//! Examples:
//! - infer entity types from file extensions or OpenAPI objects
//! - infer edges (contains, imports, foreign keys) from node attributes
//! - infer schema kind from meta/source
//! - infer stable ids from keys with a deterministic strategy
//!
//! This module is intentionally conservative and does not attempt "AI" inference.
//! Any probabilistic/LLM-based inference belongs in higher layers (plugins/services)
//! and must output a deterministic IR that can be verified.
//!
//! Core inference is rule-based and deterministic.

use std::collections::{BTreeMap, BTreeSet};

use crate::errors::{SigniaError, SigniaResult};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

#[cfg(feature = "canonical-json")]
use crate::model::ir::{IrEdge, IrGraph, IrNode};

/// A single inference action recorded for explainability.
#[derive(Debug, Clone)]
pub struct InferenceNote {
    pub code: String,
    pub message: String,
    pub data: BTreeMap<String, String>,
}

/// Result of running inference over a graph.
#[derive(Debug, Clone)]
pub struct InferenceReport {
    pub notes: Vec<InferenceNote>,
    pub added_nodes: usize,
    pub added_edges: usize,
}

impl Default for InferenceReport {
    fn default() -> Self {
        Self {
            notes: Vec::new(),
            added_nodes: 0,
            added_edges: 0,
        }
    }
}

/// Inference options.
///
/// Keep this struct additive only; do not break callers.
#[derive(Debug, Clone)]
pub struct InferenceOptions {
    /// If true, infer `contains` edges based on `path`/`parent` attributes when present.
    pub infer_contains: bool,

    /// If true, infer "language" for file entities by extension into `attrs.language`.
    pub infer_languages: bool,

    /// Maximum inferred edges to add (safety limit).
    pub max_inferred_edges: usize,
}

impl Default for InferenceOptions {
    fn default() -> Self {
        Self {
            infer_contains: true,
            infer_languages: true,
            max_inferred_edges: 50_000,
        }
    }
}

/// Run deterministic inference over an IR graph.
///
/// This function mutates the graph in-place and returns a report describing
/// what was inferred. The report is suitable for CLI/API display.
///
/// Requirements:
/// - Graph must be internally consistent (basic validation passes).
/// - Inference must not add duplicate keys.
#[cfg(feature = "canonical-json")]
pub fn infer_ir(g: &mut IrGraph, opts: &InferenceOptions) -> SigniaResult<InferenceReport> {
    g.validate_basic()?;

    let mut report = InferenceReport::default();

    if opts.infer_languages {
        let n = infer_file_languages(g, &mut report)?;
        if n > 0 {
            report.notes.push(InferenceNote {
                code: "infer.languages".to_string(),
                message: format!("inferred language for {n} file nodes"),
                data: BTreeMap::new(),
            });
        }
    }

    if opts.infer_contains {
        let n = infer_contains_edges(g, opts.max_inferred_edges, &mut report)?;
        if n > 0 {
            report.notes.push(InferenceNote {
                code: "infer.contains".to_string(),
                message: format!("inferred {n} contains edges"),
                data: BTreeMap::new(),
            });
        }
    }

    g.validate_basic()?;
    Ok(report)
}

#[cfg(feature = "canonical-json")]
fn infer_file_languages(g: &mut IrGraph, report: &mut InferenceReport) -> SigniaResult<usize> {
    let mut count = 0usize;

    for node in g.nodes.values_mut() {
        if node.node_type != "file" {
            continue;
        }

        // Look for a `name` or `path` based extension.
        let candidate = node
            .attrs
            .get("path")
            .and_then(|v| v.as_str())
            .or_else(|| Some(node.name.as_str()));

        let Some(s) = candidate else { continue };

        let ext = s.rsplit('.').next().unwrap_or("");
        if ext == s {
            continue; // no dot
        }

        let lang = match ext.to_ascii_lowercase().as_str() {
            "rs" => "rust",
            "ts" => "typescript",
            "tsx" => "typescript",
            "js" => "javascript",
            "jsx" => "javascript",
            "py" => "python",
            "go" => "go",
            "java" => "java",
            "kt" => "kotlin",
            "c" => "c",
            "h" => "c",
            "cpp" | "cc" | "cxx" | "hpp" => "cpp",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "md" => "markdown",
            "sol" => "solidity",
            _ => continue,
        };

        // Only set if not already present.
        let set = match node.attrs.get("language") {
            Some(v) if v.is_string() => false,
            _ => true,
        };

        if set {
            node.attrs
                .insert("language".to_string(), Value::String(lang.to_string()));
            count += 1;
        }
    }

    report.added_nodes += 0;
    Ok(count)
}

#[cfg(feature = "canonical-json")]
fn infer_contains_edges(
    g: &mut IrGraph,
    max_edges: usize,
    report: &mut InferenceReport,
) -> SigniaResult<usize> {
    // Strategy:
    // - If nodes have attrs.parentKey, connect parentKey -> node.key with contains edge
    // - If nodes have attrs.parentId, connect parentId -> node.id
    //
    // We do not attempt path parsing here to avoid ambiguity. Path-based inference can
    // be implemented as a plugin with stronger rules.
    let mut inferred = 0usize;

    // Build lookup maps
    let key_to_id: BTreeMap<String, String> =
        g.nodes.values().map(|n| (n.key.clone(), n.id.clone())).collect();

    // Avoid duplicates by tracking existing (from,to,type) triplets
    let mut existing: BTreeSet<(String, String, String)> = BTreeSet::new();
    for e in g.edges.values() {
        existing.insert((e.from.clone(), e.to.clone(), e.edge_type.clone()));
    }

    // Add inferred edges
    let mut to_add: Vec<IrEdge> = Vec::new();

    for n in g.nodes.values() {
        if inferred >= max_edges {
            return Err(SigniaError::invalid_argument(format!(
                "inferred edges exceeded limit ({max_edges})"
            )));
        }

        // parentKey path
        if let Some(pk) = n.attrs.get("parentKey").and_then(|v| v.as_str()) {
            if let Some(pid) = key_to_id.get(pk) {
                let trip = (pid.clone(), n.id.clone(), "contains".to_string());
                if !existing.contains(&trip) {
                    let edge_key = format!("contains:{}:{}", pk, n.key);
                    to_add.push(IrEdge {
                        id: format!("ie:{}", edge_key),
                        key: edge_key,
                        edge_type: "contains".to_string(),
                        from: pid.clone(),
                        to: n.id.clone(),
                        attrs: BTreeMap::new(),
                        provenance: Some("inference:parentKey".to_string()),
                        diagnostics: vec![],
                    });
                    existing.insert(trip);
                    inferred += 1;
                }
            }
        }

        // parentId path
        if let Some(pid) = n.attrs.get("parentId").and_then(|v| v.as_str()) {
            // Only if the referenced id exists
            if g.nodes.contains_key(pid) {
                let trip = (pid.to_string(), n.id.clone(), "contains".to_string());
                if !existing.contains(&trip) {
                    let edge_key = format!("contains:{}:{}", pid, n.id);
                    to_add.push(IrEdge {
                        id: format!("ie:{}", edge_key),
                        key: edge_key,
                        edge_type: "contains".to_string(),
                        from: pid.to_string(),
                        to: n.id.clone(),
                        attrs: BTreeMap::new(),
                        provenance: Some("inference:parentId".to_string()),
                        diagnostics: vec![],
                    });
                    existing.insert(trip);
                    inferred += 1;
                }
            }
        }
    }

    for e in to_add {
        // Insert with duplicate key handling
        if g.edges.contains_key(&e.id) {
            // Ensure unique id; deterministic suffix by counting collisions.
            let mut i = 1u32;
            loop {
                let alt = format!("{}:{}", e.id, i);
                if !g.edges.contains_key(&alt) {
                    let mut e2 = e.clone();
                    e2.id = alt.clone();
                    g.insert_edge(e2)?;
                    break;
                }
                i += 1;
                if i > 10_000 {
                    return Err(SigniaError::invariant("failed to allocate unique inferred edge id"));
                }
            }
        } else {
            g.insert_edge(e)?;
        }
    }

    report.added_edges += inferred;
    Ok(inferred)
}

/// Infer a schema kind from meta JSON.
///
/// This helper is intended for callers that have a meta object but not a stable kind.
/// It uses deterministic heuristics:
/// - if meta.labels.kind exists, use it
/// - else if source.locator suggests openapi, dataset, or repo
#[cfg(feature = "canonical-json")]
pub fn infer_schema_kind_from_meta(meta: &Value) -> SigniaResult<String> {
    let obj = meta
        .as_object()
        .ok_or_else(|| SigniaError::invalid_argument("meta must be an object"))?;

    // labels.kind
    if let Some(labels) = obj.get("labels").and_then(|v| v.as_object()) {
        if let Some(k) = labels.get("kind").and_then(|v| v.as_str()) {
            return Ok(k.to_string());
        }
    }

    // source locator heuristics
    if let Some(src) = obj.get("source").and_then(|v| v.as_object()) {
        if let Some(locator) = src.get("locator").and_then(|v| v.as_str()) {
            let l = locator.to_ascii_lowercase();
            if l.contains("openapi") || l.ends_with(".yaml") || l.ends_with(".yml") || l.ends_with(".json") {
                // Avoid misclassification: only treat obvious openapi locators as openapi.
                if l.contains("openapi") {
                    return Ok("openapi".to_string());
                }
            }
            if l.contains("dataset") || l.ends_with(".parquet") || l.ends_with(".csv") {
                return Ok("dataset".to_string());
            }
            return Ok("repo".to_string());
        }
    }

    Ok("repo".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "canonical-json")]
    fn infer_kind_labels() {
        let meta = serde_json::json!({
            "name":"x",
            "createdAt":"1970-01-01T00:00:00Z",
            "labels":{"kind":"dataset"},
            "source":{"type":"path","locator":"artifact:/x"},
            "normalization":{"policyVersion":"v1","pathRoot":"artifact:/","newline":"lf","encoding":"utf-8","symlinks":"deny","network":"deny"}
        });
        assert_eq!(infer_schema_kind_from_meta(&meta).unwrap(), "dataset");
    }

    #[test]
    #[cfg(feature = "canonical-json")]
    fn infer_kind_source_repo_default() {
        let meta = serde_json::json!({
            "name":"x",
            "createdAt":"1970-01-01T00:00:00Z",
            "source":{"type":"path","locator":"artifact:/x"},
            "normalization":{"policyVersion":"v1","pathRoot":"artifact:/","newline":"lf","encoding":"utf-8","symlinks":"deny","network":"deny"}
        });
        assert_eq!(infer_schema_kind_from_meta(&meta).unwrap(), "repo");
    }

    #[test]
    #[cfg(feature = "canonical-json")]
    fn infer_ir_languages_and_contains() {
        let mut g = IrGraph::new();

        g.insert_node(IrNode {
            id: "n1".to_string(),
            key: "repo:root".to_string(),
            node_type: "repo".to_string(),
            name: "demo".to_string(),
            attrs: BTreeMap::new(),
            digests: vec![],
            provenance: None,
            diagnostics: vec![],
        })
        .unwrap();

        let mut attrs = BTreeMap::new();
        attrs.insert("path".to_string(), Value::String("artifact:/README.md".to_string()));
        attrs.insert("parentId".to_string(), Value::String("n1".to_string()));

        g.insert_node(IrNode {
            id: "n2".to_string(),
            key: "file:readme".to_string(),
            node_type: "file".to_string(),
            name: "README.md".to_string(),
            attrs,
            digests: vec![],
            provenance: None,
            diagnostics: vec![],
        })
        .unwrap();

        let opts = InferenceOptions::default();
        let rep = infer_ir(&mut g, &opts).unwrap();
        assert!(rep.added_edges >= 1);

        let n2 = g.nodes.get("n2").unwrap();
        assert_eq!(n2.attrs.get("language").and_then(|v| v.as_str()), Some("markdown"));
    }
}
