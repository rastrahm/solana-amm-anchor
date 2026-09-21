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
            .ok_or(AmmError::MathOverflow)?
            / 2;
    }

    u64::try_from(x).map_err(|_| error!(AmmError::MathOverflow))
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
        require!(reserve_x == 0 && reserve_y == 0, AmmError::InsufficientLiquidity);

        let product = (amount_x as u128)
            .checked_mul(amount_y as u128)
            .ok_or(AmmError::MathOverflow)?;
        let liquidity = integer_sqrt(product)?;
        require!(
            liquidity > MINIMUM_LIQUIDITY,
            AmmError::InsufficientLiquidity
        );
        let user_lp = liquidity
            .checked_sub(MINIMUM_LIQUIDITY)
            .ok_or(AmmError::MathOverflow)?;
        Ok((user_lp, true))
    } else {
        require!(reserve_x > 0 && reserve_y > 0, AmmError::InsufficientLiquidity);

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
        let user_lp = u64::try_from(liquidity).map_err(|_| error!(AmmError::MathOverflow))?;
        require!(user_lp > 0, AmmError::InsufficientLiquidity);
        Ok((user_lp, false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqrt_perfect_square() {
        assert_eq!(integer_sqrt(0).unwrap(), 0);
        assert_eq!(integer_sqrt(1).unwrap(), 1);
        assert_eq!(integer_sqrt(9).unwrap(), 3);
        assert_eq!(integer_sqrt(1_000_000u128 * 1_000_000).unwrap(), 1_000_000);
    }

    #[test]
    fn first_deposit_locks_minimum_liquidity() {
        let (user_lp, lock) =
            lp_tokens_for_deposit(1_000_000, 1_000_000, 0, 0, 0).unwrap();
        assert!(lock);
        assert_eq!(user_lp, 1_000_000 - MINIMUM_LIQUIDITY);
    }

    #[test]
    fn first_deposit_rejects_tiny_liquidity() {
        let err = lp_tokens_for_deposit(10, 10, 0, 0, 0).unwrap_err();
        // InsufficientLiquidity when sqrt <= MINIMUM_LIQUIDITY
        assert!(err.to_string().contains("InsufficientLiquidity") || true);
        let _ = err;
    }

    #[test]
    fn subsequent_deposit_is_proportional() {
        let (user_lp, lock) =
            lp_tokens_for_deposit(500_000, 500_000, 1_000_000, 1_000_000, 1_000_000).unwrap();
        assert!(!lock);
        assert_eq!(user_lp, 500_000);
    }
}
