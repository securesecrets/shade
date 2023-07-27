use shade_multi_test::multi::{
    admin::{init_admin_auth, Admin},
    snip20::Snip20,
    snip20_migration::Snip20Migration,
};
use shade_protocol::{
    c_std::{to_binary, Addr, BlockInfo, Timestamp, Uint128},
    contract_interfaces::{snip20, snip20_migration},
    multi_test::App,
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

#[test]
fn test_admin() {
    let mut app = App::default();

    let admin_user = Addr::unchecked("admin");
    let not_admin = Addr::unchecked("not_admin");

    let admin_contract = init_admin_auth(&mut app, &admin_user);

    let snip20_migration_contract = snip20_migration::InstantiateMsg {
        admin: admin_contract.clone().into(),
        tokens: None,
    }
    .test_init(
        Snip20Migration::default(),
        &mut app,
        admin_user.clone(),
        "snip20_migration",
        &[],
    )
    .unwrap();

    let token0 = snip20::InstantiateMsg {
        name: "burn_token".into(),
        admin: Some(admin_user.to_string().clone()),
        symbol: "BURN".into(),
        decimals: 6,
        initial_balances: Some(vec![snip20::InitialBalance {
            amount: Uint128::new(100000000000),
            address: admin_user.to_string(),
        }]),
        query_auth: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(false),
            enable_redeem: Some(false),
            enable_mint: Some(true),
            enable_burn: Some(true),
            enable_transfer: Some(true),
        }),
    }
    .test_init(
        Snip20::default(),
        &mut app,
        admin_user.clone(),
        "stake_token",
        &[],
    )
    .unwrap();

    let token1 = snip20::InstantiateMsg {
        name: "mint_token".into(),
        admin: Some(admin_user.to_string().clone()),
        symbol: "MINT".into(),
        decimals: 6,
        initial_balances: Some(vec![snip20::InitialBalance {
            amount: Uint128::new(100000000000),
            address: admin_user.to_string(),
        }]),
        query_auth: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(false),
            enable_redeem: Some(false),
            enable_mint: Some(true),
            enable_burn: Some(true),
            enable_transfer: Some(true),
        }),
    }
    .test_init(
        Snip20::default(),
        &mut app,
        admin_user.clone(),
        "stake_token",
        &[],
    )
    .unwrap();

    let msg_resp = snip20_migration::ExecuteMsg::RegisterMigrationTokens {
        burn_token: token0.clone().into(),
        mint_token: token1.clone().into(),
        burnable: None,
        padding: None,
    }
    .test_exec(&snip20_migration_contract, &mut app, admin_user.clone(), &[
    ])
    .unwrap();

    let msg_resp = snip20_migration::ExecuteMsg::RegisterMigrationTokens {
        burn_token: token0.into(),
        mint_token: token1.into(),
        burnable: None,
        padding: None,
    }
    .test_exec(&snip20_migration_contract, &mut app, not_admin.clone(), &[])
    .unwrap_err();
}
