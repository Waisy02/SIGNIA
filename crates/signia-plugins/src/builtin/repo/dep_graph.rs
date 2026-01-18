//! Dependency graph extraction for the built-in `repo` plugin.
//!
//! This module builds a deterministic dependency graph from a repository snapshot.
//!
//! IMPORTANT:
//! - This code performs no filesystem or network I/O.
//! - Inputs are `RepoFile` entries produced by the host (optionally with bytes).
//!
//! Supported extractors (best-effort, deterministic):
//! - Rust: Cargo.toml (simple section parser; no full TOML engine)
//! - Node: package.json (JSON parser)
//! - Go: go.mod (line parser)
//! - Python: requirements.txt / requirements*.txt (line parser)
//!
//! The output graph is intended for:
//! - metadata / provenance enrichment
//! - on-chain anchoring of dependency sets
//! - security and supply-chain analysis pipelines
//!
//! Determinism rules:
//! - parsing is tolerant but stable
//! - dependency identifiers are normalized
//! - results are sorted

#![cfg(feature = "builtin")]

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};

use crate::builtin::repo::github_fetch::RepoFile;
use crate::builtin::repo::tree_walk::normalize_repo_path;

/// A dependency ecosystem.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ecosystem {
    Rust,
    Node,
    Go,
    Python,
    Unknown,
}

impl Ecosystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            Ecosystem::Rust => "rust",
            Ecosystem::Node => "node",
            Ecosystem::Go => "go",
            Ecosystem::Python => "python",
            Ecosystem::Unknown => "unknown",
        }
    }
}

/// A dependency coordinate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dep {
    pub ecosystem: Ecosystem,
    /// Dependency name (normalized).
    pub name: String,
    /// Version spec if known (normalized).
    pub version: Option<String>,
    /// Optional source (registry, git URL, etc).
    pub source: Option<String>,
}

impl Dep {
    pub fn id(&self) -> String {
        match &self.version {
            Some(v) => format!("{}:{}@{}", self.ecosystem.as_str(), self.name, v),
            None => format!("{}:{}", self.ecosystem.as_str(), self.name),
        }
    }
}

/// A dependency edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DepEdge {
    /// From component path (e.g., "Cargo.toml", "package.json" or module file).
    pub from: String,
    /// Dependency id.
    pub to: String,
    /// Edge kind ("depends_on").
    pub kind: String,
}

/// A deterministic dependency graph.
#[derive(Debug, Clone, Default)]
pub struct DepGraph {
    pub deps: BTreeSet<Dep>,
    pub edges: BTreeSet<DepEdge>,
    /// Metadata about extracted components.
    pub components: BTreeMap<String, String>,
}

impl DepGraph {
    pub fn is_empty(&self) -> bool {
        self.deps.is_empty() && self.edges.is_empty()
    }

    pub fn deps_count(&self) -> usize {
        self.deps.len()
    }

    pub fn edges_count(&self) -> usize {
        self.edges.len()
    }
}

/// Extract a dependency graph from a repo snapshot.
pub fn extract_dep_graph(files: &[RepoFile]) -> Result<DepGraph> {
    let mut g = DepGraph::default();

    // Normalize and index files by path.
    let mut by_path: BTreeMap<String, &RepoFile> = BTreeMap::new();
    for f in files {
        let p = normalize_repo_path(&f.path)?;
        by_path.insert(p, f);
    }

    // Rust: Cargo.toml files (root + workspaces).
    for (path, f) in &by_path {
        if path.ends_with("Cargo.toml") {
            if let Some(bytes) = &f.bytes {
                let text = std::str::from_utf8(bytes).map_err(|_| anyhow!("Cargo.toml not utf-8: {path}"))?;
                let deps = parse_cargo_toml_deps(text)?;
                if !deps.is_empty() {
                    g.components.insert(path.clone(), "cargo".to_string());
                }
                for d in deps {
                    let dep_id = d.id();
                    g.deps.insert(d);
                    g.edges.insert(DepEdge {
                        from: path.clone(),
                        to: dep_id,
                        kind: "depends_on".to_string(),
                    });
                }
            }
        }
    }

    // Node: package.json
    for (path, f) in &by_path {
        if path.ends_with("package.json") {
            if let Some(bytes) = &f.bytes {
                let deps = parse_package_json_deps(bytes)?;
                if !deps.is_empty() {
                    g.components.insert(path.clone(), "npm".to_string());
                }
                for d in deps {
                    let dep_id = d.id();
                    g.deps.insert(d);
                    g.edges.insert(DepEdge {
                        from: path.clone(),
                        to: dep_id,
                        kind: "depends_on".to_string(),
                    });
                }
            }
        }
    }

    // Go: go.mod
    for (path, f) in &by_path {
        if path.ends_with("go.mod") {
            if let Some(bytes) = &f.bytes {
                let text = std::str::from_utf8(bytes).map_err(|_| anyhow!("go.mod not utf-8: {path}"))?;
                let deps = parse_go_mod_deps(text)?;
                if !deps.is_empty() {
                    g.components.insert(path.clone(), "gomod".to_string());
                }
                for d in deps {
                    let dep_id = d.id();
                    g.deps.insert(d);
                    g.edges.insert(DepEdge {
                        from: path.clone(),
                        to: dep_id,
                        kind: "depends_on".to_string(),
                    });
                }
            }
        }
    }

    // Python: requirements*.txt
    for (path, f) in &by_path {
        let lower = path.to_ascii_lowercase();
        if lower.ends_with("requirements.txt") || (lower.contains("requirements") && lower.ends_with(".txt")) {
            if let Some(bytes) = &f.bytes {
                let text = std::str::from_utf8(bytes).map_err(|_| anyhow!("requirements not utf-8: {path}"))?;
                let deps = parse_requirements_txt(text)?;
                if !deps.is_empty() {
                    g.components.insert(path.clone(), "pip".to_string());
                }
                for d in deps {
                    let dep_id = d.id();
                    g.deps.insert(d);
                    g.edges.insert(DepEdge {
                        from: path.clone(),
                        to: dep_id,
                        kind: "depends_on".to_string(),
                    });
                }
            }
        }
    }

    Ok(g)
}

/// Convert graph to a deterministic JSON value for use as pipeline metadata.
pub fn dep_graph_to_json(g: &DepGraph) -> serde_json::Value {
    let deps = g
        .deps
        .iter()
        .map(|d| {
            let mut o = serde_json::Map::new();
            o.insert("ecosystem".to_string(), serde_json::Value::String(d.ecosystem.as_str().to_string()));
            o.insert("name".to_string(), serde_json::Value::String(d.name.clone()));
            if let Some(v) = &d.version {
                o.insert("version".to_string(), serde_json::Value::String(v.clone()));
            }
            if let Some(s) = &d.source {
                o.insert("source".to_string(), serde_json::Value::String(s.clone()));
            }
            serde_json::Value::Object(o)
        })
        .collect::<Vec<_>>();

    let edges = g
        .edges
        .iter()
        .map(|e| {
            serde_json::json!({
                "from": e.from,
                "to": e.to,
                "kind": e.kind
            })
        })
        .collect::<Vec<_>>();

    let components = g
        .components
        .iter()
        .map(|(k, v)| serde_json::json!({"path": k, "type": v}))
        .collect::<Vec<_>>();

    serde_json::json!({
        "deps": deps,
        "edges": edges,
        "components": components,
        "counts": {
            "deps": g.deps_count(),
            "edges": g.edges_count(),
        }
    })
}

/// Parse dependency lines from a minimal Cargo.toml section.
/// This is a best-effort parser that avoids a full TOML dependency.
fn parse_cargo_toml_deps(toml_text: &str) -> Result<Vec<Dep>> {
    let mut deps: Vec<Dep> = Vec::new();

    let mut in_deps = false;
    let mut in_dev_deps = false;
    let mut in_build_deps = false;

    for raw in toml_text.lines() {
        let line = raw.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section switches
        if line.starts_with('[') && line.ends_with(']') {
            let sec = line.trim_matches(&['[', ']'][..]).trim();
            in_deps = sec == "dependencies";
            in_dev_deps = sec == "dev-dependencies";
            in_build_deps = sec == "build-dependencies";
            // Also support workspace dependency tables:
            // [workspace.dependencies]
            if sec == "workspace.dependencies" {
                in_deps = true;
                in_dev_deps = false;
                in_build_deps = false;
            }
            continue;
        }

        if !(in_deps || in_dev_deps || in_build_deps) {
            continue;
        }

        // Very simple "name = ..." parser
        // Examples:
        // serde = "1.0"
        // anyhow = { version = "1", features = ["x"] }
        // tokio = { git = "...", rev = "..."}
        let Some((name_raw, rhs_raw)) = line.split_once('=') else {
            continue;
        };

        let name = normalize_dep_name(name_raw)?;
        let rhs = rhs_raw.trim();

        let mut d = Dep {
            ecosystem: Ecosystem::Rust,
            name,
            version: None,
            source: None,
        };

        if rhs.starts_with('"') {
            // Version string
            if let Some(v) = extract_quoted(rhs) {
                d.version = Some(normalize_version(&v));
            }
        } else if rhs.starts_with('{') {
            // Inline table: try to extract version/git/path
            if let Some(v) = find_key_quoted(rhs, "version") {
                d.version = Some(normalize_version(&v));
            }
            if let Some(git) = find_key_quoted(rhs, "git") {
                d.source = Some(git);
            } else if let Some(path) = find_key_quoted(rhs, "path") {
                d.source = Some(format!("path:{path}"));
            } else if let Some(reg) = find_key_quoted(rhs, "registry") {
                d.source = Some(format!("registry:{reg}"));
            }
        } else {
            // Unrecognized; ignore to stay stable.
        }

        deps.push(d);
    }

    // Stable sort by dep id.
    deps.sort_by(|a, b| a.id().cmp(&b.id()));
    deps.dedup_by(|a, b| a.id() == b.id());
    Ok(deps)
}

/// Parse dependencies from package.json.
fn parse_package_json_deps(bytes: &[u8]) -> Result<Vec<Dep>> {
    let v: serde_json::Value = serde_json::from_slice(bytes)?;
    let mut out = Vec::new();

    for key in ["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"] {
        if let Some(obj) = v.get(key).and_then(|x| x.as_object()) {
            for (name, ver_val) in obj {
                let name_n = normalize_dep_name(name)?;
                let ver = ver_val.as_str().map(|s| normalize_version(s));
                out.push(Dep {
                    ecosystem: Ecosystem::Node,
                    name: name_n,
                    version: ver,
                    source: None,
                });
            }
        }
    }

    out.sort_by(|a, b| a.id().cmp(&b.id()));
    out.dedup_by(|a, b| a.id() == b.id());
    Ok(out)
}

/// Parse dependencies from go.mod.
fn parse_go_mod_deps(text: &str) -> Result<Vec<Dep>> {
    let mut out = Vec::new();
    let mut in_require_block = false;

    for raw in text.lines() {
        let line = raw.trim();

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if line.starts_with("require (") {
            in_require_block = true;
            continue;
        }
        if in_require_block && line == ")" {
            in_require_block = false;
            continue;
        }

        let is_require_line = line.starts_with("require ") || in_require_block;
        if !is_require_line {
            continue;
        }

        let l = if line.starts_with("require ") {
            line.trim_start_matches("require ").trim()
        } else {
            line
        };

        // Pattern: module version [// indirect]
        let parts: Vec<&str> = l.split_whitespace().collect();
        if parts.len() >= 2 {
            let name_n = normalize_dep_name(parts[0])?;
            let ver = normalize_version(parts[1]);
            out.push(Dep {
                ecosystem: Ecosystem::Go,
                name: name_n,
                version: Some(ver),
                source: None,
            });
        }
    }

    out.sort_by(|a, b| a.id().cmp(&b.id()));
    out.dedup_by(|a, b| a.id() == b.id());
    Ok(out)
}

/// Parse dependencies from requirements.txt format.
fn parse_requirements_txt(text: &str) -> Result<Vec<Dep>> {
    let mut out = Vec::new();

    for raw in text.lines() {
        let mut line = raw.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Drop inline comments deterministically (simple split).
        if let Some((l, _c)) = line.split_once('#') {
            line = l.trim();
        }
        if line.is_empty() {
            continue;
        }

        // Skip includes and options that are not deps
        if line.starts_with("-r ") || line.starts_with("--requirement") {
            continue;
        }
        if line.starts_with("-c ") || line.starts_with("--constraint") {
            continue;
        }
        if line.starts_with("--") || line.starts_with('-') {
            continue;
        }

        // Basic formats:
        // pkg==1.2.3
        // pkg>=1.0
        // pkg
        // git+https://...#egg=pkg
        if line.starts_with("git+") || line.contains("://") {
            // Best-effort: extract egg name
            let egg = line.split("#egg=").nth(1).map(|s| s.trim()).filter(|s| !s.is_empty());
            if let Some(name) = egg {
                out.push(Dep {
                    ecosystem: Ecosystem::Python,
                    name: normalize_dep_name(name)?,
                    version: None,
                    source: Some(line.to_string()),
                });
            }
            continue;
        }

        let (name_part, ver_part) = split_req_name_version(line);
        let name_n = normalize_dep_name(name_part)?;
        let ver = ver_part.map(normalize_version);

        out.push(Dep {
            ecosystem: Ecosystem::Python,
            name: name_n,
            version: ver,
            source: None,
        });
    }

    out.sort_by(|a, b| a.id().cmp(&b.id()));
    out.dedup_by(|a, b| a.id() == b.id());
    Ok(out)
}

/// Normalize dependency name:
/// - trim
/// - lowercase for ecosystems that are case-insensitive (python, node)
/// - ensure ASCII where possible
fn normalize_dep_name(s: &str) -> Result<String> {
    let name = s.trim().trim_matches('"').trim_matches('\'').trim();
    if name.is_empty() {
        return Err(anyhow!("empty dependency name"));
    }
    // Conservative: keep ASCII requirement to make ids stable and safe.
    // If a name is non-ASCII, keep it but normalize whitespace.
    let mut out = name.to_string();
    out = out.replace(char::is_whitespace, "");
    // Lowercase common ecosystems (safe even for Rust crates).
    out = out.to_ascii_lowercase();
    Ok(out)
}

/// Normalize version strings:
/// - trim
/// - collapse whitespace
fn normalize_version(s: &str) -> String {
    s.trim().replace(char::is_whitespace, "")
}

/// Extract first quoted string if present at beginning.
fn extract_quoted(s: &str) -> Option<String> {
    let s = s.trim();
    if !s.starts_with('"') {
        return None;
    }
    let rest = &s[1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Find key="value" in a TOML inline table string (best-effort).
fn find_key_quoted(table: &str, key: &str) -> Option<String> {
    // Look for patterns like: key = "..."
    // This is a tolerant scan; deterministic by using the first match.
    let needle = format!("{key}");
    let mut idx = 0usize;
    while let Some(pos) = table[idx..].find(&needle) {
        let start = idx + pos + needle.len();
        let after = &table[start..];
        // must be followed by optional spaces then '='
        let after = after.trim_start();
        if !after.starts_with('=') {
            idx = start;
            continue;
        }
        let after = after[1..].trim_start();
        if let Some(v) = extract_quoted(after) {
            return Some(v);
        }
        idx = start;
    }
    None
}

/// Split requirement line into name and version part.
fn split_req_name_version(line: &str) -> (&str, Option<&str>) {
    for op in ["==", ">=", "<=", "~=", "!=", ">", "<"] {
        if let Some((a, b)) = line.split_once(op) {
            return (a.trim(), Some(format!("{op}{}", b.trim()).as_str()));
        }
    }
    (line.trim(), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin::repo::github_fetch::RepoFile;

    #[test]
    fn parses_package_json_deps() {
        let bytes = br#"{
            "dependencies": { "react": "^18.0.0" },
            "devDependencies": { "typescript": "^5.0.0" }
        }"#;
        let deps = parse_package_json_deps(bytes).unwrap();
        assert!(deps.iter().any(|d| d.name == "react"));
        assert!(deps.iter().any(|d| d.name == "typescript"));
    }

    #[test]
    fn parses_go_mod_deps() {
        let text = r#"
module example.com/x

go 1.22

require (
  github.com/gorilla/mux v1.8.0
  golang.org/x/crypto v0.17.0 // indirect
)
"#;
        let deps = parse_go_mod_deps(text).unwrap();
        assert!(deps.iter().any(|d| d.name == "github.com/gorilla/mux"));
    }

    #[test]
    fn extract_dep_graph_empty_without_bytes() {
        let files = vec![RepoFile {
            path: "Cargo.toml".to_string(),
            size: 10,
            sha256: None,
            mode: None,
            bytes: None,
        }];
        let g = extract_dep_graph(&files).unwrap();
        assert!(g.is_empty());
    }
}
