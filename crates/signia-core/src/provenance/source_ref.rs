//! Source reference types for SIGNIA provenance.
//!
//! A SourceRef identifies *where* a structure came from.
//! It does not fetch the source; it only records a deterministic reference.
//!
//! Examples:
//! - git repository + commit + subpath
//! - local/virtual path with declared root
//! - URL with content digest
//! - dataset identifier (e.g., huggingface dataset + revision)
//!
//! SourceRef is designed to be serializable and suitable for embedding in:
//! - Schema meta.source
//! - Manifest inputs
//! - Provenance payloads

use std::collections::BTreeMap;

use crate::errors::{SigniaError, SigniaResult};

#[cfg(feature = "canonical-json")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

/// Canonical source reference.
#[cfg_attr(feature = "canonical-json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRef {
    /// Source type (git, path, url, dataset, openapi, workflow, custom).
    pub r#type: String,

    /// Locator string. Must be deterministic (no machine-local temp paths).
    ///
    /// Examples:
    /// - git:https://github.com/org/repo.git#<commit>:<subpath>
    /// - path:artifact:/repo
    /// - url:https://example.com/spec.yaml
    /// - hf:dataset:org/name@revision
    pub locator: String,

    /// Optional content digest (hex).
    pub digest: Option<String>,

    /// Optional revision string (commit, tag, version).
    pub revision: Option<String>,

    /// Optional subpath inside a container source (git repo subdir, archive path).
    pub subpath: Option<String>,

    /// Optional extra fields.
    pub extras: BTreeMap<String, String>,

    /// Optional JSON payload for richer structured metadata.
    #[cfg(feature = "canonical-json")]
    pub payload: Option<Value>,
}

impl SourceRef {
    /// Create a new source ref with minimal fields.
    pub fn new(r#type: impl Into<String>, locator: impl Into<String>) -> Self {
        Self {
            r#type: r#type.into(),
            locator: locator.into(),
            digest: None,
            revision: None,
            subpath: None,
            extras: BTreeMap::new(),
            #[cfg(feature = "canonical-json")]
            payload: None,
        }
    }

    pub fn with_digest(mut self, digest: impl Into<String>) -> Self {
        self.digest = Some(digest.into());
        self
    }

    pub fn with_revision(mut self, revision: impl Into<String>) -> Self {
        self.revision = Some(revision.into());
        self
    }

    pub fn with_subpath(mut self, subpath: impl Into<String>) -> Self {
        self.subpath = Some(subpath.into());
        self
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

    /// Validate determinism constraints.
    ///
    /// - type and locator must be ASCII and non-empty
    /// - locator must not contain obviously machine-local temp patterns
    pub fn validate(&self) -> SigniaResult<()> {
        if self.r#type.trim().is_empty() {
            return Err(SigniaError::invalid_argument("sourceRef.type is empty"));
        }
        if self.locator.trim().is_empty() {
            return Err(SigniaError::invalid_argument("sourceRef.locator is empty"));
        }
        if !self.r#type.is_ascii() || !self.locator.is_ascii() {
            return Err(SigniaError::invalid_argument(
                "sourceRef.type and locator must be ASCII",
            ));
        }

        // Guard against temp paths and OS user paths.
        let l = self.locator.to_ascii_lowercase();
        let forbidden = [
            "c:\\users\\",
            "/users/",
            "/home/",
            "/var/folders/",
            "/tmp/",
            "appdata\\local\\temp",
        ];
        for f in forbidden {
            if l.contains(f) {
                return Err(SigniaError::invalid_argument(
                    "sourceRef.locator appears to contain a machine-local path; use a virtual artifact root",
                ));
            }
        }

        Ok(())
    }

    /// Convert to a stable "display id" string.
    ///
    /// This is not a cryptographic identifier; it is a human-friendly normalized label.
    pub fn display_id(&self) -> String {
        if let Some(r) = &self.revision {
            if let Some(s) = &self.subpath {
                return format!("{}@{}:{}", self.locator, r, s);
            }
            return format!("{}@{}", self.locator, r);
        }
        self.locator.clone()
    }

    /// Hash this source ref deterministically.
    #[cfg(feature = "canonical-json")]
    pub fn hash_hex(&self) -> SigniaResult<String> {
        let v = serde_json::to_value(self)
            .map_err(|e| SigniaError::serialization(format!("failed to serialize sourceRef: {e}")))?;
        crate::hash::hash_canonical_json_hex(&v)
    }
}

/// A convenience wrapper representing a Git source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitSource {
    pub remote: String,
    pub commit: String,
    pub subpath: Option<String>,
}

impl GitSource {
    pub fn to_source_ref(&self) -> SourceRef {
        let mut locator = format!("git:{}#{}", self.remote, self.commit);
        if let Some(s) = &self.subpath {
            locator.push(':');
            locator.push_str(s);
        }
        SourceRef::new("git", locator).with_revision(self.commit.clone())
    }
}

/// A convenience wrapper representing a URL source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlSource {
    pub url: String,
    pub digest: Option<String>,
}

impl UrlSource {
    pub fn to_source_ref(&self) -> SourceRef {
        let mut sr = SourceRef::new("url", self.url.clone());
        if let Some(d) = &self.digest {
            sr = sr.with_digest(d.clone());
        }
        sr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_ref_validate() {
        let sr = SourceRef::new("git", "git:https://github.com/x/y.git#abc");
        sr.validate().unwrap();
    }

    #[test]
    fn source_ref_rejects_local_paths() {
        let sr = SourceRef::new("path", "path:/Users/alice/tmp");
        assert!(sr.validate().is_err());
    }

    #[test]
    fn git_source_to_ref() {
        let gs = GitSource {
            remote: "https://github.com/x/y.git".to_string(),
            commit: "abc".to_string(),
            subpath: Some("src".to_string()),
        };
        let sr = gs.to_source_ref();
        assert!(sr.locator.contains("git:https://github.com/x/y.git#abc:src"));
    }
}
