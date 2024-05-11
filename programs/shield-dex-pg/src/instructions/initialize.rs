use crate::{
    constants::{CUSTOMED_FEE_BOUND, LP_MINT_DECIMALS, MAXIMUM_FEE},
    errors::ErrorCode,
    schema::{
        platform_config::PlatformConfig,
        pool::{Pool, PoolState},
    },
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::{
    associated_token,
    token::{self},
};

#[event]
pub struct InitializeEvent {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub lp_mint: Pubkey,
    pub a: u64,
    pub b: u64,
    pub lp: u64,
    pub referral_fee: u64,
    pub lp_fee: u64,
    pub tax: Pubkey,
    pub created_at: i64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Authority
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Which config the pool belongs to.
    pub platform_config: Box<Account<'info, PlatformConfig>>,
    /// Pool
    #[account(init, payer = authority,  space = Pool::LEN)]
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
      init_if_needed,
      payer = authority,
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
      init_if_needed,
      payer = authority,
      associated_token::mint = mint_b,
      associated_token::authority = escrow
    )]
    pub treasury_b: Box<Account<'info, token::TokenAccount>>,
    // LP Mint
    #[account(
      init,
      payer = authority,
      mint::decimals = LP_MINT_DECIMALS,
      mint::authority = escrow,
      mint::freeze_authority = escrow,
      seeds = ["lp_mint".as_bytes(), &pool.key().to_bytes()],
      bump
    )]
    pub lp_mint: Box<Account<'info, token::Mint>>,
    #[account(
      init_if_needed,
      payer = authority,
      associated_token::mint = lp_mint,
      associated_token::authority = authority
    )]
    pub dst_lp: Box<Account<'info, token::TokenAccount>>,
    /// CHECK: The pool fee reveiver
    #[account(
        mut,
        address= crate::create_pool_fee_reveiver::id(),
    )]
    pub taxman: AccountInfo<'info>,
    /// CHECK: The pool escrow
    #[account(seeds = ["escrow".as_bytes(), &pool.key().to_bytes()], bump)]
    pub escrow: AccountInfo<'info>,
    /// System programs
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl Initialize<'_> {
    pub fn invoke(
        ctx: Context<Initialize>,
        a: u64,
        b: u64,
        referral_fee: u64,
        sol_amount_for_custom_fee: u64,
        lp_fee: u64,
    ) -> Result<()> {
        msg!(
            "Initialize: a={}, b={}, lp_fee={}, tax={}",
            a,
            b,
            lp_fee,
            sol_amount_for_custom_fee,
        );
        let pool = &mut ctx.accounts.pool;
        let seeds: &[&[&[u8]]] = &[&[
            "escrow".as_ref(),
            &pool.key().to_bytes(),
            &[ctx.bumps.escrow],
        ]];

        if ctx.accounts.mint_a.key() == ctx.accounts.mint_b.key() {
            return err!(ErrorCode::InvalidParams);
        }

        if a <= 0 || b <= 0 {
            return err!(ErrorCode::InvalidParams);
        }
        if lp_fee > MAXIMUM_FEE {
            return err!(ErrorCode::InvalidParams);
        }

        if lp_fee > CUSTOMED_FEE_BOUND {
            // Invoke the transfer instruction
            let ix = system_instruction::transfer(
                &ctx.accounts.authority.key(),
                &ctx.accounts.taxman.key(),
                sol_amount_for_custom_fee,
            );
            anchor_lang::solana_program::program::invoke(
                &ix,
                &[
                    ctx.accounts.authority.to_account_info(),
                    ctx.accounts.taxman.to_account_info(),
                ],
            )?;
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

        pool.authority = ctx.accounts.authority.key();
        pool.lp_mint = ctx.accounts.lp_mint.key();
        pool.mint_a = ctx.accounts.mint_a.key();
        pool.mint_b = ctx.accounts.mint_b.key();
        pool.referral_fee = referral_fee;
        pool.lp_fee = lp_fee;
        pool.tax = ctx.accounts.platform_config.key();
        pool.state = PoolState::Initialized;
        pool.created_at = Clock::get()?.unix_timestamp;
        pool.updated_at = Clock::get()?.unix_timestamp;

        emit!(InitializeEvent {
            authority: ctx.accounts.authority.key(),
            pool: pool.key(),
            mint_a: ctx.accounts.mint_a.key(),
            mint_b: ctx.accounts.mint_b.key(),
            lp_mint: ctx.accounts.lp_mint.key(),
            a,
            b,
            lp,
            referral_fee,
            lp_fee,
            tax: ctx.accounts.platform_config.key(),
            created_at: pool.created_at
        });

        Ok(())
    }
}
