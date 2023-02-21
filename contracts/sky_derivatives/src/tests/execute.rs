use shade_protocol::c_std::{
    to_binary, from_binary,
    Addr, StdError, Uint128, Coin,
    Decimal,
};
use shade_protocol::contract_interfaces::{
    sky::{
        cycles::{
            ArbPair, Derivative,
            DerivativeType,
        },
        sky_derivatives::{
            Config,
            ExecuteAnswer,
            ExecuteMsg,
            QueryAnswer,
            QueryMsg,
            TradingFees,
        },
    },
    admin,
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

use crate::tests::init;

#[test]
fn update_config() {
    let (mut chain, admin, base, deriv, arb, config) = init();

    // Test no changes
    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: None,
        derivative: None,
        trading_fees: None,
        max_arb_amount: None,
        viewing_key: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::Config {}
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::Config {
            config: config.clone(),
        },
    );
 
    // Test with changes
    let new_deriv = stkd::MockInstantiateMsg {
        name: "derivative2".to_string(),
        symbol: "stkd-SCRT2".to_string(),
        decimals: 6,
        price: Uint128::new(2),
    }.test_init(
        MockStkd::default(), 
        &mut chain, 
        Addr::unchecked("admin"), 
        "stkd-SCRT2", 
        &[]
    ).unwrap();

    let new_derivative = Derivative {
        contract: new_deriv.clone().into(),
        original_asset: base.clone().into(),
        staking_type: DerivativeType::StkdScrt,
    };

    let new_fees = TradingFees {
        dex_fee: Decimal::raw(1_000_000),
        stake_fee: Decimal::raw(1_000_000),
        unbond_fee: Decimal::raw(1_000_000),
    };

    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: Some(Addr::unchecked("treasury2")),
        derivative: Some(new_derivative.clone()),
        trading_fees: Some(new_fees.clone()),
        max_arb_amount: Some(Uint128::new(1_000_000_000)),
        viewing_key: Some("key2".into()),
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::Config {}
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::Config {
            config: Config {
                shade_admin_addr: config.shade_admin_addr,
                treasury: Addr::unchecked("treasury2"),
                derivative: new_derivative,
                trading_fees: new_fees,
                max_arb_amount: Uint128::new(1_000_000_000),
                viewing_key: "key2".into(),
            },
        },
    );

    // Test new viewing keys
    assert_eq!(
        stkd::QueryMsg::Balance {
            address: arb.address,
            key: "key".into(),
        }.test_query::<stkd::QueryAnswer>(&base, &chain).unwrap(),
        stkd::QueryAnswer::Balance {
            amount: Uint128::zero(),
        },
    );

    // Test bad admin


    // Test bad trading fees
}

#[test]
fn set_dex_pairs() {
    assert!(false);
}

#[test]
fn set_pair() {
    assert!(false);
}

#[test]
fn add_pair() {
    assert!(false);
}

#[test]
fn remove_pair() {
    assert!(false);
}

#[test]
fn arb_pair() {
    assert!(false);
}

#[test]
fn arb_all_pairs() {
    assert!(false);
}

#[test]
fn adapter_unbond() {
    assert!(false);
}

#[test]
fn adapter_claim() {
    assert!(false);
}

#[test]
fn adapter_update() {
    assert!(false);
}
