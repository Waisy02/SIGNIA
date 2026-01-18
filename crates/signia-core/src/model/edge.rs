//! Edge helpers for SIGNIA graphs.
//!
//! This module provides higher-level utilities for working with schema edges.
//! It complements `node.rs` by offering:
//! - typed edge kinds
//! - safe attribute accessors
//! - deterministic builders
//! - indexing helpers
//!
//! The wire format remains `model::v1::EdgeV1`. These helpers are optional but
//! useful for compiler, verifier, and analysis tooling.

use std::collections::BTreeMap;

use crate::errors::{SigniaError, SigniaResult};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

#[cfg(feature = "canonical-json")]
use crate::model::v1::EdgeV1;

/// Common edge kind classification.
///
/// This enum is not exhaustive and does not restrict user-defined edge types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeKind {
    Contains,
    Imports,
    References,
    DependsOn,
    Defines,
    Produces,
    Consumes,
    Flow,
    UsesSchema,
    ForeignKey,
    Other(String),
}

impl EdgeKind {
    pub fn from_type_str(s: &str) -> Self {
        match s {
            "contains" => Self::Contains,
            "imports" => Self::Imports,
            "references" => Self::References,
            "depends_on" => Self::DependsOn,
            "defines" => Self::Defines,
            "produces" => Self::Produces,
            "consumes" => Self::Consumes,
            "flow" => Self::Flow,
            "uses_schema" => Self::UsesSchema,
            "foreign_key" => Self::ForeignKey,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn as_type_str(&self) -> &str {
        match self {
            Self::Contains => "contains",
            Self::Imports => "imports",
            Self::References => "references",
            Self::DependsOn => "depends_on",
            Self::Defines => "defines",
            Self::Produces => "produces",
            Self::Consumes => "consumes",
            Self::Flow => "flow",
            Self::UsesSchema => "uses_schema",
            Self::ForeignKey => "foreign_key",
            Self::Other(s) => s.as_str(),
        }
    }
}

/// Attribute helpers for edges.
#[cfg(feature = "canonical-json")]
pub mod attrs {
    use super::*;

    pub fn get_str(attrs: &Value, key: &str) -> SigniaResult<String> {
        let obj = attrs.as_object().ok_or_else(|| {
            SigniaError::invalid_argument("edge attrs must be an object")
        })?;

        let v = obj.get(key).ok_or_else(|| {
            SigniaError::invalid_argument(format!("missing edge attribute: {key}"))
        })?;

        let s = v.as_str().ok_or_else(|| {
            SigniaError::invalid_argument(format!("edge attribute {key} must be a string"))
        })?;

        Ok(s.to_string())
    }

    pub fn get_i64(attrs: &Value, key: &str) -> SigniaResult<i64> {
        let obj = attrs.as_object().ok_or_else(|| {
            SigniaError::invalid_argument("edge attrs must be an object")
        })?;

        let v = obj.get(key).ok_or_else(|| {
            SigniaError::invalid_argument(format!("missing edge attribute: {key}"))
        })?;

        v.as_i64().ok_or_else(|| {
            SigniaError::invalid_argument(format!("edge attribute {key} must be an integer"))
        })
    }
}

/// Builder for `EdgeV1` with deterministic defaults.
#[cfg(feature = "canonical-json")]
#[derive(Debug, Clone)]
pub struct EdgeBuilder {
    id: String,
    edge_type: String,
    from: String,
    to: String,
    attrs: BTreeMap<String, Value>,
}

#[cfg(feature = "canonical-json")]
impl EdgeBuilder {
    pub fn new(
        id: impl Into<String>,
        edge_type: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            edge_type: edge_type.into(),
            from: from.into(),
            to: to.into(),
            attrs: BTreeMap::new(),
        }
    }

    pub fn kind(mut self, kind: EdgeKind) -> Self {
        self.edge_type = kind.as_type_str().to_string();
        self
    }

    pub fn attr(mut self, k: impl Into<String>, v: Value) -> Self {
        self.attrs.insert(k.into(), v);
        self
    }

    pub fn build(self) -> EdgeV1 {
        let mut m = serde_json::Map::new();
        for (k, v) in self.attrs.into_iter() {
            m.insert(k, v);
        }

        EdgeV1 {
            id: self.id,
            r#type: self.edge_type,
            from: self.from,
            to: self.to,
            attrs: Value::Object(m),
        }
    }
}

/// Index edges by id and type.
#[cfg(feature = "canonical-json")]
#[derive(Debug, Clone)]
pub struct EdgeIndex<'a> {
    by_id: BTreeMap<&'a str, &'a EdgeV1>,
    by_type: BTreeMap<&'a str, Vec<&'a EdgeV1>>,
}

#[cfg(feature = "canonical-json")]
impl<'a> EdgeIndex<'a> {
    pub fn new(edges: &'a [EdgeV1]) -> Self {
        let mut by_id = BTreeMap::new();
        let mut by_type: BTreeMap<&'a str, Vec<&'a EdgeV1>> = BTreeMap::new();

        for e in edges {
            by_id.insert(e.id.as_str(), e);
            by_type.entry(e.r#type.as_str()).or_default().push(e);
        }

        // Deterministic ordering within each type by id.
        for v in by_type.values_mut() {
            v.sort_by(|a, b| a.id.cmp(&b.id));
        }

        Self { by_id, by_type }
    }

    pub fn get(&self, id: &str) -> Option<&'a EdgeV1> {
        self.by_id.get(id).copied()
    }

    pub fn list_by_type(&self, t: &str) -> Vec<&'a EdgeV1> {
        self.by_type.get(t).cloned().unwrap_or_default()
    }

    pub fn require(&self, id: &str) -> SigniaResult<&'a EdgeV1> {
        self.get(id).ok_or_else(|| SigniaError::invalid_argument(format!("missing edge id: {id}")))
    }
}

#[cfg(test)]
#[cfg(feature = "canonical-json")]
mod tests {
    use super::*;

    #[test]
    fn builder_builds_edge() {
        let e = EdgeBuilder::new("edge:x", "contains", "a", "b")
            .attr("ordinal", serde_json::Value::Number(1.into()))
            .build();

        assert_eq!(e.id, "edge:x");
        assert_eq!(e.from, "a");
        assert_eq!(e.to, "b");
    }

    #[test]
    fn index_by_type_sorted() {
        let e1 = EdgeV1 {
            id: "b".to_string(),
            r#type: "contains".to_string(),
            from: "x".to_string(),
            to: "y".to_string(),
            attrs: serde_json::json!({}),
        };
        let e2 = EdgeV1 {
            id: "a".to_string(),
            r#type: "contains".to_string(),
            from: "x".to_string(),
            to: "z".to_string(),
            attrs: serde_json::json!({}),
        };

        let idx = EdgeIndex::new(&[e1, e2]);
        let edges = idx.list_by_type("contains");
        assert_eq!(edges[0].id, "a");
        assert_eq!(edges[1].id, "b");
    }
}
