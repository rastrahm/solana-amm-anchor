//! Events emitted through `emit_cpi!` (self-CPI signed by the `__event_authority` PDA).
//!
//! Unlike `emit!` logs, self-CPI events cannot be truncated by the log limit nor forged by
//! another program in the same transaction. Fields follow the descending-size layout rule.

use anchor_lang::prelude::*;

/// @notice A new pool was created by `initialize`.
#[event]
pub struct PoolInitialized {
    /// Admin of the pool; `None` means immutable.
    pub authority: Option<Pubkey>,
    /// Config PDA identifying the pool.
    pub config: Pubkey,
    /// Mint of token X.
    pub mint_x: Pubkey,
    /// Mint of token Y.
    pub mint_y: Pubkey,
    /// LP share mint.
    pub mint_lp: Pubkey,
    /// Seed used to derive the Config PDA.
    pub seed: u64,
    /// Swap fee in basis points.
    pub fee: u16,
}

/// @notice Liquidity was added by `deposit`.
#[event]
pub struct LiquidityDeposited {
    /// Config PDA identifying the pool.
    pub config: Pubkey,
    /// Liquidity provider.
    pub user: Pubkey,
    /// Token X actually pulled from the user.
    pub amount_x: u64,
    /// Token Y actually pulled from the user.
    pub amount_y: u64,
    /// LP minted to the user.
    pub lp_minted: u64,
    /// LP locked forever in `lock_lp` (`MINIMUM_LIQUIDITY` on the first deposit, else 0).
    pub lp_locked: u64,
    /// Vault X balance after the deposit.
    pub reserve_x: u64,
    /// Vault Y balance after the deposit.
    pub reserve_y: u64,
}

/// @notice Liquidity was removed by `withdraw`.
#[event]
pub struct LiquidityWithdrawn {
    /// Config PDA identifying the pool.
    pub config: Pubkey,
    /// Liquidity provider.
    pub user: Pubkey,
    /// LP burned from the user.
    pub lp_burned: u64,
    /// Token X sent to the user.
    pub amount_x: u64,
    /// Token Y sent to the user.
    pub amount_y: u64,
    /// Vault X balance after the withdrawal.
    pub reserve_x: u64,
    /// Vault Y balance after the withdrawal.
    pub reserve_y: u64,
}

/// @notice A trade was executed by `swap`.
#[event]
pub struct SwapExecuted {
    /// Config PDA identifying the pool.
    pub config: Pubkey,
    /// Trader.
    pub user: Pubkey,
    /// Exact input sent by the trader (fee included).
    pub amount_in: u64,
    /// Output received by the trader.
    pub amount_out: u64,
    /// Vault X balance after the swap.
    pub reserve_x: u64,
    /// Vault Y balance after the swap.
    pub reserve_y: u64,
    /// `true` for X→Y, `false` for Y→X.
    pub is_x: bool,
}

/// @notice The pause flag was changed by `set_locked`.
#[event]
pub struct LockedUpdated {
    /// Config PDA identifying the pool.
    pub config: Pubkey,
    /// Admin that signed the change.
    pub authority: Pubkey,
    /// New pause flag.
    pub locked: bool,
}

/// @notice The admin was transferred or renounced by `set_authority`.
#[event]
pub struct AuthorityUpdated {
    /// Admin before the change.
    pub previous_authority: Option<Pubkey>,
    /// Admin after the change; `None` means renounced forever.
    pub new_authority: Option<Pubkey>,
    /// Config PDA identifying the pool.
    pub config: Pubkey,
}
