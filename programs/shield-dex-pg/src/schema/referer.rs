use crate::constants::*;
use anchor_lang::prelude::*;

///
/// Referrer struct
///
#[account]
pub struct Referrer {
    pub owner: Pubkey,
    pub referee: Pubkey,
    pub pool: Pubkey,
}

impl Referrer {
    pub const LEN: usize = ACCOUNT_DISCRIMINATOR + PUBKEY_SIZE + PUBKEY_SIZE + PUBKEY_SIZE;
}
