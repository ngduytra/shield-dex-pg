use crate::{errors::ErrorCode, schema::pool::Pool};
use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token};
use num::ToPrimitive;

#[event]
pub struct RemoveLiquidityEvent {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub a: u64,
    pub b: u64,
    pub lp: u64,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    /// Authority
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Pool
    #[account(
    has_one = mint_a @ ErrorCode::UnmatchPool,
    has_one = mint_b @ ErrorCode::UnmatchPool,
    has_one = lp_mint @ ErrorCode::UnmatchPool
  )]
    pub pool: Account<'info, Pool>,
    /// Mint A
    pub mint_a: Box<Account<'info, token::Mint>>,
    #[account(
    mut,
    associated_token::mint = mint_a,
    associated_token::authority = escrow
  )]
    pub treasury_a: Box<Account<'info, token::TokenAccount>>,
    #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = mint_a,
    associated_token::authority = authority
  )]
    pub dst_a: Box<Account<'info, token::TokenAccount>>,
    /// Mint B
    pub mint_b: Box<Account<'info, token::Mint>>,
    #[account(
    mut,
    associated_token::mint = mint_b,
    associated_token::authority = escrow
  )]
    pub treasury_b: Box<Account<'info, token::TokenAccount>>,
    #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = mint_b,
    associated_token::authority = authority
  )]
    pub dst_b: Box<Account<'info, token::TokenAccount>>,
    // LP Mint
    #[account(mut)]
    pub lp_mint: Box<Account<'info, token::Mint>>,
    #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = lp_mint,
    associated_token::authority = authority
  )]
    pub src_lp: Box<Account<'info, token::TokenAccount>>,
    /// CHECK: The pool escrow
    #[account(seeds = ["escrow".as_bytes(), &pool.key().to_bytes()], bump)]
    pub escrow: AccountInfo<'info>,
    /// System programs
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl RemoveLiquidity<'_> {
    pub fn invoke(ctx: Context<RemoveLiquidity>, lp: u64) -> Result<()> {
        let pool = &ctx.accounts.pool;
        let seeds: &[&[&[u8]]] = &[&[
            "escrow".as_ref(),
            &pool.key().to_bytes(),
            &[ctx.bumps.escrow],
        ]];

        if lp <= 0 {
            return err!(ErrorCode::InvalidParams);
        }

        // Burn LP tokens
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    from: ctx.accounts.src_lp.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            lp,
        )?;

        let liquidity = ctx.accounts.lp_mint.supply;

        // Current pool reserves
        let (reserve_a, reserve_b) = pool.vault_amount_without_fee(
            ctx.accounts
                .treasury_a
                .amount
                .to_u128()
                .ok_or(ErrorCode::Overflow)?,
            ctx.accounts
                .treasury_b
                .amount
                .to_u128()
                .ok_or(ErrorCode::Overflow)?,
        );

        // Withdraw token A | a = lp * reserve_a / liquidity
        let a = Pool::hydrate_liquidity(lp, reserve_a, liquidity).ok_or(ErrorCode::Overflow)?;
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.treasury_a.to_account_info(),
                    to: ctx.accounts.dst_a.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                seeds,
            ),
            a,
        )?;
        // Withdraw token B | b = lp * reserve_b / liquidity
        let b = Pool::hydrate_liquidity(lp, reserve_b, liquidity).ok_or(ErrorCode::Overflow)?;
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.treasury_b.to_account_info(),
                    to: ctx.accounts.dst_b.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                seeds,
            ),
            b,
        )?;

        emit!(RemoveLiquidityEvent {
            authority: ctx.accounts.authority.key(),
            pool: pool.key(),
            a,
            b,
            lp
        });

        Ok(())
    }
}
