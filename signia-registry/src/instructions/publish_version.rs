use anchor_lang::prelude::*;

use crate::constants::{SEED_ENTRY, SEED_REGISTRY};
use crate::errors::RegistryError;
use crate::state::{Entry, Registry};
use crate::utils::{decode_hash32, validate_namespace};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PublishVersionArgs {
    pub namespace: String,
    pub schema_hash_hex: String,
    /// If true, mark this entry as current version in its namespace.
    pub make_current: bool,
}

#[derive(Accounts)]
#[instruction(args: PublishVersionArgs)]
pub struct PublishVersion<'info> {
    pub authority: Signer<'info>,

    #[account(
        seeds = [SEED_REGISTRY],
        bump = registry.bump,
        constraint = registry.authority == authority.key() @ RegistryError::Unauthorized
    )]
    pub registry: Account<'info, Registry>,

    #[account(
        mut,
        seeds = [SEED_ENTRY, args.namespace.as_bytes(), decode_hash32(&args.schema_hash_hex)?.as_ref()],
        bump = entry.bump
    )]
    pub entry: Account<'info, Entry>,
}

pub fn handler(ctx: Context<PublishVersion>, args: PublishVersionArgs) -> Result<()> {
    validate_namespace(&args.namespace)?;
    let _ = decode_hash32(&args.schema_hash_hex)?;

    require!(!ctx.accounts.entry.revoked, RegistryError::EntryRevoked);

    ctx.accounts.entry.current = args.make_current;
    Ok(())
}
