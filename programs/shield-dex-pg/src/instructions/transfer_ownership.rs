use crate::errors::ErrorCode;
use crate::schema::pool::*;
use anchor_lang::prelude::*;

#[event]
pub struct TransferOwnershipEvent {
    pub authority: Pubkey,
    pub new_owner: Pubkey,
    pub pool: Pubkey,
    pub updated_at: i64,
}

#[derive(Accounts)]
pub struct TransferOwnership<'info> {
    /// Authority
    pub authority: Signer<'info>,
    /// Pool
    #[account(mut, has_one = authority @ ErrorCode::Unauthorized)]
    pub pool: Account<'info, Pool>,
}

impl TransferOwnership<'_> {
    pub fn invoke(ctx: Context<TransferOwnership>, new_owner: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        pool.authority = new_owner;
        pool.updated_at = Clock::get()?.unix_timestamp;

        emit!(TransferOwnershipEvent {
            authority: ctx.accounts.authority.key(),
            new_owner,
            pool: pool.key(),
            updated_at: pool.updated_at
        });

        Ok(())
    }
}
