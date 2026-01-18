//! Metadata helpers for SIGNIA.
//!
//! SIGNIA artifacts include metadata in multiple places:
//! - `SchemaV1.meta` (required)
//! - `ManifestV1` fields (`name`, optional `description`, optional `labels`)
//! - `ProofV1.meta` (optional)
//!
//! The wire model keeps some sections as untyped JSON to preserve forward compatibility.
//! This module provides:
//! - typed builders for common meta sections
//! - validation helpers for required keys
//! - deterministic constraints (no host timestamps unless explicitly set, stable defaults)
//!
//! This module performs no I/O.

use std::collections::BTreeMap;

use crate::errors::{SigniaError, SigniaResult};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

/// A strongly typed view of `schema.meta` used by compiler and tooling.
///
/// This is intentionally strict and only includes the most important core fields.
/// Additional fields may be present in the JSON but are ignored by this struct unless
/// explicitly parsed.
#[derive(Debug, Clone)]
pub struct SchemaMeta {
    pub name: String,
    pub description: Option<String>,
    pub labels: BTreeMap<String, String>,
    pub created_at: String,
    pub source: SourceRef,
    pub normalization: NormalizationPolicy,
}

/// Source reference for compilation.
#[derive(Debug, Clone)]
pub struct SourceRef {
    pub r#type: String,
    pub locator: String,
    pub content_hash: Option<String>,
}

/// Normalization policy captured at compile time.
#[derive(Debug, Clone)]
pub struct NormalizationPolicy {
    pub policy_version: String,
    pub path_root: String,
    pub newline: String,
    pub encoding: String,
    pub symlinks: String,
    pub network: String,
}

impl Default for NormalizationPolicy {
    fn default() -> Self {
        Self {
            policy_version: "v1".to_string(),
            path_root: "artifact:/".to_string(),
            newline: "lf".to_string(),
            encoding: "utf-8".to_string(),
            symlinks: "deny".to_string(),
            network: "deny".to_string(),
        }
    }
}

impl SchemaMeta {
    /// Convert this typed meta into JSON suitable for `SchemaV1.meta`.
    #[cfg(feature = "canonical-json")]
    pub fn to_json(&self) -> Value {
        let mut m = serde_json::Map::new();
        m.insert("name".to_string(), Value::String(self.name.clone()));
        if let Some(d) = &self.description {
            m.insert("description".to_string(), Value::String(d.clone()));
        }
        if !self.labels.is_empty() {
            let mut lm = serde_json::Map::new();
            for (k, v) in self.labels.iter() {
                lm.insert(k.clone(), Value::String(v.clone()));
            }
            m.insert("labels".to_string(), Value::Object(lm));
        }
        m.insert("createdAt".to_string(), Value::String(self.created_at.clone()));
        m.insert(
            "source".to_string(),
            serde_json::json!({
                "type": self.source.r#type,
                "locator": self.source.locator,
                "contentHash": self.source.content_hash
            }),
        );
        m.insert(
            "normalization".to_string(),
            serde_json::json!({
                "policyVersion": self.normalization.policy_version,
                "pathRoot": self.normalization.path_root,
                "newline": self.normalization.newline,
                "encoding": self.normalization.encoding,
                "symlinks": self.normalization.symlinks,
                "network": self.normalization.network
            }),
        );
        Value::Object(m)
    }

    /// Parse required fields from a JSON `schema.meta` object.
    #[cfg(feature = "canonical-json")]
    pub fn from_json(v: &Value) -> SigniaResult<Self> {
        let obj = v.as_object().ok_or_else(|| SigniaError::invalid_argument("schema.meta must be an object"))?;

        let name = obj
            .get("name")
            .and_then(|x| x.as_str())
            .ok_or_else(|| SigniaError::invalid_argument("schema.meta.name must be a string"))?
            .to_string();

        let description = obj.get("description").and_then(|x| x.as_str()).map(|s| s.to_string());

        let created_at = obj
            .get("createdAt")
            .and_then(|x| x.as_str())
            .ok_or_else(|| SigniaError::invalid_argument("schema.meta.createdAt must be a string"))?
            .to_string();

        let source_obj = obj
            .get("source")
            .and_then(|x| x.as_object())
            .ok_or_else(|| SigniaError::invalid_argument("schema.meta.source must be an object"))?;

        let source_type = source_obj
            .get("type")
            .and_then(|x| x.as_str())
            .ok_or_else(|| SigniaError::invalid_argument("schema.meta.source.type must be a string"))?
            .to_string();

        let source_locator = source_obj
            .get("locator")
            .and_then(|x| x.as_str())
            .ok_or_else(|| SigniaError::invalid_argument("schema.meta.source.locator must be a string"))?
            .to_string();

        let source_content_hash = source_obj.get("contentHash").and_then(|x| x.as_str()).map(|s| s.to_string());

        let norm_obj = obj
            .get("normalization")
            .and_then(|x| x.as_object())
            .ok_or_else(|| SigniaError::invalid_argument("schema.meta.normalization must be an object"))?;

        let normalization = NormalizationPolicy {
            policy_version: norm_obj.get("policyVersion").and_then(|x| x.as_str()).unwrap_or("v1").to_string(),
            path_root: norm_obj.get("pathRoot").and_then(|x| x.as_str()).unwrap_or("artifact:/").to_string(),
            newline: norm_obj.get("newline").and_then(|x| x.as_str()).unwrap_or("lf").to_string(),
            encoding: norm_obj.get("encoding").and_then(|x| x.as_str()).unwrap_or("utf-8").to_string(),
            symlinks: norm_obj.get("symlinks").and_then(|x| x.as_str()).unwrap_or("deny").to_string(),
            network: norm_obj.get("network").and_then(|x| x.as_str()).unwrap_or("deny").to_string(),
        };

        let mut labels = BTreeMap::new();
        if let Some(lv) = obj.get("labels").and_then(|x| x.as_object()) {
            for (k, v) in lv.iter() {
                if let Some(s) = v.as_str() {
                    labels.insert(k.clone(), s.to_string());
                }
            }
        }

        Ok(Self {
            name,
            description,
            labels,
            created_at,
            source: SourceRef {
                r#type: source_type,
                locator: source_locator,
                content_hash: source_content_hash,
            },
            normalization,
        })
    }
}

/// Builder for `SchemaMeta`.
#[derive(Debug, Clone)]
pub struct SchemaMetaBuilder {
    name: String,
    description: Option<String>,
    labels: BTreeMap<String, String>,
    created_at: String,
    source: SourceRef,
    normalization: NormalizationPolicy,
}

impl SchemaMetaBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            labels: BTreeMap::new(),
            created_at: "1970-01-01T00:00:00Z".to_string(),
            source: SourceRef {
                r#type: "path".to_string(),
                locator: "artifact:/".to_string(),
                content_hash: None,
            },
            normalization: NormalizationPolicy::default(),
        }
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    pub fn label(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.labels.insert(k.into(), v.into());
        self
    }

    pub fn created_at(mut self, iso8601: impl Into<String>) -> Self {
        self.created_at = iso8601.into();
        self
    }

    pub fn source(mut self, r#type: impl Into<String>, locator: impl Into<String>) -> Self {
        self.source.r#type = r#type.into();
        self.source.locator = locator.into();
        self
    }

    pub fn source_content_hash(mut self, digest_hex: impl Into<String>) -> Self {
        self.source.content_hash = Some(digest_hex.into());
        self
    }

    pub fn normalization(mut self, n: NormalizationPolicy) -> Self {
        self.normalization = n;
        self
    }

    pub fn build(self) -> SchemaMeta {
        SchemaMeta {
            name: self.name,
            description: self.description,
            labels: self.labels,
            created_at: self.created_at,
            source: self.source,
            normalization: self.normalization,
        }
    }
}

/// Validate a meta JSON object for required fields.
///
/// This is a convenience wrapper around `SchemaMeta::from_json`.
#[cfg(feature = "canonical-json")]
pub fn validate_schema_meta_json(v: &Value) -> SigniaResult<()> {
    let _ = SchemaMeta::from_json(v)?;
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "canonical-json")]
mod tests {
    use super::*;

    #[test]
    fn builder_to_json_has_required_fields() {
        let meta = SchemaMetaBuilder::new("demo")
            .created_at("1970-01-01T00:00:00Z")
            .source("path", "artifact:/demo")
            .label("category", "repo")
            .build();

        let j = meta.to_json();
        validate_schema_meta_json(&j).unwrap();

        assert_eq!(j["name"], "demo");
        assert_eq!(j["source"]["locator"], "artifact:/demo");
    }

    #[test]
    fn from_json_parses() {
        let v = serde_json::json!({
            "name": "demo",
            "createdAt": "1970-01-01T00:00:00Z",
            "source": { "type": "path", "locator": "artifact:/demo" },
            "normalization": { "policyVersion": "v1", "pathRoot": "artifact:/", "newline": "lf", "encoding": "utf-8", "symlinks": "deny", "network": "deny" }
        });

        let m = SchemaMeta::from_json(&v).unwrap();
        assert_eq!(m.name, "demo");
        assert_eq!(m.source.locator, "artifact:/demo");
        assert_eq!(m.normalization.newline, "lf");
    }
}
