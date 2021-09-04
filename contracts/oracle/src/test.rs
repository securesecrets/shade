#[cfg(test)]
mod tests {
    use crate::query;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockStorage, MockApi, MockQuerier};
    use cosmwasm_std::{coins, from_binary, Uint128};
    use shade_protocol::asset::Contract;

    macro_rules! normalize_price_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (amount, decimals, expected) = $value;
                assert_eq!(query::normalize_price(amount, decimals), expected)
            }
        )*
        }
    }

    normalize_price_tests! {
        normalize_0: (
            Uint128(1413500852332497), 
            18u8, 
            Uint128(1413500852332497)
        ),
    }

    macro_rules! translate_price_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (scrt_price, trade_price, expected) = $value;
                assert_eq!(query::translate_price(scrt_price, trade_price), expected)
            }
        )*
        }
    }

    translate_price_tests! {
        translate_0: (
            //SCRT/USD price
            Uint128(    1_622_110_000_000_000_000), 
            // 1 sSCRT -> sETH
            Uint128(        1_413_500_852_332_497),
            // sETH/USD price
            Uint128(1_147_583_319_333_175_746_166),
        ),
        translate_1: (
            //SCRT/USD price
            Uint128(    1_622_110_000_000_000_000), 
            // SCRT/ETH
            Uint128(          425_600_000_000_000),
            // sETH/USD price
            Uint128(3_811_348_684_210_526_315_789),
        ),
    }
}
