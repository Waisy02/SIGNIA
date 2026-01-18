//! Common strongly-typed helpers used by SIGNIA models and tooling.
//!
//! The wire model (`model::v1`) is intentionally permissive and JSON-friendly.
//! This module provides:
//! - validated newtypes for frequently used fields (ids, hashes, paths)
//! - parsing and formatting helpers
//! - deterministic constraints (lowercase hex, stable prefixes)
//!
//! These types are safe to use across compiler/CLI/API layers. They do not perform I/O.

use std::fmt;

use crate::errors::{SigniaError, SigniaResult};

/// A 32-byte digest represented as lowercase hex (64 chars).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HexDigest32(String);

impl HexDigest32 {
    pub fn new(hex64: impl Into<String>) -> SigniaResult<Self> {
        let s = hex64.into();
        validate_hex_digest32(&s)?;
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Debug for HexDigest32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("HexDigest32").field(&self.0).finish()
    }
}

impl fmt::Display for HexDigest32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A strongly typed entity id (e.g., `ent:file:abcd1234...`).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(String);

impl EntityId {
    pub fn new(id: impl Into<String>) -> SigniaResult<Self> {
        let s = id.into();
        validate_entity_id(&s)?;
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Debug for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EntityId").field(&self.0).finish()
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A strongly typed edge id (e.g., `edge:contains:abcd...`).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(String);

impl EdgeId {
    pub fn new(id: impl Into<String>) -> SigniaResult<Self> {
        let s = id.into();
        validate_edge_id(&s)?;
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Debug for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EdgeId").field(&self.0).finish()
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A stable, canonical leaf key used in Merkle proofs.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LeafKey(String);

impl LeafKey {
    pub fn new(key: impl Into<String>) -> SigniaResult<Self> {
        let s = key.into();
        validate_leaf_key(&s)?;
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for LeafKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("LeafKey").field(&self.0).finish()
    }
}

impl fmt::Display for LeafKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Validate a 32-byte hex digest (sha256 or blake3).
pub fn validate_hex_digest32(s: &str) -> SigniaResult<()> {
    if s.len() != 64 {
        return Err(SigniaError::invalid_argument(format!(
            "digest must be 64 lowercase hex chars, got length {}",
            s.len()
        )));
    }
    if !s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
        return Err(SigniaError::invalid_argument(
            "digest must be lowercase hex",
        ));
    }
    Ok(())
}

/// Validate entity id format.
/// We require at minimum: `ent:<type>:<suffix>` with no spaces.
pub fn validate_entity_id(s: &str) -> SigniaResult<()> {
    if s.trim() != s || s.contains(' ') {
        return Err(SigniaError::invalid_argument("entity id must not contain spaces"));
    }
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 3 || parts[0] != "ent" {
        return Err(SigniaError::invalid_argument(
            "entity id must have format ent:<type>:<suffix>",
        ));
    }
    if parts[1].is_empty() || parts[2].is_empty() {
        return Err(SigniaError::invalid_argument(
            "entity id must include a non-empty type and suffix",
        ));
    }
    Ok(())
}

/// Validate edge id format.
/// We require at minimum: `edge:<type>:<suffix>` with no spaces.
pub fn validate_edge_id(s: &str) -> SigniaResult<()> {
    if s.trim() != s || s.contains(' ') {
        return Err(SigniaError::invalid_argument("edge id must not contain spaces"));
    }
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 3 || parts[0] != "edge" {
        return Err(SigniaError::invalid_argument(
            "edge id must have format edge:<type>:<suffix>",
        ));
    }
    if parts[1].is_empty() || parts[2].is_empty() {
        return Err(SigniaError::invalid_argument(
            "edge id must include a non-empty type and suffix",
        ));
    }
    Ok(())
}

/// Validate canonical leaf keys.
/// Leaves should be short, stable, and include a domain prefix like:
/// - `file:schema.json`
/// - `digest:schemaHash`
/// - `meta:kind`
///
/// Rules:
/// - must contain a ':' separating prefix and value
/// - must not contain whitespace
/// - must be <= 1024 chars
pub fn validate_leaf_key(s: &str) -> SigniaResult<()> {
    if s.is_empty() {
        return Err(SigniaError::invalid_argument("leaf key must not be empty"));
    }
    if s.len() > 1024 {
        return Err(SigniaError::invalid_argument("leaf key too long"));
    }
    if s.chars().any(|c| c.is_whitespace()) {
        return Err(SigniaError::invalid_argument("leaf key must not contain whitespace"));
    }
    let mut it = s.splitn(2, ':');
    let prefix = it.next().unwrap_or("");
    let rest = it.next().unwrap_or("");
    if prefix.is_empty() || rest.is_empty() {
        return Err(SigniaError::invalid_argument(
            "leaf key must have format <prefix>:<value>",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_validation() {
        assert!(validate_hex_digest32(&"a".repeat(64)).is_ok());
        assert!(validate_hex_digest32(&"A".repeat(64)).is_err());
        assert!(validate_hex_digest32("abc").is_err());
    }

    #[test]
    fn entity_id_validation() {
        assert!(validate_entity_id("ent:file:abcd").is_ok());
        assert!(validate_entity_id("edge:file:abcd").is_err());
        assert!(validate_entity_id("ent::abcd").is_err());
    }

    #[test]
    fn edge_id_validation() {
        assert!(validate_edge_id("edge:contains:abcd").is_ok());
        assert!(validate_edge_id("ent:contains:abcd").is_err());
        assert!(validate_edge_id("edge::abcd").is_err());
    }

    #[test]
    fn leaf_key_validation() {
        assert!(validate_leaf_key("file:schema.json").is_ok());
        assert!(validate_leaf_key("file ").is_err());
        assert!(validate_leaf_key("file:").is_err());
        assert!(validate_leaf_key(":x").is_err());
    }

    #[test]
    fn newtypes_construct() {
        let d = HexDigest32::new("b".repeat(64)).unwrap();
        assert_eq!(d.as_str().len(), 64);

        let e = EntityId::new("ent:repo:root").unwrap();
        assert!(e.as_str().starts_with("ent:"));

        let ed = EdgeId::new("edge:contains:1").unwrap();
        assert!(ed.as_str().starts_with("edge:"));

        let k = LeafKey::new("digest:schemaHash").unwrap();
        assert!(k.as_str().contains(':'));
    }
}
