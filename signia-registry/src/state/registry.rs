use anchor_lang::prelude::*;
use crate::constants::REGISTRY_LAYOUT_VERSION;

#[account]
pub struct Registry {
    pub layout_version: u16,
    pub bump: u8,
    pub authority: Pubkey,
    pub entry_counter: u64,
}

impl Registry {
    pub const SIZE: usize = 2 + 1 + 32 + 8;

    pub fn init(&mut self, bump: u8, authority: Pubkey) {
        self.layout_version = REGISTRY_LAYOUT_VERSION;
        self.bump = bump;
        self.authority = authority;
        self.entry_counter = 0;
    }

    pub fn next_entry_id(&mut self) -> u64 {
        self.entry_counter = self.entry_counter.saturating_add(1);
        self.entry_counter
    }
}
