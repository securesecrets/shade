#[cfg(test)]
mod tests {
    use crate::contract;
    use cosmwasm_std::{coins, from_binary, testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage}, Extern, StdError, Uint128, HumanAddr};
    use shade_protocol::{utils::asset::Contract, oracle::{self, OracleConfig, QueryAnswer}};
    use crate::query;


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
            Uint128(1_413_500_852_332_497),
            18u8,
            Uint128(1_413_500_852_332_497)
        ),
        normalize_1: (
            // amount of TKN received for 1 sSCRT
            Uint128(1_000_000),
            // TKN 6 decimals
            6u8,
            // price * 10^18
            Uint128(1_000_000_000_000_000_000)
        ),
        normalize_2: (
            // amount of TKN received for 1 sSCRT
            Uint128(1_000_000),
            // TKN 8 decimals
            8u8,
            // price * 10^18
            Uint128(10_000_000_000_000_000)
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
            // 1.62 USD per SCRT
            Uint128(    1_622_110_000_000_000_000),
            // 1 sSCRT -> sETH
            Uint128(        1_413_500_852_332_497),
            // sETH/USD price
            Uint128(1_147_583_319_333_175_746_166),
        ),
        translate_1: (
            // 1.62 USD per SCRT
            Uint128(    1_622_110_000_000_000_000),
            // .000425 ETH per sSCRT
            Uint128(          425_600_000_000_000),
            // 3811.34 ETH per USD
            Uint128(3_811_348_684_210_526_315_789),
        ),
        translate_2: (
            // 1 USD per scrt
            Uint128( 1_000_000_000_000_000_000),
            // 1 sscrt for .1 SHD
            Uint128(   100_000_000_000_000_000),
            // 10 SHD per USD
            Uint128(10_000_000_000_000_000_000),
        ),
        translate_3: (
            // 1 USD per scrt
            Uint128( 1_000_000_000_000_000_000),
            // 1 sscrt for .02 SHD
            Uint128(    20_000_000_000_000_000),
            // 50 SHD per USD
            Uint128(50_000_000_000_000_000_000),
        ),
    }

    #[test]
    fn test_config(){
        let mut deps = mock_dependencies(20, &coins(0, ""));

        // Initialize oracle contract
        let env = mock_env("creator", &coins(0, ""));
        let oracle_init_msg = oracle::InitMsg{
            admin: None,
            band: Contract{
                address: HumanAddr::from(""),
                code_hash: String::from(""),
            },
            sscrt: Contract{
                address: HumanAddr::from(""),
                code_hash: String::from("")
            },
        };
        let res = contract::init(&mut deps, env, oracle_init_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let check_state = OracleConfig{
            admin: HumanAddr::from("creator"),
            band: Contract{
                address: HumanAddr::from(""),
                code_hash: String::from("")
            },
            sscrt: Contract{
                address: HumanAddr::from(""),
                code_hash: String::from("")
            }
        };
        let query_answer = query::config(&mut deps).unwrap();
        let query_result = match query_answer{
            QueryAnswer::Config{config} => config == check_state,
            _ => false,
        };
        assert_eq!(true, query_result);
    }
}
