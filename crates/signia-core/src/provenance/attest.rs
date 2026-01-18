//! Attestation utilities for SIGNIA provenance.
//!
//! An attestation binds provenance, environment, and artifacts together
//! into a verifiable statement.
//!
//! Attestations are designed to be:
//! - deterministic
//! - serializable
//! - hashable
//! - suitable for on-chain anchoring or off-chain verification
//!
//! This module does NOT perform cryptographic signing.
//! Signature layers live outside of signia-core.

use std::collections::BTreeMap;

use crate::errors::{SigniaError, SigniaResult};
use crate::provenance::{BuildEnv, ProvenanceChain, SourceRef};

#[cfg(feature = "canonical-json")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

/// Attestation subject kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttestSubjectKind {
    Schema,
    Manifest,
    Proof,
    Custom,
}

impl AttestSubjectKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AttestSubjectKind::Schema => "schema",
            AttestSubjectKind::Manifest => "manifest",
            AttestSubjectKind::Proof => "proof",
            AttestSubjectKind::Custom => "custom",
        }
    }
}

/// Attestation record.
///
/// This structure is intentionally explicit and verbose.
#[cfg_attr(feature = "canonical-json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Attestation {
    /// Subject kind.
    pub subject_kind: String,

    /// Subject identifier (hash, name, or URI).
    pub subject_id: String,

    /// Source reference of the subject.
    pub source: Option<SourceRef>,

    /// Build environment used to produce the subject.
    pub build_env: Option<BuildEnv>,

    /// Provenance chain describing transformations.
    pub provenance: Option<ProvenanceChain>,

    /// Deterministic timestamp (ISO8601, injected).
    pub at: String,

    /// Additional explicit metadata.
    pub meta: BTreeMap<String, String>,

    /// Optional structured payload.
    #[cfg(feature = "canonical-json")]
    pub payload: Option<Value>,
}

impl Attestation {
    pub fn new(
        subject_kind: AttestSubjectKind,
        subject_id: impl Into<String>,
        at: impl Into<String>,
    ) -> Self {
        Self {
            subject_kind: subject_kind.as_str().to_string(),
            subject_id: subject_id.into(),
            source: None,
            build_env: None,
            provenance: None,
            at: at.into(),
            meta: BTreeMap::new(),
            #[cfg(feature = "canonical-json")]
            payload: None,
        }
    }

    pub fn with_source(mut self, source: SourceRef) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_build_env(mut self, env: BuildEnv) -> Self {
        self.build_env = Some(env);
        self
    }

    pub fn with_provenance(mut self, chain: ProvenanceChain) -> Self {
        self.provenance = Some(chain);
        self
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.meta.insert(key.into(), value.into());
        self
    }

    #[cfg(feature = "canonical-json")]
    pub fn with_payload(mut self, payload: Value) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Validate the attestation.
    pub fn validate(&self) -> SigniaResult<()> {
        if self.subject_kind.trim().is_empty() {
            return Err(SigniaError::invalid_argument(
                "attestation.subject_kind is empty",
            ));
        }
        if self.subject_id.trim().is_empty() {
            return Err(SigniaError::invalid_argument(
                "attestation.subject_id is empty",
            ));
        }
        if self.at.trim().is_empty() {
            return Err(SigniaError::invalid_argument(
                "attestation.at is empty",
            ));
        }

        if let Some(s) = &self.source {
            s.validate()?;
        }
        if let Some(b) = &self.build_env {
            b.validate()?;
        }

        Ok(())
    }

    /// Hash this attestation deterministically.
    #[cfg(feature = "canonical-json")]
    pub fn hash_hex(&self) -> SigniaResult<String> {
        let v = serde_json::to_value(self).map_err(|e| {
            SigniaError::serialization(format!("failed to serialize attestation: {e}"))
        })?;
        crate::hash::hash_canonical_json_hex(&v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::{ProvKind, ProvenanceRecord};

    #[test]
    fn attestation_validate_ok() {
        let a = Attestation::new(
            AttestSubjectKind::Schema,
            "schema:abc",
            "1970-01-01T00:00:00Z",
        );
        a.validate().unwrap();
    }

    #[test]
    #[cfg(feature = "canonical-json")]
    fn attestation_hash_stable() {
        let mut chain = ProvenanceChain::default();
        chain.push(
            ProvenanceRecord::new(
                ProvKind::Emit,
                "signia-core",
                "1970-01-01T00:00:00Z",
                "emit",
            )
        );

        let a = Attestation::new(
            AttestSubjectKind::Manifest,
            "manifest:xyz",
            "1970-01-01T00:00:00Z",
        )
        .with_provenance(chain);

        let h1 = a.hash_hex().unwrap();
        let h2 = a.hash_hex().unwrap();
        assert_eq!(h1, h2);
    }
}
