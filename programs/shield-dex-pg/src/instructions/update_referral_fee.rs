use crate::constants::MAXIMUM_FEE;
use crate::errors::ErrorCode;
use crate::schema::pool::*;
use anchor_lang::prelude::*;

#[event]
pub struct UpdateReferralFeeEvent {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub referral_fee: u64,
    pub updated_at: i64,
}

#[derive(Accounts)]
pub struct UpdateReferralFee<'info> {
    /// Authority
    pub authority: Signer<'info>,
    /// Pool
    #[account(mut, has_one =  authority @ ErrorCode::Unauthorized)]
    pub pool: Account<'info, Pool>,
}

impl UpdateReferralFee<'_> {
    pub fn invoke(ctx: Context<UpdateReferralFee>, referral_fee: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        if referral_fee > MAXIMUM_FEE {
            return err!(ErrorCode::InvalidParams);
        }

        pool.referral_fee = referral_fee;
        pool.updated_at = Clock::get()?.unix_timestamp;

        emit!(UpdateReferralFeeEvent {
            authority: ctx.accounts.authority.key(),
            pool: pool.key(),
            referral_fee,
            updated_at: pool.updated_at
        });

        Ok(())
    }
}
