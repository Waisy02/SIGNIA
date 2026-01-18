use anchor_lang::prelude::*;

/// Reserved for future: per-namespace authority configuration.
/// This is not used by the current instruction set.
#[account]
pub struct NamespaceAuthority {
    pub bump: u8,
    pub namespace: String,
    pub authority: Pubkey,
    pub frozen: bool,
}

impl NamespaceAuthority {
    pub fn space(namespace_len: usize) -> usize {
        1 + 4 + namespace_len + 32 + 1
    }
}
