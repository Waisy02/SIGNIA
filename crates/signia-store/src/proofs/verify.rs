//! Merkle proof verification.

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

use super::merkle::MerkleProof;

pub fn verify_proof(leaf_hex: &str, root: &[u8; 32], proof: &MerkleProof) -> Result<bool> {
    let mut cur = decode32(leaf_hex)?;
    for (is_left_sibling, sib) in &proof.path {
        let (left, right) = if *is_left_sibling { (sib, &cur) } else { (&cur, sib) };
        cur = hash_pair(left, right);
    }
    Ok(&cur == root)
}

fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(left);
    h.update(right);
    let out = h.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

fn decode32(hex_str: &str) -> Result<[u8; 32]> {
    if hex_str.len() != 64 {
        return Err(anyhow!("expected 32-byte hex digest (64 chars)"));
    }
    let bytes = hex::decode(hex_str)?;
    if bytes.len() != 32 {
        return Err(anyhow!("invalid digest length after decoding"));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}
