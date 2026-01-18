//! Determinism primitives for SIGNIA.
//!
//! This module defines the contract and helpers that guarantee deterministic,
//! reproducible outputs across machines, runs, and environments.
//!
//! Scope:
//! - canonical ordering rules
//! - stable serialization expectations
//! - deterministic hashing boundaries
//! - reproducible build constraints
//!
//! Non-scope:
//! - cryptographic implementations themselves (see `hash` module)
//! - execution sandboxes (see plugins/runtime)
//!
//! Determinism is a core invariant of SIGNIA: any structure compiled twice
//! with the same inputs MUST yield identical outputs.

use std::collections::{BTreeMap, BTreeSet};

use crate::errors::{SigniaError, SigniaResult};

#[cfg(feature = "canonical-json")]
use serde_json::{Map, Value};

/// Deterministic ordering helpers.
pub mod ordering {
    use super::*;

    /// Return a sorted vector of keys from a map.
    pub fn sorted_keys<V>(map: &BTreeMap<String, V>) -> Vec<String> {
        map.keys().cloned().collect()
    }

    /// Return a sorted vector of unique strings.
    pub fn sorted_set(set: &BTreeSet<String>) -> Vec<String> {
        set.iter().cloned().collect()
    }
}

/// Canonicalization helpers.
///
/// These functions normalize data into canonical forms before hashing
/// or serialization.
#[cfg(feature = "canonical-json")]
pub mod canonical {
    use super::*;

    /// Canonicalize a JSON value.
    ///
    /// Rules:
    /// - Objects: keys sorted lexicographically
    /// - Arrays: order preserved
    /// - Numbers: preserved as-is (caller must avoid floats if non-deterministic)
    /// - Strings, bool, null: preserved
    pub fn canonicalize_json(value: &Value) -> SigniaResult<Value> {
        match value {
            Value::Object(map) => {
                let mut out = Map::new();
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                for k in keys {
                    let v = map.get(k).unwrap();
                    out.insert(k.clone(), canonicalize_json(v)?);
                }
                Ok(Value::Object(out))
            }
            Value::Array(arr) => {
                let mut out = Vec::with_capacity(arr.len());
                for v in arr {
                    out.push(canonicalize_json(v)?);
                }
                Ok(Value::Array(out))
            }
            _ => Ok(value.clone()),
        }
    }
}

/// Deterministic string helpers.
pub mod strings {
    /// Normalize line endings to LF.
    pub fn normalize_newlines(s: &str) -> String {
        s.replace("\r\n", "\n").replace('\r', "\n")
    }

    /// Normalize UTF-8 input by trimming trailing whitespace lines.
    pub fn trim_trailing_whitespace(s: &str) -> String {
        s.lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Deterministic checks for structures.
///
/// These helpers are intended to be used in validation paths.
pub mod checks {
    use super::*;

    /// Ensure a string is non-empty and ASCII-safe.
    pub fn ensure_ascii_identifier(s: &str, field: &str) -> SigniaResult<()> {
        if s.is_empty() {
            return Err(SigniaError::invalid_argument(format!("{field} is empty")));
        }
        if !s.is_ascii() {
            return Err(SigniaError::invalid_argument(format!(
                "{field} must be ASCII"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "canonical-json")]
    fn canonicalize_json_sorts_keys() {
        let v = serde_json::json!({
            "b": 1,
            "a": { "d": 2, "c": 3 }
        });
        let c = canonical::canonicalize_json(&v).unwrap();
        let obj = c.as_object().unwrap();
        let keys: Vec<_> = obj.keys().cloned().collect();
        assert_eq!(keys, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn normalize_strings() {
        let s = "a\r\nb\r\n";
        let n = strings::normalize_newlines(s);
        assert_eq!(n, "a\nb\n");
    }
}
