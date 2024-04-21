use crate::constants::MAXIMUM_FEE;
use crate::errors::ErrorCode;
use crate::schema::pool::*;
use anchor_lang::prelude::*;

#[event]
pub struct UpdateLPFeeEvent {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub lp_fee: u64,
    pub updated_at: i64,
}

#[derive(Accounts)]
pub struct UpdateLPFee<'info> {
    /// Authority
    pub authority: Signer<'info>,
    /// Pool
    #[account(mut, has_one =  authority @ ErrorCode::Unauthorized)]
    pub pool: Account<'info, Pool>,
}

impl UpdateLPFee<'_> {
    pub fn invoke(ctx: Context<UpdateLPFee>, lp_fee: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        if lp_fee > MAXIMUM_FEE {
            return err!(ErrorCode::InvalidParams);
        }

        pool.lp_fee = lp_fee;
        pool.updated_at = Clock::get()?.unix_timestamp;

        emit!(UpdateLPFeeEvent {
            authority: ctx.accounts.authority.key(),
            pool: pool.key(),
            lp_fee,
            updated_at: pool.updated_at
        });

        Ok(())
    }
}
