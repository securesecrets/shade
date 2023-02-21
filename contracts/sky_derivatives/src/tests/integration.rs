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
fn integration() {
    assert!(false);
}
