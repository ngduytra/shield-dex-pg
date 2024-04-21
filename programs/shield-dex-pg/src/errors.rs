use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Operation overflowed")]
    Overflow,
    #[msg("Not have permission")]
    Unauthorized,
    #[msg("Invalid params")]
    InvalidParams,
    #[msg("Invalid state")]
    InvalidState,
    #[msg("Unmatch pool")]
    UnmatchPool,
    #[msg("Swap failed")]
    SwapFailed,
    #[msg("Large slippage")]
    LargeSlippage,
    #[msg("Invalid platform config")]
    InvalidPlatformConfig,
    #[msg("Invalid referer")]
    InvalidReferer,
}
