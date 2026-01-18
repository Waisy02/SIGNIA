//! Repository metadata helpers for the built-in `repo` plugin.
//!
//! This module converts repo snapshots into deterministic metadata suitable for:
//! - pipeline metadata enrichment
//! - manifest generation
//! - provenance anchoring
//! - UI and indexing
//!
//! It does not perform I/O and does not depend on system time.

#![cfg(feature = "builtin")]

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use signia_core::determinism::hashing::hash_bytes_hex;
use signia_core::provenance::SourceRef;

use crate::builtin::repo::dep_graph::{dep_graph_to_json, extract_dep_graph, DepGraph};
use crate::builtin::repo::github_fetch::{RepoFile, RepoSnapshot};
use crate::builtin::repo::tree_walk::normalize_repo_path;

/// Minimal repo identity metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoIdentity {
    pub name: String,
    pub git_ref: String,
    pub source: SourceRef,
    pub snapshot_hash: String,
}

/// Aggregate repo metadata computed deterministically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoMetadata {
    pub identity: RepoIdentity,

    /// File counts and sizes.
    pub file_count: u64,
    pub total_bytes: u64,

    /// Simple language heuristics (by extension).
    pub language_bytes: BTreeMap<String, u64>,

    /// Top-level paths present.
    pub top_level: Vec<String>,

    /// Important manifests detected.
    pub manifests: Vec<String>,

    /// Dependency graph (best-effort).
    pub dep_graph: DepGraph,

    /// Additional stable tags for UI.
    pub tags: Vec<String>,
}

impl RepoMetadata {
    /// Convert to JSON metadata suitable for injecting into `PipelineContext.inputs`.
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "identity": {
                "name": self.identity.name,
                "ref": self.identity.git_ref,
                "source": {
                    "type": self.identity.source.r#type,
                    "locator": self.identity.source.locator,
                    "digest": self.identity.source.digest,
                    "revision": self.identity.source.revision,
                    "subpath": self.identity.source.subpath,
                    "extras": self.identity.source.extras,
                },
                "snapshotHash": self.identity.snapshot_hash,
            },
            "stats": {
                "fileCount": self.file_count,
                "totalBytes": self.total_bytes,
                "languageBytes": self.language_bytes,
                "topLevel": self.top_level,
                "manifests": self.manifests,
            },
            "deps": dep_graph_to_json(&self.dep_graph),
            "tags": self.tags,
        })
    }
}

/// Build deterministic metadata from a snapshot.
pub fn build_repo_metadata(
    owner: &str,
    repo: &str,
    git_ref: &str,
    snapshot: &RepoSnapshot,
) -> Result<RepoMetadata> {
    if owner.trim().is_empty() || repo.trim().is_empty() || git_ref.trim().is_empty() {
        return Err(anyhow!("owner/repo/ref must be non-empty"));
    }

    let name = format!("{}/{}", owner, repo);

    let (file_count, total_bytes) = count_files_bytes(&snapshot.files)?;
    let language_bytes = language_bytes(&snapshot.files)?;
    let top_level = top_level_paths(&snapshot.files)?;
    let manifests = detect_manifests(&snapshot.files)?;
    let dep_graph = extract_dep_graph(&snapshot.files)?;

    let mut tags = BTreeSet::<String>::new();
    if manifests.iter().any(|m| m.ends_with("Cargo.toml")) {
        tags.insert("rust".to_string());
    }
    if manifests.iter().any(|m| m.ends_with("package.json")) {
        tags.insert("node".to_string());
    }
    if manifests.iter().any(|m| m.ends_with("go.mod")) {
        tags.insert("go".to_string());
    }
    if manifests.iter().any(|m| m.to_ascii_lowercase().contains("requirements")) {
        tags.insert("python".to_string());
    }

    // If we found deps, tag as supply-chain relevant.
    if !dep_graph.is_empty() {
        tags.insert("deps".to_string());
    }

    // Stable extra tag: a content-free digest of manifest set
    let manifest_digest = hash_manifest_set(&manifests)?;
    tags.insert(format!("manifestset:{manifest_digest}"));

    Ok(RepoMetadata {
        identity: RepoIdentity {
            name,
            git_ref: git_ref.to_string(),
            source: snapshot.source.clone(),
            snapshot_hash: snapshot.snapshot_hash.clone(),
        },
        file_count,
        total_bytes,
        language_bytes,
        top_level,
        manifests,
        dep_graph,
        tags: tags.into_iter().collect(),
    })
}

fn count_files_bytes(files: &[RepoFile]) -> Result<(u64, u64)> {
    let mut total = 0u64;
    for f in files {
        let p = normalize_repo_path(&f.path)?;
        if p.is_empty() {
            return Err(anyhow!("empty path in snapshot"));
        }
        total = total.saturating_add(f.size);
    }
    Ok((files.len() as u64, total))
}

fn language_bytes(files: &[RepoFile]) -> Result<BTreeMap<String, u64>> {
    let mut map: BTreeMap<String, u64> = BTreeMap::new();

    for f in files {
        let p = normalize_repo_path(&f.path)?;
        let ext = p.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        let lang = match ext.as_str() {
            "rs" => "Rust",
            "ts" => "TypeScript",
            "tsx" => "TypeScript",
            "js" => "JavaScript",
            "jsx" => "JavaScript",
            "py" => "Python",
            "go" => "Go",
            "sol" => "Solidity",
            "md" => "Markdown",
            "yaml" | "yml" => "YAML",
            "json" => "JSON",
            "toml" => "TOML",
            "lock" => "Lockfile",
            _ => {
                if p.contains("Dockerfile") || p.ends_with("Dockerfile") {
                    "Docker"
                } else {
                    "Other"
                }
            }
        };

        *map.entry(lang.to_string()).or_insert(0) = map
            .get(lang)
            .copied()
            .unwrap_or(0)
            .saturating_add(f.size);
    }

    Ok(map)
}

fn top_level_paths(files: &[RepoFile]) -> Result<Vec<String>> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    for f in files {
        let p = normalize_repo_path(&f.path)?;
        let top = p.split('/').next().unwrap_or("").to_string();
        if !top.is_empty() {
            set.insert(top);
        }
    }
    Ok(set.into_iter().collect())
}

fn detect_manifests(files: &[RepoFile]) -> Result<Vec<String>> {
    let mut set: BTreeSet<String> = BTreeSet::new();

    for f in files {
        let p = normalize_repo_path(&f.path)?;
        let lower = p.to_ascii_lowercase();

        // Common manifests
        if p.ends_with("Cargo.toml")
            || p.ends_with("Cargo.lock")
            || p.ends_with("package.json")
            || p.ends_with("package-lock.json")
            || p.ends_with("pnpm-lock.yaml")
            || p.ends_with("yarn.lock")
            || p.ends_with("go.mod")
            || p.ends_with("go.sum")
            || lower.ends_with("requirements.txt")
            || (lower.contains("requirements") && lower.ends_with(".txt"))
            || p.ends_with("pyproject.toml")
            || p.ends_with("Pipfile")
            || p.ends_with("Pipfile.lock")
            || p.ends_with("Dockerfile")
            || p.ends_with("docker-compose.yml")
            || p.ends_with("docker-compose.yaml")
            || p.ends_with("openapi.yaml")
            || p.ends_with("openapi.yml")
            || p.ends_with("openapi.json")
        {
            set.insert(p);
        }
    }

    Ok(set.into_iter().collect())
}

fn hash_manifest_set(manifests: &[String]) -> Result<String> {
    let mut buf = Vec::new();
    for m in manifests {
        buf.extend_from_slice(m.as_bytes());
        buf.extend_from_slice(b"\n");
    }
    hash_bytes_hex(&buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin::repo::github_fetch::snapshot_from_files;

    #[test]
    fn builds_metadata() {
        let req = crate::builtin::repo::github_fetch::GitHubFetchRequest::new("o", "r", "deadbeef")
            .with_limits(10, 1024)
            .with_include("**".to_string());

        let files = vec![
            RepoFile {
                path: "Cargo.toml".to_string(),
                size: 20,
                sha256: Some("x".to_string()),
                mode: None,
                bytes: Some(br#"[dependencies]
serde = "1.0"
"#.to_vec()),
            },
            RepoFile {
                path: "src/lib.rs".to_string(),
                size: 10,
                sha256: None,
                mode: None,
                bytes: Some(b"fn main(){}".to_vec()),
            },
        ];

        let snapshot = snapshot_from_files(&req, files).unwrap();
        let meta = build_repo_metadata("o", "r", "deadbeef", &snapshot).unwrap();
        assert_eq!(meta.file_count, 2);
        assert!(meta.tags.iter().any(|t| t == "rust"));
    }
}
