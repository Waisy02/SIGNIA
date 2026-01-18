//! SIGNIA Proof v1 model.
//!
//! Proofs provide verifiable bindings between:
//! - schema bytes (canonical)
//! - manifest bytes (canonical)
//! - computed digests (schemaHash, manifestHash)
//! - Merkle roots over declared leaves
//!
//! The proof format is designed to support:
//! - full-bundle verification
//! - inclusion proofs for partial verification (e.g., verifying a single file leaf)
//! - offline verification
//!
//! This is a wire-level model. Do not introduce breaking changes for v1.

#[cfg(feature = "canonical-json")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

/// A SIGNIA proof instance.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct ProofV1 {
    /// Proof version. Must be "v1".
    pub version: String,

    /// Hash algorithm used for leaves and nodes (e.g. sha256, blake3).
    pub hash_alg: String,

    /// Merkle root for the proof set.
    pub root: String,

    /// Leaf entries used to construct the Merkle tree.
    pub leaves: Vec<LeafV1>,

    /// Optional inclusion proofs (keyed by leaf key).
    #[cfg_attr(feature = "canonical-json", serde(default, skip_serializing_if = "Option::is_none"))]
    pub inclusions: Option<Vec<InclusionProofV1>>,

    /// Optional extra metadata for tooling (must be deterministic if present).
    #[cfg_attr(feature = "canonical-json", serde(default, skip_serializing_if = "Option::is_none"))]
    pub meta: Option<Value>,
}

/// A leaf entry in a proof set.
///
/// Leaf value is typically a digest or canonical bytes digest of a specific component:
/// - schemaHash
/// - manifestHash
/// - file:README.md hash
/// - meta field hash (optional)
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct LeafV1 {
    /// Canonical leaf key (e.g. "digest:schemaHash" or "file:src/lib.rs").
    pub key: String,

    /// Value for the leaf. Usually a lowercase hex digest.
    pub value: String,
}

/// Inclusion proof for a specific leaf.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct InclusionProofV1 {
    /// Leaf key this proof is for.
    pub key: String,

    /// Leaf value this proof is for.
    pub value: String,

    /// Sibling hashes (hex) on the path from leaf to root.
    pub siblings: Vec<SiblingV1>,
}

/// One Merkle sibling entry.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct SiblingV1 {
    /// "left" or "right" indicates where the sibling hash is placed relative to the running hash.
    pub side: String,
    /// Sibling node hash.
    pub hash: String,
}

impl ProofV1 {
    pub fn new(hash_alg: impl Into<String>, root: impl Into<String>) -> Self {
        Self {
            version: "v1".to_string(),
            hash_alg: hash_alg.into(),
            root: root.into(),
            leaves: Vec::new(),
            inclusions: None,
            meta: None,
        }
    }

    pub fn push_leaf(&mut self, leaf: LeafV1) {
        self.leaves.push(leaf);
    }

    pub fn set_inclusions(&mut self, inc: Vec<InclusionProofV1>) {
        self.inclusions = Some(inc);
    }
}

#[cfg(test)]
#[cfg(feature = "canonical-json")]
mod tests {
    use super::*;

    #[test]
    fn proof_roundtrip() {
        let mut p = ProofV1::new("sha256", "a".repeat(64));
        p.push_leaf(LeafV1 {
            key: "digest:schemaHash".to_string(),
            value: "b".repeat(64),
        });

        p.set_inclusions(vec![InclusionProofV1 {
            key: "digest:schemaHash".to_string(),
            value: "b".repeat(64),
            siblings: vec![SiblingV1 {
                side: "left".to_string(),
                hash: "c".repeat(64),
            }],
        }]);

        let s = serde_json::to_string(&p).unwrap();
        let back: ProofV1 = serde_json::from_str(&s).unwrap();
        assert_eq!(back.version, "v1");
        assert_eq!(back.hash_alg, "sha256");
        assert_eq!(back.leaves.len(), 1);
        assert!(back.inclusions.is_some());
    }
}
