use mock_adapter;
use shade_multi_test::{
    interfaces,
    multi::{
        admin::init_admin_auth,
        mock_adapter::MockAdapter,
        snip20::Snip20,
        treasury::Treasury,
        treasury_manager::TreasuryManager,
    },
};
use shade_protocol::{
    c_std::{
        coins,
        from_binary,
        to_binary,
        Addr,
        Binary,
        Coin,
        ContractInfo,
        Decimal,
        Env,
        StdError,
        StdResult,
        Uint128,
        Validator,
    },
    multi_test::{App, BankSudo, StakingSudo, SudoMsg},
};
use shade_protocol::{
    contract_interfaces::{
        dao::{
            adapter,
            manager,
            //mock_adapter,
            treasury,
            treasury::{Allowance, AllowanceType, RunLevel},
            treasury_manager::{self, Allocation, AllocationType},
        },
        snip20,
    },
    utils::{
        asset::{Contract, RawContract},
        cycle::{utc_from_timestamp, Cycle},
        storage::plus::period_storage::Period,
        ExecuteCallback,
        InstantiateCallback,
        MultiTestable,
        Query,
    },
};

use serde_json;

//TODO test with manager
// Add other adapters here as they come
fn batch_balance_test(amounts: Vec<Uint128>) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
    let user = Addr::unchecked("user");
    let admin_auth = init_admin_auth(&mut app, &admin);
    let viewing_key = "veiwing_key".to_string();

    let mut tokens = vec![];

    let treasury = treasury::InstantiateMsg {
        admin_auth: admin_auth.clone().into(),
        viewing_key: viewing_key.clone(),
        multisig: admin.to_string().clone(),
    }
    .test_init(Treasury::default(), &mut app, admin.clone(), "treasury", &[
    ])
    .unwrap();

    for amount in amounts.clone() {
        let token = snip20::InstantiateMsg {
            name: "token".into(),
            admin: Some("admin".into()),
            symbol: "TKN".into(),
            decimals: 6,
            initial_balances: Some(vec![snip20::InitialBalance {
                address: treasury.address.to_string().clone(),
                amount,
            }]),
            prng_seed: to_binary("").ok().unwrap(),
            config: Some(snip20::InitConfig {
                public_total_supply: Some(true),
                enable_deposit: Some(true),
                enable_redeem: Some(true),
                enable_mint: Some(false),
                enable_burn: Some(false),
                enable_transfer: Some(true),
            }),
            query_auth: None,
        }
        .test_init(
            Snip20::default(),
            &mut app,
            admin.clone(),
            &amount.to_string(),
            &[],
        )
        .unwrap();

        treasury::ExecuteMsg::RegisterAsset {
            contract: token.clone().into(),
        }
        .test_exec(&treasury, &mut app, admin.clone(), &[])
        .unwrap();

        tokens.push(token);
    }

    // Treasury Balances
    let balances: Vec<Uint128> = treasury::QueryMsg::BatchBalance {
        assets: tokens
            .iter()
            .map(|t| t.address.to_string().clone())
            .collect(),
    }
    .test_query(&treasury, &app)
    .unwrap();

    assert!(balances == amounts, "Reported balances match inputs");
}

macro_rules! batch_balance_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                batch_balance_test($value.into_iter().map(|a| Uint128::new(a as u128)).collect());
            }
        )*
    }
}

batch_balance_tests! {
    batch_balances_0: vec![10, 23840, 8402840, 123456, 0],
}
