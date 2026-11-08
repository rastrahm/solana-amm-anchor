#![feature(prelude_import)]
//! Constant-product AMM program (`x * y = k`).
//!
//! Module layout:
//! - [`state`] — on-chain account layouts (`Config`, etc.)
//! - [`instructions`] — RPC handlers and `#[derive(Accounts)]` contexts
//! - [`errors`] — `#[error_code]` definitions (`AmmError`)
//! - [`helpers`] — checked math and liquidity constants
#![allow(unexpected_cfgs)]
#![allow(deprecated)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2021::*;
use anchor_lang::prelude::*;
pub mod errors {
    //! Custom program errors (`#[error_code]`).
    pub mod amm_error {
        //! Custom AMM program errors.
        use anchor_lang::prelude::*;
        #[repr(u32)]
        /// Errors returned by AMM instructions.
        pub enum AmmError {
            /// @notice Vault or user token accounts for X and Y must be distinct keys.
            IdenticalVaults,
            /// @notice Mint X and mint Y must be distinct.
            IdenticalMints,
            /// @notice Fee exceeds `MAX_FEE_BPS` or is otherwise invalid.
            InvalidFee,
            /// @notice Output / LP minted below the caller-specified slippage bound.
            SlippageExceeded,
            /// @notice Checked arithmetic overflow / underflow / invalid sqrt.
            MathOverflow,
            /// @notice Pool reserves or LP supply insufficient for the requested operation.
            InsufficientLiquidity,
            /// @notice Signer is not the configured authority.
            Unauthorized,
            /// @notice Pool is locked; mutating instructions are disabled.
            PoolLocked,
            /// @notice Amount argument is zero or otherwise invalid.
            InvalidAmount,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for AmmError {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        AmmError::IdenticalVaults => "IdenticalVaults",
                        AmmError::IdenticalMints => "IdenticalMints",
                        AmmError::InvalidFee => "InvalidFee",
                        AmmError::SlippageExceeded => "SlippageExceeded",
                        AmmError::MathOverflow => "MathOverflow",
                        AmmError::InsufficientLiquidity => "InsufficientLiquidity",
                        AmmError::Unauthorized => "Unauthorized",
                        AmmError::PoolLocked => "PoolLocked",
                        AmmError::InvalidAmount => "InvalidAmount",
                    },
                )
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for AmmError {}
        #[automatically_derived]
        impl ::core::clone::Clone for AmmError {
            #[inline]
            fn clone(&self) -> AmmError {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for AmmError {}
        impl AmmError {
            /// Gets the name of this [#enum_name].
            pub fn name(&self) -> String {
                match self {
                    AmmError::IdenticalVaults => "IdenticalVaults".to_string(),
                    AmmError::IdenticalMints => "IdenticalMints".to_string(),
                    AmmError::InvalidFee => "InvalidFee".to_string(),
                    AmmError::SlippageExceeded => "SlippageExceeded".to_string(),
                    AmmError::MathOverflow => "MathOverflow".to_string(),
                    AmmError::InsufficientLiquidity => {
                        "InsufficientLiquidity".to_string()
                    }
                    AmmError::Unauthorized => "Unauthorized".to_string(),
                    AmmError::PoolLocked => "PoolLocked".to_string(),
                    AmmError::InvalidAmount => "InvalidAmount".to_string(),
                }
            }
        }
        impl From<AmmError> for u32 {
            fn from(e: AmmError) -> u32 {
                e as u32 + anchor_lang::error::ERROR_CODE_OFFSET
            }
        }
        impl From<AmmError> for anchor_lang::error::Error {
            fn from(error_code: AmmError) -> anchor_lang::error::Error {
                anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                    error_name: error_code.name(),
                    error_code_number: error_code.into(),
                    error_msg: error_code.to_string(),
                    error_origin: None,
                    compared_values: None,
                })
            }
        }
        impl std::fmt::Display for AmmError {
            fn fmt(
                &self,
                fmt: &mut std::fmt::Formatter<'_>,
            ) -> std::result::Result<(), std::fmt::Error> {
                match self {
                    AmmError::IdenticalVaults => {
                        fmt.write_fmt(
                            format_args!(
                                "Vault or token accounts for X and Y must not be identical",
                            ),
                        )
                    }
                    AmmError::IdenticalMints => {
                        fmt.write_fmt(
                            format_args!("Mint X and mint Y must not be identical"),
                        )
                    }
                    AmmError::InvalidFee => {
                        fmt.write_fmt(
                            format_args!("Fee is invalid (must be <= 10000 bps)"),
                        )
                    }
                    AmmError::SlippageExceeded => {
                        fmt.write_fmt(format_args!("Slippage tolerance exceeded"))
                    }
                    AmmError::MathOverflow => {
                        fmt.write_fmt(
                            format_args!("Arithmetic overflow or invalid math"),
                        )
                    }
                    AmmError::InsufficientLiquidity => {
                        fmt.write_fmt(format_args!("Insufficient liquidity"))
                    }
                    AmmError::Unauthorized => fmt.write_fmt(format_args!("Unauthorized")),
                    AmmError::PoolLocked => fmt.write_fmt(format_args!("Pool is locked")),
                    AmmError::InvalidAmount => {
                        fmt.write_fmt(format_args!("Invalid amount"))
                    }
                }
            }
        }
    }
    pub use amm_error::*;
}
pub mod helpers {
    //! Shared helpers (math, constants).
    pub mod math {
        //! Constant-product liquidity math with checked `u128` arithmetic.
        use anchor_lang::prelude::*;
        use crate::errors::AmmError;
        /// LP tokens permanently locked on the first deposit (anti inflation attack).
        pub const MINIMUM_LIQUIDITY: u64 = 1_000;
        /// Integer square root (Babylonian method) for `u128`.
        ///
        /// @param value Non-negative radicand.
        /// @return Floor of `sqrt(value)` as `u64`, or `MathOverflow` if it exceeds `u64::MAX`.
        pub fn integer_sqrt(value: u128) -> Result<u64> {
            if value == 0 {
                return Ok(0);
            }
            let mut x = value;
            let mut y = value.saturating_add(1) / 2;
            while y < x {
                x = y;
                y = value
                    .checked_div(x)
                    .ok_or(AmmError::MathOverflow)?
                    .checked_add(x)
                    .ok_or(AmmError::MathOverflow)? / 2;
            }
            u64::try_from(x)
                .map_err(|_| anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                    error_name: AmmError::MathOverflow.name(),
                    error_code_number: AmmError::MathOverflow.into(),
                    error_msg: AmmError::MathOverflow.to_string(),
                    error_origin: Some(
                        anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                            filename: "programs/amm/src/helpers/math.rs",
                            line: 31u32,
                        }),
                    ),
                    compared_values: None,
                }))
        }
        /// Amount of LP tokens to mint to the user for a deposit.
        ///
        /// - First deposit (`total_lp == 0`): `sqrt(amount_x * amount_y) - MINIMUM_LIQUIDITY`
        /// - Later deposits: `min(amount_x * total_lp / reserve_x, amount_y * total_lp / reserve_y)`
        ///
        /// @return `(lp_to_user, lock_minimum)` where `lock_minimum` is true only on first deposit.
        pub fn lp_tokens_for_deposit(
            amount_x: u64,
            amount_y: u64,
            reserve_x: u64,
            reserve_y: u64,
            total_lp: u64,
        ) -> Result<(u64, bool)> {
            if total_lp == 0 {
                if !(reserve_x == 0 && reserve_y == 0) {
                    return Err(
                        anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                            error_name: AmmError::InsufficientLiquidity.name(),
                            error_code_number: AmmError::InsufficientLiquidity.into(),
                            error_msg: AmmError::InsufficientLiquidity.to_string(),
                            error_origin: Some(
                                anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                    filename: "programs/amm/src/helpers/math.rs",
                                    line: 48u32,
                                }),
                            ),
                            compared_values: None,
                        }),
                    );
                }
                let product = (amount_x as u128)
                    .checked_mul(amount_y as u128)
                    .ok_or(AmmError::MathOverflow)?;
                let liquidity = integer_sqrt(product)?;
                if !(liquidity > MINIMUM_LIQUIDITY) {
                    return Err(
                        anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                            error_name: AmmError::InsufficientLiquidity.name(),
                            error_code_number: AmmError::InsufficientLiquidity.into(),
                            error_msg: AmmError::InsufficientLiquidity.to_string(),
                            error_origin: Some(
                                anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                    filename: "programs/amm/src/helpers/math.rs",
                                    line: 57u32,
                                }),
                            ),
                            compared_values: None,
                        }),
                    );
                }
                let user_lp = liquidity
                    .checked_sub(MINIMUM_LIQUIDITY)
                    .ok_or(AmmError::MathOverflow)?;
                Ok((user_lp, true))
            } else {
                if !(reserve_x > 0 && reserve_y > 0) {
                    return Err(
                        anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                            error_name: AmmError::InsufficientLiquidity.name(),
                            error_code_number: AmmError::InsufficientLiquidity.into(),
                            error_msg: AmmError::InsufficientLiquidity.to_string(),
                            error_origin: Some(
                                anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                    filename: "programs/amm/src/helpers/math.rs",
                                    line: 66u32,
                                }),
                            ),
                            compared_values: None,
                        }),
                    );
                }
                let lp_from_x = (amount_x as u128)
                    .checked_mul(total_lp as u128)
                    .ok_or(AmmError::MathOverflow)?
                    .checked_div(reserve_x as u128)
                    .ok_or(AmmError::MathOverflow)?;
                let lp_from_y = (amount_y as u128)
                    .checked_mul(total_lp as u128)
                    .ok_or(AmmError::MathOverflow)?
                    .checked_div(reserve_y as u128)
                    .ok_or(AmmError::MathOverflow)?;
                let liquidity = lp_from_x.min(lp_from_y);
                let user_lp = u64::try_from(liquidity)
                    .map_err(|_| anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::MathOverflow.name(),
                        error_code_number: AmmError::MathOverflow.into(),
                        error_msg: AmmError::MathOverflow.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/helpers/math.rs",
                                line: 83u32,
                            }),
                        ),
                        compared_values: None,
                    }))?;
                if !(user_lp > 0) {
                    return Err(
                        anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                            error_name: AmmError::InsufficientLiquidity.name(),
                            error_code_number: AmmError::InsufficientLiquidity.into(),
                            error_msg: AmmError::InsufficientLiquidity.to_string(),
                            error_origin: Some(
                                anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                    filename: "programs/amm/src/helpers/math.rs",
                                    line: 84u32,
                                }),
                            ),
                            compared_values: None,
                        }),
                    );
                }
                Ok((user_lp, false))
            }
        }
        /// Token amounts returned when burning `lp_amount` shares.
        ///
        /// Uses floor division (rounds in favor of the pool):
        /// `amount_* = lp_amount * reserve_* / total_lp`.
        ///
        /// Ensures post-burn supply stays at least `MINIMUM_LIQUIDITY`.
        ///
        /// @return `(amount_x, amount_y)`.
        pub fn amounts_for_withdraw(
            lp_amount: u64,
            reserve_x: u64,
            reserve_y: u64,
            total_lp: u64,
        ) -> Result<(u64, u64)> {
            if !(lp_amount > 0) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InvalidAmount.name(),
                        error_code_number: AmmError::InvalidAmount.into(),
                        error_msg: AmmError::InvalidAmount.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/helpers/math.rs",
                                line: 103u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            if !(total_lp > 0 && reserve_x > 0 && reserve_y > 0) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InsufficientLiquidity.name(),
                        error_code_number: AmmError::InsufficientLiquidity.into(),
                        error_msg: AmmError::InsufficientLiquidity.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/helpers/math.rs",
                                line: 104u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            let remaining = total_lp
                .checked_sub(lp_amount)
                .ok_or(AmmError::InsufficientLiquidity)?;
            if !(remaining >= MINIMUM_LIQUIDITY) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InsufficientLiquidity.name(),
                        error_code_number: AmmError::InsufficientLiquidity.into(),
                        error_msg: AmmError::InsufficientLiquidity.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/helpers/math.rs",
                                line: 112u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            let amount_x = (lp_amount as u128)
                .checked_mul(reserve_x as u128)
                .ok_or(AmmError::MathOverflow)?
                .checked_div(total_lp as u128)
                .ok_or(AmmError::MathOverflow)?;
            let amount_y = (lp_amount as u128)
                .checked_mul(reserve_y as u128)
                .ok_or(AmmError::MathOverflow)?
                .checked_div(total_lp as u128)
                .ok_or(AmmError::MathOverflow)?;
            let amount_x = u64::try_from(amount_x)
                .map_err(|_| anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                    error_name: AmmError::MathOverflow.name(),
                    error_code_number: AmmError::MathOverflow.into(),
                    error_msg: AmmError::MathOverflow.to_string(),
                    error_origin: Some(
                        anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                            filename: "programs/amm/src/helpers/math.rs",
                            line: 128u32,
                        }),
                    ),
                    compared_values: None,
                }))?;
            let amount_y = u64::try_from(amount_y)
                .map_err(|_| anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                    error_name: AmmError::MathOverflow.name(),
                    error_code_number: AmmError::MathOverflow.into(),
                    error_msg: AmmError::MathOverflow.to_string(),
                    error_origin: Some(
                        anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                            filename: "programs/amm/src/helpers/math.rs",
                            line: 129u32,
                        }),
                    ),
                    compared_values: None,
                }))?;
            if !(amount_x > 0 && amount_y > 0) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InsufficientLiquidity.name(),
                        error_code_number: AmmError::InsufficientLiquidity.into(),
                        error_msg: AmmError::InsufficientLiquidity.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/helpers/math.rs",
                                line: 130u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            Ok((amount_x, amount_y))
        }
        /// Basis-points denominator for swap fees (`fee` is stored in bps on `Config`).
        pub const FEE_DENOMINATOR: u128 = 10_000;
        /// Exact-in constant-product swap output with fee applied on the input.
        ///
        /// Uses Uniswap-v2 style math to limit precision loss:
        /// `amount_out = (amount_in * (10_000 - fee) * reserve_out)
        ///             / (reserve_in * 10_000 + amount_in * (10_000 - fee))`
        ///
        /// Also enforces the post-swap invariant
        /// `(reserve_in + amount_in) * (reserve_out - amount_out) >= reserve_in * reserve_out`.
        ///
        /// @return `amount_out` of the token leaving the pool.
        pub fn amount_out_for_swap(
            amount_in: u64,
            reserve_in: u64,
            reserve_out: u64,
            fee_bps: u16,
        ) -> Result<u64> {
            if !(amount_in > 0) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InvalidAmount.name(),
                        error_code_number: AmmError::InvalidAmount.into(),
                        error_msg: AmmError::InvalidAmount.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/helpers/math.rs",
                                line: 157u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            if !(reserve_in > 0 && reserve_out > 0) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InsufficientLiquidity.name(),
                        error_code_number: AmmError::InsufficientLiquidity.into(),
                        error_msg: AmmError::InsufficientLiquidity.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/helpers/math.rs",
                                line: 158u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            if !(fee_bps as u128 <= FEE_DENOMINATOR) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InvalidFee.name(),
                        error_code_number: AmmError::InvalidFee.into(),
                        error_msg: AmmError::InvalidFee.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/helpers/math.rs",
                                line: 162u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            let fee_multiplier = FEE_DENOMINATOR
                .checked_sub(fee_bps as u128)
                .ok_or(AmmError::MathOverflow)?;
            let amount_in_u = amount_in as u128;
            let reserve_in_u = reserve_in as u128;
            let reserve_out_u = reserve_out as u128;
            let amount_in_with_fee = amount_in_u
                .checked_mul(fee_multiplier)
                .ok_or(AmmError::MathOverflow)?;
            let numerator = amount_in_with_fee
                .checked_mul(reserve_out_u)
                .ok_or(AmmError::MathOverflow)?;
            let denominator = reserve_in_u
                .checked_mul(FEE_DENOMINATOR)
                .ok_or(AmmError::MathOverflow)?
                .checked_add(amount_in_with_fee)
                .ok_or(AmmError::MathOverflow)?;
            let amount_out_u = numerator
                .checked_div(denominator)
                .ok_or(AmmError::MathOverflow)?;
            if !(amount_out_u > 0) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InsufficientLiquidity.name(),
                        error_code_number: AmmError::InsufficientLiquidity.into(),
                        error_msg: AmmError::InsufficientLiquidity.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/helpers/math.rs",
                                line: 187u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            if !(amount_out_u < reserve_out_u) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InsufficientLiquidity.name(),
                        error_code_number: AmmError::InsufficientLiquidity.into(),
                        error_msg: AmmError::InsufficientLiquidity.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/helpers/math.rs",
                                line: 188u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            let amount_out = u64::try_from(amount_out_u)
                .map_err(|_| anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                    error_name: AmmError::MathOverflow.name(),
                    error_code_number: AmmError::MathOverflow.into(),
                    error_msg: AmmError::MathOverflow.to_string(),
                    error_origin: Some(
                        anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                            filename: "programs/amm/src/helpers/math.rs",
                            line: 193u32,
                        }),
                    ),
                    compared_values: None,
                }))?;
            let k_before = reserve_in_u
                .checked_mul(reserve_out_u)
                .ok_or(AmmError::MathOverflow)?;
            let k_after = reserve_in_u
                .checked_add(amount_in_u)
                .ok_or(AmmError::MathOverflow)?
                .checked_mul(
                    reserve_out_u
                        .checked_sub(amount_out_u)
                        .ok_or(AmmError::MathOverflow)?,
                )
                .ok_or(AmmError::MathOverflow)?;
            if !(k_after >= k_before) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::MathOverflow.name(),
                        error_code_number: AmmError::MathOverflow.into(),
                        error_msg: AmmError::MathOverflow.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/helpers/math.rs",
                                line: 208u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            Ok(amount_out)
        }
    }
    pub use math::*;
}
pub mod instructions {
    //! Instruction handlers and account contexts.
    pub mod deposit {
        //! Add liquidity: transfer X/Y into vaults and mint LP shares.
        use anchor_lang::prelude::*;
        use anchor_spl::{
            associated_token::AssociatedToken,
            token_interface::{
                mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface,
                TransferChecked,
            },
        };
        use crate::errors::AmmError;
        use crate::helpers::{lp_tokens_for_deposit, MINIMUM_LIQUIDITY};
        use crate::state::{Config, CONFIG_SEED, LP_SEED};
        /// Accounts for depositing token X/Y and minting LP.
        ///
        /// Large token accounts are `Box`ed to stay under the BPF 4KB stack limit.
        pub struct Deposit<'info> {
            /// Liquidity provider funding the deposit and paying ATA rent if needed.
            #[account(mut)]
            pub user: Signer<'info>,
            /// Mint of token X (must match `config.mint_x`).
            pub mint_x: Box<InterfaceAccount<'info, Mint>>,
            /// Mint of token Y (must match `config.mint_y`).
            pub mint_y: Box<InterfaceAccount<'info, Mint>>,
            /// LP mint controlled by the Config PDA.
            #[account(
                mut,
                seeds = [LP_SEED,
                config.key().as_ref()],
                bump = config.lp_bump,
            )]
            pub mint_lp: Box<InterfaceAccount<'info, Mint>>,
            /// Pool config PDA. Uses stored canonical bumps (no client-supplied bump).
            #[account(
                has_one = mint_x,
                has_one = mint_y,
                has_one = mint_lp,
                seeds = [CONFIG_SEED,
                config.seed.to_le_bytes().as_ref()],
                bump = config.config_bump,
            )]
            pub config: Box<Account<'info, Config>>,
            /// Pool vault for token X.
            #[account(
                mut,
                associated_token::mint = mint_x,
                associated_token::authority = config,
                associated_token::token_program = token_program,
            )]
            pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,
            /// Pool vault for token Y.
            #[account(
                mut,
                associated_token::mint = mint_y,
                associated_token::authority = config,
                associated_token::token_program = token_program,
                constraint = vault_x.key()!= vault_y.key()@AmmError::IdenticalVaults
            )]
            pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,
            /// User ATA for token X.
            #[account(
                mut,
                associated_token::mint = mint_x,
                associated_token::authority = user,
                associated_token::token_program = token_program,
            )]
            pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,
            /// User ATA for token Y.
            #[account(
                mut,
                associated_token::mint = mint_y,
                associated_token::authority = user,
                associated_token::token_program = token_program,
            )]
            pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,
            /// User ATA for LP tokens (created on first deposit if needed).
            #[account(
                init_if_needed,
                payer = user,
                associated_token::mint = mint_lp,
                associated_token::authority = user,
                associated_token::token_program = token_program,
            )]
            pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,
            /// Locked LP ATA owned by Config (holds `MINIMUM_LIQUIDITY` forever).
            #[account(
                init_if_needed,
                payer = user,
                associated_token::mint = mint_lp,
                associated_token::authority = config,
                associated_token::token_program = token_program,
            )]
            pub lock_lp: Box<InterfaceAccount<'info, TokenAccount>>,
            /// SPL Token or Token-2022 program.
            pub token_program: Interface<'info, TokenInterface>,
            /// Associated Token Account program.
            pub associated_token_program: Program<'info, AssociatedToken>,
            /// System program.
            pub system_program: Program<'info, System>,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::Accounts<'info, DepositBumps> for Deposit<'info>
        where
            'info: 'info,
        {
            #[inline(never)]
            fn try_accounts(
                __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
                __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >],
                __ix_data: &[u8],
                __bumps: &mut DepositBumps,
                __reallocs: &mut std::collections::BTreeSet<
                    anchor_lang::solana_program::pubkey::Pubkey,
                >,
            ) -> anchor_lang::Result<Self> {
                let user: Signer = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("user"))?;
                let mint_x: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<Mint>,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("mint_x"))?;
                let mint_y: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<Mint>,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("mint_y"))?;
                let mint_lp: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<Mint>,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("mint_lp"))?;
                let config: Box<anchor_lang::accounts::account::Account<Config>> = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("config"))?;
                let vault_x: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("vault_x"))?;
                let vault_y: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("vault_y"))?;
                let user_x: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("user_x"))?;
                let user_y: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("user_y"))?;
                if __accounts.is_empty() {
                    return Err(
                        anchor_lang::error::ErrorCode::AccountNotEnoughKeys.into(),
                    );
                }
                let user_lp = &__accounts[0];
                *__accounts = &__accounts[1..];
                if __accounts.is_empty() {
                    return Err(
                        anchor_lang::error::ErrorCode::AccountNotEnoughKeys.into(),
                    );
                }
                let lock_lp = &__accounts[0];
                *__accounts = &__accounts[1..];
                let token_program: anchor_lang::accounts::interface::Interface<
                    TokenInterface,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("token_program"))?;
                let associated_token_program: anchor_lang::accounts::program::Program<
                    AssociatedToken,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("associated_token_program"))?;
                let system_program: anchor_lang::accounts::program::Program<System> = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("system_program"))?;
                let __anchor_rent = Rent::get()?;
                let user_lp: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = ({
                    #[inline(never)]
                    || {
                        let owner_program = AsRef::<
                            AccountInfo,
                        >::as_ref(user_lp.as_ref())
                            .owner;
                        if !true
                            || owner_program
                                == &anchor_lang::solana_program::system_program::ID
                        {
                            ::anchor_spl::associated_token::create(
                                anchor_lang::context::CpiContext::new(
                                    associated_token_program.to_account_info(),
                                    ::anchor_spl::associated_token::Create {
                                        payer: user.to_account_info(),
                                        associated_token: user_lp.to_account_info(),
                                        authority: user.to_account_info(),
                                        mint: mint_lp.to_account_info(),
                                        system_program: system_program.to_account_info(),
                                        token_program: token_program.to_account_info(),
                                    },
                                ),
                            )?;
                        }
                        let pa: Box<
                            anchor_lang::accounts::interface_account::InterfaceAccount<
                                TokenAccount,
                            >,
                        > = Box::new(
                            match anchor_lang::accounts::interface_account::InterfaceAccount::try_from_unchecked(
                                &user_lp,
                            ) {
                                Ok(val) => val,
                                Err(e) => return Err(e.with_account_name("user_lp")),
                            },
                        );
                        if true {
                            if pa.mint != mint_lp.key() {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::ConstraintTokenMint,
                                        )
                                        .with_account_name("user_lp")
                                        .with_pubkeys((pa.mint, mint_lp.key())),
                                );
                            }
                            if pa.owner != user.key() {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                        )
                                        .with_account_name("user_lp")
                                        .with_pubkeys((pa.owner, user.key())),
                                );
                            }
                            if owner_program != &token_program.key() {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram,
                                        )
                                        .with_account_name("user_lp")
                                        .with_pubkeys((*owner_program, token_program.key())),
                                );
                            }
                            if pa.key()
                                != ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                                    &user.key(),
                                    &mint_lp.key(),
                                    &token_program.key(),
                                )
                            {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::AccountNotAssociatedTokenAccount,
                                        )
                                        .with_account_name("user_lp"),
                                );
                            }
                        }
                        Ok(pa)
                    }
                })()?;
                if !AsRef::<AccountInfo>::as_ref(user_lp.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("user_lp"),
                    );
                }
                if !__anchor_rent
                    .is_exempt(
                        user_lp.to_account_info().lamports(),
                        user_lp.to_account_info().try_data_len()?,
                    )
                {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintRentExempt,
                            )
                            .with_account_name("user_lp"),
                    );
                }
                let __anchor_rent = Rent::get()?;
                let lock_lp: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = ({
                    #[inline(never)]
                    || {
                        let owner_program = AsRef::<
                            AccountInfo,
                        >::as_ref(lock_lp.as_ref())
                            .owner;
                        if !true
                            || owner_program
                                == &anchor_lang::solana_program::system_program::ID
                        {
                            ::anchor_spl::associated_token::create(
                                anchor_lang::context::CpiContext::new(
                                    associated_token_program.to_account_info(),
                                    ::anchor_spl::associated_token::Create {
                                        payer: user.to_account_info(),
                                        associated_token: lock_lp.to_account_info(),
                                        authority: config.to_account_info(),
                                        mint: mint_lp.to_account_info(),
                                        system_program: system_program.to_account_info(),
                                        token_program: token_program.to_account_info(),
                                    },
                                ),
                            )?;
                        }
                        let pa: Box<
                            anchor_lang::accounts::interface_account::InterfaceAccount<
                                TokenAccount,
                            >,
                        > = Box::new(
                            match anchor_lang::accounts::interface_account::InterfaceAccount::try_from_unchecked(
                                &lock_lp,
                            ) {
                                Ok(val) => val,
                                Err(e) => return Err(e.with_account_name("lock_lp")),
                            },
                        );
                        if true {
                            if pa.mint != mint_lp.key() {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::ConstraintTokenMint,
                                        )
                                        .with_account_name("lock_lp")
                                        .with_pubkeys((pa.mint, mint_lp.key())),
                                );
                            }
                            if pa.owner != config.key() {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                        )
                                        .with_account_name("lock_lp")
                                        .with_pubkeys((pa.owner, config.key())),
                                );
                            }
                            if owner_program != &token_program.key() {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram,
                                        )
                                        .with_account_name("lock_lp")
                                        .with_pubkeys((*owner_program, token_program.key())),
                                );
                            }
                            if pa.key()
                                != ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                                    &config.key(),
                                    &mint_lp.key(),
                                    &token_program.key(),
                                )
                            {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::AccountNotAssociatedTokenAccount,
                                        )
                                        .with_account_name("lock_lp"),
                                );
                            }
                        }
                        Ok(pa)
                    }
                })()?;
                if !AsRef::<AccountInfo>::as_ref(lock_lp.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("lock_lp"),
                    );
                }
                if !__anchor_rent
                    .is_exempt(
                        lock_lp.to_account_info().lamports(),
                        lock_lp.to_account_info().try_data_len()?,
                    )
                {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintRentExempt,
                            )
                            .with_account_name("lock_lp"),
                    );
                }
                if !AsRef::<AccountInfo>::as_ref(&user).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("user"),
                    );
                }
                let __pda_address = Pubkey::create_program_address(
                        &[LP_SEED, config.key().as_ref(), &[config.lp_bump][..]],
                        &__program_id,
                    )
                    .map_err(|_| {
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("mint_lp")
                    })?;
                if mint_lp.key() != __pda_address {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("mint_lp")
                            .with_pubkeys((mint_lp.key(), __pda_address)),
                    );
                }
                if !AsRef::<AccountInfo>::as_ref(mint_lp.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("mint_lp"),
                    );
                }
                let __pda_address = Pubkey::create_program_address(
                        &[
                            CONFIG_SEED,
                            config.seed.to_le_bytes().as_ref(),
                            &[config.config_bump][..],
                        ],
                        &__program_id,
                    )
                    .map_err(|_| {
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("config")
                    })?;
                if config.key() != __pda_address {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("config")
                            .with_pubkeys((config.key(), __pda_address)),
                    );
                }
                {
                    let my_key = config.mint_x;
                    let target_key = mint_x.key();
                    if my_key != target_key {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintHasOne,
                                )
                                .with_account_name("config")
                                .with_pubkeys((my_key, target_key)),
                        );
                    }
                }
                {
                    let my_key = config.mint_y;
                    let target_key = mint_y.key();
                    if my_key != target_key {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintHasOne,
                                )
                                .with_account_name("config")
                                .with_pubkeys((my_key, target_key)),
                        );
                    }
                }
                {
                    let my_key = config.mint_lp;
                    let target_key = mint_lp.key();
                    if my_key != target_key {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintHasOne,
                                )
                                .with_account_name("config")
                                .with_pubkeys((my_key, target_key)),
                        );
                    }
                }
                {
                    if AsRef::<AccountInfo>::as_ref(vault_x.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = vault_x.owner;
                    let wallet_address = config.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("vault_x")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_x.key(),
                        &token_program.key(),
                    );
                    let my_key = vault_x.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("vault_x")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(vault_x.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("vault_x"),
                    );
                }
                {
                    if AsRef::<AccountInfo>::as_ref(vault_y.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = vault_y.owner;
                    let wallet_address = config.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("vault_y")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_y.key(),
                        &token_program.key(),
                    );
                    let my_key = vault_y.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("vault_y")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(vault_y.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("vault_y"),
                    );
                }
                if !(vault_x.key() != vault_y.key()) {
                    return Err(
                        anchor_lang::error::Error::from(AmmError::IdenticalVaults)
                            .with_account_name("vault_y"),
                    );
                }
                {
                    if AsRef::<AccountInfo>::as_ref(user_x.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = user_x.owner;
                    let wallet_address = user.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("user_x")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_x.key(),
                        &token_program.key(),
                    );
                    let my_key = user_x.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("user_x")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(user_x.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("user_x"),
                    );
                }
                {
                    if AsRef::<AccountInfo>::as_ref(user_y.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = user_y.owner;
                    let wallet_address = user.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("user_y")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_y.key(),
                        &token_program.key(),
                    );
                    let my_key = user_y.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("user_y")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(user_y.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("user_y"),
                    );
                }
                Ok(Deposit {
                    user,
                    mint_x,
                    mint_y,
                    mint_lp,
                    config,
                    vault_x,
                    vault_y,
                    user_x,
                    user_y,
                    user_lp,
                    lock_lp,
                    token_program,
                    associated_token_program,
                    system_program,
                })
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for Deposit<'info>
        where
            'info: 'info,
        {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos.extend(self.user.to_account_infos());
                account_infos.extend(self.mint_x.to_account_infos());
                account_infos.extend(self.mint_y.to_account_infos());
                account_infos.extend(self.mint_lp.to_account_infos());
                account_infos.extend(self.config.to_account_infos());
                account_infos.extend(self.vault_x.to_account_infos());
                account_infos.extend(self.vault_y.to_account_infos());
                account_infos.extend(self.user_x.to_account_infos());
                account_infos.extend(self.user_y.to_account_infos());
                account_infos.extend(self.user_lp.to_account_infos());
                account_infos.extend(self.lock_lp.to_account_infos());
                account_infos.extend(self.token_program.to_account_infos());
                account_infos.extend(self.associated_token_program.to_account_infos());
                account_infos.extend(self.system_program.to_account_infos());
                account_infos
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for Deposit<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas.extend(self.user.to_account_metas(None));
                account_metas.extend(self.mint_x.to_account_metas(None));
                account_metas.extend(self.mint_y.to_account_metas(None));
                account_metas.extend(self.mint_lp.to_account_metas(None));
                account_metas.extend(self.config.to_account_metas(None));
                account_metas.extend(self.vault_x.to_account_metas(None));
                account_metas.extend(self.vault_y.to_account_metas(None));
                account_metas.extend(self.user_x.to_account_metas(None));
                account_metas.extend(self.user_y.to_account_metas(None));
                account_metas.extend(self.user_lp.to_account_metas(None));
                account_metas.extend(self.lock_lp.to_account_metas(None));
                account_metas.extend(self.token_program.to_account_metas(None));
                account_metas
                    .extend(self.associated_token_program.to_account_metas(None));
                account_metas.extend(self.system_program.to_account_metas(None));
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::AccountsExit<'info> for Deposit<'info>
        where
            'info: 'info,
        {
            fn exit(
                &self,
                program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            ) -> anchor_lang::Result<()> {
                anchor_lang::AccountsExit::exit(&self.user, program_id)
                    .map_err(|e| e.with_account_name("user"))?;
                anchor_lang::AccountsExit::exit(&self.mint_lp, program_id)
                    .map_err(|e| e.with_account_name("mint_lp"))?;
                anchor_lang::AccountsExit::exit(&self.vault_x, program_id)
                    .map_err(|e| e.with_account_name("vault_x"))?;
                anchor_lang::AccountsExit::exit(&self.vault_y, program_id)
                    .map_err(|e| e.with_account_name("vault_y"))?;
                anchor_lang::AccountsExit::exit(&self.user_x, program_id)
                    .map_err(|e| e.with_account_name("user_x"))?;
                anchor_lang::AccountsExit::exit(&self.user_y, program_id)
                    .map_err(|e| e.with_account_name("user_y"))?;
                anchor_lang::AccountsExit::exit(&self.user_lp, program_id)
                    .map_err(|e| e.with_account_name("user_lp"))?;
                anchor_lang::AccountsExit::exit(&self.lock_lp, program_id)
                    .map_err(|e| e.with_account_name("lock_lp"))?;
                Ok(())
            }
        }
        pub struct DepositBumps {}
        #[automatically_derived]
        impl ::core::fmt::Debug for DepositBumps {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "DepositBumps")
            }
        }
        impl Default for DepositBumps {
            fn default() -> Self {
                DepositBumps {}
            }
        }
        impl<'info> anchor_lang::Bumps for Deposit<'info>
        where
            'info: 'info,
        {
            type Bumps = DepositBumps;
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
        /// instead of an `AccountInfo`. This is useful for clients that want
        /// to generate a list of accounts, without explicitly knowing the
        /// order all the fields should be in.
        ///
        /// To access the struct in this module, one should use the sibling
        /// `accounts` module (also generated), which re-exports this.
        pub(crate) mod __client_accounts_deposit {
            use super::*;
            use anchor_lang::prelude::borsh;
            /// Generated client accounts for [`Deposit`].
            pub struct Deposit {
                ///Liquidity provider funding the deposit and paying ATA rent if needed.
                pub user: Pubkey,
                ///Mint of token X (must match `config.mint_x`).
                pub mint_x: Pubkey,
                ///Mint of token Y (must match `config.mint_y`).
                pub mint_y: Pubkey,
                ///LP mint controlled by the Config PDA.
                pub mint_lp: Pubkey,
                ///Pool config PDA. Uses stored canonical bumps (no client-supplied bump).
                pub config: Pubkey,
                ///Pool vault for token X.
                pub vault_x: Pubkey,
                ///Pool vault for token Y.
                pub vault_y: Pubkey,
                ///User ATA for token X.
                pub user_x: Pubkey,
                ///User ATA for token Y.
                pub user_y: Pubkey,
                ///User ATA for LP tokens (created on first deposit if needed).
                pub user_lp: Pubkey,
                ///Locked LP ATA owned by Config (holds `MINIMUM_LIQUIDITY` forever).
                pub lock_lp: Pubkey,
                ///SPL Token or Token-2022 program.
                pub token_program: Pubkey,
                ///Associated Token Account program.
                pub associated_token_program: Pubkey,
                ///System program.
                pub system_program: Pubkey,
            }
            impl borsh::ser::BorshSerialize for Deposit
            where
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.user, writer)?;
                    borsh::BorshSerialize::serialize(&self.mint_x, writer)?;
                    borsh::BorshSerialize::serialize(&self.mint_y, writer)?;
                    borsh::BorshSerialize::serialize(&self.mint_lp, writer)?;
                    borsh::BorshSerialize::serialize(&self.config, writer)?;
                    borsh::BorshSerialize::serialize(&self.vault_x, writer)?;
                    borsh::BorshSerialize::serialize(&self.vault_y, writer)?;
                    borsh::BorshSerialize::serialize(&self.user_x, writer)?;
                    borsh::BorshSerialize::serialize(&self.user_y, writer)?;
                    borsh::BorshSerialize::serialize(&self.user_lp, writer)?;
                    borsh::BorshSerialize::serialize(&self.lock_lp, writer)?;
                    borsh::BorshSerialize::serialize(&self.token_program, writer)?;
                    borsh::BorshSerialize::serialize(
                        &self.associated_token_program,
                        writer,
                    )?;
                    borsh::BorshSerialize::serialize(&self.system_program, writer)?;
                    Ok(())
                }
            }
            #[automatically_derived]
            impl anchor_lang::ToAccountMetas for Deposit {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.user,
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.mint_x,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.mint_y,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.mint_lp,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.config,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.vault_x,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.vault_y,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.user_x,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.user_y,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.user_lp,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.lock_lp,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.token_program,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.associated_token_program,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.system_program,
                                false,
                            ),
                        );
                    account_metas
                }
            }
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a CPI struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is an
        /// AccountInfo.
        ///
        /// To access the struct in this module, one should use the sibling
        /// [`cpi::accounts`] module (also generated), which re-exports this.
        pub(crate) mod __cpi_client_accounts_deposit {
            use super::*;
            /// Generated CPI struct of the accounts for [`Deposit`].
            pub struct Deposit<'info> {
                ///Liquidity provider funding the deposit and paying ATA rent if needed.
                pub user: anchor_lang::solana_program::account_info::AccountInfo<'info>,
                ///Mint of token X (must match `config.mint_x`).
                pub mint_x: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Mint of token Y (must match `config.mint_y`).
                pub mint_y: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///LP mint controlled by the Config PDA.
                pub mint_lp: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Pool config PDA. Uses stored canonical bumps (no client-supplied bump).
                pub config: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Pool vault for token X.
                pub vault_x: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Pool vault for token Y.
                pub vault_y: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///User ATA for token X.
                pub user_x: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///User ATA for token Y.
                pub user_y: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///User ATA for LP tokens (created on first deposit if needed).
                pub user_lp: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Locked LP ATA owned by Config (holds `MINIMUM_LIQUIDITY` forever).
                pub lock_lp: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///SPL Token or Token-2022 program.
                pub token_program: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Associated Token Account program.
                pub associated_token_program: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///System program.
                pub system_program: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountMetas for Deposit<'info> {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.user),
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.mint_x),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.mint_y),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.mint_lp),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.config),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.vault_x),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.vault_y),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.user_x),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.user_y),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.user_lp),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.lock_lp),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.token_program),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.associated_token_program),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.system_program),
                                false,
                            ),
                        );
                    account_metas
                }
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountInfos<'info> for Deposit<'info> {
                fn to_account_infos(
                    &self,
                ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                    let mut account_infos = ::alloc::vec::Vec::new();
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.user),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.mint_x),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.mint_y),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.mint_lp),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.config),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.vault_x),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.vault_y),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.user_x),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.user_y),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.user_lp),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.lock_lp),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.token_program,
                            ),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.associated_token_program,
                            ),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.system_program,
                            ),
                        );
                    account_infos
                }
            }
        }
        /// @notice Deposits token X and Y into the pool and mints LP shares.
        /// @dev First deposit locks `MINIMUM_LIQUIDITY` LP in `lock_lp` (Config-owned ATA).
        /// @param ctx Deposit accounts (user, mints, config, vaults, ATAs, token programs).
        /// @param amount_x Exact amount of token X to transfer from the user.
        /// @param amount_y Exact amount of token Y to transfer from the user.
        /// @param min_lp Slippage guard: minimum LP tokens the user must receive.
        /// @return Result<()> Ok when transfers and mints succeed.
        pub(crate) fn handler(
            ctx: Context<Deposit>,
            amount_x: u64,
            amount_y: u64,
            min_lp: u64,
        ) -> Result<()> {
            if !(!ctx.accounts.config.locked) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::PoolLocked.name(),
                        error_code_number: AmmError::PoolLocked.into(),
                        error_msg: AmmError::PoolLocked.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/instructions/deposit.rs",
                                line: 128u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            if !(amount_x > 0 && amount_y > 0) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InvalidAmount.name(),
                        error_code_number: AmmError::InvalidAmount.into(),
                        error_msg: AmmError::InvalidAmount.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/instructions/deposit.rs",
                                line: 129u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            if ctx.accounts.user_x.key() == ctx.accounts.user_y.key() {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                            error_name: AmmError::IdenticalVaults.name(),
                            error_code_number: AmmError::IdenticalVaults.into(),
                            error_msg: AmmError::IdenticalVaults.to_string(),
                            error_origin: Some(
                                anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                    filename: "programs/amm/src/instructions/deposit.rs",
                                    line: 130u32,
                                }),
                            ),
                            compared_values: None,
                        })
                        .with_pubkeys((
                            ctx.accounts.user_x.key(),
                            ctx.accounts.user_y.key(),
                        )),
                );
            }
            let reserve_x = ctx.accounts.vault_x.amount;
            let reserve_y = ctx.accounts.vault_y.amount;
            let total_lp = ctx.accounts.mint_lp.supply;
            let (lp_to_user, lock_minimum) = lp_tokens_for_deposit(
                amount_x,
                amount_y,
                reserve_x,
                reserve_y,
                total_lp,
            )?;
            if !(lp_to_user >= min_lp) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::SlippageExceeded.name(),
                        error_code_number: AmmError::SlippageExceeded.into(),
                        error_msg: AmmError::SlippageExceeded.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/instructions/deposit.rs",
                                line: 142u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            transfer_checked(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.user_x.to_account_info(),
                        mint: ctx.accounts.mint_x.to_account_info(),
                        to: ctx.accounts.vault_x.to_account_info(),
                        authority: ctx.accounts.user.to_account_info(),
                    },
                ),
                amount_x,
                ctx.accounts.mint_x.decimals,
            )?;
            transfer_checked(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.user_y.to_account_info(),
                        mint: ctx.accounts.mint_y.to_account_info(),
                        to: ctx.accounts.vault_y.to_account_info(),
                        authority: ctx.accounts.user.to_account_info(),
                    },
                ),
                amount_y,
                ctx.accounts.mint_y.decimals,
            )?;
            let seed_bytes = ctx.accounts.config.seed.to_le_bytes();
            let bump = [ctx.accounts.config.config_bump];
            let signer_seeds: &[&[&[u8]]] = &[
                &[CONFIG_SEED, seed_bytes.as_ref(), bump.as_ref()],
            ];
            if lock_minimum {
                mint_to(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        MintTo {
                            mint: ctx.accounts.mint_lp.to_account_info(),
                            to: ctx.accounts.lock_lp.to_account_info(),
                            authority: ctx.accounts.config.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    MINIMUM_LIQUIDITY,
                )?;
            }
            mint_to(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    MintTo {
                        mint: ctx.accounts.mint_lp.to_account_info(),
                        to: ctx.accounts.user_lp.to_account_info(),
                        authority: ctx.accounts.config.to_account_info(),
                    },
                    signer_seeds,
                ),
                lp_to_user,
            )?;
            Ok(())
        }
    }
    pub mod initialize {
        //! Pool initialization: Config PDA, LP mint, and X/Y vaults.
        use anchor_lang::prelude::*;
        use anchor_spl::{
            associated_token::{self, AssociatedToken},
            token_interface::{Mint, TokenInterface},
        };
        use crate::errors::AmmError;
        use crate::state::{Config, CONFIG_SEED, LP_SEED, MAX_FEE_BPS};
        /// Accounts required to initialize a new constant-product pool.
        ///
        /// Creates the `Config` PDA and LP mint in-constraints, then creates vault ATAs
        /// in the handler **after** mint/fee checks (so invalid inputs never spend CPI
        /// on vault creation and surface `AmmError` cleanly).
        #[instruction(seed:u64)]
        pub struct Initialize<'info> {
            /// Payer and transaction signer that funds account creation.
            #[account(mut)]
            pub initializer: Signer<'info>,
            /// Mint of token X in the trading pair.
            pub mint_x: InterfaceAccount<'info, Mint>,
            /// Mint of token Y in the trading pair.
            pub mint_y: InterfaceAccount<'info, Mint>,
            /// Pool configuration PDA. Space is exactly `Config::ACCOUNT_SPACE`.
            #[account(
                init,
                payer = initializer,
                seeds = [CONFIG_SEED,
                seed.to_le_bytes().as_ref()],
                bump,
                space = Config::ACCOUNT_SPACE,
            )]
            pub config: Account<'info, Config>,
            /// LP share mint. Mint authority is the Config PDA.
            #[account(
                init,
                payer = initializer,
                seeds = [LP_SEED,
                config.key().as_ref()],
                bump,
                mint::decimals = 6,
                mint::authority = config,
                mint::token_program = token_program,
            )]
            pub mint_lp: InterfaceAccount<'info, Mint>,
            /// Vault holding reserves of token X. Created in the handler via ATA CPI.
            /// CHECK: validated and initialized in `handler` after mint distinctness checks.
            #[account(mut)]
            pub vault_x: UncheckedAccount<'info>,
            /// Vault holding reserves of token Y. Created in the handler via ATA CPI.
            /// CHECK: validated and initialized in `handler` after mint distinctness checks.
            #[account(mut)]
            pub vault_y: UncheckedAccount<'info>,
            /// SPL Token or Token-2022 program (validated Interface — anti arbitrary CPI).
            pub token_program: Interface<'info, TokenInterface>,
            /// Associated Token Account program.
            pub associated_token_program: Program<'info, AssociatedToken>,
            /// System program for account creation.
            pub system_program: Program<'info, System>,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::Accounts<'info, InitializeBumps> for Initialize<'info>
        where
            'info: 'info,
        {
            #[inline(never)]
            fn try_accounts(
                __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
                __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >],
                __ix_data: &[u8],
                __bumps: &mut InitializeBumps,
                __reallocs: &mut std::collections::BTreeSet<
                    anchor_lang::solana_program::pubkey::Pubkey,
                >,
            ) -> anchor_lang::Result<Self> {
                let mut __ix_data = __ix_data;
                struct __Args {
                    seed: u64,
                }
                impl borsh::ser::BorshSerialize for __Args
                where
                    u64: borsh::ser::BorshSerialize,
                {
                    fn serialize<W: borsh::maybestd::io::Write>(
                        &self,
                        writer: &mut W,
                    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                        borsh::BorshSerialize::serialize(&self.seed, writer)?;
                        Ok(())
                    }
                }
                impl borsh::de::BorshDeserialize for __Args
                where
                    u64: borsh::BorshDeserialize,
                {
                    fn deserialize_reader<R: borsh::maybestd::io::Read>(
                        reader: &mut R,
                    ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                        Ok(Self {
                            seed: borsh::BorshDeserialize::deserialize_reader(reader)?,
                        })
                    }
                }
                let __Args { seed } = __Args::deserialize(&mut __ix_data)
                    .map_err(|_| {
                        anchor_lang::error::ErrorCode::InstructionDidNotDeserialize
                    })?;
                let initializer: Signer = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("initializer"))?;
                let mint_x: anchor_lang::accounts::interface_account::InterfaceAccount<
                    Mint,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("mint_x"))?;
                let mint_y: anchor_lang::accounts::interface_account::InterfaceAccount<
                    Mint,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("mint_y"))?;
                if __accounts.is_empty() {
                    return Err(
                        anchor_lang::error::ErrorCode::AccountNotEnoughKeys.into(),
                    );
                }
                let config = &__accounts[0];
                *__accounts = &__accounts[1..];
                if __accounts.is_empty() {
                    return Err(
                        anchor_lang::error::ErrorCode::AccountNotEnoughKeys.into(),
                    );
                }
                let mint_lp = &__accounts[0];
                *__accounts = &__accounts[1..];
                let vault_x: UncheckedAccount = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("vault_x"))?;
                let vault_y: UncheckedAccount = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("vault_y"))?;
                let token_program: anchor_lang::accounts::interface::Interface<
                    TokenInterface,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("token_program"))?;
                let associated_token_program: anchor_lang::accounts::program::Program<
                    AssociatedToken,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("associated_token_program"))?;
                let system_program: anchor_lang::accounts::program::Program<System> = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("system_program"))?;
                let __anchor_rent = Rent::get()?;
                let (__pda_address, __bump) = Pubkey::find_program_address(
                    &[CONFIG_SEED, seed.to_le_bytes().as_ref()],
                    __program_id,
                );
                __bumps.config = __bump;
                if config.key() != __pda_address {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("config")
                            .with_pubkeys((config.key(), __pda_address)),
                    );
                }
                let config = ({
                    #[inline(never)]
                    || {
                        let actual_field = AsRef::<AccountInfo>::as_ref(&config);
                        let actual_owner = actual_field.owner;
                        let space = Config::ACCOUNT_SPACE;
                        let pa: anchor_lang::accounts::account::Account<Config> = if !false
                            || actual_owner
                                == &anchor_lang::solana_program::system_program::ID
                        {
                            let __current_lamports = config.lamports();
                            if __current_lamports == 0 {
                                let space = space;
                                let lamports = __anchor_rent.minimum_balance(space);
                                let cpi_accounts = anchor_lang::system_program::CreateAccount {
                                    from: initializer.to_account_info(),
                                    to: config.to_account_info(),
                                };
                                let cpi_context = anchor_lang::context::CpiContext::new(
                                    system_program.to_account_info(),
                                    cpi_accounts,
                                );
                                anchor_lang::system_program::create_account(
                                    cpi_context
                                        .with_signer(
                                            &[
                                                &[
                                                    CONFIG_SEED,
                                                    seed.to_le_bytes().as_ref(),
                                                    &[__bump][..],
                                                ][..],
                                            ],
                                        ),
                                    lamports,
                                    space as u64,
                                    __program_id,
                                )?;
                            } else {
                                if initializer.key() == config.key() {
                                    return Err(
                                        anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                                                error_name: anchor_lang::error::ErrorCode::TryingToInitPayerAsProgramAccount
                                                    .name(),
                                                error_code_number: anchor_lang::error::ErrorCode::TryingToInitPayerAsProgramAccount
                                                    .into(),
                                                error_msg: anchor_lang::error::ErrorCode::TryingToInitPayerAsProgramAccount
                                                    .to_string(),
                                                error_origin: Some(
                                                    anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                                        filename: "programs/amm/src/instructions/initialize.rs",
                                                        line: 17u32,
                                                    }),
                                                ),
                                                compared_values: None,
                                            })
                                            .with_pubkeys((initializer.key(), config.key())),
                                    );
                                }
                                let required_lamports = __anchor_rent
                                    .minimum_balance(space)
                                    .max(1)
                                    .saturating_sub(__current_lamports);
                                if required_lamports > 0 {
                                    let cpi_accounts = anchor_lang::system_program::Transfer {
                                        from: initializer.to_account_info(),
                                        to: config.to_account_info(),
                                    };
                                    let cpi_context = anchor_lang::context::CpiContext::new(
                                        system_program.to_account_info(),
                                        cpi_accounts,
                                    );
                                    anchor_lang::system_program::transfer(
                                        cpi_context,
                                        required_lamports,
                                    )?;
                                }
                                let cpi_accounts = anchor_lang::system_program::Allocate {
                                    account_to_allocate: config.to_account_info(),
                                };
                                let cpi_context = anchor_lang::context::CpiContext::new(
                                    system_program.to_account_info(),
                                    cpi_accounts,
                                );
                                anchor_lang::system_program::allocate(
                                    cpi_context
                                        .with_signer(
                                            &[
                                                &[
                                                    CONFIG_SEED,
                                                    seed.to_le_bytes().as_ref(),
                                                    &[__bump][..],
                                                ][..],
                                            ],
                                        ),
                                    space as u64,
                                )?;
                                let cpi_accounts = anchor_lang::system_program::Assign {
                                    account_to_assign: config.to_account_info(),
                                };
                                let cpi_context = anchor_lang::context::CpiContext::new(
                                    system_program.to_account_info(),
                                    cpi_accounts,
                                );
                                anchor_lang::system_program::assign(
                                    cpi_context
                                        .with_signer(
                                            &[
                                                &[
                                                    CONFIG_SEED,
                                                    seed.to_le_bytes().as_ref(),
                                                    &[__bump][..],
                                                ][..],
                                            ],
                                        ),
                                    __program_id,
                                )?;
                            }
                            match anchor_lang::accounts::account::Account::try_from_unchecked(
                                &config,
                            ) {
                                Ok(val) => val,
                                Err(e) => return Err(e.with_account_name("config")),
                            }
                        } else {
                            match anchor_lang::accounts::account::Account::try_from(
                                &config,
                            ) {
                                Ok(val) => val,
                                Err(e) => return Err(e.with_account_name("config")),
                            }
                        };
                        if false {
                            if space != actual_field.data_len() {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::ConstraintSpace,
                                        )
                                        .with_account_name("config")
                                        .with_values((space, actual_field.data_len())),
                                );
                            }
                            if actual_owner != __program_id {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::ConstraintOwner,
                                        )
                                        .with_account_name("config")
                                        .with_pubkeys((*actual_owner, *__program_id)),
                                );
                            }
                            {
                                let required_lamports = __anchor_rent
                                    .minimum_balance(space);
                                if pa.to_account_info().lamports() < required_lamports {
                                    return Err(
                                        anchor_lang::error::Error::from(
                                                anchor_lang::error::ErrorCode::ConstraintRentExempt,
                                            )
                                            .with_account_name("config"),
                                    );
                                }
                            }
                        }
                        Ok(pa)
                    }
                })()?;
                if !AsRef::<AccountInfo>::as_ref(&config).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("config"),
                    );
                }
                if !__anchor_rent
                    .is_exempt(
                        config.to_account_info().lamports(),
                        config.to_account_info().try_data_len()?,
                    )
                {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintRentExempt,
                            )
                            .with_account_name("config"),
                    );
                }
                let __anchor_rent = Rent::get()?;
                let (__pda_address, __bump) = Pubkey::find_program_address(
                    &[LP_SEED, config.key().as_ref()],
                    __program_id,
                );
                __bumps.mint_lp = __bump;
                if mint_lp.key() != __pda_address {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("mint_lp")
                            .with_pubkeys((mint_lp.key(), __pda_address)),
                    );
                }
                let mint_lp: anchor_lang::accounts::interface_account::InterfaceAccount<
                    Mint,
                > = ({
                    #[inline(never)]
                    || {
                        let owner_program = AsRef::<AccountInfo>::as_ref(&mint_lp).owner;
                        if !false
                            || owner_program
                                == &anchor_lang::solana_program::system_program::ID
                        {
                            let __current_lamports = mint_lp.lamports();
                            if __current_lamports == 0 {
                                let space = ::anchor_spl::token::Mint::LEN;
                                let lamports = __anchor_rent.minimum_balance(space);
                                let cpi_accounts = anchor_lang::system_program::CreateAccount {
                                    from: initializer.to_account_info(),
                                    to: mint_lp.to_account_info(),
                                };
                                let cpi_context = anchor_lang::context::CpiContext::new(
                                    system_program.to_account_info(),
                                    cpi_accounts,
                                );
                                anchor_lang::system_program::create_account(
                                    cpi_context
                                        .with_signer(
                                            &[&[LP_SEED, config.key().as_ref(), &[__bump][..]][..]],
                                        ),
                                    lamports,
                                    space as u64,
                                    &token_program.key(),
                                )?;
                            } else {
                                if initializer.key() == mint_lp.key() {
                                    return Err(
                                        anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                                                error_name: anchor_lang::error::ErrorCode::TryingToInitPayerAsProgramAccount
                                                    .name(),
                                                error_code_number: anchor_lang::error::ErrorCode::TryingToInitPayerAsProgramAccount
                                                    .into(),
                                                error_msg: anchor_lang::error::ErrorCode::TryingToInitPayerAsProgramAccount
                                                    .to_string(),
                                                error_origin: Some(
                                                    anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                                        filename: "programs/amm/src/instructions/initialize.rs",
                                                        line: 17u32,
                                                    }),
                                                ),
                                                compared_values: None,
                                            })
                                            .with_pubkeys((initializer.key(), mint_lp.key())),
                                    );
                                }
                                let required_lamports = __anchor_rent
                                    .minimum_balance(::anchor_spl::token::Mint::LEN)
                                    .max(1)
                                    .saturating_sub(__current_lamports);
                                if required_lamports > 0 {
                                    let cpi_accounts = anchor_lang::system_program::Transfer {
                                        from: initializer.to_account_info(),
                                        to: mint_lp.to_account_info(),
                                    };
                                    let cpi_context = anchor_lang::context::CpiContext::new(
                                        system_program.to_account_info(),
                                        cpi_accounts,
                                    );
                                    anchor_lang::system_program::transfer(
                                        cpi_context,
                                        required_lamports,
                                    )?;
                                }
                                let cpi_accounts = anchor_lang::system_program::Allocate {
                                    account_to_allocate: mint_lp.to_account_info(),
                                };
                                let cpi_context = anchor_lang::context::CpiContext::new(
                                    system_program.to_account_info(),
                                    cpi_accounts,
                                );
                                anchor_lang::system_program::allocate(
                                    cpi_context
                                        .with_signer(
                                            &[&[LP_SEED, config.key().as_ref(), &[__bump][..]][..]],
                                        ),
                                    ::anchor_spl::token::Mint::LEN as u64,
                                )?;
                                let cpi_accounts = anchor_lang::system_program::Assign {
                                    account_to_assign: mint_lp.to_account_info(),
                                };
                                let cpi_context = anchor_lang::context::CpiContext::new(
                                    system_program.to_account_info(),
                                    cpi_accounts,
                                );
                                anchor_lang::system_program::assign(
                                    cpi_context
                                        .with_signer(
                                            &[&[LP_SEED, config.key().as_ref(), &[__bump][..]][..]],
                                        ),
                                    &token_program.key(),
                                )?;
                            }
                            if let Some(extensions) = Option::<
                                &::anchor_spl::token_interface::ExtensionsVec,
                            >::None {
                                for e in extensions {
                                    match e {
                                        ::anchor_spl::token_interface::spl_token_2022::extension::ExtensionType::GroupPointer => {
                                            ::anchor_spl::token_interface::group_pointer_initialize(
                                                anchor_lang::context::CpiContext::new(
                                                    token_program.to_account_info(),
                                                    ::anchor_spl::token_interface::GroupPointerInitialize {
                                                        token_program_id: token_program.to_account_info(),
                                                        mint: mint_lp.to_account_info(),
                                                    },
                                                ),
                                                Option::<anchor_lang::prelude::Pubkey>::None,
                                                Option::<anchor_lang::prelude::Pubkey>::None,
                                            )?;
                                        }
                                        ::anchor_spl::token_interface::spl_token_2022::extension::ExtensionType::GroupMemberPointer => {
                                            ::anchor_spl::token_interface::group_member_pointer_initialize(
                                                anchor_lang::context::CpiContext::new(
                                                    token_program.to_account_info(),
                                                    ::anchor_spl::token_interface::GroupMemberPointerInitialize {
                                                        token_program_id: token_program.to_account_info(),
                                                        mint: mint_lp.to_account_info(),
                                                    },
                                                ),
                                                Option::<anchor_lang::prelude::Pubkey>::None,
                                                Option::<anchor_lang::prelude::Pubkey>::None,
                                            )?;
                                        }
                                        ::anchor_spl::token_interface::spl_token_2022::extension::ExtensionType::MetadataPointer => {
                                            ::anchor_spl::token_interface::metadata_pointer_initialize(
                                                anchor_lang::context::CpiContext::new(
                                                    token_program.to_account_info(),
                                                    ::anchor_spl::token_interface::MetadataPointerInitialize {
                                                        token_program_id: token_program.to_account_info(),
                                                        mint: mint_lp.to_account_info(),
                                                    },
                                                ),
                                                Option::<anchor_lang::prelude::Pubkey>::None,
                                                Option::<anchor_lang::prelude::Pubkey>::None,
                                            )?;
                                        }
                                        ::anchor_spl::token_interface::spl_token_2022::extension::ExtensionType::MintCloseAuthority => {
                                            ::anchor_spl::token_interface::mint_close_authority_initialize(
                                                anchor_lang::context::CpiContext::new(
                                                    token_program.to_account_info(),
                                                    ::anchor_spl::token_interface::MintCloseAuthorityInitialize {
                                                        token_program_id: token_program.to_account_info(),
                                                        mint: mint_lp.to_account_info(),
                                                    },
                                                ),
                                                Option::<&anchor_lang::prelude::Pubkey>::None,
                                            )?;
                                        }
                                        ::anchor_spl::token_interface::spl_token_2022::extension::ExtensionType::TransferHook => {
                                            ::anchor_spl::token_interface::transfer_hook_initialize(
                                                anchor_lang::context::CpiContext::new(
                                                    token_program.to_account_info(),
                                                    ::anchor_spl::token_interface::TransferHookInitialize {
                                                        token_program_id: token_program.to_account_info(),
                                                        mint: mint_lp.to_account_info(),
                                                    },
                                                ),
                                                Option::<anchor_lang::prelude::Pubkey>::None,
                                                Option::<anchor_lang::prelude::Pubkey>::None,
                                            )?;
                                        }
                                        ::anchor_spl::token_interface::spl_token_2022::extension::ExtensionType::NonTransferable => {
                                            ::anchor_spl::token_interface::non_transferable_mint_initialize(
                                                anchor_lang::context::CpiContext::new(
                                                    token_program.to_account_info(),
                                                    ::anchor_spl::token_interface::NonTransferableMintInitialize {
                                                        token_program_id: token_program.to_account_info(),
                                                        mint: mint_lp.to_account_info(),
                                                    },
                                                ),
                                            )?;
                                        }
                                        ::anchor_spl::token_interface::spl_token_2022::extension::ExtensionType::PermanentDelegate => {
                                            ::anchor_spl::token_interface::permanent_delegate_initialize(
                                                anchor_lang::context::CpiContext::new(
                                                    token_program.to_account_info(),
                                                    ::anchor_spl::token_interface::PermanentDelegateInitialize {
                                                        token_program_id: token_program.to_account_info(),
                                                        mint: mint_lp.to_account_info(),
                                                    },
                                                ),
                                                Option::<&anchor_lang::prelude::Pubkey>::None.unwrap(),
                                            )?;
                                        }
                                        _ => {
                                            ::core::panicking::panic_fmt(
                                                format_args!(
                                                    "not implemented: {0}",
                                                    format_args!("{0:?}", e),
                                                ),
                                            );
                                        }
                                    }
                                }
                            }
                            let cpi_program = token_program.to_account_info();
                            let accounts = ::anchor_spl::token_interface::InitializeMint2 {
                                mint: mint_lp.to_account_info(),
                            };
                            let cpi_ctx = anchor_lang::context::CpiContext::new(
                                cpi_program,
                                accounts,
                            );
                            ::anchor_spl::token_interface::initialize_mint2(
                                cpi_ctx,
                                6,
                                &config.key(),
                                Option::<&anchor_lang::prelude::Pubkey>::None,
                            )?;
                        }
                        let pa: anchor_lang::accounts::interface_account::InterfaceAccount<
                            Mint,
                        > = match anchor_lang::accounts::interface_account::InterfaceAccount::try_from_unchecked(
                            &mint_lp,
                        ) {
                            Ok(val) => val,
                            Err(e) => return Err(e.with_account_name("mint_lp")),
                        };
                        if false {
                            if pa.mint_authority
                                != anchor_lang::solana_program::program_option::COption::Some(
                                    config.key(),
                                )
                            {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::ConstraintMintMintAuthority,
                                        )
                                        .with_account_name("mint_lp"),
                                );
                            }
                            if pa
                                .freeze_authority
                                .as_ref()
                                .map(|fa| {
                                    Option::<&anchor_lang::prelude::Pubkey>::None
                                        .as_ref()
                                        .map(|expected_fa| fa != *expected_fa)
                                        .unwrap_or(true)
                                })
                                .unwrap_or(
                                    Option::<&anchor_lang::prelude::Pubkey>::None.is_some(),
                                )
                            {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::ConstraintMintFreezeAuthority,
                                        )
                                        .with_account_name("mint_lp"),
                                );
                            }
                            if pa.decimals != 6 {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::ConstraintMintDecimals,
                                        )
                                        .with_account_name("mint_lp")
                                        .with_values((pa.decimals, 6)),
                                );
                            }
                            if owner_program != &token_program.key() {
                                return Err(
                                    anchor_lang::error::Error::from(
                                            anchor_lang::error::ErrorCode::ConstraintMintTokenProgram,
                                        )
                                        .with_account_name("mint_lp")
                                        .with_pubkeys((*owner_program, token_program.key())),
                                );
                            }
                        }
                        Ok(pa)
                    }
                })()?;
                if !AsRef::<AccountInfo>::as_ref(&mint_lp).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("mint_lp"),
                    );
                }
                if !__anchor_rent
                    .is_exempt(
                        mint_lp.to_account_info().lamports(),
                        mint_lp.to_account_info().try_data_len()?,
                    )
                {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintRentExempt,
                            )
                            .with_account_name("mint_lp"),
                    );
                }
                if !AsRef::<AccountInfo>::as_ref(&initializer).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("initializer"),
                    );
                }
                if !AsRef::<AccountInfo>::as_ref(&vault_x).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("vault_x"),
                    );
                }
                if !AsRef::<AccountInfo>::as_ref(&vault_y).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("vault_y"),
                    );
                }
                Ok(Initialize {
                    initializer,
                    mint_x,
                    mint_y,
                    config,
                    mint_lp,
                    vault_x,
                    vault_y,
                    token_program,
                    associated_token_program,
                    system_program,
                })
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for Initialize<'info>
        where
            'info: 'info,
        {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos.extend(self.initializer.to_account_infos());
                account_infos.extend(self.mint_x.to_account_infos());
                account_infos.extend(self.mint_y.to_account_infos());
                account_infos.extend(self.config.to_account_infos());
                account_infos.extend(self.mint_lp.to_account_infos());
                account_infos.extend(self.vault_x.to_account_infos());
                account_infos.extend(self.vault_y.to_account_infos());
                account_infos.extend(self.token_program.to_account_infos());
                account_infos.extend(self.associated_token_program.to_account_infos());
                account_infos.extend(self.system_program.to_account_infos());
                account_infos
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for Initialize<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas.extend(self.initializer.to_account_metas(None));
                account_metas.extend(self.mint_x.to_account_metas(None));
                account_metas.extend(self.mint_y.to_account_metas(None));
                account_metas.extend(self.config.to_account_metas(None));
                account_metas.extend(self.mint_lp.to_account_metas(None));
                account_metas.extend(self.vault_x.to_account_metas(None));
                account_metas.extend(self.vault_y.to_account_metas(None));
                account_metas.extend(self.token_program.to_account_metas(None));
                account_metas
                    .extend(self.associated_token_program.to_account_metas(None));
                account_metas.extend(self.system_program.to_account_metas(None));
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::AccountsExit<'info> for Initialize<'info>
        where
            'info: 'info,
        {
            fn exit(
                &self,
                program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            ) -> anchor_lang::Result<()> {
                anchor_lang::AccountsExit::exit(&self.initializer, program_id)
                    .map_err(|e| e.with_account_name("initializer"))?;
                anchor_lang::AccountsExit::exit(&self.config, program_id)
                    .map_err(|e| e.with_account_name("config"))?;
                anchor_lang::AccountsExit::exit(&self.mint_lp, program_id)
                    .map_err(|e| e.with_account_name("mint_lp"))?;
                anchor_lang::AccountsExit::exit(&self.vault_x, program_id)
                    .map_err(|e| e.with_account_name("vault_x"))?;
                anchor_lang::AccountsExit::exit(&self.vault_y, program_id)
                    .map_err(|e| e.with_account_name("vault_y"))?;
                Ok(())
            }
        }
        pub struct InitializeBumps {
            pub config: u8,
            pub mint_lp: u8,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for InitializeBumps {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "InitializeBumps",
                    "config",
                    &self.config,
                    "mint_lp",
                    &&self.mint_lp,
                )
            }
        }
        impl Default for InitializeBumps {
            fn default() -> Self {
                InitializeBumps {
                    config: u8::MAX,
                    mint_lp: u8::MAX,
                }
            }
        }
        impl<'info> anchor_lang::Bumps for Initialize<'info>
        where
            'info: 'info,
        {
            type Bumps = InitializeBumps;
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
        /// instead of an `AccountInfo`. This is useful for clients that want
        /// to generate a list of accounts, without explicitly knowing the
        /// order all the fields should be in.
        ///
        /// To access the struct in this module, one should use the sibling
        /// `accounts` module (also generated), which re-exports this.
        pub(crate) mod __client_accounts_initialize {
            use super::*;
            use anchor_lang::prelude::borsh;
            /// Generated client accounts for [`Initialize`].
            pub struct Initialize {
                ///Payer and transaction signer that funds account creation.
                pub initializer: Pubkey,
                ///Mint of token X in the trading pair.
                pub mint_x: Pubkey,
                ///Mint of token Y in the trading pair.
                pub mint_y: Pubkey,
                ///Pool configuration PDA. Space is exactly `Config::ACCOUNT_SPACE`.
                pub config: Pubkey,
                ///LP share mint. Mint authority is the Config PDA.
                pub mint_lp: Pubkey,
                ///Vault holding reserves of token X. Created in the handler via ATA CPI.
                pub vault_x: Pubkey,
                ///Vault holding reserves of token Y. Created in the handler via ATA CPI.
                pub vault_y: Pubkey,
                ///SPL Token or Token-2022 program (validated Interface — anti arbitrary CPI).
                pub token_program: Pubkey,
                ///Associated Token Account program.
                pub associated_token_program: Pubkey,
                ///System program for account creation.
                pub system_program: Pubkey,
            }
            impl borsh::ser::BorshSerialize for Initialize
            where
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.initializer, writer)?;
                    borsh::BorshSerialize::serialize(&self.mint_x, writer)?;
                    borsh::BorshSerialize::serialize(&self.mint_y, writer)?;
                    borsh::BorshSerialize::serialize(&self.config, writer)?;
                    borsh::BorshSerialize::serialize(&self.mint_lp, writer)?;
                    borsh::BorshSerialize::serialize(&self.vault_x, writer)?;
                    borsh::BorshSerialize::serialize(&self.vault_y, writer)?;
                    borsh::BorshSerialize::serialize(&self.token_program, writer)?;
                    borsh::BorshSerialize::serialize(
                        &self.associated_token_program,
                        writer,
                    )?;
                    borsh::BorshSerialize::serialize(&self.system_program, writer)?;
                    Ok(())
                }
            }
            #[automatically_derived]
            impl anchor_lang::ToAccountMetas for Initialize {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.initializer,
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.mint_x,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.mint_y,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.config,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.mint_lp,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.vault_x,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.vault_y,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.token_program,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.associated_token_program,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.system_program,
                                false,
                            ),
                        );
                    account_metas
                }
            }
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a CPI struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is an
        /// AccountInfo.
        ///
        /// To access the struct in this module, one should use the sibling
        /// [`cpi::accounts`] module (also generated), which re-exports this.
        pub(crate) mod __cpi_client_accounts_initialize {
            use super::*;
            /// Generated CPI struct of the accounts for [`Initialize`].
            pub struct Initialize<'info> {
                ///Payer and transaction signer that funds account creation.
                pub initializer: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Mint of token X in the trading pair.
                pub mint_x: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Mint of token Y in the trading pair.
                pub mint_y: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Pool configuration PDA. Space is exactly `Config::ACCOUNT_SPACE`.
                pub config: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///LP share mint. Mint authority is the Config PDA.
                pub mint_lp: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Vault holding reserves of token X. Created in the handler via ATA CPI.
                pub vault_x: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Vault holding reserves of token Y. Created in the handler via ATA CPI.
                pub vault_y: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///SPL Token or Token-2022 program (validated Interface — anti arbitrary CPI).
                pub token_program: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Associated Token Account program.
                pub associated_token_program: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///System program for account creation.
                pub system_program: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountMetas for Initialize<'info> {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.initializer),
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.mint_x),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.mint_y),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.config),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.mint_lp),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.vault_x),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.vault_y),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.token_program),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.associated_token_program),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.system_program),
                                false,
                            ),
                        );
                    account_metas
                }
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountInfos<'info> for Initialize<'info> {
                fn to_account_infos(
                    &self,
                ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                    let mut account_infos = ::alloc::vec::Vec::new();
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.initializer,
                            ),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.mint_x),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.mint_y),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.config),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.mint_lp),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.vault_x),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.vault_y),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.token_program,
                            ),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.associated_token_program,
                            ),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.system_program,
                            ),
                        );
                    account_infos
                }
            }
        }
        /// @notice Initializes a new AMM pool (Config, LP mint, vaults X/Y).
        /// @dev Persists canonical bumps in `Config` for later instructions.
        /// @param ctx Accounts: initializer, mints, config PDA, mint_lp, vaults, token programs.
        /// @param seed Client seed used in the Config PDA derivation.
        /// @param fee Swap fee in basis points (`<= MAX_FEE_BPS`).
        /// @param authority Optional admin pubkey; `None` leaves the pool without an admin.
        /// @return Result<()> Ok if accounts were created and Config was written.
        pub(crate) fn handler(
            ctx: Context<Initialize>,
            seed: u64,
            fee: u16,
            authority: Option<Pubkey>,
        ) -> Result<()> {
            if !(fee <= MAX_FEE_BPS) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InvalidFee.name(),
                        error_code_number: AmmError::InvalidFee.into(),
                        error_msg: AmmError::InvalidFee.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/instructions/initialize.rs",
                                line: 85u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            if ctx.accounts.mint_x.key() == ctx.accounts.mint_y.key() {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                            error_name: AmmError::IdenticalMints.name(),
                            error_code_number: AmmError::IdenticalMints.into(),
                            error_msg: AmmError::IdenticalMints.to_string(),
                            error_origin: Some(
                                anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                    filename: "programs/amm/src/instructions/initialize.rs",
                                    line: 86u32,
                                }),
                            ),
                            compared_values: None,
                        })
                        .with_pubkeys((
                            ctx.accounts.mint_x.key(),
                            ctx.accounts.mint_y.key(),
                        )),
                );
            }
            if ctx.accounts.vault_x.key() == ctx.accounts.vault_y.key() {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                            error_name: AmmError::IdenticalVaults.name(),
                            error_code_number: AmmError::IdenticalVaults.into(),
                            error_msg: AmmError::IdenticalVaults.to_string(),
                            error_origin: Some(
                                anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                    filename: "programs/amm/src/instructions/initialize.rs",
                                    line: 91u32,
                                }),
                            ),
                            compared_values: None,
                        })
                        .with_pubkeys((
                            ctx.accounts.vault_x.key(),
                            ctx.accounts.vault_y.key(),
                        )),
                );
            }
            associated_token::create(
                CpiContext::new(
                    ctx.accounts.associated_token_program.to_account_info(),
                    associated_token::Create {
                        payer: ctx.accounts.initializer.to_account_info(),
                        associated_token: ctx.accounts.vault_x.to_account_info(),
                        authority: ctx.accounts.config.to_account_info(),
                        mint: ctx.accounts.mint_x.to_account_info(),
                        system_program: ctx.accounts.system_program.to_account_info(),
                        token_program: ctx.accounts.token_program.to_account_info(),
                    },
                ),
            )?;
            associated_token::create(
                CpiContext::new(
                    ctx.accounts.associated_token_program.to_account_info(),
                    associated_token::Create {
                        payer: ctx.accounts.initializer.to_account_info(),
                        associated_token: ctx.accounts.vault_y.to_account_info(),
                        authority: ctx.accounts.config.to_account_info(),
                        mint: ctx.accounts.mint_y.to_account_info(),
                        system_program: ctx.accounts.system_program.to_account_info(),
                        token_program: ctx.accounts.token_program.to_account_info(),
                    },
                ),
            )?;
            let expected_vault_x = associated_token::get_associated_token_address_with_program_id(
                &ctx.accounts.config.key(),
                &ctx.accounts.mint_x.key(),
                &ctx.accounts.token_program.key(),
            );
            let expected_vault_y = associated_token::get_associated_token_address_with_program_id(
                &ctx.accounts.config.key(),
                &ctx.accounts.mint_y.key(),
                &ctx.accounts.token_program.key(),
            );
            if ctx.accounts.vault_x.key() != expected_vault_x {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                            error_name: anchor_lang::error::ErrorCode::RequireKeysEqViolated
                                .name(),
                            error_code_number: anchor_lang::error::ErrorCode::RequireKeysEqViolated
                                .into(),
                            error_msg: anchor_lang::error::ErrorCode::RequireKeysEqViolated
                                .to_string(),
                            error_origin: Some(
                                anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                    filename: "programs/amm/src/instructions/initialize.rs",
                                    line: 133u32,
                                }),
                            ),
                            compared_values: None,
                        })
                        .with_pubkeys((ctx.accounts.vault_x.key(), expected_vault_x)),
                );
            }
            if ctx.accounts.vault_y.key() != expected_vault_y {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                            error_name: anchor_lang::error::ErrorCode::RequireKeysEqViolated
                                .name(),
                            error_code_number: anchor_lang::error::ErrorCode::RequireKeysEqViolated
                                .into(),
                            error_msg: anchor_lang::error::ErrorCode::RequireKeysEqViolated
                                .to_string(),
                            error_origin: Some(
                                anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                    filename: "programs/amm/src/instructions/initialize.rs",
                                    line: 134u32,
                                }),
                            ),
                            compared_values: None,
                        })
                        .with_pubkeys((ctx.accounts.vault_y.key(), expected_vault_y)),
                );
            }
            let config = &mut ctx.accounts.config;
            config.authority = authority;
            config.mint_x = ctx.accounts.mint_x.key();
            config.mint_y = ctx.accounts.mint_y.key();
            config.mint_lp = ctx.accounts.mint_lp.key();
            config.seed = seed;
            config.fee = fee;
            config.locked = false;
            config.config_bump = ctx.bumps.config;
            config.lp_bump = ctx.bumps.mint_lp;
            Ok(())
        }
    }
    pub mod swap {
        //! Constant-product swap with fee on input.
        use anchor_lang::prelude::*;
        use anchor_spl::token_interface::{
            transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
        };
        use crate::errors::AmmError;
        use crate::helpers::amount_out_for_swap;
        use crate::state::{Config, CONFIG_SEED};
        /// Accounts for swapping between token X and token Y.
        ///
        /// Large token accounts are `Box`ed to stay under the BPF 4KB stack limit.
        pub struct Swap<'info> {
            /// Trader funding the input transfer and receiving the output.
            pub user: Signer<'info>,
            /// Mint of token X (must match `config.mint_x`).
            pub mint_x: Box<InterfaceAccount<'info, Mint>>,
            /// Mint of token Y (must match `config.mint_y`).
            pub mint_y: Box<InterfaceAccount<'info, Mint>>,
            /// Pool config PDA. Uses stored canonical bumps (no client-supplied bump).
            #[account(
                has_one = mint_x,
                has_one = mint_y,
                seeds = [CONFIG_SEED,
                config.seed.to_le_bytes().as_ref()],
                bump = config.config_bump,
            )]
            pub config: Box<Account<'info, Config>>,
            /// Pool vault for token X.
            #[account(
                mut,
                associated_token::mint = mint_x,
                associated_token::authority = config,
                associated_token::token_program = token_program,
            )]
            pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,
            /// Pool vault for token Y.
            #[account(
                mut,
                associated_token::mint = mint_y,
                associated_token::authority = config,
                associated_token::token_program = token_program,
                constraint = vault_x.key()!= vault_y.key()@AmmError::IdenticalVaults
            )]
            pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,
            /// User ATA for token X.
            #[account(
                mut,
                associated_token::mint = mint_x,
                associated_token::authority = user,
                associated_token::token_program = token_program,
            )]
            pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,
            /// User ATA for token Y.
            #[account(
                mut,
                associated_token::mint = mint_y,
                associated_token::authority = user,
                associated_token::token_program = token_program,
                constraint = user_x.key()!= user_y.key()@AmmError::IdenticalVaults
            )]
            pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,
            /// SPL Token or Token-2022 program.
            pub token_program: Interface<'info, TokenInterface>,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::Accounts<'info, SwapBumps> for Swap<'info>
        where
            'info: 'info,
        {
            #[inline(never)]
            fn try_accounts(
                __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
                __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >],
                __ix_data: &[u8],
                __bumps: &mut SwapBumps,
                __reallocs: &mut std::collections::BTreeSet<
                    anchor_lang::solana_program::pubkey::Pubkey,
                >,
            ) -> anchor_lang::Result<Self> {
                let user: Signer = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("user"))?;
                let mint_x: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<Mint>,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("mint_x"))?;
                let mint_y: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<Mint>,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("mint_y"))?;
                let config: Box<anchor_lang::accounts::account::Account<Config>> = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("config"))?;
                let vault_x: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("vault_x"))?;
                let vault_y: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("vault_y"))?;
                let user_x: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("user_x"))?;
                let user_y: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("user_y"))?;
                let token_program: anchor_lang::accounts::interface::Interface<
                    TokenInterface,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("token_program"))?;
                let __pda_address = Pubkey::create_program_address(
                        &[
                            CONFIG_SEED,
                            config.seed.to_le_bytes().as_ref(),
                            &[config.config_bump][..],
                        ],
                        &__program_id,
                    )
                    .map_err(|_| {
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("config")
                    })?;
                if config.key() != __pda_address {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("config")
                            .with_pubkeys((config.key(), __pda_address)),
                    );
                }
                {
                    let my_key = config.mint_x;
                    let target_key = mint_x.key();
                    if my_key != target_key {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintHasOne,
                                )
                                .with_account_name("config")
                                .with_pubkeys((my_key, target_key)),
                        );
                    }
                }
                {
                    let my_key = config.mint_y;
                    let target_key = mint_y.key();
                    if my_key != target_key {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintHasOne,
                                )
                                .with_account_name("config")
                                .with_pubkeys((my_key, target_key)),
                        );
                    }
                }
                {
                    if AsRef::<AccountInfo>::as_ref(vault_x.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = vault_x.owner;
                    let wallet_address = config.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("vault_x")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_x.key(),
                        &token_program.key(),
                    );
                    let my_key = vault_x.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("vault_x")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(vault_x.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("vault_x"),
                    );
                }
                {
                    if AsRef::<AccountInfo>::as_ref(vault_y.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = vault_y.owner;
                    let wallet_address = config.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("vault_y")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_y.key(),
                        &token_program.key(),
                    );
                    let my_key = vault_y.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("vault_y")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(vault_y.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("vault_y"),
                    );
                }
                if !(vault_x.key() != vault_y.key()) {
                    return Err(
                        anchor_lang::error::Error::from(AmmError::IdenticalVaults)
                            .with_account_name("vault_y"),
                    );
                }
                {
                    if AsRef::<AccountInfo>::as_ref(user_x.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = user_x.owner;
                    let wallet_address = user.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("user_x")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_x.key(),
                        &token_program.key(),
                    );
                    let my_key = user_x.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("user_x")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(user_x.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("user_x"),
                    );
                }
                {
                    if AsRef::<AccountInfo>::as_ref(user_y.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = user_y.owner;
                    let wallet_address = user.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("user_y")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_y.key(),
                        &token_program.key(),
                    );
                    let my_key = user_y.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("user_y")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(user_y.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("user_y"),
                    );
                }
                if !(user_x.key() != user_y.key()) {
                    return Err(
                        anchor_lang::error::Error::from(AmmError::IdenticalVaults)
                            .with_account_name("user_y"),
                    );
                }
                Ok(Swap {
                    user,
                    mint_x,
                    mint_y,
                    config,
                    vault_x,
                    vault_y,
                    user_x,
                    user_y,
                    token_program,
                })
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for Swap<'info>
        where
            'info: 'info,
        {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos.extend(self.user.to_account_infos());
                account_infos.extend(self.mint_x.to_account_infos());
                account_infos.extend(self.mint_y.to_account_infos());
                account_infos.extend(self.config.to_account_infos());
                account_infos.extend(self.vault_x.to_account_infos());
                account_infos.extend(self.vault_y.to_account_infos());
                account_infos.extend(self.user_x.to_account_infos());
                account_infos.extend(self.user_y.to_account_infos());
                account_infos.extend(self.token_program.to_account_infos());
                account_infos
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for Swap<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas.extend(self.user.to_account_metas(None));
                account_metas.extend(self.mint_x.to_account_metas(None));
                account_metas.extend(self.mint_y.to_account_metas(None));
                account_metas.extend(self.config.to_account_metas(None));
                account_metas.extend(self.vault_x.to_account_metas(None));
                account_metas.extend(self.vault_y.to_account_metas(None));
                account_metas.extend(self.user_x.to_account_metas(None));
                account_metas.extend(self.user_y.to_account_metas(None));
                account_metas.extend(self.token_program.to_account_metas(None));
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::AccountsExit<'info> for Swap<'info>
        where
            'info: 'info,
        {
            fn exit(
                &self,
                program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            ) -> anchor_lang::Result<()> {
                anchor_lang::AccountsExit::exit(&self.vault_x, program_id)
                    .map_err(|e| e.with_account_name("vault_x"))?;
                anchor_lang::AccountsExit::exit(&self.vault_y, program_id)
                    .map_err(|e| e.with_account_name("vault_y"))?;
                anchor_lang::AccountsExit::exit(&self.user_x, program_id)
                    .map_err(|e| e.with_account_name("user_x"))?;
                anchor_lang::AccountsExit::exit(&self.user_y, program_id)
                    .map_err(|e| e.with_account_name("user_y"))?;
                Ok(())
            }
        }
        pub struct SwapBumps {}
        #[automatically_derived]
        impl ::core::fmt::Debug for SwapBumps {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "SwapBumps")
            }
        }
        impl Default for SwapBumps {
            fn default() -> Self {
                SwapBumps {}
            }
        }
        impl<'info> anchor_lang::Bumps for Swap<'info>
        where
            'info: 'info,
        {
            type Bumps = SwapBumps;
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
        /// instead of an `AccountInfo`. This is useful for clients that want
        /// to generate a list of accounts, without explicitly knowing the
        /// order all the fields should be in.
        ///
        /// To access the struct in this module, one should use the sibling
        /// `accounts` module (also generated), which re-exports this.
        pub(crate) mod __client_accounts_swap {
            use super::*;
            use anchor_lang::prelude::borsh;
            /// Generated client accounts for [`Swap`].
            pub struct Swap {
                ///Trader funding the input transfer and receiving the output.
                pub user: Pubkey,
                ///Mint of token X (must match `config.mint_x`).
                pub mint_x: Pubkey,
                ///Mint of token Y (must match `config.mint_y`).
                pub mint_y: Pubkey,
                ///Pool config PDA. Uses stored canonical bumps (no client-supplied bump).
                pub config: Pubkey,
                ///Pool vault for token X.
                pub vault_x: Pubkey,
                ///Pool vault for token Y.
                pub vault_y: Pubkey,
                ///User ATA for token X.
                pub user_x: Pubkey,
                ///User ATA for token Y.
                pub user_y: Pubkey,
                ///SPL Token or Token-2022 program.
                pub token_program: Pubkey,
            }
            impl borsh::ser::BorshSerialize for Swap
            where
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.user, writer)?;
                    borsh::BorshSerialize::serialize(&self.mint_x, writer)?;
                    borsh::BorshSerialize::serialize(&self.mint_y, writer)?;
                    borsh::BorshSerialize::serialize(&self.config, writer)?;
                    borsh::BorshSerialize::serialize(&self.vault_x, writer)?;
                    borsh::BorshSerialize::serialize(&self.vault_y, writer)?;
                    borsh::BorshSerialize::serialize(&self.user_x, writer)?;
                    borsh::BorshSerialize::serialize(&self.user_y, writer)?;
                    borsh::BorshSerialize::serialize(&self.token_program, writer)?;
                    Ok(())
                }
            }
            #[automatically_derived]
            impl anchor_lang::ToAccountMetas for Swap {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.user,
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.mint_x,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.mint_y,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.config,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.vault_x,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.vault_y,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.user_x,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.user_y,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.token_program,
                                false,
                            ),
                        );
                    account_metas
                }
            }
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a CPI struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is an
        /// AccountInfo.
        ///
        /// To access the struct in this module, one should use the sibling
        /// [`cpi::accounts`] module (also generated), which re-exports this.
        pub(crate) mod __cpi_client_accounts_swap {
            use super::*;
            /// Generated CPI struct of the accounts for [`Swap`].
            pub struct Swap<'info> {
                ///Trader funding the input transfer and receiving the output.
                pub user: anchor_lang::solana_program::account_info::AccountInfo<'info>,
                ///Mint of token X (must match `config.mint_x`).
                pub mint_x: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Mint of token Y (must match `config.mint_y`).
                pub mint_y: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Pool config PDA. Uses stored canonical bumps (no client-supplied bump).
                pub config: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Pool vault for token X.
                pub vault_x: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Pool vault for token Y.
                pub vault_y: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///User ATA for token X.
                pub user_x: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///User ATA for token Y.
                pub user_y: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///SPL Token or Token-2022 program.
                pub token_program: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountMetas for Swap<'info> {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.user),
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.mint_x),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.mint_y),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.config),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.vault_x),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.vault_y),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.user_x),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.user_y),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.token_program),
                                false,
                            ),
                        );
                    account_metas
                }
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountInfos<'info> for Swap<'info> {
                fn to_account_infos(
                    &self,
                ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                    let mut account_infos = ::alloc::vec::Vec::new();
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.user),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.mint_x),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.mint_y),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.config),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.vault_x),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.vault_y),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.user_x),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.user_y),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.token_program,
                            ),
                        );
                    account_infos
                }
            }
        }
        /// @notice Swaps an exact input amount for the other token in the pool.
        /// @dev Fee is taken on input; post-swap invariant `(x+dx)*(y-dy) >= x*y` is enforced.
        /// @param ctx Swap accounts (user, mints, config, vaults, user ATAs, token program).
        /// @param is_x `true` swaps X→Y; `false` swaps Y→X.
        /// @param amount_in Exact input amount transferred from the user.
        /// @param min_amount_out Slippage guard on the output amount.
        /// @return Result<()> Ok when both transfers succeed.
        pub(crate) fn handler(
            ctx: Context<Swap>,
            is_x: bool,
            amount_in: u64,
            min_amount_out: u64,
        ) -> Result<()> {
            if !(!ctx.accounts.config.locked) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::PoolLocked.name(),
                        error_code_number: AmmError::PoolLocked.into(),
                        error_msg: AmmError::PoolLocked.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/instructions/swap.rs",
                                line: 90u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            if !(amount_in > 0) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InvalidAmount.name(),
                        error_code_number: AmmError::InvalidAmount.into(),
                        error_msg: AmmError::InvalidAmount.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/instructions/swap.rs",
                                line: 91u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            let (reserve_in, reserve_out, mint_in_decimals, mint_out_decimals) = if is_x {
                (
                    ctx.accounts.vault_x.amount,
                    ctx.accounts.vault_y.amount,
                    ctx.accounts.mint_x.decimals,
                    ctx.accounts.mint_y.decimals,
                )
            } else {
                (
                    ctx.accounts.vault_y.amount,
                    ctx.accounts.vault_x.amount,
                    ctx.accounts.mint_y.decimals,
                    ctx.accounts.mint_x.decimals,
                )
            };
            let amount_out = amount_out_for_swap(
                amount_in,
                reserve_in,
                reserve_out,
                ctx.accounts.config.fee,
            )?;
            if !(amount_out >= min_amount_out) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::SlippageExceeded.name(),
                        error_code_number: AmmError::SlippageExceeded.into(),
                        error_msg: AmmError::SlippageExceeded.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/instructions/swap.rs",
                                line: 111u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            let seed_bytes = ctx.accounts.config.seed.to_le_bytes();
            let bump = [ctx.accounts.config.config_bump];
            let signer_seeds: &[&[&[u8]]] = &[
                &[CONFIG_SEED, seed_bytes.as_ref(), bump.as_ref()],
            ];
            if is_x {
                transfer_checked(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        TransferChecked {
                            from: ctx.accounts.user_x.to_account_info(),
                            mint: ctx.accounts.mint_x.to_account_info(),
                            to: ctx.accounts.vault_x.to_account_info(),
                            authority: ctx.accounts.user.to_account_info(),
                        },
                    ),
                    amount_in,
                    mint_in_decimals,
                )?;
                transfer_checked(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        TransferChecked {
                            from: ctx.accounts.vault_y.to_account_info(),
                            mint: ctx.accounts.mint_y.to_account_info(),
                            to: ctx.accounts.user_y.to_account_info(),
                            authority: ctx.accounts.config.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    amount_out,
                    mint_out_decimals,
                )?;
            } else {
                transfer_checked(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        TransferChecked {
                            from: ctx.accounts.user_y.to_account_info(),
                            mint: ctx.accounts.mint_y.to_account_info(),
                            to: ctx.accounts.vault_y.to_account_info(),
                            authority: ctx.accounts.user.to_account_info(),
                        },
                    ),
                    amount_in,
                    mint_in_decimals,
                )?;
                transfer_checked(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        TransferChecked {
                            from: ctx.accounts.vault_x.to_account_info(),
                            mint: ctx.accounts.mint_x.to_account_info(),
                            to: ctx.accounts.user_x.to_account_info(),
                            authority: ctx.accounts.config.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    amount_out,
                    mint_out_decimals,
                )?;
            }
            Ok(())
        }
    }
    pub mod withdraw {
        //! Remove liquidity: burn LP and return proportional X/Y from vaults.
        use anchor_lang::prelude::*;
        use anchor_spl::token_interface::{
            burn, transfer_checked, Burn, Mint, TokenAccount, TokenInterface,
            TransferChecked,
        };
        use crate::errors::AmmError;
        use crate::helpers::amounts_for_withdraw;
        use crate::state::{Config, CONFIG_SEED, LP_SEED};
        /// Accounts for withdrawing liquidity by burning LP tokens.
        ///
        /// Large token accounts are `Box`ed to stay under the BPF 4KB stack limit.
        pub struct Withdraw<'info> {
            /// Liquidity provider burning LP and receiving X/Y.
            pub user: Signer<'info>,
            /// Mint of token X (must match `config.mint_x`).
            pub mint_x: Box<InterfaceAccount<'info, Mint>>,
            /// Mint of token Y (must match `config.mint_y`).
            pub mint_y: Box<InterfaceAccount<'info, Mint>>,
            /// LP mint controlled by the Config PDA.
            #[account(
                mut,
                seeds = [LP_SEED,
                config.key().as_ref()],
                bump = config.lp_bump,
            )]
            pub mint_lp: Box<InterfaceAccount<'info, Mint>>,
            /// Pool config PDA. Uses stored canonical bumps (no client-supplied bump).
            #[account(
                has_one = mint_x,
                has_one = mint_y,
                has_one = mint_lp,
                seeds = [CONFIG_SEED,
                config.seed.to_le_bytes().as_ref()],
                bump = config.config_bump,
            )]
            pub config: Box<Account<'info, Config>>,
            /// Pool vault for token X.
            #[account(
                mut,
                associated_token::mint = mint_x,
                associated_token::authority = config,
                associated_token::token_program = token_program,
            )]
            pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,
            /// Pool vault for token Y.
            #[account(
                mut,
                associated_token::mint = mint_y,
                associated_token::authority = config,
                associated_token::token_program = token_program,
                constraint = vault_x.key()!= vault_y.key()@AmmError::IdenticalVaults
            )]
            pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,
            /// User ATA for token X.
            #[account(
                mut,
                associated_token::mint = mint_x,
                associated_token::authority = user,
                associated_token::token_program = token_program,
            )]
            pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,
            /// User ATA for token Y.
            #[account(
                mut,
                associated_token::mint = mint_y,
                associated_token::authority = user,
                associated_token::token_program = token_program,
            )]
            pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,
            /// User ATA holding LP tokens to burn.
            #[account(
                mut,
                associated_token::mint = mint_lp,
                associated_token::authority = user,
                associated_token::token_program = token_program,
            )]
            pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,
            /// SPL Token or Token-2022 program.
            pub token_program: Interface<'info, TokenInterface>,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::Accounts<'info, WithdrawBumps> for Withdraw<'info>
        where
            'info: 'info,
        {
            #[inline(never)]
            fn try_accounts(
                __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
                __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >],
                __ix_data: &[u8],
                __bumps: &mut WithdrawBumps,
                __reallocs: &mut std::collections::BTreeSet<
                    anchor_lang::solana_program::pubkey::Pubkey,
                >,
            ) -> anchor_lang::Result<Self> {
                let user: Signer = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("user"))?;
                let mint_x: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<Mint>,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("mint_x"))?;
                let mint_y: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<Mint>,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("mint_y"))?;
                let mint_lp: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<Mint>,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("mint_lp"))?;
                let config: Box<anchor_lang::accounts::account::Account<Config>> = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("config"))?;
                let vault_x: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("vault_x"))?;
                let vault_y: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("vault_y"))?;
                let user_x: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("user_x"))?;
                let user_y: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("user_y"))?;
                let user_lp: Box<
                    anchor_lang::accounts::interface_account::InterfaceAccount<
                        TokenAccount,
                    >,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("user_lp"))?;
                let token_program: anchor_lang::accounts::interface::Interface<
                    TokenInterface,
                > = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("token_program"))?;
                let __pda_address = Pubkey::create_program_address(
                        &[LP_SEED, config.key().as_ref(), &[config.lp_bump][..]],
                        &__program_id,
                    )
                    .map_err(|_| {
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("mint_lp")
                    })?;
                if mint_lp.key() != __pda_address {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("mint_lp")
                            .with_pubkeys((mint_lp.key(), __pda_address)),
                    );
                }
                if !AsRef::<AccountInfo>::as_ref(mint_lp.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("mint_lp"),
                    );
                }
                let __pda_address = Pubkey::create_program_address(
                        &[
                            CONFIG_SEED,
                            config.seed.to_le_bytes().as_ref(),
                            &[config.config_bump][..],
                        ],
                        &__program_id,
                    )
                    .map_err(|_| {
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("config")
                    })?;
                if config.key() != __pda_address {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("config")
                            .with_pubkeys((config.key(), __pda_address)),
                    );
                }
                {
                    let my_key = config.mint_x;
                    let target_key = mint_x.key();
                    if my_key != target_key {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintHasOne,
                                )
                                .with_account_name("config")
                                .with_pubkeys((my_key, target_key)),
                        );
                    }
                }
                {
                    let my_key = config.mint_y;
                    let target_key = mint_y.key();
                    if my_key != target_key {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintHasOne,
                                )
                                .with_account_name("config")
                                .with_pubkeys((my_key, target_key)),
                        );
                    }
                }
                {
                    let my_key = config.mint_lp;
                    let target_key = mint_lp.key();
                    if my_key != target_key {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintHasOne,
                                )
                                .with_account_name("config")
                                .with_pubkeys((my_key, target_key)),
                        );
                    }
                }
                {
                    if AsRef::<AccountInfo>::as_ref(vault_x.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = vault_x.owner;
                    let wallet_address = config.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("vault_x")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_x.key(),
                        &token_program.key(),
                    );
                    let my_key = vault_x.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("vault_x")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(vault_x.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("vault_x"),
                    );
                }
                {
                    if AsRef::<AccountInfo>::as_ref(vault_y.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = vault_y.owner;
                    let wallet_address = config.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("vault_y")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_y.key(),
                        &token_program.key(),
                    );
                    let my_key = vault_y.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("vault_y")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(vault_y.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("vault_y"),
                    );
                }
                if !(vault_x.key() != vault_y.key()) {
                    return Err(
                        anchor_lang::error::Error::from(AmmError::IdenticalVaults)
                            .with_account_name("vault_y"),
                    );
                }
                {
                    if AsRef::<AccountInfo>::as_ref(user_x.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = user_x.owner;
                    let wallet_address = user.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("user_x")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_x.key(),
                        &token_program.key(),
                    );
                    let my_key = user_x.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("user_x")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(user_x.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("user_x"),
                    );
                }
                {
                    if AsRef::<AccountInfo>::as_ref(user_y.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = user_y.owner;
                    let wallet_address = user.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("user_y")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_y.key(),
                        &token_program.key(),
                    );
                    let my_key = user_y.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("user_y")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(user_y.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("user_y"),
                    );
                }
                {
                    if AsRef::<AccountInfo>::as_ref(user_lp.as_ref()).owner
                        != &token_program.key()
                    {
                        return Err(
                            anchor_lang::error::ErrorCode::ConstraintAssociatedTokenTokenProgram
                                .into(),
                        );
                    }
                    let my_owner = user_lp.owner;
                    let wallet_address = user.key();
                    if my_owner != wallet_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintTokenOwner,
                                )
                                .with_account_name("user_lp")
                                .with_pubkeys((my_owner, wallet_address)),
                        );
                    }
                    let __associated_token_address = ::anchor_spl::associated_token::get_associated_token_address_with_program_id(
                        &wallet_address,
                        &mint_lp.key(),
                        &token_program.key(),
                    );
                    let my_key = user_lp.key();
                    if my_key != __associated_token_address {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintAssociated,
                                )
                                .with_account_name("user_lp")
                                .with_pubkeys((my_key, __associated_token_address)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(user_lp.as_ref()).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("user_lp"),
                    );
                }
                Ok(Withdraw {
                    user,
                    mint_x,
                    mint_y,
                    mint_lp,
                    config,
                    vault_x,
                    vault_y,
                    user_x,
                    user_y,
                    user_lp,
                    token_program,
                })
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for Withdraw<'info>
        where
            'info: 'info,
        {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos.extend(self.user.to_account_infos());
                account_infos.extend(self.mint_x.to_account_infos());
                account_infos.extend(self.mint_y.to_account_infos());
                account_infos.extend(self.mint_lp.to_account_infos());
                account_infos.extend(self.config.to_account_infos());
                account_infos.extend(self.vault_x.to_account_infos());
                account_infos.extend(self.vault_y.to_account_infos());
                account_infos.extend(self.user_x.to_account_infos());
                account_infos.extend(self.user_y.to_account_infos());
                account_infos.extend(self.user_lp.to_account_infos());
                account_infos.extend(self.token_program.to_account_infos());
                account_infos
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for Withdraw<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas.extend(self.user.to_account_metas(None));
                account_metas.extend(self.mint_x.to_account_metas(None));
                account_metas.extend(self.mint_y.to_account_metas(None));
                account_metas.extend(self.mint_lp.to_account_metas(None));
                account_metas.extend(self.config.to_account_metas(None));
                account_metas.extend(self.vault_x.to_account_metas(None));
                account_metas.extend(self.vault_y.to_account_metas(None));
                account_metas.extend(self.user_x.to_account_metas(None));
                account_metas.extend(self.user_y.to_account_metas(None));
                account_metas.extend(self.user_lp.to_account_metas(None));
                account_metas.extend(self.token_program.to_account_metas(None));
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::AccountsExit<'info> for Withdraw<'info>
        where
            'info: 'info,
        {
            fn exit(
                &self,
                program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            ) -> anchor_lang::Result<()> {
                anchor_lang::AccountsExit::exit(&self.mint_lp, program_id)
                    .map_err(|e| e.with_account_name("mint_lp"))?;
                anchor_lang::AccountsExit::exit(&self.vault_x, program_id)
                    .map_err(|e| e.with_account_name("vault_x"))?;
                anchor_lang::AccountsExit::exit(&self.vault_y, program_id)
                    .map_err(|e| e.with_account_name("vault_y"))?;
                anchor_lang::AccountsExit::exit(&self.user_x, program_id)
                    .map_err(|e| e.with_account_name("user_x"))?;
                anchor_lang::AccountsExit::exit(&self.user_y, program_id)
                    .map_err(|e| e.with_account_name("user_y"))?;
                anchor_lang::AccountsExit::exit(&self.user_lp, program_id)
                    .map_err(|e| e.with_account_name("user_lp"))?;
                Ok(())
            }
        }
        pub struct WithdrawBumps {}
        #[automatically_derived]
        impl ::core::fmt::Debug for WithdrawBumps {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "WithdrawBumps")
            }
        }
        impl Default for WithdrawBumps {
            fn default() -> Self {
                WithdrawBumps {}
            }
        }
        impl<'info> anchor_lang::Bumps for Withdraw<'info>
        where
            'info: 'info,
        {
            type Bumps = WithdrawBumps;
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
        /// instead of an `AccountInfo`. This is useful for clients that want
        /// to generate a list of accounts, without explicitly knowing the
        /// order all the fields should be in.
        ///
        /// To access the struct in this module, one should use the sibling
        /// `accounts` module (also generated), which re-exports this.
        pub(crate) mod __client_accounts_withdraw {
            use super::*;
            use anchor_lang::prelude::borsh;
            /// Generated client accounts for [`Withdraw`].
            pub struct Withdraw {
                ///Liquidity provider burning LP and receiving X/Y.
                pub user: Pubkey,
                ///Mint of token X (must match `config.mint_x`).
                pub mint_x: Pubkey,
                ///Mint of token Y (must match `config.mint_y`).
                pub mint_y: Pubkey,
                ///LP mint controlled by the Config PDA.
                pub mint_lp: Pubkey,
                ///Pool config PDA. Uses stored canonical bumps (no client-supplied bump).
                pub config: Pubkey,
                ///Pool vault for token X.
                pub vault_x: Pubkey,
                ///Pool vault for token Y.
                pub vault_y: Pubkey,
                ///User ATA for token X.
                pub user_x: Pubkey,
                ///User ATA for token Y.
                pub user_y: Pubkey,
                ///User ATA holding LP tokens to burn.
                pub user_lp: Pubkey,
                ///SPL Token or Token-2022 program.
                pub token_program: Pubkey,
            }
            impl borsh::ser::BorshSerialize for Withdraw
            where
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.user, writer)?;
                    borsh::BorshSerialize::serialize(&self.mint_x, writer)?;
                    borsh::BorshSerialize::serialize(&self.mint_y, writer)?;
                    borsh::BorshSerialize::serialize(&self.mint_lp, writer)?;
                    borsh::BorshSerialize::serialize(&self.config, writer)?;
                    borsh::BorshSerialize::serialize(&self.vault_x, writer)?;
                    borsh::BorshSerialize::serialize(&self.vault_y, writer)?;
                    borsh::BorshSerialize::serialize(&self.user_x, writer)?;
                    borsh::BorshSerialize::serialize(&self.user_y, writer)?;
                    borsh::BorshSerialize::serialize(&self.user_lp, writer)?;
                    borsh::BorshSerialize::serialize(&self.token_program, writer)?;
                    Ok(())
                }
            }
            #[automatically_derived]
            impl anchor_lang::ToAccountMetas for Withdraw {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.user,
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.mint_x,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.mint_y,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.mint_lp,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.config,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.vault_x,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.vault_y,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.user_x,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.user_y,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.user_lp,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.token_program,
                                false,
                            ),
                        );
                    account_metas
                }
            }
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a CPI struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is an
        /// AccountInfo.
        ///
        /// To access the struct in this module, one should use the sibling
        /// [`cpi::accounts`] module (also generated), which re-exports this.
        pub(crate) mod __cpi_client_accounts_withdraw {
            use super::*;
            /// Generated CPI struct of the accounts for [`Withdraw`].
            pub struct Withdraw<'info> {
                ///Liquidity provider burning LP and receiving X/Y.
                pub user: anchor_lang::solana_program::account_info::AccountInfo<'info>,
                ///Mint of token X (must match `config.mint_x`).
                pub mint_x: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Mint of token Y (must match `config.mint_y`).
                pub mint_y: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///LP mint controlled by the Config PDA.
                pub mint_lp: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Pool config PDA. Uses stored canonical bumps (no client-supplied bump).
                pub config: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Pool vault for token X.
                pub vault_x: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///Pool vault for token Y.
                pub vault_y: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///User ATA for token X.
                pub user_x: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///User ATA for token Y.
                pub user_y: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///User ATA holding LP tokens to burn.
                pub user_lp: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                ///SPL Token or Token-2022 program.
                pub token_program: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountMetas for Withdraw<'info> {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.user),
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.mint_x),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.mint_y),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.mint_lp),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.config),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.vault_x),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.vault_y),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.user_x),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.user_y),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.user_lp),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.token_program),
                                false,
                            ),
                        );
                    account_metas
                }
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountInfos<'info> for Withdraw<'info> {
                fn to_account_infos(
                    &self,
                ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                    let mut account_infos = ::alloc::vec::Vec::new();
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.user),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.mint_x),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.mint_y),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.mint_lp),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.config),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.vault_x),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.vault_y),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.user_x),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.user_y),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.user_lp),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.token_program,
                            ),
                        );
                    account_infos
                }
            }
        }
        /// @notice Burns LP tokens and withdraws proportional X/Y from the pool vaults.
        /// @dev Floor division favors the pool; supply cannot fall below `MINIMUM_LIQUIDITY`.
        /// @param ctx Withdraw accounts (user, mints, config, vaults, user ATAs, token program).
        /// @param lp_amount Exact LP amount to burn from the user.
        /// @param min_x Slippage guard: minimum token X received.
        /// @param min_y Slippage guard: minimum token Y received.
        /// @return Result<()> Ok when burn and transfers succeed.
        pub(crate) fn handler(
            ctx: Context<Withdraw>,
            lp_amount: u64,
            min_x: u64,
            min_y: u64,
        ) -> Result<()> {
            if !(!ctx.accounts.config.locked) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::PoolLocked.name(),
                        error_code_number: AmmError::PoolLocked.into(),
                        error_msg: AmmError::PoolLocked.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/instructions/withdraw.rs",
                                line: 107u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            if !(lp_amount > 0) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InvalidAmount.name(),
                        error_code_number: AmmError::InvalidAmount.into(),
                        error_msg: AmmError::InvalidAmount.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/instructions/withdraw.rs",
                                line: 108u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            if !(ctx.accounts.user_lp.amount >= lp_amount) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::InsufficientLiquidity.name(),
                        error_code_number: AmmError::InsufficientLiquidity.into(),
                        error_msg: AmmError::InsufficientLiquidity.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/instructions/withdraw.rs",
                                line: 109u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            if ctx.accounts.user_x.key() == ctx.accounts.user_y.key() {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                            error_name: AmmError::IdenticalVaults.name(),
                            error_code_number: AmmError::IdenticalVaults.into(),
                            error_msg: AmmError::IdenticalVaults.to_string(),
                            error_origin: Some(
                                anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                    filename: "programs/amm/src/instructions/withdraw.rs",
                                    line: 113u32,
                                }),
                            ),
                            compared_values: None,
                        })
                        .with_pubkeys((
                            ctx.accounts.user_x.key(),
                            ctx.accounts.user_y.key(),
                        )),
                );
            }
            let reserve_x = ctx.accounts.vault_x.amount;
            let reserve_y = ctx.accounts.vault_y.amount;
            let total_lp = ctx.accounts.mint_lp.supply;
            let (amount_x, amount_y) = amounts_for_withdraw(
                lp_amount,
                reserve_x,
                reserve_y,
                total_lp,
            )?;
            if !(amount_x >= min_x && amount_y >= min_y) {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                        error_name: AmmError::SlippageExceeded.name(),
                        error_code_number: AmmError::SlippageExceeded.into(),
                        error_msg: AmmError::SlippageExceeded.to_string(),
                        error_origin: Some(
                            anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                filename: "programs/amm/src/instructions/withdraw.rs",
                                line: 124u32,
                            }),
                        ),
                        compared_values: None,
                    }),
                );
            }
            burn(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Burn {
                        mint: ctx.accounts.mint_lp.to_account_info(),
                        from: ctx.accounts.user_lp.to_account_info(),
                        authority: ctx.accounts.user.to_account_info(),
                    },
                ),
                lp_amount,
            )?;
            let seed_bytes = ctx.accounts.config.seed.to_le_bytes();
            let bump = [ctx.accounts.config.config_bump];
            let signer_seeds: &[&[&[u8]]] = &[
                &[CONFIG_SEED, seed_bytes.as_ref(), bump.as_ref()],
            ];
            transfer_checked(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.vault_x.to_account_info(),
                        mint: ctx.accounts.mint_x.to_account_info(),
                        to: ctx.accounts.user_x.to_account_info(),
                        authority: ctx.accounts.config.to_account_info(),
                    },
                    signer_seeds,
                ),
                amount_x,
                ctx.accounts.mint_x.decimals,
            )?;
            transfer_checked(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.vault_y.to_account_info(),
                        mint: ctx.accounts.mint_y.to_account_info(),
                        to: ctx.accounts.user_y.to_account_info(),
                        authority: ctx.accounts.config.to_account_info(),
                    },
                    signer_seeds,
                ),
                amount_y,
                ctx.accounts.mint_y.decimals,
            )?;
            Ok(())
        }
    }
    pub use deposit::*;
    pub use initialize::*;
    pub use swap::*;
    pub use withdraw::*;
}
pub mod state {
    //! On-chain account state layouts for the AMM.
    pub mod config {
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
        #[automatically_derived]
        impl anchor_lang::Space for Config {
            const INIT_SPACE: usize = 0 + (1 + 32) + 32 + 32 + 32 + 8 + 2 + 1 + 1 + 1;
        }
        impl borsh::ser::BorshSerialize for Config
        where
            Option<Pubkey>: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            u64: borsh::ser::BorshSerialize,
            u16: borsh::ser::BorshSerialize,
            bool: borsh::ser::BorshSerialize,
            u8: borsh::ser::BorshSerialize,
            u8: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.authority, writer)?;
                borsh::BorshSerialize::serialize(&self.mint_x, writer)?;
                borsh::BorshSerialize::serialize(&self.mint_y, writer)?;
                borsh::BorshSerialize::serialize(&self.mint_lp, writer)?;
                borsh::BorshSerialize::serialize(&self.seed, writer)?;
                borsh::BorshSerialize::serialize(&self.fee, writer)?;
                borsh::BorshSerialize::serialize(&self.locked, writer)?;
                borsh::BorshSerialize::serialize(&self.config_bump, writer)?;
                borsh::BorshSerialize::serialize(&self.lp_bump, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for Config
        where
            Option<Pubkey>: borsh::BorshDeserialize,
            Pubkey: borsh::BorshDeserialize,
            Pubkey: borsh::BorshDeserialize,
            Pubkey: borsh::BorshDeserialize,
            u64: borsh::BorshDeserialize,
            u16: borsh::BorshDeserialize,
            bool: borsh::BorshDeserialize,
            u8: borsh::BorshDeserialize,
            u8: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    authority: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    mint_x: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    mint_y: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    mint_lp: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    seed: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    fee: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    locked: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    config_bump: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    lp_bump: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Config {
            #[inline]
            fn clone(&self) -> Config {
                Config {
                    authority: ::core::clone::Clone::clone(&self.authority),
                    mint_x: ::core::clone::Clone::clone(&self.mint_x),
                    mint_y: ::core::clone::Clone::clone(&self.mint_y),
                    mint_lp: ::core::clone::Clone::clone(&self.mint_lp),
                    seed: ::core::clone::Clone::clone(&self.seed),
                    fee: ::core::clone::Clone::clone(&self.fee),
                    locked: ::core::clone::Clone::clone(&self.locked),
                    config_bump: ::core::clone::Clone::clone(&self.config_bump),
                    lp_bump: ::core::clone::Clone::clone(&self.lp_bump),
                }
            }
        }
        #[automatically_derived]
        impl anchor_lang::AccountSerialize for Config {
            fn try_serialize<W: std::io::Write>(
                &self,
                writer: &mut W,
            ) -> anchor_lang::Result<()> {
                if writer.write_all(Config::DISCRIMINATOR).is_err() {
                    return Err(
                        anchor_lang::error::ErrorCode::AccountDidNotSerialize.into(),
                    );
                }
                if AnchorSerialize::serialize(self, writer).is_err() {
                    return Err(
                        anchor_lang::error::ErrorCode::AccountDidNotSerialize.into(),
                    );
                }
                Ok(())
            }
        }
        #[automatically_derived]
        impl anchor_lang::AccountDeserialize for Config {
            fn try_deserialize(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
                if buf.len() < Config::DISCRIMINATOR.len() {
                    return Err(
                        anchor_lang::error::ErrorCode::AccountDiscriminatorNotFound
                            .into(),
                    );
                }
                let given_disc = &buf[..Config::DISCRIMINATOR.len()];
                if Config::DISCRIMINATOR != given_disc {
                    return Err(
                        anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                                error_name: anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch
                                    .name(),
                                error_code_number: anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch
                                    .into(),
                                error_msg: anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch
                                    .to_string(),
                                error_origin: Some(
                                    anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                        filename: "programs/amm/src/state/config.rs",
                                        line: 22u32,
                                    }),
                                ),
                                compared_values: None,
                            })
                            .with_account_name("Config"),
                    );
                }
                Self::try_deserialize_unchecked(buf)
            }
            fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
                let mut data: &[u8] = &buf[Config::DISCRIMINATOR.len()..];
                AnchorDeserialize::deserialize(&mut data)
                    .map_err(|_| {
                        anchor_lang::error::ErrorCode::AccountDidNotDeserialize.into()
                    })
            }
        }
        #[automatically_derived]
        impl anchor_lang::Discriminator for Config {
            const DISCRIMINATOR: &'static [u8] = &[155, 12, 170, 224, 30, 250, 204, 130];
        }
        #[automatically_derived]
        impl anchor_lang::Owner for Config {
            fn owner() -> Pubkey {
                crate::ID
            }
        }
        impl Config {
            /// Total account bytes including the 8-byte Anchor discriminator.
            pub const ACCOUNT_SPACE: usize = 8 + Self::INIT_SPACE;
        }
    }
    pub use config::*;
}
pub use errors::*;
pub use helpers::*;
pub use state::*;
use instructions::*;
/// The static program ID
pub static ID: anchor_lang::solana_program::pubkey::Pubkey = anchor_lang::solana_program::pubkey::Pubkey::new_from_array([
    184u8, 118u8, 30u8, 135u8, 58u8, 248u8, 180u8, 55u8, 116u8, 58u8, 23u8, 139u8, 248u8,
    218u8, 15u8, 186u8, 157u8, 197u8, 190u8, 15u8, 134u8, 228u8, 212u8, 5u8, 174u8, 38u8,
    240u8, 39u8, 43u8, 251u8, 71u8, 228u8,
]);
/// Const version of `ID`
pub const ID_CONST: anchor_lang::solana_program::pubkey::Pubkey = anchor_lang::solana_program::pubkey::Pubkey::new_from_array([
    184u8, 118u8, 30u8, 135u8, 58u8, 248u8, 180u8, 55u8, 116u8, 58u8, 23u8, 139u8, 248u8,
    218u8, 15u8, 186u8, 157u8, 197u8, 190u8, 15u8, 134u8, 228u8, 212u8, 5u8, 174u8, 38u8,
    240u8, 39u8, 43u8, 251u8, 71u8, 228u8,
]);
/// Confirms that a given pubkey is equivalent to the program ID
pub fn check_id(id: &anchor_lang::solana_program::pubkey::Pubkey) -> bool {
    id == &ID
}
/// Returns the program ID
pub fn id() -> anchor_lang::solana_program::pubkey::Pubkey {
    ID
}
/// Const version of `ID`
pub const fn id_const() -> anchor_lang::solana_program::pubkey::Pubkey {
    ID_CONST
}
use self::amm::*;
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    let (program_id, accounts, instruction_data) = unsafe {
        ::solana_program_entrypoint::deserialize(input)
    };
    match entry(program_id, &accounts, instruction_data) {
        Ok(()) => ::solana_program_entrypoint::SUCCESS,
        Err(error) => error.into(),
    }
}
/// The Anchor codegen exposes a programming model where a user defines
/// a set of methods inside of a `#[program]` module in a way similar
/// to writing RPC request handlers. The macro then generates a bunch of
/// code wrapping these user defined methods into something that can be
/// executed on Solana.
///
/// These methods fall into one category for now.
///
/// Global methods - regular methods inside of the `#[program]`.
///
/// Care must be taken by the codegen to prevent collisions between
/// methods in these different namespaces. For this reason, Anchor uses
/// a variant of sighash to perform method dispatch, rather than
/// something like a simple enum variant discriminator.
///
/// The execution flow of the generated code can be roughly outlined:
///
/// * Start program via the entrypoint.
/// * Check whether the declared program id matches the input program
///   id. If it's not, return an error.
/// * Find and invoke the method based on whether the instruction data
///   starts with the method's discriminator.
/// * Run the method handler wrapper. This wraps the code the user
///   actually wrote, deserializing the accounts, constructing the
///   context, invoking the user's code, and finally running the exit
///   routine, which typically persists account changes.
///
/// The `entry` function here, defines the standard entry to a Solana
/// program, where execution begins.
pub fn entry<'info>(
    program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    data: &[u8],
) -> anchor_lang::solana_program::entrypoint::ProgramResult {
    try_entry(program_id, accounts, data)
        .map_err(|e| {
            e.log();
            e.into()
        })
}
fn try_entry<'info>(
    program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    data: &[u8],
) -> anchor_lang::Result<()> {
    if *program_id != ID {
        return Err(anchor_lang::error::ErrorCode::DeclaredProgramIdMismatch.into());
    }
    dispatch(program_id, accounts, data)
}
/// Module representing the program.
pub mod program {
    use super::*;
    /// Type representing the program.
    pub struct Amm;
    #[automatically_derived]
    impl ::core::clone::Clone for Amm {
        #[inline]
        fn clone(&self) -> Amm {
            Amm
        }
    }
    impl anchor_lang::Id for Amm {
        fn id() -> Pubkey {
            ID
        }
    }
}
/// Performs method dispatch.
///
/// Each instruction's discriminator is checked until the given instruction data starts with
/// the current discriminator.
///
/// If a match is found, the instruction handler is called using the given instruction data
/// excluding the prepended discriminator bytes.
///
/// If no match is found, the fallback function is executed if it exists, or an error is
/// returned if it doesn't exist.
fn dispatch<'info>(
    program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    data: &[u8],
) -> anchor_lang::Result<()> {
    if data.starts_with(instruction::Initialize::DISCRIMINATOR) {
        return __private::__global::initialize(
            program_id,
            accounts,
            &data[instruction::Initialize::DISCRIMINATOR.len()..],
        );
    }
    if data.starts_with(instruction::Deposit::DISCRIMINATOR) {
        return __private::__global::deposit(
            program_id,
            accounts,
            &data[instruction::Deposit::DISCRIMINATOR.len()..],
        );
    }
    if data.starts_with(instruction::Withdraw::DISCRIMINATOR) {
        return __private::__global::withdraw(
            program_id,
            accounts,
            &data[instruction::Withdraw::DISCRIMINATOR.len()..],
        );
    }
    if data.starts_with(instruction::Swap::DISCRIMINATOR) {
        return __private::__global::swap(
            program_id,
            accounts,
            &data[instruction::Swap::DISCRIMINATOR.len()..],
        );
    }
    if data.starts_with(anchor_lang::idl::IDL_IX_TAG_LE) {
        return __private::__idl::__idl_dispatch(
            program_id,
            accounts,
            &data[anchor_lang::idl::IDL_IX_TAG_LE.len()..],
        );
    }
    if data.starts_with(anchor_lang::event::EVENT_IX_TAG_LE) {
        return Err(anchor_lang::error::ErrorCode::EventInstructionStub.into());
    }
    Err(anchor_lang::error::ErrorCode::InstructionFallbackNotFound.into())
}
/// Create a private module to not clutter the program's namespace.
/// Defines an entrypoint for each individual instruction handler
/// wrapper.
mod __private {
    use super::*;
    /// __idl mod defines handlers for injected Anchor IDL instructions.
    pub mod __idl {
        use super::*;
        #[inline(never)]
        pub fn __idl_dispatch<'info>(
            program_id: &Pubkey,
            accounts: &'info [AccountInfo<'info>],
            idl_ix_data: &[u8],
        ) -> anchor_lang::Result<()> {
            let mut accounts = accounts;
            let mut data: &[u8] = idl_ix_data;
            let ix = anchor_lang::idl::IdlInstruction::deserialize(&mut data)
                .map_err(|_| {
                    anchor_lang::error::ErrorCode::InstructionDidNotDeserialize
                })?;
            match ix {
                anchor_lang::idl::IdlInstruction::Create { data_len } => {
                    let mut bumps = <IdlCreateAccounts as anchor_lang::Bumps>::Bumps::default();
                    let mut reallocs = std::collections::BTreeSet::new();
                    let mut accounts = IdlCreateAccounts::try_accounts(
                        program_id,
                        &mut accounts,
                        &[],
                        &mut bumps,
                        &mut reallocs,
                    )?;
                    __idl_create_account(program_id, &mut accounts, data_len)?;
                    accounts.exit(program_id)?;
                }
                anchor_lang::idl::IdlInstruction::Resize { data_len } => {
                    let mut bumps = <IdlResizeAccount as anchor_lang::Bumps>::Bumps::default();
                    let mut reallocs = std::collections::BTreeSet::new();
                    let mut accounts = IdlResizeAccount::try_accounts(
                        program_id,
                        &mut accounts,
                        &[],
                        &mut bumps,
                        &mut reallocs,
                    )?;
                    __idl_resize_account(program_id, &mut accounts, data_len)?;
                    accounts.exit(program_id)?;
                }
                anchor_lang::idl::IdlInstruction::Close => {
                    let mut bumps = <IdlCloseAccount as anchor_lang::Bumps>::Bumps::default();
                    let mut reallocs = std::collections::BTreeSet::new();
                    let mut accounts = IdlCloseAccount::try_accounts(
                        program_id,
                        &mut accounts,
                        &[],
                        &mut bumps,
                        &mut reallocs,
                    )?;
                    __idl_close_account(program_id, &mut accounts)?;
                    accounts.exit(program_id)?;
                }
                anchor_lang::idl::IdlInstruction::CreateBuffer => {
                    let mut bumps = <IdlCreateBuffer as anchor_lang::Bumps>::Bumps::default();
                    let mut reallocs = std::collections::BTreeSet::new();
                    let mut accounts = IdlCreateBuffer::try_accounts(
                        program_id,
                        &mut accounts,
                        &[],
                        &mut bumps,
                        &mut reallocs,
                    )?;
                    __idl_create_buffer(program_id, &mut accounts)?;
                    accounts.exit(program_id)?;
                }
                anchor_lang::idl::IdlInstruction::Write { data } => {
                    let mut bumps = <IdlAccounts as anchor_lang::Bumps>::Bumps::default();
                    let mut reallocs = std::collections::BTreeSet::new();
                    let mut accounts = IdlAccounts::try_accounts(
                        program_id,
                        &mut accounts,
                        &[],
                        &mut bumps,
                        &mut reallocs,
                    )?;
                    __idl_write(program_id, &mut accounts, data)?;
                    accounts.exit(program_id)?;
                }
                anchor_lang::idl::IdlInstruction::SetAuthority { new_authority } => {
                    let mut bumps = <IdlAccounts as anchor_lang::Bumps>::Bumps::default();
                    let mut reallocs = std::collections::BTreeSet::new();
                    let mut accounts = IdlAccounts::try_accounts(
                        program_id,
                        &mut accounts,
                        &[],
                        &mut bumps,
                        &mut reallocs,
                    )?;
                    __idl_set_authority(program_id, &mut accounts, new_authority)?;
                    accounts.exit(program_id)?;
                }
                anchor_lang::idl::IdlInstruction::SetBuffer => {
                    let mut bumps = <IdlSetBuffer as anchor_lang::Bumps>::Bumps::default();
                    let mut reallocs = std::collections::BTreeSet::new();
                    let mut accounts = IdlSetBuffer::try_accounts(
                        program_id,
                        &mut accounts,
                        &[],
                        &mut bumps,
                        &mut reallocs,
                    )?;
                    __idl_set_buffer(program_id, &mut accounts)?;
                    accounts.exit(program_id)?;
                }
            }
            Ok(())
        }
        use anchor_lang::idl::ERASED_AUTHORITY;
        pub struct IdlAccount {
            pub authority: Pubkey,
            pub data_len: u32,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for IdlAccount {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "IdlAccount",
                    "authority",
                    &self.authority,
                    "data_len",
                    &&self.data_len,
                )
            }
        }
        impl borsh::ser::BorshSerialize for IdlAccount
        where
            Pubkey: borsh::ser::BorshSerialize,
            u32: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.authority, writer)?;
                borsh::BorshSerialize::serialize(&self.data_len, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for IdlAccount
        where
            Pubkey: borsh::BorshDeserialize,
            u32: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    authority: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    data_len: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for IdlAccount {
            #[inline]
            fn clone(&self) -> IdlAccount {
                IdlAccount {
                    authority: ::core::clone::Clone::clone(&self.authority),
                    data_len: ::core::clone::Clone::clone(&self.data_len),
                }
            }
        }
        #[automatically_derived]
        impl anchor_lang::AccountSerialize for IdlAccount {
            fn try_serialize<W: std::io::Write>(
                &self,
                writer: &mut W,
            ) -> anchor_lang::Result<()> {
                if writer.write_all(IdlAccount::DISCRIMINATOR).is_err() {
                    return Err(
                        anchor_lang::error::ErrorCode::AccountDidNotSerialize.into(),
                    );
                }
                if AnchorSerialize::serialize(self, writer).is_err() {
                    return Err(
                        anchor_lang::error::ErrorCode::AccountDidNotSerialize.into(),
                    );
                }
                Ok(())
            }
        }
        #[automatically_derived]
        impl anchor_lang::AccountDeserialize for IdlAccount {
            fn try_deserialize(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
                if buf.len() < IdlAccount::DISCRIMINATOR.len() {
                    return Err(
                        anchor_lang::error::ErrorCode::AccountDiscriminatorNotFound
                            .into(),
                    );
                }
                let given_disc = &buf[..IdlAccount::DISCRIMINATOR.len()];
                if IdlAccount::DISCRIMINATOR != given_disc {
                    return Err(
                        anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                                error_name: anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch
                                    .name(),
                                error_code_number: anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch
                                    .into(),
                                error_msg: anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch
                                    .to_string(),
                                error_origin: Some(
                                    anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                        filename: "programs/amm/src/lib.rs",
                                        line: 29u32,
                                    }),
                                ),
                                compared_values: None,
                            })
                            .with_account_name("IdlAccount"),
                    );
                }
                Self::try_deserialize_unchecked(buf)
            }
            fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
                let mut data: &[u8] = &buf[IdlAccount::DISCRIMINATOR.len()..];
                AnchorDeserialize::deserialize(&mut data)
                    .map_err(|_| {
                        anchor_lang::error::ErrorCode::AccountDidNotDeserialize.into()
                    })
            }
        }
        #[automatically_derived]
        impl anchor_lang::Discriminator for IdlAccount {
            const DISCRIMINATOR: &'static [u8] = &[24, 70, 98, 191, 58, 144, 123, 158];
        }
        impl IdlAccount {
            pub fn address(program_id: &Pubkey) -> Pubkey {
                let program_signer = Pubkey::find_program_address(&[], program_id).0;
                Pubkey::create_with_seed(&program_signer, IdlAccount::seed(), program_id)
                    .expect("Seed is always valid")
            }
            pub fn seed() -> &'static str {
                "anchor:idl"
            }
        }
        impl anchor_lang::Owner for IdlAccount {
            fn owner() -> Pubkey {
                crate::ID
            }
        }
        pub struct IdlCreateAccounts<'info> {
            #[account(signer)]
            pub from: AccountInfo<'info>,
            #[account(mut)]
            pub to: AccountInfo<'info>,
            #[account(seeds = [], bump)]
            pub base: AccountInfo<'info>,
            pub system_program: Program<'info, System>,
            #[account(executable)]
            pub program: AccountInfo<'info>,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::Accounts<'info, IdlCreateAccountsBumps>
        for IdlCreateAccounts<'info>
        where
            'info: 'info,
        {
            #[inline(never)]
            fn try_accounts(
                __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
                __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >],
                __ix_data: &[u8],
                __bumps: &mut IdlCreateAccountsBumps,
                __reallocs: &mut std::collections::BTreeSet<
                    anchor_lang::solana_program::pubkey::Pubkey,
                >,
            ) -> anchor_lang::Result<Self> {
                let from: AccountInfo = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("from"))?;
                let to: AccountInfo = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("to"))?;
                let base: AccountInfo = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("base"))?;
                let system_program: anchor_lang::accounts::program::Program<System> = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("system_program"))?;
                let program: AccountInfo = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("program"))?;
                if !&from.is_signer {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSigner,
                            )
                            .with_account_name("from"),
                    );
                }
                if !&to.is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("to"),
                    );
                }
                let (__pda_address, __bump) = Pubkey::find_program_address(
                    &[],
                    &__program_id,
                );
                __bumps.base = __bump;
                if base.key() != __pda_address {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintSeeds,
                            )
                            .with_account_name("base")
                            .with_pubkeys((base.key(), __pda_address)),
                    );
                }
                if !&program.executable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintExecutable,
                            )
                            .with_account_name("program"),
                    );
                }
                Ok(IdlCreateAccounts {
                    from,
                    to,
                    base,
                    system_program,
                    program,
                })
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for IdlCreateAccounts<'info>
        where
            'info: 'info,
        {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos.extend(self.from.to_account_infos());
                account_infos.extend(self.to.to_account_infos());
                account_infos.extend(self.base.to_account_infos());
                account_infos.extend(self.system_program.to_account_infos());
                account_infos.extend(self.program.to_account_infos());
                account_infos
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for IdlCreateAccounts<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas.extend(self.from.to_account_metas(Some(true)));
                account_metas.extend(self.to.to_account_metas(None));
                account_metas.extend(self.base.to_account_metas(None));
                account_metas.extend(self.system_program.to_account_metas(None));
                account_metas.extend(self.program.to_account_metas(None));
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::AccountsExit<'info> for IdlCreateAccounts<'info>
        where
            'info: 'info,
        {
            fn exit(
                &self,
                program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            ) -> anchor_lang::Result<()> {
                anchor_lang::AccountsExit::exit(&self.to, program_id)
                    .map_err(|e| e.with_account_name("to"))?;
                Ok(())
            }
        }
        pub struct IdlCreateAccountsBumps {
            pub base: u8,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for IdlCreateAccountsBumps {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "IdlCreateAccountsBumps",
                    "base",
                    &&self.base,
                )
            }
        }
        impl Default for IdlCreateAccountsBumps {
            fn default() -> Self {
                IdlCreateAccountsBumps {
                    base: u8::MAX,
                }
            }
        }
        impl<'info> anchor_lang::Bumps for IdlCreateAccounts<'info>
        where
            'info: 'info,
        {
            type Bumps = IdlCreateAccountsBumps;
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
        /// instead of an `AccountInfo`. This is useful for clients that want
        /// to generate a list of accounts, without explicitly knowing the
        /// order all the fields should be in.
        ///
        /// To access the struct in this module, one should use the sibling
        /// `accounts` module (also generated), which re-exports this.
        pub(crate) mod __client_accounts_idl_create_accounts {
            use super::*;
            use anchor_lang::prelude::borsh;
            /// Generated client accounts for [`IdlCreateAccounts`].
            pub struct IdlCreateAccounts {
                pub from: Pubkey,
                pub to: Pubkey,
                pub base: Pubkey,
                pub system_program: Pubkey,
                pub program: Pubkey,
            }
            impl borsh::ser::BorshSerialize for IdlCreateAccounts
            where
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.from, writer)?;
                    borsh::BorshSerialize::serialize(&self.to, writer)?;
                    borsh::BorshSerialize::serialize(&self.base, writer)?;
                    borsh::BorshSerialize::serialize(&self.system_program, writer)?;
                    borsh::BorshSerialize::serialize(&self.program, writer)?;
                    Ok(())
                }
            }
            #[automatically_derived]
            impl anchor_lang::ToAccountMetas for IdlCreateAccounts {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.from,
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.to,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.base,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.system_program,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.program,
                                false,
                            ),
                        );
                    account_metas
                }
            }
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a CPI struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is an
        /// AccountInfo.
        ///
        /// To access the struct in this module, one should use the sibling
        /// [`cpi::accounts`] module (also generated), which re-exports this.
        pub(crate) mod __cpi_client_accounts_idl_create_accounts {
            use super::*;
            /// Generated CPI struct of the accounts for [`IdlCreateAccounts`].
            pub struct IdlCreateAccounts<'info> {
                pub from: anchor_lang::solana_program::account_info::AccountInfo<'info>,
                pub to: anchor_lang::solana_program::account_info::AccountInfo<'info>,
                pub base: anchor_lang::solana_program::account_info::AccountInfo<'info>,
                pub system_program: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                pub program: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountMetas for IdlCreateAccounts<'info> {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.from),
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.to),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.base),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.system_program),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.program),
                                false,
                            ),
                        );
                    account_metas
                }
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountInfos<'info> for IdlCreateAccounts<'info> {
                fn to_account_infos(
                    &self,
                ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                    let mut account_infos = ::alloc::vec::Vec::new();
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.from),
                        );
                    account_infos
                        .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.to));
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.base),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.system_program,
                            ),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.program),
                        );
                    account_infos
                }
            }
        }
        pub struct IdlAccounts<'info> {
            #[account(mut, has_one = authority)]
            pub idl: Account<'info, IdlAccount>,
            #[account(constraint = authority.key!= &ERASED_AUTHORITY)]
            pub authority: Signer<'info>,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::Accounts<'info, IdlAccountsBumps> for IdlAccounts<'info>
        where
            'info: 'info,
        {
            #[inline(never)]
            fn try_accounts(
                __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
                __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >],
                __ix_data: &[u8],
                __bumps: &mut IdlAccountsBumps,
                __reallocs: &mut std::collections::BTreeSet<
                    anchor_lang::solana_program::pubkey::Pubkey,
                >,
            ) -> anchor_lang::Result<Self> {
                let idl: anchor_lang::accounts::account::Account<IdlAccount> = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("idl"))?;
                let authority: Signer = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("authority"))?;
                if !AsRef::<AccountInfo>::as_ref(&idl).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("idl"),
                    );
                }
                {
                    let my_key = idl.authority;
                    let target_key = authority.key();
                    if my_key != target_key {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintHasOne,
                                )
                                .with_account_name("idl")
                                .with_pubkeys((my_key, target_key)),
                        );
                    }
                }
                if !(authority.key != &ERASED_AUTHORITY) {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintRaw,
                            )
                            .with_account_name("authority"),
                    );
                }
                Ok(IdlAccounts { idl, authority })
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for IdlAccounts<'info>
        where
            'info: 'info,
        {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos.extend(self.idl.to_account_infos());
                account_infos.extend(self.authority.to_account_infos());
                account_infos
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for IdlAccounts<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas.extend(self.idl.to_account_metas(None));
                account_metas.extend(self.authority.to_account_metas(None));
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::AccountsExit<'info> for IdlAccounts<'info>
        where
            'info: 'info,
        {
            fn exit(
                &self,
                program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            ) -> anchor_lang::Result<()> {
                anchor_lang::AccountsExit::exit(&self.idl, program_id)
                    .map_err(|e| e.with_account_name("idl"))?;
                Ok(())
            }
        }
        pub struct IdlAccountsBumps {}
        #[automatically_derived]
        impl ::core::fmt::Debug for IdlAccountsBumps {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "IdlAccountsBumps")
            }
        }
        impl Default for IdlAccountsBumps {
            fn default() -> Self {
                IdlAccountsBumps {}
            }
        }
        impl<'info> anchor_lang::Bumps for IdlAccounts<'info>
        where
            'info: 'info,
        {
            type Bumps = IdlAccountsBumps;
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
        /// instead of an `AccountInfo`. This is useful for clients that want
        /// to generate a list of accounts, without explicitly knowing the
        /// order all the fields should be in.
        ///
        /// To access the struct in this module, one should use the sibling
        /// `accounts` module (also generated), which re-exports this.
        pub(crate) mod __client_accounts_idl_accounts {
            use super::*;
            use anchor_lang::prelude::borsh;
            /// Generated client accounts for [`IdlAccounts`].
            pub struct IdlAccounts {
                pub idl: Pubkey,
                pub authority: Pubkey,
            }
            impl borsh::ser::BorshSerialize for IdlAccounts
            where
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.idl, writer)?;
                    borsh::BorshSerialize::serialize(&self.authority, writer)?;
                    Ok(())
                }
            }
            #[automatically_derived]
            impl anchor_lang::ToAccountMetas for IdlAccounts {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.idl,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.authority,
                                true,
                            ),
                        );
                    account_metas
                }
            }
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a CPI struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is an
        /// AccountInfo.
        ///
        /// To access the struct in this module, one should use the sibling
        /// [`cpi::accounts`] module (also generated), which re-exports this.
        pub(crate) mod __cpi_client_accounts_idl_accounts {
            use super::*;
            /// Generated CPI struct of the accounts for [`IdlAccounts`].
            pub struct IdlAccounts<'info> {
                pub idl: anchor_lang::solana_program::account_info::AccountInfo<'info>,
                pub authority: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountMetas for IdlAccounts<'info> {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.idl),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.authority),
                                true,
                            ),
                        );
                    account_metas
                }
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountInfos<'info> for IdlAccounts<'info> {
                fn to_account_infos(
                    &self,
                ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                    let mut account_infos = ::alloc::vec::Vec::new();
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.idl),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.authority,
                            ),
                        );
                    account_infos
                }
            }
        }
        pub struct IdlResizeAccount<'info> {
            #[account(mut, has_one = authority)]
            pub idl: Account<'info, IdlAccount>,
            #[account(mut, constraint = authority.key!= &ERASED_AUTHORITY)]
            pub authority: Signer<'info>,
            pub system_program: Program<'info, System>,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::Accounts<'info, IdlResizeAccountBumps>
        for IdlResizeAccount<'info>
        where
            'info: 'info,
        {
            #[inline(never)]
            fn try_accounts(
                __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
                __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >],
                __ix_data: &[u8],
                __bumps: &mut IdlResizeAccountBumps,
                __reallocs: &mut std::collections::BTreeSet<
                    anchor_lang::solana_program::pubkey::Pubkey,
                >,
            ) -> anchor_lang::Result<Self> {
                let idl: anchor_lang::accounts::account::Account<IdlAccount> = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("idl"))?;
                let authority: Signer = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("authority"))?;
                let system_program: anchor_lang::accounts::program::Program<System> = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("system_program"))?;
                if !AsRef::<AccountInfo>::as_ref(&idl).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("idl"),
                    );
                }
                {
                    let my_key = idl.authority;
                    let target_key = authority.key();
                    if my_key != target_key {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintHasOne,
                                )
                                .with_account_name("idl")
                                .with_pubkeys((my_key, target_key)),
                        );
                    }
                }
                if !AsRef::<AccountInfo>::as_ref(&authority).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("authority"),
                    );
                }
                if !(authority.key != &ERASED_AUTHORITY) {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintRaw,
                            )
                            .with_account_name("authority"),
                    );
                }
                Ok(IdlResizeAccount {
                    idl,
                    authority,
                    system_program,
                })
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for IdlResizeAccount<'info>
        where
            'info: 'info,
        {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos.extend(self.idl.to_account_infos());
                account_infos.extend(self.authority.to_account_infos());
                account_infos.extend(self.system_program.to_account_infos());
                account_infos
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for IdlResizeAccount<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas.extend(self.idl.to_account_metas(None));
                account_metas.extend(self.authority.to_account_metas(None));
                account_metas.extend(self.system_program.to_account_metas(None));
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::AccountsExit<'info> for IdlResizeAccount<'info>
        where
            'info: 'info,
        {
            fn exit(
                &self,
                program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            ) -> anchor_lang::Result<()> {
                anchor_lang::AccountsExit::exit(&self.idl, program_id)
                    .map_err(|e| e.with_account_name("idl"))?;
                anchor_lang::AccountsExit::exit(&self.authority, program_id)
                    .map_err(|e| e.with_account_name("authority"))?;
                Ok(())
            }
        }
        pub struct IdlResizeAccountBumps {}
        #[automatically_derived]
        impl ::core::fmt::Debug for IdlResizeAccountBumps {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "IdlResizeAccountBumps")
            }
        }
        impl Default for IdlResizeAccountBumps {
            fn default() -> Self {
                IdlResizeAccountBumps {}
            }
        }
        impl<'info> anchor_lang::Bumps for IdlResizeAccount<'info>
        where
            'info: 'info,
        {
            type Bumps = IdlResizeAccountBumps;
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
        /// instead of an `AccountInfo`. This is useful for clients that want
        /// to generate a list of accounts, without explicitly knowing the
        /// order all the fields should be in.
        ///
        /// To access the struct in this module, one should use the sibling
        /// `accounts` module (also generated), which re-exports this.
        pub(crate) mod __client_accounts_idl_resize_account {
            use super::*;
            use anchor_lang::prelude::borsh;
            /// Generated client accounts for [`IdlResizeAccount`].
            pub struct IdlResizeAccount {
                pub idl: Pubkey,
                pub authority: Pubkey,
                pub system_program: Pubkey,
            }
            impl borsh::ser::BorshSerialize for IdlResizeAccount
            where
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.idl, writer)?;
                    borsh::BorshSerialize::serialize(&self.authority, writer)?;
                    borsh::BorshSerialize::serialize(&self.system_program, writer)?;
                    Ok(())
                }
            }
            #[automatically_derived]
            impl anchor_lang::ToAccountMetas for IdlResizeAccount {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.idl,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.authority,
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.system_program,
                                false,
                            ),
                        );
                    account_metas
                }
            }
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a CPI struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is an
        /// AccountInfo.
        ///
        /// To access the struct in this module, one should use the sibling
        /// [`cpi::accounts`] module (also generated), which re-exports this.
        pub(crate) mod __cpi_client_accounts_idl_resize_account {
            use super::*;
            /// Generated CPI struct of the accounts for [`IdlResizeAccount`].
            pub struct IdlResizeAccount<'info> {
                pub idl: anchor_lang::solana_program::account_info::AccountInfo<'info>,
                pub authority: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                pub system_program: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountMetas for IdlResizeAccount<'info> {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.idl),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.authority),
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.system_program),
                                false,
                            ),
                        );
                    account_metas
                }
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountInfos<'info> for IdlResizeAccount<'info> {
                fn to_account_infos(
                    &self,
                ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                    let mut account_infos = ::alloc::vec::Vec::new();
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.idl),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.authority,
                            ),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.system_program,
                            ),
                        );
                    account_infos
                }
            }
        }
        pub struct IdlCreateBuffer<'info> {
            #[account(zero)]
            pub buffer: Account<'info, IdlAccount>,
            #[account(constraint = authority.key!= &ERASED_AUTHORITY)]
            pub authority: Signer<'info>,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::Accounts<'info, IdlCreateBufferBumps>
        for IdlCreateBuffer<'info>
        where
            'info: 'info,
        {
            #[inline(never)]
            fn try_accounts(
                __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
                __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >],
                __ix_data: &[u8],
                __bumps: &mut IdlCreateBufferBumps,
                __reallocs: &mut std::collections::BTreeSet<
                    anchor_lang::solana_program::pubkey::Pubkey,
                >,
            ) -> anchor_lang::Result<Self> {
                if __accounts.is_empty() {
                    return Err(
                        anchor_lang::error::ErrorCode::AccountNotEnoughKeys.into(),
                    );
                }
                let buffer = &__accounts[0];
                *__accounts = &__accounts[1..];
                let authority: Signer = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("authority"))?;
                let __anchor_rent = Rent::get()?;
                let buffer: anchor_lang::accounts::account::Account<IdlAccount> = {
                    let mut __data: &[u8] = &buffer.try_borrow_data()?;
                    let __disc = &__data[..IdlAccount::DISCRIMINATOR.len()];
                    let __has_disc = __disc.iter().any(|b| *b != 0);
                    if __has_disc {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintZero,
                                )
                                .with_account_name("buffer"),
                        );
                    }
                    match anchor_lang::accounts::account::Account::try_from_unchecked(
                        &buffer,
                    ) {
                        Ok(val) => val,
                        Err(e) => return Err(e.with_account_name("buffer")),
                    }
                };
                if !AsRef::<AccountInfo>::as_ref(&buffer).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("buffer"),
                    );
                }
                if !__anchor_rent
                    .is_exempt(
                        buffer.to_account_info().lamports(),
                        buffer.to_account_info().try_data_len()?,
                    )
                {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintRentExempt,
                            )
                            .with_account_name("buffer"),
                    );
                }
                if !(authority.key != &ERASED_AUTHORITY) {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintRaw,
                            )
                            .with_account_name("authority"),
                    );
                }
                Ok(IdlCreateBuffer {
                    buffer,
                    authority,
                })
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for IdlCreateBuffer<'info>
        where
            'info: 'info,
        {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos.extend(self.buffer.to_account_infos());
                account_infos.extend(self.authority.to_account_infos());
                account_infos
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for IdlCreateBuffer<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas.extend(self.buffer.to_account_metas(None));
                account_metas.extend(self.authority.to_account_metas(None));
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::AccountsExit<'info> for IdlCreateBuffer<'info>
        where
            'info: 'info,
        {
            fn exit(
                &self,
                program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            ) -> anchor_lang::Result<()> {
                anchor_lang::AccountsExit::exit(&self.buffer, program_id)
                    .map_err(|e| e.with_account_name("buffer"))?;
                Ok(())
            }
        }
        pub struct IdlCreateBufferBumps {}
        #[automatically_derived]
        impl ::core::fmt::Debug for IdlCreateBufferBumps {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "IdlCreateBufferBumps")
            }
        }
        impl Default for IdlCreateBufferBumps {
            fn default() -> Self {
                IdlCreateBufferBumps {}
            }
        }
        impl<'info> anchor_lang::Bumps for IdlCreateBuffer<'info>
        where
            'info: 'info,
        {
            type Bumps = IdlCreateBufferBumps;
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
        /// instead of an `AccountInfo`. This is useful for clients that want
        /// to generate a list of accounts, without explicitly knowing the
        /// order all the fields should be in.
        ///
        /// To access the struct in this module, one should use the sibling
        /// `accounts` module (also generated), which re-exports this.
        pub(crate) mod __client_accounts_idl_create_buffer {
            use super::*;
            use anchor_lang::prelude::borsh;
            /// Generated client accounts for [`IdlCreateBuffer`].
            pub struct IdlCreateBuffer {
                pub buffer: Pubkey,
                pub authority: Pubkey,
            }
            impl borsh::ser::BorshSerialize for IdlCreateBuffer
            where
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.buffer, writer)?;
                    borsh::BorshSerialize::serialize(&self.authority, writer)?;
                    Ok(())
                }
            }
            #[automatically_derived]
            impl anchor_lang::ToAccountMetas for IdlCreateBuffer {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.buffer,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.authority,
                                true,
                            ),
                        );
                    account_metas
                }
            }
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a CPI struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is an
        /// AccountInfo.
        ///
        /// To access the struct in this module, one should use the sibling
        /// [`cpi::accounts`] module (also generated), which re-exports this.
        pub(crate) mod __cpi_client_accounts_idl_create_buffer {
            use super::*;
            /// Generated CPI struct of the accounts for [`IdlCreateBuffer`].
            pub struct IdlCreateBuffer<'info> {
                pub buffer: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                pub authority: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountMetas for IdlCreateBuffer<'info> {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.buffer),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.authority),
                                true,
                            ),
                        );
                    account_metas
                }
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountInfos<'info> for IdlCreateBuffer<'info> {
                fn to_account_infos(
                    &self,
                ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                    let mut account_infos = ::alloc::vec::Vec::new();
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.buffer),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.authority,
                            ),
                        );
                    account_infos
                }
            }
        }
        pub struct IdlSetBuffer<'info> {
            #[account(mut, constraint = buffer.authority = = idl.authority)]
            pub buffer: Account<'info, IdlAccount>,
            #[account(mut, has_one = authority)]
            pub idl: Account<'info, IdlAccount>,
            #[account(constraint = authority.key!= &ERASED_AUTHORITY)]
            pub authority: Signer<'info>,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::Accounts<'info, IdlSetBufferBumps>
        for IdlSetBuffer<'info>
        where
            'info: 'info,
        {
            #[inline(never)]
            fn try_accounts(
                __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
                __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >],
                __ix_data: &[u8],
                __bumps: &mut IdlSetBufferBumps,
                __reallocs: &mut std::collections::BTreeSet<
                    anchor_lang::solana_program::pubkey::Pubkey,
                >,
            ) -> anchor_lang::Result<Self> {
                let buffer: anchor_lang::accounts::account::Account<IdlAccount> = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("buffer"))?;
                let idl: anchor_lang::accounts::account::Account<IdlAccount> = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("idl"))?;
                let authority: Signer = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("authority"))?;
                if !AsRef::<AccountInfo>::as_ref(&buffer).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("buffer"),
                    );
                }
                if !(buffer.authority == idl.authority) {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintRaw,
                            )
                            .with_account_name("buffer"),
                    );
                }
                if !AsRef::<AccountInfo>::as_ref(&idl).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("idl"),
                    );
                }
                {
                    let my_key = idl.authority;
                    let target_key = authority.key();
                    if my_key != target_key {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintHasOne,
                                )
                                .with_account_name("idl")
                                .with_pubkeys((my_key, target_key)),
                        );
                    }
                }
                if !(authority.key != &ERASED_AUTHORITY) {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintRaw,
                            )
                            .with_account_name("authority"),
                    );
                }
                Ok(IdlSetBuffer {
                    buffer,
                    idl,
                    authority,
                })
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for IdlSetBuffer<'info>
        where
            'info: 'info,
        {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos.extend(self.buffer.to_account_infos());
                account_infos.extend(self.idl.to_account_infos());
                account_infos.extend(self.authority.to_account_infos());
                account_infos
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for IdlSetBuffer<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas.extend(self.buffer.to_account_metas(None));
                account_metas.extend(self.idl.to_account_metas(None));
                account_metas.extend(self.authority.to_account_metas(None));
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::AccountsExit<'info> for IdlSetBuffer<'info>
        where
            'info: 'info,
        {
            fn exit(
                &self,
                program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            ) -> anchor_lang::Result<()> {
                anchor_lang::AccountsExit::exit(&self.buffer, program_id)
                    .map_err(|e| e.with_account_name("buffer"))?;
                anchor_lang::AccountsExit::exit(&self.idl, program_id)
                    .map_err(|e| e.with_account_name("idl"))?;
                Ok(())
            }
        }
        pub struct IdlSetBufferBumps {}
        #[automatically_derived]
        impl ::core::fmt::Debug for IdlSetBufferBumps {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "IdlSetBufferBumps")
            }
        }
        impl Default for IdlSetBufferBumps {
            fn default() -> Self {
                IdlSetBufferBumps {}
            }
        }
        impl<'info> anchor_lang::Bumps for IdlSetBuffer<'info>
        where
            'info: 'info,
        {
            type Bumps = IdlSetBufferBumps;
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
        /// instead of an `AccountInfo`. This is useful for clients that want
        /// to generate a list of accounts, without explicitly knowing the
        /// order all the fields should be in.
        ///
        /// To access the struct in this module, one should use the sibling
        /// `accounts` module (also generated), which re-exports this.
        pub(crate) mod __client_accounts_idl_set_buffer {
            use super::*;
            use anchor_lang::prelude::borsh;
            /// Generated client accounts for [`IdlSetBuffer`].
            pub struct IdlSetBuffer {
                pub buffer: Pubkey,
                pub idl: Pubkey,
                pub authority: Pubkey,
            }
            impl borsh::ser::BorshSerialize for IdlSetBuffer
            where
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.buffer, writer)?;
                    borsh::BorshSerialize::serialize(&self.idl, writer)?;
                    borsh::BorshSerialize::serialize(&self.authority, writer)?;
                    Ok(())
                }
            }
            #[automatically_derived]
            impl anchor_lang::ToAccountMetas for IdlSetBuffer {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.buffer,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.idl,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.authority,
                                true,
                            ),
                        );
                    account_metas
                }
            }
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a CPI struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is an
        /// AccountInfo.
        ///
        /// To access the struct in this module, one should use the sibling
        /// [`cpi::accounts`] module (also generated), which re-exports this.
        pub(crate) mod __cpi_client_accounts_idl_set_buffer {
            use super::*;
            /// Generated CPI struct of the accounts for [`IdlSetBuffer`].
            pub struct IdlSetBuffer<'info> {
                pub buffer: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                pub idl: anchor_lang::solana_program::account_info::AccountInfo<'info>,
                pub authority: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountMetas for IdlSetBuffer<'info> {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.buffer),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.idl),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.authority),
                                true,
                            ),
                        );
                    account_metas
                }
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountInfos<'info> for IdlSetBuffer<'info> {
                fn to_account_infos(
                    &self,
                ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                    let mut account_infos = ::alloc::vec::Vec::new();
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.buffer),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.idl),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.authority,
                            ),
                        );
                    account_infos
                }
            }
        }
        pub struct IdlCloseAccount<'info> {
            #[account(mut, has_one = authority, close = sol_destination)]
            pub account: Account<'info, IdlAccount>,
            #[account(constraint = authority.key!= &ERASED_AUTHORITY)]
            pub authority: Signer<'info>,
            #[account(mut)]
            pub sol_destination: AccountInfo<'info>,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::Accounts<'info, IdlCloseAccountBumps>
        for IdlCloseAccount<'info>
        where
            'info: 'info,
        {
            #[inline(never)]
            fn try_accounts(
                __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
                __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >],
                __ix_data: &[u8],
                __bumps: &mut IdlCloseAccountBumps,
                __reallocs: &mut std::collections::BTreeSet<
                    anchor_lang::solana_program::pubkey::Pubkey,
                >,
            ) -> anchor_lang::Result<Self> {
                let account: anchor_lang::accounts::account::Account<IdlAccount> = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("account"))?;
                let authority: Signer = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("authority"))?;
                let sol_destination: AccountInfo = anchor_lang::Accounts::try_accounts(
                        __program_id,
                        __accounts,
                        __ix_data,
                        __bumps,
                        __reallocs,
                    )
                    .map_err(|e| e.with_account_name("sol_destination"))?;
                if !AsRef::<AccountInfo>::as_ref(&account).is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("account"),
                    );
                }
                {
                    let my_key = account.authority;
                    let target_key = authority.key();
                    if my_key != target_key {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintHasOne,
                                )
                                .with_account_name("account")
                                .with_pubkeys((my_key, target_key)),
                        );
                    }
                }
                {
                    if account.key() == sol_destination.key() {
                        return Err(
                            anchor_lang::error::Error::from(
                                    anchor_lang::error::ErrorCode::ConstraintClose,
                                )
                                .with_account_name("account"),
                        );
                    }
                }
                if !(authority.key != &ERASED_AUTHORITY) {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintRaw,
                            )
                            .with_account_name("authority"),
                    );
                }
                if !&sol_destination.is_writable {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintMut,
                            )
                            .with_account_name("sol_destination"),
                    );
                }
                Ok(IdlCloseAccount {
                    account,
                    authority,
                    sol_destination,
                })
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for IdlCloseAccount<'info>
        where
            'info: 'info,
        {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos.extend(self.account.to_account_infos());
                account_infos.extend(self.authority.to_account_infos());
                account_infos.extend(self.sol_destination.to_account_infos());
                account_infos
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for IdlCloseAccount<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas.extend(self.account.to_account_metas(None));
                account_metas.extend(self.authority.to_account_metas(None));
                account_metas.extend(self.sol_destination.to_account_metas(None));
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::AccountsExit<'info> for IdlCloseAccount<'info>
        where
            'info: 'info,
        {
            fn exit(
                &self,
                program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            ) -> anchor_lang::Result<()> {
                {
                    let sol_destination = &self.sol_destination;
                    anchor_lang::AccountsClose::close(
                            &self.account,
                            sol_destination.to_account_info(),
                        )
                        .map_err(|e| e.with_account_name("account"))?;
                }
                anchor_lang::AccountsExit::exit(&self.sol_destination, program_id)
                    .map_err(|e| e.with_account_name("sol_destination"))?;
                Ok(())
            }
        }
        pub struct IdlCloseAccountBumps {}
        #[automatically_derived]
        impl ::core::fmt::Debug for IdlCloseAccountBumps {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "IdlCloseAccountBumps")
            }
        }
        impl Default for IdlCloseAccountBumps {
            fn default() -> Self {
                IdlCloseAccountBumps {}
            }
        }
        impl<'info> anchor_lang::Bumps for IdlCloseAccount<'info>
        where
            'info: 'info,
        {
            type Bumps = IdlCloseAccountBumps;
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
        /// instead of an `AccountInfo`. This is useful for clients that want
        /// to generate a list of accounts, without explicitly knowing the
        /// order all the fields should be in.
        ///
        /// To access the struct in this module, one should use the sibling
        /// `accounts` module (also generated), which re-exports this.
        pub(crate) mod __client_accounts_idl_close_account {
            use super::*;
            use anchor_lang::prelude::borsh;
            /// Generated client accounts for [`IdlCloseAccount`].
            pub struct IdlCloseAccount {
                pub account: Pubkey,
                pub authority: Pubkey,
                pub sol_destination: Pubkey,
            }
            impl borsh::ser::BorshSerialize for IdlCloseAccount
            where
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
                Pubkey: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.account, writer)?;
                    borsh::BorshSerialize::serialize(&self.authority, writer)?;
                    borsh::BorshSerialize::serialize(&self.sol_destination, writer)?;
                    Ok(())
                }
            }
            #[automatically_derived]
            impl anchor_lang::ToAccountMetas for IdlCloseAccount {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.account,
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                self.authority,
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                self.sol_destination,
                                false,
                            ),
                        );
                    account_metas
                }
            }
        }
        /// An internal, Anchor generated module. This is used (as an
        /// implementation detail), to generate a CPI struct for a given
        /// `#[derive(Accounts)]` implementation, where each field is an
        /// AccountInfo.
        ///
        /// To access the struct in this module, one should use the sibling
        /// [`cpi::accounts`] module (also generated), which re-exports this.
        pub(crate) mod __cpi_client_accounts_idl_close_account {
            use super::*;
            /// Generated CPI struct of the accounts for [`IdlCloseAccount`].
            pub struct IdlCloseAccount<'info> {
                pub account: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                pub authority: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
                pub sol_destination: anchor_lang::solana_program::account_info::AccountInfo<
                    'info,
                >,
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountMetas for IdlCloseAccount<'info> {
                fn to_account_metas(
                    &self,
                    is_signer: Option<bool>,
                ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                    let mut account_metas = ::alloc::vec::Vec::new();
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.account),
                                false,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                                anchor_lang::Key::key(&self.authority),
                                true,
                            ),
                        );
                    account_metas
                        .push(
                            anchor_lang::solana_program::instruction::AccountMeta::new(
                                anchor_lang::Key::key(&self.sol_destination),
                                false,
                            ),
                        );
                    account_metas
                }
            }
            #[automatically_derived]
            impl<'info> anchor_lang::ToAccountInfos<'info> for IdlCloseAccount<'info> {
                fn to_account_infos(
                    &self,
                ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                    let mut account_infos = ::alloc::vec::Vec::new();
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(&self.account),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.authority,
                            ),
                        );
                    account_infos
                        .extend(
                            anchor_lang::ToAccountInfos::to_account_infos(
                                &self.sol_destination,
                            ),
                        );
                    account_infos
                }
            }
        }
        use std::cell::{Ref, RefMut};
        pub trait IdlTrailingData<'info> {
            fn trailing_data(self) -> Ref<'info, [u8]>;
            fn trailing_data_mut(self) -> RefMut<'info, [u8]>;
        }
        impl<'a, 'info: 'a> IdlTrailingData<'a> for &'a Account<'info, IdlAccount> {
            fn trailing_data(self) -> Ref<'a, [u8]> {
                let info: &AccountInfo<'info> = self.as_ref();
                Ref::map(info.try_borrow_data().unwrap(), |d| &d[44..])
            }
            fn trailing_data_mut(self) -> RefMut<'a, [u8]> {
                let info: &AccountInfo<'info> = self.as_ref();
                RefMut::map(info.try_borrow_mut_data().unwrap(), |d| &mut d[44..])
            }
        }
        #[inline(never)]
        pub fn __idl_create_account(
            program_id: &Pubkey,
            accounts: &mut IdlCreateAccounts,
            data_len: u64,
        ) -> anchor_lang::Result<()> {
            ::solana_msg::sol_log("Instruction: IdlCreateAccount");
            if program_id != accounts.program.key {
                return Err(
                    anchor_lang::error::ErrorCode::IdlInstructionInvalidProgram.into(),
                );
            }
            let from = accounts.from.key;
            let (base, nonce) = Pubkey::find_program_address(&[], program_id);
            let seed = IdlAccount::seed();
            let owner = accounts.program.key;
            let to = Pubkey::create_with_seed(&base, seed, owner).unwrap();
            let space = std::cmp::min(
                IdlAccount::DISCRIMINATOR.len() + 32 + 4 + data_len as usize,
                10_000,
            );
            let rent = Rent::get()?;
            let lamports = rent.minimum_balance(space);
            let seeds = &[&[nonce][..]];
            let ix = anchor_lang::solana_program::system_instruction::create_account_with_seed(
                from,
                &to,
                &base,
                seed,
                lamports,
                space as u64,
                owner,
            );
            anchor_lang::solana_program::program::invoke_signed(
                &ix,
                &[
                    accounts.from.clone(),
                    accounts.to.clone(),
                    accounts.base.clone(),
                    accounts.system_program.to_account_info(),
                ],
                &[seeds],
            )?;
            let mut idl_account = {
                let mut account_data = accounts.to.try_borrow_data()?;
                let mut account_data_slice: &[u8] = &account_data;
                IdlAccount::try_deserialize_unchecked(&mut account_data_slice)?
            };
            idl_account.authority = *accounts.from.key;
            let mut data = accounts.to.try_borrow_mut_data()?;
            let dst: &mut [u8] = &mut data;
            let mut cursor = std::io::Cursor::new(dst);
            idl_account.try_serialize(&mut cursor)?;
            Ok(())
        }
        #[inline(never)]
        pub fn __idl_resize_account(
            program_id: &Pubkey,
            accounts: &mut IdlResizeAccount,
            data_len: u64,
        ) -> anchor_lang::Result<()> {
            ::solana_msg::sol_log("Instruction: IdlResizeAccount");
            let data_len: usize = data_len as usize;
            if accounts.idl.data_len != 0 {
                return Err(anchor_lang::error::ErrorCode::IdlAccountNotEmpty.into());
            }
            let idl_ref = AsRef::<AccountInfo>::as_ref(&accounts.idl);
            let new_account_space = idl_ref
                .data_len()
                .checked_add(
                    std::cmp::min(
                        data_len
                            .checked_sub(idl_ref.data_len())
                            .expect(
                                "data_len should always be >= the current account space",
                            ),
                        10_000,
                    ),
                )
                .unwrap();
            if new_account_space > idl_ref.data_len() {
                let sysvar_rent = Rent::get()?;
                let new_rent_minimum = sysvar_rent.minimum_balance(new_account_space);
                anchor_lang::system_program::transfer(
                    anchor_lang::context::CpiContext::new(
                        accounts.system_program.to_account_info(),
                        anchor_lang::system_program::Transfer {
                            from: accounts.authority.to_account_info(),
                            to: accounts.idl.to_account_info(),
                        },
                    ),
                    new_rent_minimum.checked_sub(idl_ref.lamports()).unwrap(),
                )?;
                idl_ref.realloc(new_account_space, false)?;
            }
            Ok(())
        }
        #[inline(never)]
        pub fn __idl_close_account(
            program_id: &Pubkey,
            accounts: &mut IdlCloseAccount,
        ) -> anchor_lang::Result<()> {
            ::solana_msg::sol_log("Instruction: IdlCloseAccount");
            Ok(())
        }
        #[inline(never)]
        pub fn __idl_create_buffer(
            program_id: &Pubkey,
            accounts: &mut IdlCreateBuffer,
        ) -> anchor_lang::Result<()> {
            ::solana_msg::sol_log("Instruction: IdlCreateBuffer");
            let mut buffer = &mut accounts.buffer;
            buffer.authority = *accounts.authority.key;
            Ok(())
        }
        #[inline(never)]
        pub fn __idl_write(
            program_id: &Pubkey,
            accounts: &mut IdlAccounts,
            idl_data: Vec<u8>,
        ) -> anchor_lang::Result<()> {
            ::solana_msg::sol_log("Instruction: IdlWrite");
            let prev_len: usize = ::std::convert::TryInto::<
                usize,
            >::try_into(accounts.idl.data_len)
                .unwrap();
            let new_len: usize = prev_len.checked_add(idl_data.len()).unwrap() as usize;
            accounts.idl.data_len = accounts
                .idl
                .data_len
                .checked_add(
                    ::std::convert::TryInto::<u32>::try_into(idl_data.len()).unwrap(),
                )
                .unwrap();
            use IdlTrailingData;
            let mut idl_bytes = accounts.idl.trailing_data_mut();
            let idl_expansion = &mut idl_bytes[prev_len..new_len];
            if idl_expansion.len() != idl_data.len() {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                            error_name: anchor_lang::error::ErrorCode::RequireEqViolated
                                .name(),
                            error_code_number: anchor_lang::error::ErrorCode::RequireEqViolated
                                .into(),
                            error_msg: anchor_lang::error::ErrorCode::RequireEqViolated
                                .to_string(),
                            error_origin: Some(
                                anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                    filename: "programs/amm/src/lib.rs",
                                    line: 29u32,
                                }),
                            ),
                            compared_values: None,
                        })
                        .with_values((idl_expansion.len(), idl_data.len())),
                );
            }
            idl_expansion.copy_from_slice(&idl_data[..]);
            Ok(())
        }
        #[inline(never)]
        pub fn __idl_set_authority(
            program_id: &Pubkey,
            accounts: &mut IdlAccounts,
            new_authority: Pubkey,
        ) -> anchor_lang::Result<()> {
            ::solana_msg::sol_log("Instruction: IdlSetAuthority");
            accounts.idl.authority = new_authority;
            Ok(())
        }
        #[inline(never)]
        pub fn __idl_set_buffer(
            program_id: &Pubkey,
            accounts: &mut IdlSetBuffer,
        ) -> anchor_lang::Result<()> {
            ::solana_msg::sol_log("Instruction: IdlSetBuffer");
            accounts.idl.data_len = accounts.buffer.data_len;
            use IdlTrailingData;
            let buffer_len = ::std::convert::TryInto::<
                usize,
            >::try_into(accounts.buffer.data_len)
                .unwrap();
            let mut target = accounts.idl.trailing_data_mut();
            let source = &accounts.buffer.trailing_data()[..buffer_len];
            if target.len() < buffer_len {
                return Err(
                    anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                            error_name: anchor_lang::error::ErrorCode::RequireGteViolated
                                .name(),
                            error_code_number: anchor_lang::error::ErrorCode::RequireGteViolated
                                .into(),
                            error_msg: anchor_lang::error::ErrorCode::RequireGteViolated
                                .to_string(),
                            error_origin: Some(
                                anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                                    filename: "programs/amm/src/lib.rs",
                                    line: 29u32,
                                }),
                            ),
                            compared_values: None,
                        })
                        .with_values((target.len(), buffer_len)),
                );
            }
            target[..buffer_len].copy_from_slice(source);
            Ok(())
        }
    }
    /// __global mod defines wrapped handlers for global instructions.
    pub mod __global {
        use super::*;
        #[inline(never)]
        pub fn initialize<'info>(
            __program_id: &Pubkey,
            __accounts: &'info [AccountInfo<'info>],
            __ix_data: &[u8],
        ) -> anchor_lang::Result<()> {
            ::solana_msg::sol_log("Instruction: Initialize");
            let ix = instruction::Initialize::deserialize(&mut &__ix_data[..])
                .map_err(|_| {
                    anchor_lang::error::ErrorCode::InstructionDidNotDeserialize
                })?;
            let instruction::Initialize { seed, fee, authority } = ix;
            let mut __bumps = <Initialize as anchor_lang::Bumps>::Bumps::default();
            let mut __reallocs = std::collections::BTreeSet::new();
            let mut __remaining_accounts: &[AccountInfo] = __accounts;
            let mut __accounts = Initialize::try_accounts(
                __program_id,
                &mut __remaining_accounts,
                __ix_data,
                &mut __bumps,
                &mut __reallocs,
            )?;
            let result = amm::initialize(
                anchor_lang::context::Context::new(
                    __program_id,
                    &mut __accounts,
                    __remaining_accounts,
                    __bumps,
                ),
                seed,
                fee,
                authority,
            )?;
            __accounts.exit(__program_id)
        }
        #[inline(never)]
        pub fn deposit<'info>(
            __program_id: &Pubkey,
            __accounts: &'info [AccountInfo<'info>],
            __ix_data: &[u8],
        ) -> anchor_lang::Result<()> {
            ::solana_msg::sol_log("Instruction: Deposit");
            let ix = instruction::Deposit::deserialize(&mut &__ix_data[..])
                .map_err(|_| {
                    anchor_lang::error::ErrorCode::InstructionDidNotDeserialize
                })?;
            let instruction::Deposit { amount_x, amount_y, min_lp } = ix;
            let mut __bumps = <Deposit as anchor_lang::Bumps>::Bumps::default();
            let mut __reallocs = std::collections::BTreeSet::new();
            let mut __remaining_accounts: &[AccountInfo] = __accounts;
            let mut __accounts = Deposit::try_accounts(
                __program_id,
                &mut __remaining_accounts,
                __ix_data,
                &mut __bumps,
                &mut __reallocs,
            )?;
            let result = amm::deposit(
                anchor_lang::context::Context::new(
                    __program_id,
                    &mut __accounts,
                    __remaining_accounts,
                    __bumps,
                ),
                amount_x,
                amount_y,
                min_lp,
            )?;
            __accounts.exit(__program_id)
        }
        #[inline(never)]
        pub fn withdraw<'info>(
            __program_id: &Pubkey,
            __accounts: &'info [AccountInfo<'info>],
            __ix_data: &[u8],
        ) -> anchor_lang::Result<()> {
            ::solana_msg::sol_log("Instruction: Withdraw");
            let ix = instruction::Withdraw::deserialize(&mut &__ix_data[..])
                .map_err(|_| {
                    anchor_lang::error::ErrorCode::InstructionDidNotDeserialize
                })?;
            let instruction::Withdraw { lp_amount, min_x, min_y } = ix;
            let mut __bumps = <Withdraw as anchor_lang::Bumps>::Bumps::default();
            let mut __reallocs = std::collections::BTreeSet::new();
            let mut __remaining_accounts: &[AccountInfo] = __accounts;
            let mut __accounts = Withdraw::try_accounts(
                __program_id,
                &mut __remaining_accounts,
                __ix_data,
                &mut __bumps,
                &mut __reallocs,
            )?;
            let result = amm::withdraw(
                anchor_lang::context::Context::new(
                    __program_id,
                    &mut __accounts,
                    __remaining_accounts,
                    __bumps,
                ),
                lp_amount,
                min_x,
                min_y,
            )?;
            __accounts.exit(__program_id)
        }
        #[inline(never)]
        pub fn swap<'info>(
            __program_id: &Pubkey,
            __accounts: &'info [AccountInfo<'info>],
            __ix_data: &[u8],
        ) -> anchor_lang::Result<()> {
            ::solana_msg::sol_log("Instruction: Swap");
            let ix = instruction::Swap::deserialize(&mut &__ix_data[..])
                .map_err(|_| {
                    anchor_lang::error::ErrorCode::InstructionDidNotDeserialize
                })?;
            let instruction::Swap { is_x, amount_in, min_amount_out } = ix;
            let mut __bumps = <Swap as anchor_lang::Bumps>::Bumps::default();
            let mut __reallocs = std::collections::BTreeSet::new();
            let mut __remaining_accounts: &[AccountInfo] = __accounts;
            let mut __accounts = Swap::try_accounts(
                __program_id,
                &mut __remaining_accounts,
                __ix_data,
                &mut __bumps,
                &mut __reallocs,
            )?;
            let result = amm::swap(
                anchor_lang::context::Context::new(
                    __program_id,
                    &mut __accounts,
                    __remaining_accounts,
                    __bumps,
                ),
                is_x,
                amount_in,
                min_amount_out,
            )?;
            __accounts.exit(__program_id)
        }
    }
}
/// Program entrypoints for the AMM.
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
/// An Anchor generated module containing the program's set of
/// instructions, where each method handler in the `#[program]` mod is
/// associated with a struct defining the input arguments to the
/// method. These should be used directly, when one wants to serialize
/// Anchor instruction data, for example, when speciying
/// instructions on a client.
pub mod instruction {
    use super::*;
    /// Instruction.
    pub struct Initialize {
        pub seed: u64,
        pub fee: u16,
        pub authority: Option<Pubkey>,
    }
    impl borsh::ser::BorshSerialize for Initialize
    where
        u64: borsh::ser::BorshSerialize,
        u16: borsh::ser::BorshSerialize,
        Option<Pubkey>: borsh::ser::BorshSerialize,
    {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            borsh::BorshSerialize::serialize(&self.seed, writer)?;
            borsh::BorshSerialize::serialize(&self.fee, writer)?;
            borsh::BorshSerialize::serialize(&self.authority, writer)?;
            Ok(())
        }
    }
    impl borsh::de::BorshDeserialize for Initialize
    where
        u64: borsh::BorshDeserialize,
        u16: borsh::BorshDeserialize,
        Option<Pubkey>: borsh::BorshDeserialize,
    {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            Ok(Self {
                seed: borsh::BorshDeserialize::deserialize_reader(reader)?,
                fee: borsh::BorshDeserialize::deserialize_reader(reader)?,
                authority: borsh::BorshDeserialize::deserialize_reader(reader)?,
            })
        }
    }
    impl anchor_lang::Discriminator for Initialize {
        const DISCRIMINATOR: &'static [u8] = &[175, 175, 109, 31, 13, 152, 155, 237];
    }
    impl anchor_lang::InstructionData for Initialize {}
    impl anchor_lang::Owner for Initialize {
        fn owner() -> Pubkey {
            ID
        }
    }
    /// Instruction.
    pub struct Deposit {
        pub amount_x: u64,
        pub amount_y: u64,
        pub min_lp: u64,
    }
    impl borsh::ser::BorshSerialize for Deposit
    where
        u64: borsh::ser::BorshSerialize,
        u64: borsh::ser::BorshSerialize,
        u64: borsh::ser::BorshSerialize,
    {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            borsh::BorshSerialize::serialize(&self.amount_x, writer)?;
            borsh::BorshSerialize::serialize(&self.amount_y, writer)?;
            borsh::BorshSerialize::serialize(&self.min_lp, writer)?;
            Ok(())
        }
    }
    impl borsh::de::BorshDeserialize for Deposit
    where
        u64: borsh::BorshDeserialize,
        u64: borsh::BorshDeserialize,
        u64: borsh::BorshDeserialize,
    {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            Ok(Self {
                amount_x: borsh::BorshDeserialize::deserialize_reader(reader)?,
                amount_y: borsh::BorshDeserialize::deserialize_reader(reader)?,
                min_lp: borsh::BorshDeserialize::deserialize_reader(reader)?,
            })
        }
    }
    impl anchor_lang::Discriminator for Deposit {
        const DISCRIMINATOR: &'static [u8] = &[242, 35, 198, 137, 82, 225, 242, 182];
    }
    impl anchor_lang::InstructionData for Deposit {}
    impl anchor_lang::Owner for Deposit {
        fn owner() -> Pubkey {
            ID
        }
    }
    /// Instruction.
    pub struct Withdraw {
        pub lp_amount: u64,
        pub min_x: u64,
        pub min_y: u64,
    }
    impl borsh::ser::BorshSerialize for Withdraw
    where
        u64: borsh::ser::BorshSerialize,
        u64: borsh::ser::BorshSerialize,
        u64: borsh::ser::BorshSerialize,
    {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            borsh::BorshSerialize::serialize(&self.lp_amount, writer)?;
            borsh::BorshSerialize::serialize(&self.min_x, writer)?;
            borsh::BorshSerialize::serialize(&self.min_y, writer)?;
            Ok(())
        }
    }
    impl borsh::de::BorshDeserialize for Withdraw
    where
        u64: borsh::BorshDeserialize,
        u64: borsh::BorshDeserialize,
        u64: borsh::BorshDeserialize,
    {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            Ok(Self {
                lp_amount: borsh::BorshDeserialize::deserialize_reader(reader)?,
                min_x: borsh::BorshDeserialize::deserialize_reader(reader)?,
                min_y: borsh::BorshDeserialize::deserialize_reader(reader)?,
            })
        }
    }
    impl anchor_lang::Discriminator for Withdraw {
        const DISCRIMINATOR: &'static [u8] = &[183, 18, 70, 156, 148, 109, 161, 34];
    }
    impl anchor_lang::InstructionData for Withdraw {}
    impl anchor_lang::Owner for Withdraw {
        fn owner() -> Pubkey {
            ID
        }
    }
    /// Instruction.
    pub struct Swap {
        pub is_x: bool,
        pub amount_in: u64,
        pub min_amount_out: u64,
    }
    impl borsh::ser::BorshSerialize for Swap
    where
        bool: borsh::ser::BorshSerialize,
        u64: borsh::ser::BorshSerialize,
        u64: borsh::ser::BorshSerialize,
    {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            borsh::BorshSerialize::serialize(&self.is_x, writer)?;
            borsh::BorshSerialize::serialize(&self.amount_in, writer)?;
            borsh::BorshSerialize::serialize(&self.min_amount_out, writer)?;
            Ok(())
        }
    }
    impl borsh::de::BorshDeserialize for Swap
    where
        bool: borsh::BorshDeserialize,
        u64: borsh::BorshDeserialize,
        u64: borsh::BorshDeserialize,
    {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            Ok(Self {
                is_x: borsh::BorshDeserialize::deserialize_reader(reader)?,
                amount_in: borsh::BorshDeserialize::deserialize_reader(reader)?,
                min_amount_out: borsh::BorshDeserialize::deserialize_reader(reader)?,
            })
        }
    }
    impl anchor_lang::Discriminator for Swap {
        const DISCRIMINATOR: &'static [u8] = &[248, 198, 158, 145, 225, 117, 135, 200];
    }
    impl anchor_lang::InstructionData for Swap {}
    impl anchor_lang::Owner for Swap {
        fn owner() -> Pubkey {
            ID
        }
    }
}
/// An Anchor generated module, providing a set of structs
/// mirroring the structs deriving `Accounts`, where each field is
/// a `Pubkey`. This is useful for specifying accounts for a client.
pub mod accounts {
    pub use crate::__client_accounts_deposit::*;
    pub use crate::__client_accounts_swap::*;
    pub use crate::__client_accounts_initialize::*;
    pub use crate::__client_accounts_withdraw::*;
}
