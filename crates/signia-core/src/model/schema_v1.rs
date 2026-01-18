//! SIGNIA Schema v1 model.
//!
//! This file defines the canonical Rust representation for SIGNIA Schema v1.
//! The schema instance represents a *structure graph*:
//! - entities (nodes) with deterministic attributes
//! - edges (relationships) with deterministic attributes
//! - a meta section describing provenance and normalization policy
//!
//! Notes:
//! - This is a *wire model* and must remain backward compatible for v1.
//! - Do not add breaking field changes. Additive optional fields are allowed.
//! - Canonical hashing must use `crate::canonical` rather than default serde JSON encoding.

#[cfg(feature = "canonical-json")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

/// A SIGNIA schema instance.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct SchemaV1 {
    /// Schema version. Must be "v1".
    pub version: String,

    /// High-level schema kind (repo, dataset, openapi, workflow, etc).
    pub kind: String,

    /// Metadata describing the origin and normalization policy.
    pub meta: Value,

    /// Graph entities.
    pub entities: Vec<EntityV1>,

    /// Graph edges.
    pub edges: Vec<EdgeV1>,
}

/// A graph entity (node).
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct EntityV1 {
    pub id: String,
    #[cfg_attr(feature = "canonical-json", serde(rename = "type"))]
    pub r#type: String,
    pub name: String,

    /// Deterministic attribute map stored as JSON object.
    pub attrs: Value,

    /// Optional content digests (e.g. file hashes).
    #[cfg_attr(feature = "canonical-json", serde(skip_serializing_if = "Option::is_none"))]
    pub digests: Option<Vec<DigestV1>>,
}

/// Digest information for entities.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct DigestV1 {
    /// Hash algorithm name ("sha256" | "blake3").
    pub alg: String,

    /// Lowercase hex digest (64 chars).
    pub hex: String,
}

/// A graph edge (relationship).
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct EdgeV1 {
    pub id: String,
    #[cfg_attr(feature = "canonical-json", serde(rename = "type"))]
    pub r#type: String,
    pub from: String,
    pub to: String,

    /// Deterministic attribute map stored as JSON object.
    pub attrs: Value,
}

/// Minimal strongly typed view for the meta section of v1.
///
/// This is used by compilers and verifiers, but `SchemaV1.meta` remains generic JSON.
/// Keeping meta as generic JSON gives forward compatibility for new meta fields.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct SchemaMetaV1 {
    pub name: String,
    #[cfg_attr(feature = "canonical-json", serde(default))]
    pub description: Option<String>,

    pub created_at: String,

    pub source: SourceRefV1,
    pub normalization: NormalizationV1,

    #[cfg_attr(feature = "canonical-json", serde(default))]
    pub labels: Option<std::collections::BTreeMap<String, String>>,
}

/// Source reference for schema compilation.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct SourceRefV1 {
    pub r#type: String,
    pub locator: String,
    #[cfg_attr(feature = "canonical-json", serde(default))]
    pub content_hash: Option<String>,
}

/// Normalization policy recorded in meta.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct NormalizationV1 {
    pub policy_version: String,
    pub path_root: String,
    pub newline: String,
    pub encoding: String,
    pub symlinks: String,
    pub network: String,
}

impl SchemaV1 {
    /// Create a new schema with empty entities/edges.
    pub fn new(kind: impl Into<String>, meta: Value) -> Self {
        Self {
            version: "v1".to_string(),
            kind: kind.into(),
            meta,
            entities: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add an entity.
    pub fn push_entity(&mut self, e: EntityV1) {
        self.entities.push(e);
    }

    /// Add an edge.
    pub fn push_edge(&mut self, e: EdgeV1) {
        self.edges.push(e);
    }
}

#[cfg(test)]
#[cfg(feature = "canonical-json")]
mod tests {
    use super::*;

    #[test]
    fn schema_roundtrip_json() {
        let schema = SchemaV1::new("repo", serde_json::json!({"name":"demo","createdAt":"1970-01-01T00:00:00Z","source":{"type":"path","locator":"artifact:/demo"},"normalization":{"policyVersion":"v1","pathRoot":"artifact:/","newline":"lf","encoding":"utf-8","symlinks":"deny","network":"deny"}}));
        let s = serde_json::to_string(&schema).unwrap();
        let back: SchemaV1 = serde_json::from_str(&s).unwrap();
        assert_eq!(back.version, "v1");
        assert_eq!(back.kind, "repo");
    }

    #[test]
    fn entity_with_digest_serializes() {
        let e = EntityV1 {
            id: "ent:file:x".to_string(),
            r#type: "file".to_string(),
            name: "x".to_string(),
            attrs: serde_json::json!({"path":"artifact:/x"}),
            digests: Some(vec![DigestV1 { alg: "sha256".to_string(), hex: "a".repeat(64) }]),
        };
        let s = serde_json::to_string(&e).unwrap();
        assert!(s.contains("sha256"));
        assert!(s.contains("digests"));
    }
}
