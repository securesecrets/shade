use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{
    self,
    coins,
    from_binary,
    to_binary,
    Binary,
    Env,
    DepsMut,
    Response,
    Addr,
    Response,
    StdError,
    StdResult,
};

use shade_protocol::{
    contract_interfaces::{
        mint::mint::{ExecuteMsg, InstantiateMsg, QueryAnswer, QueryMsg},
        oracles::band::{BandQuery, ReferenceData},
    },
    utils::{
        asset::Contract,
        price::{normalize_price, translate_price},
    },
};

#[test]
fn capture_calc() {
    let amount = Uint128::new(1_000_000_000_000_000_000u128);
    //10%
    let capture = Uint128::new(100_000_000_000_000_000u128);
    let expected = Uint128::new(100_000_000_000_000_000u128);
    let value = mint::handle::calculate_portion(amount, capture);
    assert_eq!(value, expected);
}

macro_rules! mint_algorithm_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (in_price, in_amount, in_decimals, target_price, target_decimals, expected_value) = $value;
                assert_eq!(mint::handle::calculate_mint(in_price, in_amount, in_decimals, target_price, target_decimals), expected_value);
            }
        )*
    }
}

mint_algorithm_tests! {
    mint_simple_0: (
        // In this example the "sent" value is 1 with 6 decimal places
        // The mint value will be 1 with 3 decimal places
        Uint128::new(1_000_000_000_000_000_000), //Burn price
        Uint128::new(1_000_000),                 //Burn amount
        6u8,                                //Burn decimals
        Uint128::new(1_000_000_000_000_000_000), //Mint price
        3u8,                                //Mint decimals
        Uint128::new(1_000),                     //Expected value
    ),
    mint_simple_1: (
        // In this example the "sent" value is 1 with 8 decimal places
        // The mint value will be 1 with 3 decimal places
        Uint128::new(1_000_000_000_000_000_000), //Burn price
        Uint128::new(1_000_000),                 //Burn amount
        6u8,                                //Burn decimals
        Uint128::new(1_000_000_000_000_000_000), //Mint price
        8u8,                                //Mint decimals
        Uint128::new(100_000_000),                     //Expected value
    ),
    mint_complex_0: (
        // In this example the "sent" value is 1.8 with 6 decimal places
        // The mint value will be 3.6 with 12 decimal places
        Uint128::new(2_000_000_000_000_000_000),
        Uint128::new(1_800_000),
        6u8,
        Uint128::new(1_000_000_000_000_000_000),
        12u8,
        Uint128::new(3_600_000_000_000),
    ),
    mint_complex_1: (
        // In amount is 50.000 valued at 20
        // target price is 100$ with 6 decimals
        Uint128::new(20_000_000_000_000_000_000),
        Uint128::new(50_000),
        3u8,
        Uint128::new(100_000_000_000_000_000_000),
        6u8,
        Uint128::new(10_000_000),
    ),
    mint_complex_2: (
        // In amount is 10,000,000 valued at 100
        // Target price is $10 with 6 decimals
        Uint128::new(1_000_000_000_000_000_000_000),
        Uint128::new(100_000_000_000_000),
        8u8,
        Uint128::new(10_000_000_000_000_000_000),
        6u8,
        Uint128::new(100_000_000_000_000),
    ),
    /*
    mint_overflow_0: (
        // In amount is 1,000,000,000,000,000,000,000,000 valued at 1,000
        // Target price is $5 with 6 decimals
        Uint128::new(1_000_000_000_000_000_000_000),
        Uint128::new(100_000_000_000_000_000_000_000_000_000_000),
        8u8,
        Uint128::new(5_000_000_000_000_000_000),
        6u8,
        Uint128::new(500_000_000_000_000_000_000_000_000_000_000_000),
    ),
    */
}
