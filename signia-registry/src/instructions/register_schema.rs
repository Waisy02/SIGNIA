use anchor_lang::prelude::*;

use crate::constants::{SEED_ENTRY, SEED_REGISTRY};
use crate::errors::RegistryError;
use crate::state::{Entry, Registry};
use crate::utils::{decode_hash32, validate_kind, validate_namespace, validate_uri, validate_version_tag};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RegisterSchemaArgs {
    /// Namespace MUST be pre-normalized by the client (lowercase + hyphens).
    pub namespace: String,
    /// 32-byte sha256 hex (lowercase recommended).
    pub schema_hash_hex: String,
    pub kind: String,
    pub uri: String,
    pub version_tag: String,
}

#[derive(Accounts)]
#[instruction(args: RegisterSchemaArgs)]
pub struct RegisterSchema<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Registry authority must sign for publication.
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_REGISTRY],
        bump = registry.bump,
        constraint = registry.authority == authority.key() @ RegistryError::Unauthorized
    )]
    pub registry: Account<'info, Registry>,

    #[account(
        init,
        payer = payer,
        space = 8 + Entry::space(
            args.namespace.len(),
            args.kind.len(),
            args.uri.len(),
            args.version_tag.len()
        ),
        seeds = [SEED_ENTRY, args.namespace.as_bytes(), decode_hash32(&args.schema_hash_hex)?.as_ref()],
        bump
    )]
    pub entry: Account<'info, Entry>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterSchema>, args: RegisterSchemaArgs) -> Result<()> {
    validate_namespace(&args.namespace)?;
    validate_kind(&args.kind)?;
    validate_uri(&args.uri)?;
    validate_version_tag(&args.version_tag)?;
    let schema_hash = decode_hash32(&args.schema_hash_hex)?;

    let bump = *ctx.bumps.get("entry").unwrap();
    let entry = &mut ctx.accounts.entry;

    entry.bump = bump;
    entry.namespace = args.namespace;
    entry.schema_hash = schema_hash;
    entry.kind = args.kind;
    entry.uri = args.uri;
    entry.version_tag = args.version_tag;
    entry.publisher = ctx.accounts.authority.key();
    entry.created_at = Clock::get()?.unix_timestamp;
    entry.revoked = false;
    entry.current = false;

    // advance counter
    ctx.accounts.registry.next_entry_id();

    Ok(())
}
