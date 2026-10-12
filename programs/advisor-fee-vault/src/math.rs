use crate::{FeeError, MAX_ADVISORY_BPS, PERFORMANCE_BPS};
use anchor_lang::prelude::*;

pub const PRICE_SCALE: u128 = 1_000_000_000_000;
pub const YEAR_SECONDS: u128 = 365 * 24 * 60 * 60;
pub const AUM_DENOMINATOR: u128 = PRICE_SCALE * 10_000 * YEAR_SECONDS;
pub const PERFORMANCE_DENOMINATOR: u128 = PRICE_SCALE * 10_000;

#[derive(Debug, PartialEq, Eq)]
pub struct Accrual {
    pub amount: u64,
    pub remainder: u128,
}

pub fn validate_rates(office: u16, advisor: u16, allotment: u16) -> Result<()> {
    require!(
        u32::from(office) + u32::from(allotment) <= u32::from(MAX_ADVISORY_BPS),
        FeeError::FeeCeiling
    );
    require!(advisor <= allotment, FeeError::AdvisorAllotment);
    Ok(())
}

pub fn advisory_fee(
    shares: u64,
    price_time_delta: u128,
    bps: u16,
    remainder: u128,
) -> Result<Accrual> {
    require!(bps <= MAX_ADVISORY_BPS, FeeError::FeeCeiling);
    accrue(shares, price_time_delta, bps, remainder, AUM_DENOMINATOR)
}

pub fn performance_fee(
    shares: u64,
    price: u128,
    high_water_mark: u128,
    remainder: u128,
) -> Result<Accrual> {
    require!(price > 0 && high_water_mark > 0, FeeError::InvalidMathInput);
    accrue(
        shares,
        price.saturating_sub(high_water_mark),
        PERFORMANCE_BPS,
        remainder,
        PERFORMANCE_DENOMINATOR,
    )
}

fn accrue(
    shares: u64,
    delta: u128,
    bps: u16,
    remainder: u128,
    denominator: u128,
) -> Result<Accrual> {
    require!(remainder < denominator, FeeError::InvalidMathInput);
    let numerator = u128::from(shares)
        .checked_mul(delta)
        .and_then(|n| n.checked_mul(u128::from(bps)))
        .and_then(|n| n.checked_add(remainder))
        .ok_or(FeeError::Overflow)?;
    Ok(Accrual {
        amount: u64::try_from(numerator / denominator).map_err(|_| FeeError::Overflow)?,
        remainder: numerator % denominator,
    })
}

pub fn blended_high_water_mark(
    old_shares: u64,
    old_mark: u128,
    new_shares: u64,
    price: u128,
) -> Result<u128> {
    require!(
        price > 0 && (old_shares == 0 || old_mark >= price),
        FeeError::InvalidMathInput
    );
    let shares = u128::from(old_shares) + u128::from(new_shares);
    require!(shares > 0, FeeError::InvalidMathInput);
    let numerator = u128::from(old_shares)
        .checked_mul(old_mark)
        .and_then(|old| {
            u128::from(new_shares)
                .checked_mul(price)
                .and_then(|new| old.checked_add(new))
        })
        .ok_or(FeeError::Overflow)?;
    // Round the basis up so sub-share precision never creates chargeable profit.
    Ok(numerator.div_ceil(shares))
}

pub fn shares_to_redeem(assets: u64, price: u128) -> Result<u64> {
    require!(price > 0, FeeError::InvalidMathInput);
    let numerator = u128::from(assets) * PRICE_SCALE;
    u64::try_from(numerator.div_ceil(price)).map_err(|_| error!(FeeError::Overflow))
}

pub fn cap_fees(owed: [u64; 3], assets: u64) -> [u64; 3] {
    let total: u128 = owed.iter().map(|&n| u128::from(n)).sum();
    if total <= u128::from(assets) {
        return owed;
    }
    let mut paid = [0u64; 3];
    let mut remainders = [0u128; 3];
    for i in 0..3 {
        let numerator = u128::from(owed[i]) * u128::from(assets);
        paid[i] = (numerator / total) as u64;
        remainders[i] = numerator % total;
    }
    // Largest remainders receive the at-most-two leftover atomic units.
    for _ in 0..assets - paid.iter().sum::<u64>() {
        let i = (0..3).max_by_key(|&i| remainders[i]).unwrap();
        paid[i] += 1;
        remainders[i] = 0;
    }
    paid
}

#[cfg(test)]
mod tests {
    use super::*;
    const USDC: u64 = 1_000_000;

    #[test]
    fn ceiling_and_allotment_are_independent() {
        assert!(validate_rates(40, 60, 60).is_ok());
        assert!(validate_rates(40, 61, 60).is_err());
        assert!(validate_rates(41, 0, 60).is_err());
        assert!(validate_rates(u16::MAX, 0, u16::MAX).is_err());
    }

    #[test]
    fn thirty_day_example_separates_capital_and_profit() {
        let integral = PRICE_SCALE * 30 * 24 * 60 * 60;
        assert_eq!(
            advisory_fee(1000 * USDC, integral, 40, 0).unwrap().amount,
            328_767
        );
        assert_eq!(
            advisory_fee(1000 * USDC, integral, 60, 0).unwrap().amount,
            493_150
        );
        assert_eq!(
            performance_fee(1000 * USDC, PRICE_SCALE * 104 / 100, PRICE_SCALE, 0)
                .unwrap()
                .amount,
            6 * USDC
        );
    }

    #[test]
    fn frequent_accrual_preserves_fractional_fees() {
        let once = advisory_fee(1000 * USDC, PRICE_SCALE * 86_400, 40, 0).unwrap();
        let (mut amount, mut remainder) = (0, 0);
        for _ in 0..86_400 {
            let next = advisory_fee(1000 * USDC, PRICE_SCALE, 40, remainder).unwrap();
            amount += next.amount;
            remainder = next.remainder;
        }
        assert_eq!((amount, remainder), (once.amount, once.remainder));
    }

    #[test]
    fn different_prices_charge_only_their_own_elapsed_periods() {
        let day = 86_400;
        let integral = PRICE_SCALE * day + 2 * PRICE_SCALE * day;
        let fee = advisory_fee(1000 * USDC, integral, 100, 0).unwrap();
        let equivalent = advisory_fee(1000 * USDC, PRICE_SCALE * 3 * day, 100, 0).unwrap();
        assert_eq!(fee, equivalent);
    }

    #[test]
    fn loss_and_recovery_to_the_mark_do_not_charge_performance() {
        for price in [PRICE_SCALE / 2, PRICE_SCALE] {
            assert_eq!(
                performance_fee(1000 * USDC, price, PRICE_SCALE, 0)
                    .unwrap()
                    .amount,
                0
            );
        }
    }

    #[test]
    fn top_up_blends_loss_basis_without_treating_deposit_as_profit() {
        let price = PRICE_SCALE / 2;
        let mark = blended_high_water_mark(100 * USDC, PRICE_SCALE, 100 * USDC, price).unwrap();
        assert_eq!(mark, PRICE_SCALE * 3 / 4);
        assert_eq!(
            performance_fee(200 * USDC, price, mark, 0).unwrap().amount,
            0
        );
        assert_eq!(
            blended_high_water_mark(0, 0, 100 * USDC, price).unwrap(),
            price
        );
    }

    #[test]
    fn tiny_gains_preserve_performance_remainders() {
        let first = performance_fee(1, PRICE_SCALE + 1, PRICE_SCALE, 0).unwrap();
        let second = performance_fee(1, 2 * PRICE_SCALE, PRICE_SCALE + 1, first.remainder).unwrap();
        assert_eq!(
            second,
            performance_fee(1, 2 * PRICE_SCALE, PRICE_SCALE, 0).unwrap()
        );
    }

    #[test]
    fn redemption_rounds_shares_up() {
        assert_eq!(shares_to_redeem(10, PRICE_SCALE * 3).unwrap(), 4);
        assert_eq!(shares_to_redeem(9, PRICE_SCALE * 3).unwrap(), 3);
    }

    #[test]
    fn exhausted_position_pays_proportionally_without_creating_debt() {
        assert_eq!(cap_fees([40, 60, 0], 50), [20, 30, 0]);
        assert_eq!(cap_fees([0, 0, 0], 0), [0, 0, 0]);
        assert_eq!(cap_fees([40, 60, 15], 200), [40, 60, 15]);
        for assets in 0..115 {
            let paid = cap_fees([40, 60, 15], assets);
            assert_eq!(paid.iter().sum::<u64>(), assets);
            assert!(paid.iter().zip([40, 60, 15]).all(|(&p, owed)| p <= owed));
        }
        assert_eq!(
            cap_fees([u64::MAX; 3], u64::MAX)
                .iter()
                .map(|&n| u128::from(n))
                .sum::<u128>(),
            u128::from(u64::MAX)
        );
    }

    #[test]
    fn invalid_or_overflowing_inputs_fail() {
        assert!(advisory_fee(u64::MAX, u128::MAX, 100, 0).is_err());
        assert!(advisory_fee(1, 1, 1, AUM_DENOMINATOR).is_err());
        assert!(performance_fee(1, 0, PRICE_SCALE, 0).is_err());
        assert!(shares_to_redeem(1, 0).is_err());
        assert!(blended_high_water_mark(0, 0, 0, PRICE_SCALE).is_err());
    }
}
