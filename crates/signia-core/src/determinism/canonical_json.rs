//! Canonical JSON utilities for SIGNIA.
//!
//! This module defines strict canonical JSON rules used for hashing,
//! comparison, and reproducible builds.
//!
//! Canonical JSON rules enforced here:
//! - Object keys are sorted lexicographically
//! - Arrays preserve order
//! - Numbers are preserved exactly (callers must avoid non-deterministic floats)
//! - Strings are preserved as UTF-8
//! - No implicit defaults are inserted
//!
//! These helpers are intentionally minimal and deterministic.

use crate::errors::{SigniaError, SigniaResult};

use serde_json::{Map, Value};

/// Canonicalize a JSON value recursively.
///
/// This function produces a new `Value` where:
/// - All objects have keys sorted
/// - All nested objects are also canonicalized
///
/// This function does not modify arrays order.
pub fn canonicalize(value: &Value) -> SigniaResult<Value> {
    match value {
        Value::Object(map) => canonicalize_object(map),
        Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for v in arr {
                out.push(canonicalize(v)?);
            }
            Ok(Value::Array(out))
        }
        _ => Ok(value.clone()),
    }
}

fn canonicalize_object(map: &Map<String, Value>) -> SigniaResult<Value> {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();

    let mut out = Map::new();
    for k in keys {
        let v = map.get(k).ok_or_else(|| {
            SigniaError::invariant("key disappeared during canonicalization")
        })?;
        out.insert(k.clone(), canonicalize(v)?);
    }

    Ok(Value::Object(out))
}

/// Convert a JSON value into a canonical UTF-8 byte representation.
///
/// This representation is stable across machines and runs.
pub fn to_canonical_bytes(value: &Value) -> SigniaResult<Vec<u8>> {
    let canonical = canonicalize(value)?;
    serde_json::to_vec(&canonical)
        .map_err(|e| SigniaError::serialization(format!("failed to serialize canonical JSON: {e}")))
}

/// Compare two JSON values for canonical equality.
///
/// Returns true if their canonical forms are byte-equal.
pub fn canonical_eq(a: &Value, b: &Value) -> SigniaResult<bool> {
    let ba = to_canonical_bytes(a)?;
    let bb = to_canonical_bytes(b)?;
    Ok(ba == bb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalize_sorts_keys() {
        let v = serde_json::json!({
            "b": 1,
            "a": {
                "d": 2,
                "c": 3
            }
        });

        let c = canonicalize(&v).unwrap();
        let obj = c.as_object().unwrap();
        let keys: Vec<_> = obj.keys().cloned().collect();
        assert_eq!(keys, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn canonical_eq_true_for_different_order() {
        let a = serde_json::json!({"a":1,"b":2});
        let b = serde_json::json!({"b":2,"a":1});
        assert!(canonical_eq(&a, &b).unwrap());
    }

    #[test]
    fn canonical_eq_false_for_different_values() {
        let a = serde_json::json!({"a":1});
        let b = serde_json::json!({"a":2});
        assert!(!canonical_eq(&a, &b).unwrap());
    }
}
