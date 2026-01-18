//! Node helpers for SIGNIA graphs.
//!
//! This module provides higher-level utilities for working with schema nodes/entities.
//! The wire model (`model::v1`) is intentionally minimal. These helpers provide:
//! - typed views over common entity kinds
//! - convenience builders with deterministic defaults
//! - indexing utilities for fast lookup
//! - safe attribute accessors with clear error messages
//!
//! This module is designed to be used by the compiler pipeline, plugins, and tooling.
//! It does not implement I/O.

use std::collections::BTreeMap;

use crate::errors::{SigniaError, SigniaResult};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

#[cfg(feature = "canonical-json")]
use crate::model::v1::EntityV1;

/// A typed "entity kind" classification used by helpers.
///
/// This is *not* a fixed enumeration of all possible types in SIGNIA.
/// It covers common types used by built-in plugins and tools.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityKind {
    Repo,
    Dir,
    File,
    Module,
    Endpoint,
    Dataset,
    Table,
    Column,
    Workflow,
    Step,
    Artifact,
    Other(String),
}

impl EntityKind {
    pub fn from_type_str(s: &str) -> Self {
        match s {
            "repo" => Self::Repo,
            "dir" => Self::Dir,
            "file" => Self::File,
            "module" => Self::Module,
            "endpoint" => Self::Endpoint,
            "dataset" => Self::Dataset,
            "table" => Self::Table,
            "column" => Self::Column,
            "workflow" => Self::Workflow,
            "step" => Self::Step,
            "artifact" => Self::Artifact,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn as_type_str(&self) -> &str {
        match self {
            Self::Repo => "repo",
            Self::Dir => "dir",
            Self::File => "file",
            Self::Module => "module",
            Self::Endpoint => "endpoint",
            Self::Dataset => "dataset",
            Self::Table => "table",
            Self::Column => "column",
            Self::Workflow => "workflow",
            Self::Step => "step",
            Self::Artifact => "artifact",
            Self::Other(s) => s.as_str(),
        }
    }
}

/// Deterministic attribute getter helpers.
///
/// These helpers avoid panics and produce stable error messages.
#[cfg(feature = "canonical-json")]
pub mod attrs {
    use super::*;

    pub fn get_str(attrs: &Value, key: &str) -> SigniaResult<String> {
        let obj = attrs.as_object().ok_or_else(|| {
            SigniaError::invalid_argument("entity attrs must be an object")
        })?;

        let v = obj.get(key).ok_or_else(|| {
            SigniaError::invalid_argument(format!("missing entity attribute: {key}"))
        })?;

        let s = v.as_str().ok_or_else(|| {
            SigniaError::invalid_argument(format!("attribute {key} must be a string"))
        })?;

        Ok(s.to_string())
    }

    pub fn get_bool(attrs: &Value, key: &str) -> SigniaResult<bool> {
        let obj = attrs.as_object().ok_or_else(|| {
            SigniaError::invalid_argument("entity attrs must be an object")
        })?;

        let v = obj.get(key).ok_or_else(|| {
            SigniaError::invalid_argument(format!("missing entity attribute: {key}"))
        })?;

        v.as_bool().ok_or_else(|| {
            SigniaError::invalid_argument(format!("attribute {key} must be a boolean"))
        })
    }

    pub fn get_i64(attrs: &Value, key: &str) -> SigniaResult<i64> {
        let obj = attrs.as_object().ok_or_else(|| {
            SigniaError::invalid_argument("entity attrs must be an object")
        })?;

        let v = obj.get(key).ok_or_else(|| {
            SigniaError::invalid_argument(format!("missing entity attribute: {key}"))
        })?;

        v.as_i64().ok_or_else(|| {
            SigniaError::invalid_argument(format!("attribute {key} must be an integer"))
        })
    }
}

/// A builder for `EntityV1` with deterministic defaults.
#[cfg(feature = "canonical-json")]
#[derive(Debug, Clone)]
pub struct EntityBuilder {
    id: String,
    entity_type: String,
    name: String,
    attrs: BTreeMap<String, Value>,
    digests: Vec<crate::model::v1::DigestV1>,
}

#[cfg(feature = "canonical-json")]
impl EntityBuilder {
    pub fn new(id: impl Into<String>, entity_type: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            entity_type: entity_type.into(),
            name: name.into(),
            attrs: BTreeMap::new(),
            digests: Vec::new(),
        }
    }

    pub fn kind(mut self, kind: EntityKind) -> Self {
        self.entity_type = kind.as_type_str().to_string();
        self
    }

    pub fn attr(mut self, k: impl Into<String>, v: Value) -> Self {
        self.attrs.insert(k.into(), v);
        self
    }

    pub fn digest_sha256(mut self, hex64: impl Into<String>) -> Self {
        self.digests.push(crate::model::v1::DigestV1 {
            alg: "sha256".to_string(),
            hex: hex64.into(),
        });
        self
    }

    pub fn build(self) -> EntityV1 {
        let mut m = serde_json::Map::new();
        for (k, v) in self.attrs.into_iter() {
            m.insert(k, v);
        }

        EntityV1 {
            id: self.id,
            r#type: self.entity_type,
            name: self.name,
            attrs: Value::Object(m),
            digests: if self.digests.is_empty() { None } else { Some(self.digests) },
        }
    }
}

/// An index for entities keyed by id and by type.
#[cfg(feature = "canonical-json")]
#[derive(Debug, Clone)]
pub struct EntityIndex<'a> {
    by_id: BTreeMap<&'a str, &'a EntityV1>,
    by_type: BTreeMap<&'a str, Vec<&'a EntityV1>>,
}

#[cfg(feature = "canonical-json")]
impl<'a> EntityIndex<'a> {
    pub fn new(entities: &'a [EntityV1]) -> Self {
        let mut by_id = BTreeMap::new();
        let mut by_type: BTreeMap<&'a str, Vec<&'a EntityV1>> = BTreeMap::new();

        for e in entities {
            by_id.insert(e.id.as_str(), e);
            by_type.entry(e.r#type.as_str()).or_default().push(e);
        }

        // Deterministic order within each type by id.
        for v in by_type.values_mut() {
            v.sort_by(|a, b| a.id.cmp(&b.id));
        }

        Self { by_id, by_type }
    }

    pub fn get(&self, id: &str) -> Option<&'a EntityV1> {
        self.by_id.get(id).copied()
    }

    pub fn list_by_type(&self, t: &str) -> Vec<&'a EntityV1> {
        self.by_type.get(t).cloned().unwrap_or_default()
    }

    pub fn require(&self, id: &str) -> SigniaResult<&'a EntityV1> {
        self.get(id).ok_or_else(|| SigniaError::invalid_argument(format!("missing entity id: {id}")))
    }
}

#[cfg(test)]
#[cfg(feature = "canonical-json")]
mod tests {
    use super::*;
    use crate::model::v1::EntityV1;

    #[test]
    fn builder_builds_entity() {
        let e = EntityBuilder::new("ent:file:x", "file", "x")
            .attr("path", serde_json::Value::String("artifact:/x".to_string()))
            .digest_sha256("a".repeat(64))
            .build();

        assert_eq!(e.id, "ent:file:x");
        assert_eq!(e.r#type, "file");
        assert!(e.digests.is_some());
    }

    #[test]
    fn index_by_type_is_sorted() {
        let e1 = EntityV1 {
            id: "b".to_string(),
            r#type: "file".to_string(),
            name: "b".to_string(),
            attrs: serde_json::json!({}),
            digests: None,
        };
        let e2 = EntityV1 {
            id: "a".to_string(),
            r#type: "file".to_string(),
            name: "a".to_string(),
            attrs: serde_json::json!({}),
            digests: None,
        };

        let idx = EntityIndex::new(&[e1, e2]);
        let files = idx.list_by_type("file");
        assert_eq!(files[0].id, "a");
        assert_eq!(files[1].id, "b");
    }

    #[test]
    fn attr_helpers_work() {
        let attrs = serde_json::json!({"path": "artifact:/x", "ok": true, "n": 7});
        assert_eq!(attrs::get_str(&attrs, "path").unwrap(), "artifact:/x");
        assert_eq!(attrs::get_bool(&attrs, "ok").unwrap(), true);
        assert_eq!(attrs::get_i64(&attrs, "n").unwrap(), 7);
    }
}
