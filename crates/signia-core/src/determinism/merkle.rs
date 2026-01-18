//! Merkle tree implementation for SIGNIA.
//!
//! This module provides a deterministic, auditable Merkle tree used for proofs.
//! Design goals:
//! - deterministic ordering
//! - explicit domain separation
//! - no hidden defaults
//! - reproducible roots across machines
//!
//! This implementation is append-only and intended for small to medium leaf sets
//! used in schema/manifest proofs.

use crate::errors::{SigniaError, SigniaResult};

use crate::determinism::hashing::{hash_merkle_leaf_hex, hash_merkle_node_hex};

/// Domain constants are defined in `crate::domain`.
#[derive(Debug, Clone)]
pub struct MerkleTreeOptions {
    pub hash_alg: String,
    pub domain_leaf: String,
    pub domain_node: String,
}

/// Deterministic Merkle tree.
///
/// Leaves are hashed in insertion order.
/// Internal nodes are built bottom-up with left/right concatenation.
#[derive(Debug, Clone)]
pub struct MerkleTree {
    opts: MerkleTreeOptions,
    leaves: Vec<String>, // hex-encoded leaf hashes
}

impl MerkleTree {
    /// Create a new empty Merkle tree.
    pub fn new(opts: MerkleTreeOptions) -> Self {
        Self {
            opts,
            leaves: Vec::new(),
        }
    }

    /// Number of leaves.
    pub fn leaf_count(&self) -> usize {
        self.leaves.len()
    }

    /// Push a raw leaf payload.
    ///
    /// The payload is hashed using:
    /// hash(domain_leaf || payload)
    pub fn push_leaf(&mut self, payload: &[u8]) -> SigniaResult<()> {
        let h = hash_merkle_leaf_hex(self.opts.hash_alg.as_str(), payload)?;
        self.leaves.push(h);
        Ok(())
    }

    /// Return the Merkle root as a lowercase hex string.
    pub fn root_hex(&self) -> SigniaResult<String> {
        if self.leaves.is_empty() {
            return Err(SigniaError::invalid_argument("cannot compute Merkle root of empty tree"));
        }

        let mut level = self.leaves.clone();

        while level.len() > 1 {
            let mut next = Vec::new();
            let mut i = 0;
            while i < level.len() {
                let left = &level[i];
                let right = if i + 1 < level.len() {
                    &level[i + 1]
                } else {
                    // Duplicate last hash if odd number of nodes
                    &level[i]
                };

                let parent = hash_merkle_node_hex(
                    self.opts.hash_alg.as_str(),
                    left,
                    right,
                )?;
                next.push(parent);
                i += 2;
            }
            level = next;
        }

        Ok(level[0].clone())
    }

    /// Return all leaf hashes (hex-encoded) in insertion order.
    pub fn leaf_hashes(&self) -> &[String] {
        &self.leaves
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merkle_single_leaf() {
        let mut t = MerkleTree::new(MerkleTreeOptions {
            hash_alg: "sha256".to_string(),
            domain_leaf: crate::domain::MERKLE_LEAF.to_string(),
            domain_node: crate::domain::MERKLE_NODE.to_string(),
        });
        t.push_leaf(b"hello").unwrap();
        let root = t.root_hex().unwrap();
        assert!(!root.is_empty());
    }

    #[test]
    fn merkle_two_leaves_deterministic() {
        let mut t1 = MerkleTree::new(MerkleTreeOptions {
            hash_alg: "sha256".to_string(),
            domain_leaf: crate::domain::MERKLE_LEAF.to_string(),
            domain_node: crate::domain::MERKLE_NODE.to_string(),
        });
        let mut t2 = t1.clone();

        t1.push_leaf(b"a").unwrap();
        t1.push_leaf(b"b").unwrap();

        t2.push_leaf(b"a").unwrap();
        t2.push_leaf(b"b").unwrap();

        assert_eq!(t1.root_hex().unwrap(), t2.root_hex().unwrap());
    }

    #[test]
    fn merkle_odd_leaves() {
        let mut t = MerkleTree::new(MerkleTreeOptions {
            hash_alg: "sha256".to_string(),
            domain_leaf: crate::domain::MERKLE_LEAF.to_string(),
            domain_node: crate::domain::MERKLE_NODE.to_string(),
        });
        t.push_leaf(b"a").unwrap();
        t.push_leaf(b"b").unwrap();
        t.push_leaf(b"c").unwrap();

        let root = t.root_hex().unwrap();
        assert!(!root.is_empty());
    }
}
