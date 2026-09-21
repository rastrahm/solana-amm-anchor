//! Constant-product AMM program (`x * y = k`).
//!
//! Module layout:
//! - [`state`] — on-chain account layouts (`Config`, etc.)
//! - [`instructions`] — RPC handlers and `#[derive(Accounts)]` contexts
//! - [`errors`] — `#[error_code]` definitions (`AmmError`)

use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

pub use errors::*;
pub use state::*;

use instructions::*;

declare_id!("DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD");

/// Program entrypoints for the AMM.
#[program]
pub mod amm {
    use super::*;

    /// @notice Initializes a new constant-product pool.
    /// @dev Creates Config PDA, LP mint, and vault ATAs; stores canonical bumps.
    /// @param ctx Accounts context (`Initialize`).
    /// @param seed PDA seed for Config.
    /// @param fee Swap fee in basis points.
    /// @param authority Optional admin; `None` for immutable / no admin.
    /// @return Result<()> Ok when the pool accounts are initialized.
    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        fee: u16,
        authority: Option<Pubkey>,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, seed, fee, authority)
    }
}
