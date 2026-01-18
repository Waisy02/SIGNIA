use anchor_lang::prelude::*;
use crate::errors::RegistryError;

/// Decode a 32-byte sha256 hash from lowercase hex.
pub fn decode_hash32(hex_str: &str) -> Result<[u8; 32]> {
    let s = hex_str.trim();
    if s.len() != 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(error!(RegistryError::InvalidSchemaHash));
    }
    let bytes = hex::decode(s).map_err(|_| error!(RegistryError::InvalidSchemaHash))?;
    if bytes.len() != 32 {
        return Err(error!(RegistryError::InvalidSchemaHash));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub fn encode_hash32(h: &[u8; 32]) -> String {
    hex::encode(h)
}
