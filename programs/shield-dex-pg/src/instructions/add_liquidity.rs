use crate::{errors::ErrorCode, schema::pool::Pool};
use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token};

#[event]
pub struct AddLiquidityEvent {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub a: u64,
    pub b: u64,
    pub lp: u64,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
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
    associated_token::authority = authority
  )]
    pub src_a: Box<Account<'info, token::TokenAccount>>,
    #[account(
    mut,
    associated_token::mint = mint_a,
    associated_token::authority = escrow
  )]
    pub treasury_a: Box<Account<'info, token::TokenAccount>>,
    /// Mint B
    pub mint_b: Box<Account<'info, token::Mint>>,
    #[account(
    mut,
    associated_token::mint = mint_b,
    associated_token::authority = authority
  )]
    pub src_b: Box<Account<'info, token::TokenAccount>>,
    #[account(
    mut,
    associated_token::mint = mint_b,
    associated_token::authority = escrow
  )]
    pub treasury_b: Box<Account<'info, token::TokenAccount>>,
    // LP Mint
    #[account(mut)]
    pub lp_mint: Box<Account<'info, token::Mint>>,
    #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = lp_mint,
    associated_token::authority = authority
  )]
    pub dst_lp: Box<Account<'info, token::TokenAccount>>,
    /// CHECK: The pool escrow
    #[account(seeds = ["escrow".as_bytes(), &pool.key().to_bytes()], bump)]
    pub escrow: AccountInfo<'info>,
    /// System programs
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl AddLiquidity<'_> {
    pub fn invoke(ctx: Context<AddLiquidity>, a: u64, b: u64) -> Result<()> {
        let pool = &ctx.accounts.pool;
        let seeds: &[&[&[u8]]] = &[&[
            "escrow".as_ref(),
            &pool.key().to_bytes(),
            &[ctx.bumps.escrow],
        ]];
        msg!(
            "referrer: system_program: {}",
            ctx.accounts.system_program.key()
        );

        if a <= 0 || b <= 0 {
            return err!(ErrorCode::InvalidParams);
        }

        // Deposit token A
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.src_a.to_account_info(),
                    to: ctx.accounts.treasury_a.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            a,
        )?;
        // Deposit token B
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.src_b.to_account_info(),
                    to: ctx.accounts.treasury_b.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            b,
        )?;

        // Mint LP tokens
        let lp = Pool::calc_liquidity(a, b).ok_or(ErrorCode::Overflow)?;
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    to: ctx.accounts.dst_lp.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                seeds,
            ),
            lp,
        )?;

        emit!(AddLiquidityEvent {
            authority: ctx.accounts.authority.key(),
            pool: pool.key(),
            a,
            b,
            lp
        });

        Ok(())
    }
}
