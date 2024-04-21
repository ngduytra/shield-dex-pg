use crate::constants::MAXIMUM_FEE;
use crate::errors::ErrorCode;
use crate::schema::platform_config::PlatformConfig;
use anchor_lang::prelude::*;

#[event]
pub struct UpdateTaxEvent {
    pub authority: Pubkey,
    pub new_tax: u64,
    pub platform_config: Pubkey,
    pub updated_at: i64,
}

#[derive(Accounts)]
pub struct UpdateTax<'info> {
    /// Authority
    pub authority: Signer<'info>,
    /// Pool
    #[account(mut, address = crate::admin::id() @ ErrorCode::Unauthorized)]
    pub platform_config: Account<'info, PlatformConfig>,
}

impl UpdateTax<'_> {
    pub fn invoke(ctx: Context<UpdateTax>, tax: u64) -> Result<()> {
        let platform_config = &mut ctx.accounts.platform_config;

        if tax > MAXIMUM_FEE {
            return err!(ErrorCode::InvalidParams);
        }

        platform_config.tax = tax;
        platform_config.updated_at = Clock::get()?.unix_timestamp;

        emit!(UpdateTaxEvent {
            authority: ctx.accounts.authority.key(),
            platform_config: platform_config.key(),
            new_tax: tax,
            updated_at: platform_config.updated_at
        });

        Ok(())
    }
}
