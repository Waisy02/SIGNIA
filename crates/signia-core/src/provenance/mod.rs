//! Provenance tracking for SIGNIA.
//!
//! Provenance is the record of *how* an artifact was produced.
//! It is not the artifact's content; it is metadata that enables auditability.
//!
//! Provenance fields can be embedded in:
//! - IR nodes/edges (provenance strings and diagnostics)
//! - ManifestV1 (plugins, inputs, limits)
//! - Proof leaves (hashes of key metadata)
//!
//! This module provides:
//! - standard provenance record types
//! - canonical serialization for provenance events
//! - helpers to attach provenance in a deterministic way
//!
//! Determinism:
//! - provenance must be deterministic for the same build inputs
//! - timestamps must be injected (no system time reads)

use std::collections::BTreeMap;

use crate::errors::{SigniaError, SigniaResult};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

/// Standard provenance event kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvKind {
    Input,
    Transform,
    Plugin,
    Emit,
    Verify,
}

impl ProvKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProvKind::Input => "input",
            ProvKind::Transform => "transform",
            ProvKind::Plugin => "plugin",
            ProvKind::Emit => "emit",
            ProvKind::Verify => "verify",
        }
    }
}

/// A deterministic provenance record.
///
/// This record is intended to be serialized as canonical JSON when needed.
#[derive(Debug, Clone)]
pub struct ProvenanceRecord {
    /// Kind of provenance record.
    pub kind: ProvKind,

    /// Stable identifier for the producer (e.g., "signia-cli", "plugin:repo:v1").
    pub producer: String,

    /// Deterministic timestamp (ISO8601).
    pub at: String,

    /// Short message.
    pub message: String,

    /// Structured data fields.
    pub data: BTreeMap<String, String>,

    /// Optional JSON payload for richer structured records.
    #[cfg(feature = "canonical-json")]
    pub payload: Option<Value>,
}

impl ProvenanceRecord {
    pub fn new(kind: ProvKind, producer: impl Into<String>, at: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind,
            producer: producer.into(),
            at: at.into(),
            message: message.into(),
            data: BTreeMap::new(),
            #[cfg(feature = "canonical-json")]
            payload: None,
        }
    }

    pub fn with_kv(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    #[cfg(feature = "canonical-json")]
    pub fn with_payload(mut self, v: Value) -> Self {
        self.payload = Some(v);
        self
    }

    /// Convert to canonical JSON (keys sorted).
    #[cfg(feature = "canonical-json")]
    pub fn to_canonical_json(&self) -> SigniaResult<Value> {
        let mut obj = serde_json::Map::new();
        obj.insert("kind".to_string(), Value::String(self.kind.as_str().to_string()));
        obj.insert("producer".to_string(), Value::String(self.producer.clone()));
        obj.insert("at".to_string(), Value::String(self.at.clone()));
        obj.insert("message".to_string(), Value::String(self.message.clone()));

        let mut data_obj = serde_json::Map::new();
        for (k, v) in &self.data {
            data_obj.insert(k.clone(), Value::String(v.clone()));
        }
        obj.insert("data".to_string(), Value::Object(data_obj));

        if let Some(p) = &self.payload {
            obj.insert("payload".to_string(), p.clone());
        }

        let v = Value::Object(obj);
        crate::determinism::canonical_json::canonicalize(&v)
    }

    /// Hash this provenance record deterministically (sha256 of canonical JSON).
    #[cfg(feature = "canonical-json")]
    pub fn hash_hex(&self) -> SigniaResult<String> {
        let v = self.to_canonical_json()?;
        crate::hash::hash_canonical_json_hex(&v)
    }
}

/// A provenance chain.
///
/// Chains are ordered records; order must be deterministic.
#[derive(Debug, Clone, Default)]
pub struct ProvenanceChain {
    pub records: Vec<ProvenanceRecord>,
}

impl ProvenanceChain {
    pub fn push(&mut self, r: ProvenanceRecord) {
        self.records.push(r);
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Return a stable digest over the entire chain.
    ///
    /// Hashing strategy:
    /// - each record hashed independently
    /// - concatenated as "i:hash\n"
    /// - hashed again to produce chain digest
    #[cfg(feature = "canonical-json")]
    pub fn chain_hash_hex(&self) -> SigniaResult<String> {
        let mut buf = Vec::new();
        for (i, r) in self.records.iter().enumerate() {
            let h = r.hash_hex()?;
            buf.extend_from_slice(format!("{i}:{h}\n").as_bytes());
        }
        crate::hash::hash_bytes_hex(&buf)
    }
}

/// Attach a provenance string to an existing field (deterministically).
///
/// This helper provides a conventional "provenance:" prefix format.
pub fn format_provenance(kind: ProvKind, producer: &str) -> String {
    format!("provenance:{}:{}", kind.as_str(), producer)
}

/// Validate a producer identifier.
///
/// Producer identifiers must be ASCII and non-empty.
pub fn validate_producer_id(s: &str) -> SigniaResult<()> {
    if s.trim().is_empty() {
        return Err(SigniaError::invalid_argument("producer id is empty"));
    }
    if !s.is_ascii() {
        return Err(SigniaError::invalid_argument("producer id must be ASCII"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_producer() {
        validate_producer_id("signia-cli").unwrap();
        assert!(validate_producer_id("").is_err());
    }

    #[test]
    fn format_prov() {
        let s = format_provenance(ProvKind::Plugin, "repo:v1");
        assert!(s.starts_with("provenance:plugin:"));
    }

    #[test]
    #[cfg(feature = "canonical-json")]
    fn provenance_hash_stable() {
        let r = ProvenanceRecord::new(
            ProvKind::Emit,
            "signia-core",
            "1970-01-01T00:00:00Z",
            "emit schema",
        )
        .with_kv("kind", "repo")
        .with_kv("version", "v1");
        let h1 = r.hash_hex().unwrap();
        let h2 = r.hash_hex().unwrap();
        assert_eq!(h1, h2);

        let mut chain = ProvenanceChain::default();
        chain.push(r);
        let ch = chain.chain_hash_hex().unwrap();
        assert!(!ch.is_empty());
    }
}
