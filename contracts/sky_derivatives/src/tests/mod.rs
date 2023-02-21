//mod integration;
mod query;
mod execute;

use shade_protocol::c_std::{
    to_binary, from_binary,
    Addr, StdError, Uint128, Coin,
    ContractInfo, Decimal,
};
use shade_protocol::contract_interfaces::{
    sky::{
        cycles::{
            ArbPair, Derivative,
            DerivativeType,
        },
        sky_derivatives::{
            Config,
            InstantiateMsg,
            TradingFees,
        },
    },
    snip20,
};
use shade_protocol_temp::{
    mock,
    stkd,
};
use shade_protocol::utils::{
    asset::Contract,
    ExecuteCallback,
    InstantiateCallback,
    MultiTestable,
    Query,
};
use shade_protocol_temp::utils::{
    InstantiateCallback as OtherInstantiateCallback,
    MultiTestable as OtherMultiTestable,
    ExecuteCallback as OtherExecuteCallback,
    Query as OtherQuery,
};
use shade_protocol::multi_test::App;
use shade_multi_test::multi::{
    admin::init_admin_auth,
    snip20::Snip20,
    sky_derivatives::SkyDerivatives,
};
use shade_multi_test_temp::multi::mock_stkd::MockStkd;

fn init() -> (App, ContractInfo, ContractInfo, ContractInfo, ContractInfo, Config) {
    let mut chain = App::default();

    // Init balances
    let admin = Addr::unchecked("admin");
    chain.init_modules(|router, _, storage| {
        router.bank.init_balance(storage, &admin, vec![Coin {
            amount: Uint128::new(1_000_000),
            denom: "uscrt".into(),
        }]).unwrap();
    });

    // Base snip20
    let base_snip20 = snip20::InstantiateMsg {
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

    // Stkd
    let deriv = stkd::MockInstantiateMsg {
        name: "derivative".to_string(),
        symbol: "stkd-SCRT".to_string(),
        decimals: 6,
        price: Uint128::new(2),
    }.test_init(MockStkd::default(), &mut chain, admin.clone(), "stkd-SCRT", &[]).unwrap();

    // Sky Derivatives
    let shd_admin = init_admin_auth(&mut chain, &admin);
    let treasury = Addr::unchecked("treasury");
    let derivative = Derivative {
        contract: deriv.clone().into(),
        original_asset: base_snip20.clone().into(),
        staking_type: DerivativeType::StkdScrt,
    };

    let dex_fee = Decimal::raw(999_500);
    let stake_fee = Decimal::raw(998_000);
    let unbond_fee = Decimal::raw(997_000);
    let trading_fees = TradingFees { dex_fee, stake_fee, unbond_fee };
    let dex_pairs: Vec<ArbPair> = vec![];
    let config = Config {
        shade_admin_addr: shd_admin.clone().into(),
        treasury: treasury.clone(),
        derivative: derivative.clone(),
        trading_fees: trading_fees.clone(),
        max_arb_amount: Uint128::MAX,
        viewing_key: "key".into(),
    };
    let sky_arb = InstantiateMsg {
        shade_admin_addr: shd_admin.clone().into(),
        treasury,
        derivative,
        trading_fees,
        dex_pairs,
        max_arb_amount: Uint128::MAX,
        viewing_key: "key".into(),
    }.test_init(SkyDerivatives::default(), &mut chain, admin.clone(), "arb", &[]).unwrap();

    (chain, shd_admin, base_snip20, deriv, sky_arb, config)
}

#[test]
fn instantiate() {
    let mut chain = App::default();

    let admin = Addr::unchecked("admin");

    let base_snip20 = snip20::InstantiateMsg {
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

    let deriv = stkd::MockInstantiateMsg {
        name: "derivative".to_string(),
        symbol: "stkd-SCRT".to_string(),
        decimals: 6,
        price: Uint128::new(2),
    }.test_init(MockStkd::default(), &mut chain, admin.clone(), "stkd-SCRT", &[]).unwrap();

    let shd_admin = init_admin_auth(&mut chain, &admin);
    let treasury = Addr::unchecked("treasury");
    let derivative = Derivative {
        contract: deriv.into(),
        original_asset: base_snip20.into(),
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
