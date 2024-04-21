use crate::errors::ErrorCode;
use crate::schema::platform_config::PlatformConfig;
use crate::schema::pool::Pool;
use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::token;
use anchor_spl::token::Token;

#[derive(Accounts)]
pub struct DistributeLpFee<'info> {
    /// Only admin or owner can collect fee now
    #[account(constraint = (owner.key() == pool.authority || owner.key() == crate::admin::id()) @ ErrorCode::Unauthorized)]
    pub owner: Signer<'info>,

    /// CHECK: The pool escrow
    #[account(seeds = ["escrow".as_bytes(), &pool.key().to_bytes()], bump)]
    pub escrow: AccountInfo<'info>,

    /// Pool state stores accumulated protocol fee amount
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Platform config account stores owner
    #[account(address = pool.tax)]
    pub platform_config: Account<'info, PlatformConfig>,

    /// Mint B
    pub mint_a: Box<Account<'info, token::Mint>>,
    /// Mint B
    pub mint_b: Box<Account<'info, token::Mint>>,

    /// The address that holds pool tokens for token_a
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow
      )]
    pub treasury_a: Box<Account<'info, token::TokenAccount>>,
    /// The address that holds pool tokens for token_b
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = escrow
      )]
    pub treasury_b: Box<Account<'info, token::TokenAccount>>,
    /// The address that receives the collected token_0 protocol fees
    #[account(mut)]
    pub recipient_token_a_account: Box<Account<'info, token::TokenAccount>>,

    /// The address that receives the collected token_1 protocol fees
    #[account(mut)]
    pub recipient_token_b_account: Box<Account<'info, token::TokenAccount>>,

    /// The SPL program to perform token transfers
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

impl DistributeLpFee<'_> {
    pub fn invoke(
        ctx: Context<DistributeLpFee>,
        amount_a_requested: u64,
        amount_b_requested: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let amount_a: u64;
        let amount_b: u64;
        let auth_bump: &[&[&[u8]]] = &[&[
            "escrow".as_ref(),
            &pool.key().to_bytes(),
            &[ctx.bumps.escrow],
        ]];
        {
            amount_a = amount_a_requested.min(pool.lp_fees_mint_a);
            amount_b = amount_b_requested.min(pool.lp_fees_mint_b);

            pool.lp_fees_mint_a = pool.lp_fees_mint_a.checked_sub(amount_a).unwrap();
            pool.lp_fees_mint_b = pool.lp_fees_mint_b.checked_sub(amount_b).unwrap();
        }

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.treasury_a.to_account_info(),
                    to: ctx.accounts.recipient_token_a_account.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                auth_bump,
            ),
            amount_a,
        )?;

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.treasury_b.to_account_info(),
                    to: ctx.accounts.recipient_token_b_account.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                auth_bump,
            ),
            amount_b,
        )?;

        Ok(())
    }
}
