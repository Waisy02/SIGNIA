//! Built-in API module.
//!
//! This module exposes read-only, deterministic API helpers for built-in plugins.
//! It is intended to be used by HTTP servers, CLIs, and embedded hosts.
//!
//! Design constraints:
//! - No network or filesystem I/O.
//! - Pure functions over in-memory data.
//! - Deterministic output suitable for hashing and caching.

#![cfg(feature = "builtin")]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::builtin::spec::{builtin_specs};
use crate::builtin::spec::link_graph::{build_link_graph, link_graph_to_json};
use crate::spec::PluginSpec;

/// Top-level API response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub data: T,
}

/// Return all built-in plugin specs.
pub fn get_builtin_specs() -> ApiResponse<Vec<PluginSpec>> {
    ApiResponse {
        ok: true,
        data: builtin_specs(),
    }
}

/// Return built-in specs rendered as a link graph.
pub fn get_builtin_link_graph() -> ApiResponse<Value> {
    let specs = builtin_specs();
    let graph = build_link_graph(&specs);
    ApiResponse {
        ok: true,
        data: link_graph_to_json(&graph),
    }
}

/// Return a single built-in plugin spec by id.
pub fn get_builtin_spec_by_id(id: &str) -> ApiResponse<Option<PluginSpec>> {
    let spec = builtin_specs().into_iter().find(|s| s.id == id);
    ApiResponse { ok: true, data: spec }
}

/// Health check for embedded usage.
pub fn health() -> ApiResponse<&'static str> {
    ApiResponse {
        ok: true,
        data: "ok",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs_endpoint_returns_data() {
        let resp = get_builtin_specs();
        assert!(resp.ok);
        assert!(!resp.data.is_empty());
    }

    #[test]
    fn graph_endpoint_returns_nodes() {
        let resp = get_builtin_link_graph();
        assert!(resp.ok);
        assert!(resp.data.get("nodes").is_some());
    }

    #[test]
    fn lookup_by_id() {
        let resp = get_builtin_spec_by_id("builtin.repo");
        assert!(resp.ok);
        assert!(resp.data.is_some());
    }
}
