//! Constant-product AMM program (`x * y = k`).
//!
//! Module layout:
//! - [`state`] — on-chain account layouts (`Config`, etc.)
//! - [`instructions`] — RPC handlers and `#[derive(Accounts)]` contexts
//! - [`errors`] — `#[error_code]` definitions (`AmmError`)
//! - [`helpers`] — checked math and liquidity constants

use anchor_lang::prelude::*;

pub mod errors;
pub mod helpers;
pub mod instructions;
pub mod state;

pub use errors::*;
pub use helpers::*;
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

    /// @notice Deposits X/Y liquidity and mints LP tokens to the user.
    /// @dev Locks `MINIMUM_LIQUIDITY` on the first deposit; enforces `min_lp` slippage.
    /// @param ctx Accounts context (`Deposit`).
    /// @param amount_x Amount of token X to deposit.
    /// @param amount_y Amount of token Y to deposit.
    /// @param min_lp Minimum acceptable LP minted (slippage protection).
    /// @return Result<()> Ok when the deposit completes.
    pub fn deposit(
        ctx: Context<Deposit>,
        amount_x: u64,
        amount_y: u64,
        min_lp: u64,
    ) -> Result<()> {
        instructions::deposit::handler(ctx, amount_x, amount_y, min_lp)
    }

    /// @notice Burns LP and withdraws proportional X/Y to the user.
    /// @dev Floor division favors the pool; cannot burn below `MINIMUM_LIQUIDITY` supply.
    /// @param ctx Accounts context (`Withdraw`).
    /// @param lp_amount LP tokens to burn.
    /// @param min_x Minimum token X out (slippage).
    /// @param min_y Minimum token Y out (slippage).
    /// @return Result<()> Ok when burn and transfers complete.
    pub fn withdraw(
        ctx: Context<Withdraw>,
        lp_amount: u64,
        min_x: u64,
        min_y: u64,
    ) -> Result<()> {
        instructions::withdraw::handler(ctx, lp_amount, min_x, min_y)
    }

    /// @notice Swaps an exact input amount for the other pool token.
    /// @dev Fee on input; enforces constant-product invariant and `min_amount_out`.
    /// @param ctx Accounts context (`Swap`).
    /// @param is_x `true` for X→Y, `false` for Y→X.
    /// @param amount_in Exact tokens sent by the user.
    /// @param min_amount_out Minimum acceptable output (slippage).
    /// @return Result<()> Ok when both transfers complete.
    pub fn swap(
        ctx: Context<Swap>,
        is_x: bool,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::swap::handler(ctx, is_x, amount_in, min_amount_out)
    }
}
