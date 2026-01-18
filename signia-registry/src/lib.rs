use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("Signia1111111111111111111111111111111111111");

#[program]
pub mod signia_registry {
    use super::*;

    pub fn init_registry(ctx: Context<InitRegistry>, args: InitRegistryArgs) -> Result<()> {
        instructions::init_registry::handler(ctx, args)
    }

    pub fn register_schema(ctx: Context<RegisterSchema>, args: RegisterSchemaArgs) -> Result<()> {
        instructions::register_schema::handler(ctx, args)
    }

    pub fn publish_version(ctx: Context<PublishVersion>, args: PublishVersionArgs) -> Result<()> {
        instructions::publish_version::handler(ctx, args)
    }

    pub fn transfer_authority(ctx: Context<TransferAuthority>, args: TransferAuthorityArgs) -> Result<()> {
        instructions::transfer_authority::handler(ctx, args)
    }

    pub fn revoke_entry(ctx: Context<RevokeEntry>, args: RevokeEntryArgs) -> Result<()> {
        instructions::revoke_entry::handler(ctx, args)
    }
}
