use crate::errors::ErrorCode;
use crate::schema::pool::*;
use anchor_lang::prelude::*;

#[event]
pub struct PauseEvent {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub updated_at: i64,
}

#[derive(Accounts)]
pub struct Pause<'info> {
    /// Authority
    pub authority: Signer<'info>,
    /// Pool
    #[account(mut, has_one = authority @ ErrorCode::Unauthorized)]
    pub pool: Account<'info, Pool>,
}

impl Pause<'_> {
    pub fn invoke(ctx: Context<Pause>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        if pool.state != PoolState::Initialized {
            return err!(ErrorCode::InvalidState);
        }

        pool.state = PoolState::Paused;
        pool.updated_at = Clock::get()?.unix_timestamp;

        emit!(PauseEvent {
            authority: ctx.accounts.authority.key(),
            pool: pool.key(),
            updated_at: pool.updated_at
        });

        Ok(())
    }
}
