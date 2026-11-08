//! Custom AMM program errors.

use anchor_lang::prelude::*;

/// Errors returned by AMM instructions.
#[error_code]
pub enum AmmError {
    /// @notice Vault or user token accounts for X and Y must be distinct keys.
    #[msg("Vault or token accounts for X and Y must not be identical")]
    IdenticalVaults,

    /// @notice Mint X and mint Y must be distinct.
    #[msg("Mint X and mint Y must not be identical")]
    IdenticalMints,

    /// @notice Fee exceeds `MAX_FEE_BPS` or is otherwise invalid.
    #[msg("Fee is invalid (must be <= 10000 bps)")]
    InvalidFee,

    /// @notice Output / LP minted below the caller-specified slippage bound.
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,

    /// @notice Checked arithmetic overflow / underflow / invalid sqrt.
    #[msg("Arithmetic overflow or invalid math")]
    MathOverflow,

    /// @notice Pool reserves or LP supply insufficient for the requested operation.
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,

    /// @notice Signer is not the configured authority.
    #[msg("Unauthorized")]
    Unauthorized,

    /// @notice Pool is locked; mutating instructions are disabled.
    #[msg("Pool is locked")]
    PoolLocked,

    /// @notice Amount argument is zero or otherwise invalid.
    #[msg("Invalid amount")]
    InvalidAmount,

    /// @notice Token-2022 mint carries an extension that breaks vault accounting or custody.
    #[msg("Mint has an unsupported Token-2022 extension")]
    UnsupportedMintExtension,

    /// @notice Mint has a freeze authority that could freeze the pool vaults.
    #[msg("Mint has a freeze authority")]
    MintHasFreezeAuthority,
}
