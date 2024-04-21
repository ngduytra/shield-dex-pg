use crate::{errors::ErrorCode, schema::platform_config::PlatformConfig};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreatePlatformConfig<'info> {
    /// Address to be set as protocol owner.
    #[account(
        mut,
        address = crate::admin::id() @ ErrorCode::Unauthorized
    )]
    pub owner: Signer<'info>,

    /// Initialize config state account to store protocol owner address and fee rates.
    #[account(
        init,
        payer = owner,
        space = PlatformConfig::LEN,
        
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    pub system_program: Program<'info, System>,
}

impl CreatePlatformConfig<'_> {
    pub fn invoke(ctx: Context<CreatePlatformConfig>, tax: u64) -> Result<()> {
        let platform_config = &mut ctx.accounts.platform_config;
        platform_config.tax = tax;
        platform_config.created_at = Clock::get()?.unix_timestamp;
        platform_config.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }
}
