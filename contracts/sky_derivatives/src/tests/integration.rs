use shade_protocol::c_std::{
    coins, from_binary, to_binary,
    Addr, StdError,
    Binary, StdResult, Env,
    Uint128,
    Coin, Decimal,
    Validator,
};

use shade_protocol::{
    contract_interfaces::{
        sky::{
            cycles::{
                ArbPair, Derivative,
                DerivativeType,
            },
            sky_derivatives::{
                InstantiateMsg,
                TradingFees,
            },
        },
        snip20,
    },
    utils::{
        MultiTestable,
        InstantiateCallback,
        ExecuteCallback,
        Query,
        asset::Contract,
    },
};

use shade_protocol::multi_test::App;
use shade_multi_test::multi::{
    admin::init_admin_auth,
    snip20::Snip20,
    sky_derivatives::SkyDerivatives,
};

#[test]
fn instantiate() {
    let mut chain = App::default();

    let admin = Addr::unchecked("admin");
    let user = Addr::unchecked("user");

    let original_token = snip20::InstantiateMsg {
        name: "secret SCRT".into(),
        admin: Some("admin".into()),
        symbol: "SSCRT".into(),
        decimals: 6,
        initial_balances: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(true),
            enable_transfer: Some(true),
        }),
        query_auth: None,
    }.test_init(Snip20::default(), &mut chain, admin.clone(), "token", &[]).unwrap();

    let derivative_token = snip20::InstantiateMsg {
        name: "staked Secret".into(),
        admin: Some("admin".into()),
        symbol: "DERIV".into(),
        decimals: 6,
        initial_balances: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(true),
            enable_transfer: Some(true),
        }),
        query_auth: None,
    }.test_init(Snip20::default(), &mut chain, admin.clone(), "token", &[]).unwrap();

    let shd_admin = init_admin_auth(&mut chain, &admin);
    let treasury = Addr::unchecked("treasury");
    let derivative = Derivative {
        contract: derivative_token.into(),
        original_asset: original_token.into(),
        staking_type: DerivativeType::StkdScrt,
    };

    let dex_fee = Decimal::raw(999_500);
    let stake_fee = Decimal::raw(998_000);
    let unbond_fee = Decimal::raw(997_000);
    let trading_fees = TradingFees { dex_fee, stake_fee, unbond_fee };

    let dex_pairs: Vec<ArbPair> = vec![];

    let sky_arb = InstantiateMsg {
        shade_admin_addr: shd_admin.into(),
        treasury,
        derivative,
        trading_fees,
        dex_pairs,
        max_arb_amount: Uint128::MAX,
        viewing_key: "key".into(),
    }.test_init(SkyDerivatives::default(), &mut chain, admin.clone(), "arb", &[]).unwrap();
}

#[test]
fn integration() {
    assert!(false);
}
