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

// Add other adapters here as they come
fn wrap_coins_test(coins: Vec<Coin>) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
    let user = Addr::unchecked("user");
    //let validator = Addr::unchecked("validator");
    let admin_auth = init_admin_auth(&mut app, &admin);

    let viewing_key = "viewing_key".to_string();

    let mut tokens = vec![];

    for coin in coins.clone() {
        let token = snip20::InstantiateMsg {
            name: coin.denom.clone(),
            admin: Some("admin".into()),
            symbol: coin.denom.to_uppercase().clone(),
            decimals: 6,
            initial_balances: None,
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
        .test_init(Snip20::default(), &mut app, admin.clone(), &coin.denom, &[])
        .unwrap();

        tokens.push(token);
    }

    let treasury = treasury::InstantiateMsg {
        admin_auth: admin_auth.clone().into(),
        viewing_key: viewing_key.clone(),
        multisig: admin.to_string().clone(),
    }
    .test_init(Treasury::default(), &mut app, admin.clone(), "treasury", &[
    ])
    .unwrap();

    /*
    // Set admin viewing key
    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();
    */

    // Register treasury assets
    for (token, coin) in tokens.iter().zip(coins.clone().iter()) {
        treasury::ExecuteMsg::RegisterAsset {
            contract: token.clone().into(),
        }
        .test_exec(&treasury, &mut app, admin.clone(), &[])
        .unwrap();

        treasury::ExecuteMsg::RegisterWrap {
            denom: coin.denom.clone(),
            contract: RawContract {
                address: token.address.clone().into(),
                code_hash: token.code_hash.clone(),
            },
        }
        .test_exec(&treasury, &mut app, admin.clone(), &[])
        .unwrap();
    }

    app.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &treasury.address.clone(), coins.clone())
            .unwrap();
    });

    // Wrap
    treasury::ExecuteMsg::WrapCoins {}
        .test_exec(&treasury, &mut app, admin.clone(), &[])
        .unwrap();

    // Treasury Balances
    for (token, coin) in tokens.iter().zip(coins.iter()) {
        match (treasury::QueryMsg::Balance {
            asset: token.address.to_string().clone(),
        }
        .test_query(&treasury, &app)
        .unwrap())
        {
            treasury::QueryAnswer::Balance { amount } => {
                assert_eq!(amount, coin.amount, "Treasury Balance");
            }
            _ => panic!("Query Failed"),
        };
    }
}

macro_rules! wrap_coins_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let coins = $value;
                wrap_coins_test(coins);
            }
        )*
    }
}

wrap_coins_tests! {
    wrap_sscrt: vec![Coin { denom: "uscrt".into(), amount: Uint128::new(100) }],
    //wrap_other: vec![Coin { denom: "other".into(), amount: Uint128::new(100) }],
}
