use crate::schema::{pool::Pool, referer::Referrer};
use anchor_lang::prelude::*;

#[event]
pub struct CreateReferrerEvent {
    pub owner: Pubkey,
    // pub pool: Pubkey,
    pub referee: Pubkey,
}

#[derive(Accounts)]
pub struct CreateReferrer<'info> {
    /// Authority
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,
    // Referrer
    #[account(init, payer = authority,seeds = ["referrer".as_bytes(), &authority.key().to_bytes() ], space = Referrer::LEN, bump)]
    pub referrer: Account<'info, Referrer>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl CreateReferrer<'_> {
    pub fn invoke(ctx: Context<CreateReferrer>, referer_address: Pubkey) -> Result<()> {
        let referrer = &mut ctx.accounts.referrer;
        msg!(
            "referrer: system_program: {}",
            ctx.accounts.system_program.key()
        );

        referrer.owner = referer_address;
        // referrer.pool = ctx.accounts.pool.key();
        referrer.referee = ctx.accounts.authority.key();

        emit!(CreateReferrerEvent {
            owner: referer_address,
            // pool: ctx.accounts.pool.key(),
            referee: ctx.accounts.authority.key(),
        });

        Ok(())
    }
}
