//! Pool configuration account layout.
//!
//! Field order follows the descending-size BPF alignment rule from `.cursorrules`:
//! `Option<Pubkey>` → `Pubkey` → `u64` → `u16` → `u8` / `bool`.

use anchor_lang::prelude::*;

/// PDA seed prefix for the pool `Config` account.
pub const CONFIG_SEED: &[u8] = b"config";

/// PDA seed prefix for the LP mint controlled by this pool.
pub const LP_SEED: &[u8] = b"lp";

/// Maximum swap fee in basis points (100% = 10_000). Values above this are invalid.
pub const MAX_FEE_BPS: u16 = 10_000;

/// On-chain configuration for a constant-product AMM pool (`x * y = k`).
///
/// Allocated with exact space `8 (discriminator) + Config::INIT_SPACE`.
/// Canonical bumps are persisted here so later instructions use
/// `bump = config.config_bump` instead of client-supplied bumps.
#[account]
#[derive(InitSpace)]
pub struct Config {
    /// Optional protocol / admin authority. `None` means immutable / no admin.
    pub authority: Option<Pubkey>,
    /// Mint of token X in the pair.
    pub mint_x: Pubkey,
    /// Mint of token Y in the pair.
    pub mint_y: Pubkey,
    /// LP share mint whose mint authority is the Config PDA.
    pub mint_lp: Pubkey,
    /// Client-chosen seed used in the Config PDA derivation.
    pub seed: u64,
    /// Swap fee in basis points (1 bps = 0.01%). Must be `<= MAX_FEE_BPS`.
    pub fee: u16,
    /// When `true`, deposits / swaps / withdraws that mutate pool state are rejected.
    pub locked: bool,
    /// Canonical bump for the Config PDA.
    pub config_bump: u8,
    /// Canonical bump for the LP mint PDA.
    pub lp_bump: u8,
}

impl Config {
    /// Total account bytes including the 8-byte Anchor discriminator.
    pub const ACCOUNT_SPACE: usize = 8 + Self::INIT_SPACE;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Manual layout check (size-descending field packing, no implicit padding assumed by InitSpace).
    ///
    /// Option<Pubkey>=33, Pubkey×3=96, u64=8, u16=2, bool=1, u8×2=2 → 142
    const EXPECTED_INIT_SPACE: usize = 33 + 32 + 32 + 32 + 8 + 2 + 1 + 1 + 1;

    #[test]
    fn config_init_space_matches_manual_layout() {
        assert_eq!(
            Config::INIT_SPACE, EXPECTED_INIT_SPACE,
            "InitSpace diverged from the documented size-descending layout"
        );
    }

    #[test]
    fn config_account_space_is_discriminator_plus_init_space() {
        assert_eq!(Config::ACCOUNT_SPACE, 8 + Config::INIT_SPACE);
        assert_eq!(Config::ACCOUNT_SPACE, 8 + EXPECTED_INIT_SPACE);
    }
}
