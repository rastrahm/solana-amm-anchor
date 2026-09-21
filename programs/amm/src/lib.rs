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

    /// @notice Smoke-test entrypoint (expanded in Phase 2 with real pool init).
    /// @dev No accounts mutated yet; confirms build/deploy/test wiring.
    /// @param ctx Empty accounts context (`Initialize`).
    /// @return Result<()> Ok when the instruction is invoked successfully.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(_ctx)
    }
}
