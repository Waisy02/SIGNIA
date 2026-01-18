use anchor_lang::prelude::*;

use crate::constants::SEED_REGISTRY;
use crate::errors::RegistryError;
use crate::state::Registry;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferAuthorityArgs {
    pub new_authority: Pubkey,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_REGISTRY],
        bump = registry.bump,
        constraint = registry.authority == authority.key() @ RegistryError::Unauthorized
    )]
    pub registry: Account<'info, Registry>,
}

pub fn handler(ctx: Context<TransferAuthority>, args: TransferAuthorityArgs) -> Result<()> {
    ctx.accounts.registry.authority = args.new_authority;
    Ok(())
}
