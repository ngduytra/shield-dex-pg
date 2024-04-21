use crate::constants::*;
use anchor_lang::prelude::*;
use num::{integer::Roots, ToPrimitive};

///
/// Pool state
///
#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum PoolState {
    Uninitialized,
    Initialized,
    Paused,
    Canceled,
}
impl Default for PoolState {
    fn default() -> Self {
        PoolState::Uninitialized
    }
}

///
/// Pool struct
///
#[account]
pub struct Pool {
    pub authority: Pubkey,
    pub lp_mint: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub referral_fee: u64,
    pub lp_fee: u64,
    pub tax: Pubkey,
    pub state: PoolState,
    pub lp_fees_mint_a: u64,
    pub lp_fees_mint_b: u64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Pool {
    pub const LEN: usize = ACCOUNT_DISCRIMINATOR
        + PUBKEY_SIZE
        + PUBKEY_SIZE
        + PUBKEY_SIZE
        + PUBKEY_SIZE
        + U64_SIZE
        + U64_SIZE
        + PUBKEY_SIZE
        + U8_SIZE
        + U64_SIZE
        + U64_SIZE
        + I64_SIZE
        + I64_SIZE;

    ///
    /// The pool is active
    ///
    pub fn is_active(&self) -> bool {
        self.state == PoolState::Initialized
    }

    ///
    /// The pool is paused
    ///
    pub fn is_paused(&self) -> bool {
        self.state == PoolState::Paused
    }

    ///
    /// Determine the number of LP tokens corresponding to the amount of A and B
    /// lp = √(a*b)
    ///
    pub fn calc_liquidity(a: u64, b: u64) -> Option<u64> {
        a.to_u128()?.checked_mul(b.to_u128()?)?.sqrt().to_u64()
    }

    ///
    /// Determine the number of tokens corresponding to the amount of burned LP
    /// amount = lp * reserve / liquidity
    ///
    pub fn hydrate_liquidity(lp: u64, reserve: u128, liquidity: u64) -> Option<u64> {
        lp.to_u128()?
            .checked_mul(reserve.to_u128()?)?
            .checked_div(liquidity.to_u128()?)?
            .to_u64()
    }

    ///
    /// This function will detect the trading direction
    /// If true, it means the swap is from A to B.
    /// If false, the swap is from B to A.
    /// If the return is None, the pair of mints is invalid.
    ///
    pub fn detect_direction(&self, bid_mint: Pubkey, ask_mint: Pubkey) -> Option<bool> {
        if bid_mint == self.mint_a && ask_mint == self.mint_b {
            return Some(true);
        }
        if bid_mint == self.mint_b && ask_mint == self.mint_a {
            return Some(false);
        }
        None
    }

    ///
    /// Estimate fee amount
    ///
    pub fn calc_fee(&self, ask_amount: u64) -> Option<u64> {
        ask_amount
            .to_u128()?
            .checked_mul(self.lp_fee.to_u128()?)?
            .checked_div(PRECISION_U128)?
            .to_u64()
    }

    ///
    /// Estimate tax amount
    ///
    pub fn calc_referral_fee(&self) -> Option<u64> {
        self.lp_fee
            .to_u128()?
            .checked_mul(self.referral_fee.to_u128()?)?
            .checked_div(PRECISION_U128)?
            .to_u64()
    }

    pub fn vault_amount_without_fee(&self, vault_0: u128, vault_1: u128) -> (u128, u128) {
        (
            vault_0
                .checked_sub(self.lp_fees_mint_a.to_u128().unwrap())
                .unwrap(),
            vault_1
                .checked_sub(self.lp_fees_mint_b.to_u128().unwrap())
                .unwrap(),
        )
    }
}
