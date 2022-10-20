use shade_multi_test::multi::{admin::init_admin_auth, snip20::Snip20, treasury::Treasury};
use shade_protocol::{
    c_std::{from_binary, to_binary, Addr, Coin, Uint128},
    contract_interfaces::{dao::treasury, snip20},
    multi_test::App,
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

// Add other adapters here as they come
fn wrap_coins_test(coins: Vec<Coin>) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
    let _user = Addr::unchecked("user");
    //let validator = Addr::unchecked("validator");
    let admin_auth = init_admin_auth(&mut app, &admin);

    let viewing_key = "viewing_key".to_string();

    let mut tokens = vec![];

    let fail_coin = Coin {
        denom: "fail".into(),
        amount: Uint128::new(100),
    };

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

    let mut all_coins = coins.clone();
    all_coins.push(fail_coin.clone());

    app.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &treasury.address.clone(), all_coins.clone())
            .unwrap();
    });

    // Wrap
    let wrap_resp = treasury::ExecuteMsg::WrapCoins {}
        .test_exec(&treasury, &mut app, admin.clone(), &[])
        .unwrap();

    match from_binary(&wrap_resp.data.unwrap()).ok().unwrap() {
        treasury::ExecuteAnswer::WrapCoins { success, failed } => {
            assert!(success == coins, "All coins succeed");
            assert!(failed == vec![fail_coin], "Unconfigured coin fails");
        }
        _ => {
            panic!("WrapCoins bad response");
        }
    }

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
            _ => panic!("Balance query Failed"),
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
