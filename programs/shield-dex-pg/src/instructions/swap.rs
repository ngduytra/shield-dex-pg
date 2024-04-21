use crate::{
    errors::ErrorCode,
    schema::{platform_config::PlatformConfig, pool::Pool},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token,
    token::{self},
};
use num::ToPrimitive;

#[event]
pub struct SwapEvent {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub bid_mint: Pubkey,
    pub ask_mint: Pubkey,
    pub bid_amount: u64,
    pub ask_amount: u64,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    /// Authority
    #[account(mut)]
    pub authority: Signer<'info>,
    // AMM config
    #[account(address = pool.tax)]
    pub platform_config: Box<Account<'info, PlatformConfig>>,
    /// Pool
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    /// CHECK: The pool fee reveiver
    #[account(
        mut,
        address= crate::create_pool_fee_reveiver::id(),
    )]
    pub taxman: AccountInfo<'info>,
    /// Bid Mint
    pub bid_mint: Box<Account<'info, token::Mint>>,
    #[account(
    mut,
    associated_token::mint = bid_mint,
    associated_token::authority = authority
  )]
    pub bid_src: Box<Account<'info, token::TokenAccount>>,
    #[account(
    mut,
    associated_token::mint = bid_mint,
    associated_token::authority = escrow
  )]
    pub bid_treasury: Box<Account<'info, token::TokenAccount>>,
    /// Ask Mint
    pub ask_mint: Box<Account<'info, token::Mint>>,
    #[account(
    mut,
    associated_token::mint = ask_mint,
    associated_token::authority = escrow
  )]
    pub ask_treasury: Box<Account<'info, token::TokenAccount>>,
    #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = ask_mint,
    associated_token::authority = authority
  )]
    pub ask_dst: Box<Account<'info, token::TokenAccount>>,
    /// CHECK: The pool escrow
    #[account(seeds = ["escrow".as_bytes(), &pool.key().to_bytes()], bump)]
    pub escrow: AccountInfo<'info>,
    #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = bid_mint,
    associated_token::authority = taxman,
   
  )]
    pub tax_dst: Box<Account<'info, token::TokenAccount>>,
    /// System programs
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl Swap<'_> {
    pub fn invoke(ctx: Context<Swap>, bid_amount: u64, limit: u64) -> Result<u64> {
        let pool = &mut ctx.accounts.pool;
        let platform_config: &Account<'_, PlatformConfig> = &ctx.accounts.platform_config;
        let seeds: &[&[&[u8]]] = &[&[
            "escrow".as_ref(),
            &pool.key().to_bytes(),
            &[ctx.bumps.escrow],
        ]];

        if !pool.is_active() {
            return err!(ErrorCode::InvalidState);
        }
        if bid_amount <= 0 {
            return err!(ErrorCode::InvalidParams);
        }

        // We ignore the value but MUST keep the function call for mints validation
        let _direction = pool
            .detect_direction(ctx.accounts.bid_mint.key(), ctx.accounts.ask_mint.key())
            .ok_or(ErrorCode::UnmatchPool)?;

        // Bid amount
        let fee = pool.calc_fee(bid_amount).ok_or(ErrorCode::Overflow)?;
        let tax = platform_config
            .calc_tax(bid_amount)
            .ok_or(ErrorCode::Overflow)?;
        let bid_amount_after_fee_and_tax = bid_amount
            .checked_sub(fee)
            .ok_or(ErrorCode::Overflow)?
            .checked_sub(tax)
            .ok_or(ErrorCode::Overflow)?;
        // Current pool reserves
        let (bid_reserve, ask_reserve) = pool.vault_amount_without_fee(
            ctx.accounts
                .bid_treasury
                .amount
                .to_u128()
                .ok_or(ErrorCode::Overflow)?,
            ctx.accounts
                .ask_treasury
                .amount
                .to_u128()
                .ok_or(ErrorCode::Overflow)?,
        );
        // Current pool liquidity aka. the product constant
        let liquidity = bid_reserve
            .checked_mul(ask_reserve)
            .ok_or(ErrorCode::Overflow)?;
        // Next pool reserves
        let next_bid_reserve = bid_amount_after_fee_and_tax
            .to_u128()
            .ok_or(ErrorCode::Overflow)?
            .checked_add(bid_reserve)
            .ok_or(ErrorCode::Overflow)?;
        let next_ask_reserve = liquidity
            .checked_div(next_bid_reserve)
            .ok_or(ErrorCode::Overflow)?;
        // Ask amount, fee amount, tax amount
        let ask_amount = ask_reserve
            .checked_sub(next_ask_reserve)
            .ok_or(ErrorCode::Overflow)?
            .to_u64()
            .ok_or(ErrorCode::Overflow)?;

        if ask_amount < limit {
            return err!(ErrorCode::LargeSlippage);
        }
        let last_bid_amount = bid_amount.checked_sub(tax).ok_or(ErrorCode::Overflow)?;

        // Transfer bid tokens
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.bid_src.to_account_info(),
                    to: ctx.accounts.bid_treasury.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            last_bid_amount,
        )?;
        // Transfer ask tokens
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.ask_treasury.to_account_info(),
                    to: ctx.accounts.ask_dst.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                seeds,
            ),
            ask_amount,
        )?;
        msg!("ask_amount:sww {}, bid_amount: {}, tax: {}, fee: {}", ask_amount, bid_amount, tax, fee);
        // Transfer the tax aka. the platform fee

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.bid_src.to_account_info(),
                    to: ctx.accounts.tax_dst.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            tax,
        )?;

        match _direction {
            true => {
                pool.lp_fees_mint_a = pool
                    .lp_fees_mint_a
                    .checked_add(fee)
                    .ok_or(ErrorCode::Overflow)?;
            }
            false => {
                pool.lp_fees_mint_b = pool
                    .lp_fees_mint_b
                    .checked_add(fee)
                    .ok_or(ErrorCode::Overflow)?;
            }
        }

        emit!(SwapEvent {
            authority: ctx.accounts.authority.key(),
            pool: pool.key(),
            bid_mint: ctx.accounts.bid_mint.key(),
            ask_mint: ctx.accounts.ask_mint.key(),
            bid_amount,
            ask_amount
        });

        Ok(ask_amount)
    }
}
