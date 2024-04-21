use crate::constants::*;
use anchor_lang::prelude::*;
use num::ToPrimitive;

///
/// Referrer struct
///
#[account]
pub struct PlatformConfig {
    pub tax: u64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl PlatformConfig {
    pub const LEN: usize = ACCOUNT_DISCRIMINATOR + U64_SIZE + I64_SIZE + I64_SIZE;

    ///
    /// Estimate tax amount
    ///
    pub fn calc_tax(&self, ask_amount: u64) -> Option<u64> {
        msg!(
            "ask_amount: {} self.tax: {} :ss {}",
            ask_amount,
            self.tax,
            PRECISION_U128
        );
        ask_amount
            .to_u128()?
            .checked_mul(self.tax.to_u128()?)?
            .checked_div(PRECISION_U128)?
            .to_u64()
    }
}
