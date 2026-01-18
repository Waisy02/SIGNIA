//! Schema detection utilities for SIGNIA built-in config.
//!
//! This module attempts to determine which "input schema" a JSON payload belongs to,
//! based on lightweight structural checks.
//!
//! Use cases:
//! - CLI: accept `--input payload.json` without requiring `--type repo|dataset|workflow`
//! - API: infer input type based on payload content
//! - tooling: quick routing to a suitable plugin
//!
//! Design constraints:
//! - Deterministic: same input -> same result
//! - Conservative: prefer `Unknown` over false positives
//! - No I/O
//!
//! This is NOT a replacement for plugin validation. Plugins perform strict checks.

#![cfg(feature = "builtin")]

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Detected schema kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectedKind {
    Repo,
    Dataset,
    Workflow,
    OpenApi,
    Unknown,
}

/// Detection result with confidence and hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub kind: DetectedKind,
    /// 0..=100, conservative by design.
    pub confidence: u8,
    /// Human-readable hints for UIs/CLIs.
    #[serde(default)]
    pub hints: Vec<String>,
    /// Extra metadata for tooling.
    #[serde(default)]
    pub meta: BTreeMap<String, String>,
}

impl DetectionResult {
    pub fn unknown() -> Self {
        Self {
            kind: DetectedKind::Unknown,
            confidence: 0,
            hints: vec!["No known schema matched".to_string()],
            meta: BTreeMap::new(),
        }
    }
}

/// Detect an input kind from a JSON payload.
///
/// This function assumes the payload has already been parsed from JSON.
/// It does not mutate the input.
///
/// Rules (high-level):
/// - Repo: keys like `repo` fields OR common repo snapshot shapes
/// - Dataset: keys like `files` with `rows`/`columns` or `dataset` descriptors
/// - Workflow: `name` + `nodes` array + optional `edges`
/// - OpenAPI: `openapi` string + `paths` object
pub fn detect_input_kind(v: &Value) -> Result<DetectionResult> {
    if v.is_null() {
        return Ok(DetectionResult::unknown());
    }

    // OpenAPI is very distinctive.
    if looks_like_openapi(v) {
        return Ok(DetectionResult {
            kind: DetectedKind::OpenApi,
            confidence: 95,
            hints: vec!["Found top-level `openapi` and `paths`".to_string()],
            meta: BTreeMap::new(),
        });
    }

    // Workflow
    if looks_like_workflow(v) {
        return Ok(DetectionResult {
            kind: DetectedKind::Workflow,
            confidence: 90,
            hints: vec!["Found workflow shape: name + nodes array".to_string()],
            meta: BTreeMap::new(),
        });
    }

    // Repo
    if looks_like_repo(v) {
        return Ok(DetectionResult {
            kind: DetectedKind::Repo,
            confidence: 80,
            hints: vec!["Found repo snapshot shape: files with paths".to_string()],
            meta: BTreeMap::new(),
        });
    }

    // Dataset
    if looks_like_dataset(v) {
        return Ok(DetectionResult {
            kind: DetectedKind::Dataset,
            confidence: 70,
            hints: vec!["Found dataset shape: files/records/columns".to_string()],
            meta: BTreeMap::new(),
        });
    }

    Ok(DetectionResult::unknown())
}

fn looks_like_openapi(v: &Value) -> bool {
    let obj = match v.as_object() {
        Some(o) => o,
        None => return false,
    };
    let openapi = obj.get("openapi").and_then(|x| x.as_str());
    let paths = obj.get("paths").and_then(|x| x.as_object());
    openapi.is_some() && paths.is_some()
}

fn looks_like_workflow(v: &Value) -> bool {
    let obj = match v.as_object() {
        Some(o) => o,
        None => return false,
    };
    let name = obj.get("name").and_then(|x| x.as_str());
    let nodes = obj.get("nodes").and_then(|x| x.as_array());
    // `edges` optional but if present should be array
    let edges_ok = match obj.get("edges") {
        None => true,
        Some(e) => e.is_array(),
    };
    name.is_some() && nodes.is_some() && edges_ok
}

fn looks_like_repo(v: &Value) -> bool {
    // Accept common shapes:
    // - { "repo": { ... }, "files": [ { "path": "...", "bytes": "..." } ] }
    // - { "files": [ { "path": "...", "sha256": "..."} ], "root": "..." }
    let obj = match v.as_object() {
        Some(o) => o,
        None => return false,
    };

    if let Some(files) = obj.get("files").and_then(|x| x.as_array()) {
        // We require at least one element with a `path` field.
        for f in files {
            if f.get("path").and_then(|x| x.as_str()).is_some() {
                return true;
            }
        }
    }

    if let Some(repo) = obj.get("repo").and_then(|x| x.as_object()) {
        if repo.get("owner").and_then(|x| x.as_str()).is_some()
            && repo.get("name").and_then(|x| x.as_str()).is_some()
        {
            return true;
        }
    }

    false
}

fn looks_like_dataset(v: &Value) -> bool {
    let obj = match v.as_object() {
        Some(o) => o,
        None => return false,
    };

    // Dataset may have:
    // - { "dataset": { "name": "...", ... }, "files": [...] }
    // - { "files": [ { "path": "...", "format": "csv", "columns": [...] } ] }
    // - { "records": [ {...}, {...} ] } (small)
    if obj.get("records").and_then(|x| x.as_array()).is_some() {
        return true;
    }

    if let Some(dataset) = obj.get("dataset").and_then(|x| x.as_object()) {
        if dataset.get("name").and_then(|x| x.as_str()).is_some() {
            return true;
        }
    }

    if let Some(files) = obj.get("files").and_then(|x| x.as_array()) {
        for f in files {
            let has_format = f.get("format").and_then(|x| x.as_str()).is_some();
            let has_cols = f.get("columns").and_then(|x| x.as_array()).is_some();
            if has_format || has_cols {
                return true;
            }
        }
    }

    false
}

/// Validate that a detection result matches an expected kind.
pub fn require_kind(res: &DetectionResult, expected: DetectedKind) -> Result<()> {
    if res.kind != expected {
        return Err(anyhow!(
            "detected kind {:?} does not match expected {:?}",
            res.kind,
            expected
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_openapi() {
        let v = json!({"openapi":"3.0.0","paths":{}});
        let r = detect_input_kind(&v).unwrap();
        assert_eq!(r.kind, DetectedKind::OpenApi);
        assert!(r.confidence >= 90);
    }

    #[test]
    fn detects_workflow() {
        let v = json!({"name":"x","nodes":[{"id":"a","type":"t"}],"edges":[]});
        let r = detect_input_kind(&v).unwrap();
        assert_eq!(r.kind, DetectedKind::Workflow);
    }

    #[test]
    fn detects_repo_by_files() {
        let v = json!({"files":[{"path":"README.md","sha256":"x"}]});
        let r = detect_input_kind(&v).unwrap();
        assert_eq!(r.kind, DetectedKind::Repo);
    }

    #[test]
    fn detects_dataset_by_records() {
        let v = json!({"records":[{"a":1}]});
        let r = detect_input_kind(&v).unwrap();
        assert_eq!(r.kind, DetectedKind::Dataset);
    }
}
