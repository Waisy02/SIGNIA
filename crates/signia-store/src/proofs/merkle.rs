//! Merkle tree implementation (SHA-256).

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof {
    pub index: usize,
    pub path: Vec<(bool, [u8; 32])>,
}

pub fn merkle_root_hex(leaves_hex: &[String]) -> Result<String> {
    Ok(hex::encode(merkle_root(leaves_hex)?))
}

pub fn merkle_root(leaves_hex: &[String]) -> Result<[u8; 32]> {
    if leaves_hex.is_empty() {
        return Err(anyhow!("cannot build Merkle root for empty leaves"));
    }
    let mut level: Vec<[u8; 32]> = leaves_hex.iter().map(|h| decode32(h)).collect::<Result<_>>()?;
    while level.len() > 1 {
        level = parent_level(&level);
    }
    Ok(level[0])
}

pub fn merkle_proof(leaves_hex: &[String], index: usize) -> Result<MerkleProof> {
    if leaves_hex.is_empty() {
        return Err(anyhow!("cannot build proof for empty leaves"));
    }
    if index >= leaves_hex.len() {
        return Err(anyhow!("leaf index out of range"));
    }

    let mut idx = index;
    let mut level: Vec<[u8; 32]> = leaves_hex.iter().map(|h| decode32(h)).collect::<Result<_>>()?;
    let mut path: Vec<(bool, [u8; 32])> = Vec::new();

    while level.len() > 1 {
        let is_right = idx % 2 == 1;
        let sibling_idx = if is_right { idx - 1 } else { idx + 1 };

        let sib = if sibling_idx < level.len() { level[sibling_idx] } else { level[idx] };
        let is_left_sibling = is_right;
        path.push((is_left_sibling, sib));

        level = parent_level(&level);
        idx /= 2;
    }

    Ok(MerkleProof { index, path })
}

fn parent_level(children: &[[u8; 32]]) -> Vec<[u8; 32]> {
    let mut out = Vec::with_capacity((children.len() + 1) / 2);
    let mut i = 0usize;
    while i < children.len() {
        let left = children[i];
        let right = if i + 1 < children.len() { children[i + 1] } else { children[i] };
        out.push(hash_pair(&left, &right));
        i += 2;
    }
    out
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
