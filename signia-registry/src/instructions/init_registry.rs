use anchor_lang::prelude::*;
use crate::constants::SEED_REGISTRY;
use crate::state::Registry;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitRegistryArgs {
    pub authority: Pubkey,
}

#[derive(Accounts)]
pub struct InitRegistry<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + Registry::SIZE,
        seeds = [SEED_REGISTRY],
        bump
    )]
    pub registry: Account<'info, Registry>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitRegistry>, args: InitRegistryArgs) -> Result<()> {
    let bump = *ctx.bumps.get("registry").unwrap();
    ctx.accounts.registry.init(bump, args.authority);
    Ok(())
}
