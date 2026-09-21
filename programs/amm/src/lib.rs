//! Constant-product AMM program (`x * y = k`).
//!
//! Module layout (Phase 0 scaffold):
//! - [`state`] — on-chain account layouts (Config, etc.)
//! - [`instructions`] — RPC handlers and `#[derive(Accounts)]` contexts
//! - [`errors`] — `#[error_code]` definitions

use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD");

/// Program entrypoints for the AMM.
#[program]
pub mod amm {
    use super::*;

    /// @notice Smoke-test entrypoint for Phase 0 scaffold validation.
    /// @dev No accounts mutated; confirms build/deploy/test wiring.
    /// @param ctx Empty accounts context (`Initialize`).
    /// @return Result<()> Ok when the instruction is invoked successfully.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(_ctx)
    }
}
