use anchor_lang::prelude::*;

#[account]
pub struct Entry {
    pub bump: u8,
    pub namespace: String,   // normalized
    pub schema_hash: [u8; 32],
    pub kind: String,
    pub uri: String,
    pub version_tag: String,
    pub publisher: Pubkey,
    pub created_at: i64,
    pub revoked: bool,

    /// Optional pointer to the "current" published version.
    pub current: bool,
}

impl Entry {
    pub fn space(namespace_len: usize, kind_len: usize, uri_len: usize, ver_len: usize) -> usize {
        1
        + 4 + namespace_len
        + 32
        + 4 + kind_len
        + 4 + uri_len
        + 4 + ver_len
        + 32
        + 8
        + 1
        + 1
    }
}
