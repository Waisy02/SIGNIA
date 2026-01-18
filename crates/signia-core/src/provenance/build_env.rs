//! Build environment recording for SIGNIA provenance.
//!
//! This module defines a deterministic representation of the build environment.
//!
//! Important:
//! - Core MUST NOT read environment variables or system properties implicitly.
//! - Higher layers (CLI/API/CI) may choose what to record and inject it here.
//!
//! This module provides:
//! - `BuildEnv` (stable, serializable record)
//! - hashing helpers for embedding into proofs
//! - validation helpers to avoid leaking sensitive info
//!
//! Security & privacy:
//! - do not record usernames, home paths, machine names, IPs
//! - prefer toolchain versions and explicit config only

use std::collections::BTreeMap;

use crate::errors::{SigniaError, SigniaResult};

#[cfg(feature = "canonical-json")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

/// Deterministic build environment record.
#[cfg_attr(feature = "canonical-json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildEnv {
    /// Producer tool identifier, e.g. "signia-cli".
    pub producer: String,

    /// Producer version string, e.g. "0.1.0".
    pub producer_version: String,

    /// Rust toolchain (rustc) version, injected by caller.
    pub rustc: Option<String>,

    /// Solana toolchain version, injected by caller.
    pub solana: Option<String>,

    /// Anchor version, injected by caller.
    pub anchor: Option<String>,

    /// Node.js version, injected by caller.
    pub node: Option<String>,

    /// OS family string (linux/windows/macos) without kernel details.
    pub os_family: Option<String>,

    /// CPU arch string (x86_64/aarch64).
    pub arch: Option<String>,

    /// Build configuration (release/debug).
    pub profile: Option<String>,

    /// Reproducible build flag.
    pub reproducible: Option<bool>,

    /// Extra explicit fields (avoid secrets).
    pub extras: BTreeMap<String, String>,

    /// Optional JSON payload.
    #[cfg(feature = "canonical-json")]
    pub payload: Option<Value>,
}

impl BuildEnv {
    pub fn new(producer: impl Into<String>, producer_version: impl Into<String>) -> Self {
        Self {
            producer: producer.into(),
            producer_version: producer_version.into(),
            ..Default::default()
        }
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extras.insert(key.into(), value.into());
        self
    }

    #[cfg(feature = "canonical-json")]
    pub fn with_payload(mut self, payload: Value) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Validate that the record is safe and deterministic-friendly.
    pub fn validate(&self) -> SigniaResult<()> {
        if self.producer.trim().is_empty() {
            return Err(SigniaError::invalid_argument("buildEnv.producer is empty"));
        }
        if self.producer_version.trim().is_empty() {
            return Err(SigniaError::invalid_argument(
                "buildEnv.producerVersion is empty",
            ));
        }
        if !self.producer.is_ascii() || !self.producer_version.is_ascii() {
            return Err(SigniaError::invalid_argument(
                "buildEnv fields must be ASCII",
            ));
        }

        // Basic secret leakage guard.
        for (k, v) in &self.extras {
            let kl = k.to_ascii_lowercase();
            let vl = v.to_ascii_lowercase();

            let forbidden_keys = ["token", "secret", "password", "apikey", "api_key", "key"];
            for fk in forbidden_keys {
                if kl.contains(fk) {
                    return Err(SigniaError::invalid_argument(
                        "buildEnv.extras appears to contain a secret-like key; do not record secrets",
                    ));
                }
            }

            let forbidden_values = ["ssh-rsa", "-----begin", "private key"];
            for fv in forbidden_values {
                if vl.contains(fv) {
                    return Err(SigniaError::invalid_argument(
                        "buildEnv.extras appears to contain secret material; do not record secrets",
                    ));
                }
            }
        }

        Ok(())
    }

    /// Convert to a stable identifier string for display.
    pub fn display_id(&self) -> String {
        format!("{}@{}", self.producer, self.producer_version)
    }

    /// Hash this build environment deterministically.
    #[cfg(feature = "canonical-json")]
    pub fn hash_hex(&self) -> SigniaResult<String> {
        let v = serde_json::to_value(self).map_err(|e| {
            SigniaError::serialization(format!("failed to serialize buildEnv: {e}"))
        })?;
        crate::hash::hash_canonical_json_hex(&v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_env_validate_ok() {
        let b = BuildEnv::new("signia-cli", "0.1.0").with_extra("profile", "release");
        b.validate().unwrap();
    }

    #[test]
    fn build_env_rejects_secret_key() {
        let b = BuildEnv::new("signia-cli", "0.1.0").with_extra("apiKey", "123");
        assert!(b.validate().is_err());
    }

    #[test]
    #[cfg(feature = "canonical-json")]
    fn build_env_hash_stable() {
        let b = BuildEnv::new("signia-cli", "0.1.0").with_extra("profile", "release");
        let h1 = b.hash_hex().unwrap();
        let h2 = b.hash_hex().unwrap();
        assert_eq!(h1, h2);
    }
}
