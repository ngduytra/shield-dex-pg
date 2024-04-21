use crate::errors::ErrorCode;
use crate::schema::platform_config::PlatformConfig;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdatePlatformConfig<'info> {
    /// Address to be set as protocol owner.
    #[account(
        mut,
        address = crate::admin::id() @ ErrorCode::Unauthorized
    )]
    pub owner: Signer<'info>,

    /// Initialize config state account to store protocol owner address and fee rates.
    #[account(mut)]
    pub platform_config: Account<'info, PlatformConfig>,
}

impl UpdatePlatformConfig<'_> {
    pub fn invoke(ctx: Context<UpdatePlatformConfig>, tax: u64) -> Result<()> {
        let platform_config = &mut ctx.accounts.platform_config;
        platform_config.tax = tax;

        Ok(())
    }
}
