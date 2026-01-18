//! JSON helpers for built-in configuration.
//!
//! This module provides deterministic JSON parsing, validation, and merging
//! utilities for `BuiltinConfig`.
//!
//! Design goals:
//! - Hosts may load config from files, env, flags, etc.
//! - This module only operates on JSON values and structs.
//! - Deterministic behavior suitable for hashing and reproducibility.

#![cfg(feature = "builtin")]

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::BuiltinConfig;

/// Parse a JSON value into `BuiltinConfig`.
///
/// This performs strict type checking via Serde.
pub fn parse_config_json(v: &Value) -> Result<BuiltinConfig> {
    let cfg: BuiltinConfig = serde_json::from_value(v.clone())
        .map_err(|e| anyhow!("invalid builtin config json: {e}"))?;
    Ok(cfg)
}

/// Merge two configs, with `override_cfg` taking precedence.
///
/// Rules:
/// - Scalars and structs in override replace base.
/// - For vectors, override replaces base entirely.
/// - This is deterministic and explicit (no deep magic).
pub fn merge_configs(base: BuiltinConfig, override_cfg: BuiltinConfig) -> BuiltinConfig {
    BuiltinConfig {
        repo: merge_repo(base.repo, override_cfg.repo),
        dataset: merge_dataset(base.dataset, override_cfg.dataset),
        workflow: merge_workflow(base.workflow, override_cfg.workflow),
        api: merge_api(base.api, override_cfg.api),
    }
}

fn merge_repo(base: super::RepoConfig, o: super::RepoConfig) -> super::RepoConfig {
    super::RepoConfig {
        max_files: o.max_files,
        max_total_bytes: o.max_total_bytes,
        max_file_bytes: o.max_file_bytes,
        include: if o.include.is_empty() { base.include } else { o.include },
        exclude: if o.exclude.is_empty() { base.exclude } else { o.exclude },
        allow_binary: o.allow_binary,
    }
}

fn merge_dataset(base: super::DatasetConfig, o: super::DatasetConfig) -> super::DatasetConfig {
    super::DatasetConfig {
        max_files: o.max_files,
        max_total_bytes: o.max_total_bytes,
        enable_merkle: o.enable_merkle,
    }
}

fn merge_workflow(base: super::WorkflowConfig, o: super::WorkflowConfig) -> super::WorkflowConfig {
    super::WorkflowConfig {
        max_nodes: o.max_nodes,
        max_edges: o.max_edges,
        enable_yaml: o.enable_yaml,
    }
}

fn merge_api(base: super::ApiConfig, o: super::ApiConfig) -> super::ApiConfig {
    super::ApiConfig {
        enabled: o.enabled,
        version: if o.version.is_empty() { base.version } else { o.version },
    }
}

/// Convert a config to a canonical JSON representation.
///
/// This is useful for hashing or debugging.
pub fn config_to_canonical_json(cfg: &BuiltinConfig) -> Result<Value> {
    let v = serde_json::to_value(cfg)?;
    let c = signia_core::determinism::canonical_json::canonicalize_json(&v)?;
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_merge_config() {
        let base = BuiltinConfig::default();
        let override_json = serde_json::json!({
            "repo": { "max_files": 10 },
            "api": { "enabled": false }
        });
        let override_cfg = parse_config_json(&override_json).unwrap();
        let merged = merge_configs(base, override_cfg);
        assert_eq!(merged.repo.max_files, 10);
        assert!(!merged.api.enabled);
    }

    #[test]
    fn canonical_json_is_stable() {
        let cfg = BuiltinConfig::default();
        let a = config_to_canonical_json(&cfg).unwrap();
        let b = config_to_canonical_json(&cfg).unwrap();
        assert_eq!(a, b);
    }
}
