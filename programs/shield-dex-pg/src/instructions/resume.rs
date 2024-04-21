use crate::errors::ErrorCode;
use crate::schema::pool::*;
use anchor_lang::prelude::*;

#[event]
pub struct ResumeEvent {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub updated_at: i64,
}

#[derive(Accounts)]
pub struct Resume<'info> {
    /// Authority
    pub authority: Signer<'info>,
    /// Pool
    #[account(mut, has_one = authority @ ErrorCode::Unauthorized)]
    pub pool: Account<'info, Pool>,
}

impl Resume<'_> {
    pub fn invoke(ctx: Context<Resume>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        if pool.state != PoolState::Paused {
            return err!(ErrorCode::InvalidState);
        }

        pool.state = PoolState::Initialized;
        pool.updated_at = Clock::get()?.unix_timestamp;

        emit!(ResumeEvent {
            authority: ctx.accounts.authority.key(),
            pool: pool.key(),
            updated_at: pool.updated_at
        });

        Ok(())
    }
}
