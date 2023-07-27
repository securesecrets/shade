use crate::c_std::Uint128;
use std::convert::TryFrom;

/// Returns price of `token_1` in `denom` from `token_1/token_2` pair
/// 
/// Denom is whatever `token_2` is quoted in.
/// 
/// ### Arguments (normalized to 10^18)
/// * `scrt_price` - amount of denom per 1 token_2 from some oracle (ex: 9.18USD per 1 SCRT)
/// * `trade_price` - amount of 1 token_2 in token_1 (ex: 0.1 SHD per 1 SCRT)
pub fn translate_price(token_2_price: Uint128, trade_amount: Uint128) -> Uint128 {
    token_2_price.multiply_ratio(10u128.pow(18), trade_amount)
}

/// Normalize the price from some amount (usually snip20) with decimals to 10^18.
/// 
/// ### Arguments
/// * `amount` - unsigned quantity
/// * `decimals` -  number of decimals for received quantity
pub fn normalize_price(amount: Uint128, decimals: u8) -> Uint128 {
    (amount.u128() * 10u128.pow(18u32 - u32::try_from(decimals).unwrap())).into()
}

/// Returns 1 * 10^factor.
pub fn get_precision(factor: u8) -> Uint128 {
    Uint128::from(10u128.pow(factor.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::c_std::Uint128;

    macro_rules! normalize_price_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (amount, decimals, expected) = $value;
                assert_eq!(normalize_price(amount, decimals), expected)
            }
        )*
        }
    }

    normalize_price_tests! {
        normalize_0: (
            Uint128::new(1_413_500_852_332_497),
            18u8,
            Uint128::new(1_413_500_852_332_497)
        ),
        normalize_1: (
            // amount of TKN received for 1 sSCRT
            Uint128::new(1_000_000),
            // TKN 6 decimals
            6u8,
            // price * 10^18
            Uint128::new(1_000_000_000_000_000_000)
        ),
        normalize_2: (
            // amount of TKN received for 1 sSCRT
            Uint128::new(1_000_000),
            // TKN 6 decimals
            6u8,
            // price * 10^18
            Uint128::new(1_000_000_000_000_000_000)
        ),
    }

    macro_rules! translate_price_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (scrt_price, trade_price, expected) = $value;
                assert_eq!(translate_price(scrt_price, trade_price), expected)
            }
        )*
        }
    }

    translate_price_tests! {
        translate_0: (
            // 1.62 USD per SCRT
            Uint128::new(    1_622_110_000_000_000_000),
            // 1 sSCRT -> sETH
            Uint128::new(        1_413_500_852_332_497),
            // sETH/USD price
            Uint128::new(1_147_583_319_333_175_746_166),
        ),
        translate_1: (
            // 1.62 USD per SCRT
            Uint128::new(    1_622_110_000_000_000_000),
            // .000425 ETH per sSCRT
            Uint128::new(          425_600_000_000_000),
            // 3811.34 ETH per USD
            Uint128::new(3_811_348_684_210_526_315_789),
        ),
        translate_2: (
            // 1 USD per scrt
            Uint128::new( 1_000_000_000_000_000_000),
            // 1 sscrt for .1 SHD
            Uint128::new(   100_000_000_000_000_000),
            // 10 SHD per USD
            Uint128::new(10_000_000_000_000_000_000),
        ),
        translate_3: (
            // 1 USD per scrt
            Uint128::new( 1_000_000_000_000_000_000),
            // 1 sscrt for .02 SHD
            Uint128::new(    20_000_000_000_000_000),
            // 50 SHD per USD
            Uint128::new(50_000_000_000_000_000_000),
        ),
    }
}
