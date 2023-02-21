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
            QueryAnswer,
            QueryMsg,
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

use crate::tests::init;

#[test]
fn get_config() {
    let (chain, _, base, deriv, arb, config) = init();

    assert_eq!(
        QueryMsg::Config { }
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::Config {
            config,
        },
    );

    // Make sure viewing keys work
    assert_eq!(
        snip20::QueryMsg::Balance {
            address: arb.address.to_string(),
            key: "key".into(),
        }.test_query::<snip20::QueryAnswer>(&base, &chain).unwrap(),
        snip20::QueryAnswer::Balance {
            amount: Uint128::zero(),
        },
    )
}

#[test]
fn dex_pairs() {
    assert!(false);
}

#[test]
fn is_profitable() {
    assert!(false);
}

#[test]
fn is_any_pair_profitable() {
    assert!(false);
}

// Adapter Tests
#[test]
fn adapter_balance() {
    assert!(false);
}

#[test]
fn adapter_claimable() {
    assert!(false);
}

#[test]
fn adapter_unbonding() {
    assert!(false);
}

#[test]
fn adapter_unbondable() {
    assert!(false);
}

#[test]
fn adapter_reserves() {
    assert!(false);
}
