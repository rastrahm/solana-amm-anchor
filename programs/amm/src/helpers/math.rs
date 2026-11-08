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

/// Result of quoting a deposit against the current pool state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepositQuote {
    /// Token X actually pulled from the user (`<= max_x`).
    pub amount_x: u64,
    /// Token Y actually pulled from the user (`<= max_y`).
    pub amount_y: u64,
    /// LP minted to the user.
    pub lp_to_user: u64,
    /// True only on the first deposit, when `MINIMUM_LIQUIDITY` must be locked.
    pub lock_minimum: bool,
}

/// Quotes a deposit: how much X/Y to pull and how much LP to mint.
///
/// - First deposit (`total_lp == 0`): pulls `max_x`/`max_y` as given and mints
///   `sqrt(max_x * max_y) - MINIMUM_LIQUIDITY`. Tokens already sitting in the vaults
///   (direct donations to the predictable ATAs) are ignored for pricing and absorbed by
///   the pool, so they cannot block initialization.
/// - Later deposits: `lp = min(max_x * total_lp / reserve_x, max_y * total_lp / reserve_y)`
///   (floor), then pulls only `ceil(lp * reserve / total_lp)` of each token. The excess of
///   an unbalanced offer stays with the user; rounding up favors the pool.
///
/// @param max_x Maximum token X the user is willing to deposit.
/// @param max_y Maximum token Y the user is willing to deposit.
/// @return `DepositQuote` with the exact amounts to transfer and LP to mint.
pub fn deposit_quote(
    max_x: u64,
    max_y: u64,
    reserve_x: u64,
    reserve_y: u64,
    total_lp: u64,
) -> Result<DepositQuote> {
    if total_lp == 0 {
        let product = (max_x as u128)
            .checked_mul(max_y as u128)
            .ok_or(AmmError::MathOverflow)?;
        let liquidity = integer_sqrt(product)?;
        require!(
            liquidity > MINIMUM_LIQUIDITY,
            AmmError::InsufficientLiquidity
        );
        let lp_to_user = liquidity
            .checked_sub(MINIMUM_LIQUIDITY)
            .ok_or(AmmError::MathOverflow)?;
        return Ok(DepositQuote {
            amount_x: max_x,
            amount_y: max_y,
            lp_to_user,
            lock_minimum: true,
        });
    }

    require!(
        reserve_x > 0 && reserve_y > 0,
        AmmError::InsufficientLiquidity
    );

    let total = total_lp as u128;
    let lp_from_x = (max_x as u128)
        .checked_mul(total)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(reserve_x as u128)
        .ok_or(AmmError::MathOverflow)?;
    let lp_from_y = (max_y as u128)
        .checked_mul(total)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(reserve_y as u128)
        .ok_or(AmmError::MathOverflow)?;
    let lp = lp_from_x.min(lp_from_y);
    require!(lp > 0, AmmError::InsufficientLiquidity);

    let amount_x = ceil_mul_div(lp, reserve_x as u128, total)?;
    let amount_y = ceil_mul_div(lp, reserve_y as u128, total)?;

    Ok(DepositQuote {
        amount_x: u64::try_from(amount_x).map_err(|_| error!(AmmError::MathOverflow))?,
        amount_y: u64::try_from(amount_y).map_err(|_| error!(AmmError::MathOverflow))?,
        lp_to_user: u64::try_from(lp).map_err(|_| error!(AmmError::MathOverflow))?,
        lock_minimum: false,
    })
}

/// `ceil(a * b / denominator)` with checked `u128` arithmetic.
fn ceil_mul_div(a: u128, b: u128, denominator: u128) -> Result<u128> {
    require!(denominator > 0, AmmError::MathOverflow);
    let product = a.checked_mul(b).ok_or(AmmError::MathOverflow)?;
    let quotient = product
        .checked_div(denominator)
        .ok_or(AmmError::MathOverflow)?;
    if product % denominator == 0 {
        Ok(quotient)
    } else {
        quotient
            .checked_add(1)
            .ok_or(error!(AmmError::MathOverflow))
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
        let q = deposit_quote(1_000_000, 1_000_000, 0, 0, 0).unwrap();
        assert!(q.lock_minimum);
        assert_eq!(q.lp_to_user, 1_000_000 - MINIMUM_LIQUIDITY);
        assert_eq!((q.amount_x, q.amount_y), (1_000_000, 1_000_000));
    }

    #[test]
    fn first_deposit_ignores_pre_existing_donations() {
        // Attacker pre-funds the predictable vault ATA before the first deposit.
        let q = deposit_quote(1_000_000, 1_000_000, 1, 0, 0).unwrap();
        assert!(q.lock_minimum);
        assert_eq!(q.lp_to_user, 1_000_000 - MINIMUM_LIQUIDITY);

        let q = deposit_quote(1_000_000, 1_000_000, 7, 9_999, 0).unwrap();
        assert_eq!(q.lp_to_user, 1_000_000 - MINIMUM_LIQUIDITY);
    }

    #[test]
    fn first_deposit_rejects_tiny_liquidity() {
        assert!(deposit_quote(10, 10, 0, 0, 0).is_err());
    }

    #[test]
    fn subsequent_deposit_is_proportional() {
        let q = deposit_quote(500_000, 500_000, 1_000_000, 1_000_000, 1_000_000).unwrap();
        assert!(!q.lock_minimum);
        assert_eq!(q.lp_to_user, 500_000);
        assert_eq!((q.amount_x, q.amount_y), (500_000, 500_000));
    }

    #[test]
    fn unbalanced_deposit_only_pulls_the_pool_ratio() {
        // 1:1 pool, user offers 100k X + 500k Y → only 100k Y is taken.
        let q = deposit_quote(100_000, 500_000, 1_000_000, 1_000_000, 1_000_000).unwrap();
        assert_eq!(q.lp_to_user, 100_000);
        assert_eq!((q.amount_x, q.amount_y), (100_000, 100_000));

        // Y is the scarce side: 1:2 pool (X:Y), offer 900k X + 200k Y → 100k X.
        let q = deposit_quote(900_000, 200_000, 1_000_000, 2_000_000, 1_000_000).unwrap();
        assert_eq!(q.lp_to_user, 100_000);
        assert_eq!((q.amount_x, q.amount_y), (100_000, 200_000));
    }

    #[test]
    fn deposit_rounds_pulled_amounts_up_and_never_exceeds_max() {
        // reserves 3:7, total 5 → non-integer ratios exercise ceil rounding.
        let (rx, ry, total) = (3_000_001u64, 7_000_003u64, 5_000_000u64);
        for (max_x, max_y) in [(1u64, 1_000u64), (12_345, 99_999), (1_000_000, 1)] {
            let Ok(q) = deposit_quote(max_x, max_y, rx, ry, total) else {
                continue;
            };
            assert!(q.amount_x <= max_x && q.amount_y <= max_y);
            // Value contributed per LP is never below the pool's current value per LP.
            assert!(q.amount_x as u128 * total as u128 >= q.lp_to_user as u128 * rx as u128);
            assert!(q.amount_y as u128 * total as u128 >= q.lp_to_user as u128 * ry as u128);
        }
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
