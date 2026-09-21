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
        require!(
            reserve_x == 0 && reserve_y == 0,
            AmmError::InsufficientLiquidity
        );

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
        require!(
            reserve_x > 0 && reserve_y > 0,
            AmmError::InsufficientLiquidity
        );

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
    require!(lp_amount > 0, AmmError::InvalidAmount);
    require!(
        total_lp > 0 && reserve_x > 0 && reserve_y > 0,
        AmmError::InsufficientLiquidity
    );

    let remaining = total_lp
        .checked_sub(lp_amount)
        .ok_or(AmmError::InsufficientLiquidity)?;
    require!(
        remaining >= MINIMUM_LIQUIDITY,
        AmmError::InsufficientLiquidity
    );

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

    let amount_x = u64::try_from(amount_x).map_err(|_| error!(AmmError::MathOverflow))?;
    let amount_y = u64::try_from(amount_y).map_err(|_| error!(AmmError::MathOverflow))?;
    require!(
        amount_x > 0 && amount_y > 0,
        AmmError::InsufficientLiquidity
    );

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
    require!(amount_in > 0, AmmError::InvalidAmount);
    require!(
        reserve_in > 0 && reserve_out > 0,
        AmmError::InsufficientLiquidity
    );
    require!(fee_bps as u128 <= FEE_DENOMINATOR, AmmError::InvalidFee);

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
    require!(amount_out_u > 0, AmmError::InsufficientLiquidity);
    require!(
        amount_out_u < reserve_out_u,
        AmmError::InsufficientLiquidity
    );

    let amount_out = u64::try_from(amount_out_u).map_err(|_| error!(AmmError::MathOverflow))?;

    // Invariant: k must not decrease after the full input is added to reserves.
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
    require!(k_after >= k_before, AmmError::MathOverflow);

    Ok(amount_out)
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
        let (user_lp, lock) = lp_tokens_for_deposit(1_000_000, 1_000_000, 0, 0, 0).unwrap();
        assert!(lock);
        assert_eq!(user_lp, 1_000_000 - MINIMUM_LIQUIDITY);
    }

    #[test]
    fn first_deposit_rejects_tiny_liquidity() {
        assert!(lp_tokens_for_deposit(10, 10, 0, 0, 0).is_err());
    }

    #[test]
    fn subsequent_deposit_is_proportional() {
        let (user_lp, lock) =
            lp_tokens_for_deposit(500_000, 500_000, 1_000_000, 1_000_000, 1_000_000).unwrap();
        assert!(!lock);
        assert_eq!(user_lp, 500_000);
    }

    #[test]
    fn withdraw_is_proportional_floor() {
        let (x, y) = amounts_for_withdraw(500_000, 1_000_000, 1_000_000, 1_000_000).unwrap();
        assert_eq!(x, 500_000);
        assert_eq!(y, 500_000);
    }

    #[test]
    fn withdraw_rejects_burning_below_minimum_lock() {
        // total 1_000_000, try burn 999_001 → remaining 999 < MINIMUM_LIQUIDITY
        assert!(amounts_for_withdraw(999_001, 1_000_000, 1_000_000, 1_000_000).is_err());
    }

    #[test]
    fn withdraw_allows_burning_all_user_lp_leaving_lock() {
        let (x, y) = amounts_for_withdraw(999_000, 1_000_000, 1_000_000, 1_000_000).unwrap();
        assert_eq!(x, 999_000);
        assert_eq!(y, 999_000);
    }

    #[test]
    fn swap_zero_fee_matches_constant_product() {
        // amount_out = 1000 * 1000 / (1000 + 1000) = 500 with 0 fee?
        // formula: (1000 * 10000 * 1000) / (1000 * 10000 + 1000 * 10000) = 10_000_000_000 / 20_000_000 = 500
        let out = amount_out_for_swap(1_000, 1_000, 1_000, 0).unwrap();
        assert_eq!(out, 500);
    }

    #[test]
    fn swap_with_fee_returns_less_than_zero_fee() {
        let no_fee = amount_out_for_swap(10_000, 1_000_000, 1_000_000, 0).unwrap();
        let with_fee = amount_out_for_swap(10_000, 1_000_000, 1_000_000, 30).unwrap();
        assert!(with_fee < no_fee);
        assert!(with_fee > 0);
    }

    #[test]
    fn swap_rejects_empty_reserves() {
        assert!(amount_out_for_swap(100, 0, 1_000, 30).is_err());
    }
}
