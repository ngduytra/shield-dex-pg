use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod schema;
pub mod utils;

declare_id!("7xCZgNDZ6da6Rup5eztPfPxuVNwVuvRac3nQK9U5ggEg");

pub mod admin {
    use anchor_lang::prelude::declare_id;
    // #[cfg(feature = "devnet")]
    // declare_id!("adMCyoCgfkg7bQiJ9aBJ59H3BXLY3r5LNLfPpQfMzBe");
    // #[cfg(not(feature = "devnet"))]
    declare_id!("CVkbpNdrD1hb6TDwiyaoEyrDUft4T7aM5PQifmtCnGb1");
}

pub mod create_pool_fee_reveiver {
    use anchor_lang::prelude::declare_id;
    // #[cfg(feature = "devnet")]
    // declare_id!("G11FKBRaAkHAKuLCgLM6K6NUc9rTjPAznRCjZifrTQe2");
    // #[cfg(not(feature = "devnet"))]
    declare_id!("CVkbpNdrD1hb6TDwiyaoEyrDUft4T7aM5PQifmtCnGb1");
}

#[program]
pub mod shield_dex_pg {
    use super::*;

    pub use instructions::{
        add_liquidity::*, create_platform_config::*, create_referrer::*, distribute_lp_fee::*,
        initialize::*, pause::*, remove_liquidity::*, resume::*, swap::*, transfer_ownership::*,
        update_lp_fee::*, update_platform_config::*, update_referral_fee::*, update_tax::*,
    };

    pub fn initialize(
        ctx: Context<Initialize>,
        a: u64,
        b: u64,
        referral_fee: u64,
        sol_amount_for_custom_fee: u64,
        fee: u64,
    ) -> Result<()> {
        Initialize::invoke(ctx, a, b, referral_fee, sol_amount_for_custom_fee, fee)
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, a: u64, b: u64) -> Result<()> {
        AddLiquidity::invoke(ctx, a, b)
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, lp: u64) -> Result<()> {
        RemoveLiquidity::invoke(ctx, lp)
    }

    pub fn swap(ctx: Context<Swap>, bid_amount: u64, limit: u64) -> Result<u64> {
        Swap::invoke(ctx, bid_amount, limit)
    }

    pub fn update_fee(ctx: Context<UpdateLPFee>, fee: u64) -> Result<()> {
        UpdateLPFee::invoke(ctx, fee)
    }

    pub fn update_referral_fee(ctx: Context<UpdateReferralFee>, fee: u64) -> Result<()> {
        UpdateReferralFee::invoke(ctx, fee)
    }

    pub fn update_tax(ctx: Context<UpdateTax>, tax: u64) -> Result<()> {
        UpdateTax::invoke(ctx, tax)
    }

    pub fn transfer_ownership(ctx: Context<TransferOwnership>, new_owner: Pubkey) -> Result<()> {
        TransferOwnership::invoke(ctx, new_owner)
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        Pause::invoke(ctx)
    }

    pub fn resume(ctx: Context<Resume>) -> Result<()> {
        Resume::invoke(ctx)
    }

    pub fn create_platform_config(ctx: Context<CreatePlatformConfig>, tax: u64) -> Result<()> {
        CreatePlatformConfig::invoke(ctx, tax)
    }

    pub fn update_platform_config(ctx: Context<UpdatePlatformConfig>, tax: u64) -> Result<()> {
        UpdatePlatformConfig::invoke(ctx, tax)
    }

    pub fn distribute_lp_fee(
        ctx: Context<DistributeLpFee>,
        amount_a_requested: u64,
        amount_b_requested: u64,
    ) -> Result<()> {
        DistributeLpFee::invoke(ctx, amount_a_requested, amount_b_requested)
    }

    pub fn create_referrer(ctx: Context<CreateReferrer>, referer_address: Pubkey) -> Result<()> {
        CreateReferrer::invoke(ctx, referer_address)
    }
}
