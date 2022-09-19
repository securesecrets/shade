use shade_multi_test::multi::{admin::init_admin_auth, snip20::Snip20, treasury::Treasury};
use shade_protocol::{
    c_std::{to_binary, Addr, Uint128},
    contract_interfaces::{dao::treasury, snip20},
    multi_test::App,
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

//TODO test with manager
// Add other adapters here as they come
fn batch_balance_test(amounts: Vec<Uint128>) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
    let _user = Addr::unchecked("user");
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
