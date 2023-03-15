use shade_multi_test::multi::{admin::Admin, snip20::Snip20, snip20_migration::Snip20Migration};
use shade_protocol::{
    admin,
    c_std::{to_binary, Addr, StdResult, Uint128},
    contract_interfaces::snip20_migration,
    multi_test::App,
    snip20::{self, InitialBalance},
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

#[test]
pub fn migration_test() {
    let mut chain = App::default();

    let admin = Addr::unchecked("admin");

    let admin_auth = admin::InstantiateMsg {
        super_admin: Some(admin.clone().to_string()),
    }
    .test_init(
        Admin::default(),
        &mut chain,
        admin.clone(),
        "admin_auth",
        &[],
    )
    .unwrap();

    let token0 = snip20::InstantiateMsg {
        name: "token0".into(),
        admin: Some(admin.clone().into()),
        symbol: "TZERO".into(),
        decimals: 6,
        initial_balances: Some(vec![InitialBalance {
            address: admin.clone().into(),
            amount: Uint128::new(1000000000),
        }]),
        prng_seed: to_binary("").ok().unwrap(),
        query_auth: None,
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
            enable_transfer: Some(true),
        }),
    }
    .test_init(Snip20::default(), &mut chain, admin.clone(), "token0", &[])
    .unwrap();

    snip20::ExecuteMsg::SetViewingKey {
        key: "vk".into(),
        padding: None,
    }
    .test_exec(&token0, &mut chain, admin.clone(), &[])
    .unwrap();

    let token1 = snip20::InstantiateMsg {
        name: "token1".into(),
        admin: Some(admin.clone().into()),
        symbol: "TONE".into(),
        decimals: 6,
        initial_balances: None,
        prng_seed: to_binary("").ok().unwrap(),
        query_auth: None,
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
            enable_transfer: Some(true),
        }),
    }
    .test_init(Snip20::default(), &mut chain, admin.clone(), "token1", &[])
    .unwrap();

    snip20::ExecuteMsg::SetViewingKey {
        key: "vk".into(),
        padding: None,
    }
    .test_exec(&token1, &mut chain, admin.clone(), &[])
    .unwrap();

    let migration_contract = snip20_migration::InstantiateMsg {
        admin: admin_auth.clone().into(),
        tokens: None,
    }
    .test_init(
        Snip20Migration::default(),
        &mut chain,
        admin.clone().into(),
        "migration",
        &[],
    )
    .unwrap();

    match (snip20_migration::QueryMsg::Config {}
        .test_query(&migration_contract, &mut chain)
        .unwrap())
    {
        snip20_migration::QueryAnswer::Config { config } => {
            let expected_config = snip20_migration::Config {
                admin: admin_auth.clone().into(),
            };
            if config == expected_config {
                assert!(true);
            } else {
                assert!(false);
            }
        }
        _ => assert!(false),
    }

    match (snip20_migration::QueryMsg::Metrics {
        token: "lala".into(),
    }
    .test_query(&migration_contract, &mut chain)
    .unwrap())
    {
        snip20_migration::QueryAnswer::Metrics { amount_minted } => match amount_minted {
            None => assert!(true),
            _ => assert!(false),
        },
        _ => assert!(false),
    }

    match (snip20_migration::QueryMsg::RegistragionStatus {
        token: "lala".into(),
    }
    .test_query(&migration_contract, &mut chain)
    .unwrap())
    {
        snip20_migration::QueryAnswer::RegistrationStatus { status } => match status {
            None => assert!(true),
            _ => assert!(false),
        },
        _ => assert!(false),
    }

    snip20::ExecuteMsg::AddMinters {
        minters: vec![migration_contract.address.clone().into()],
        padding: None,
    }
    .test_exec(&token1, &mut chain, admin.clone().into(), &[])
    .unwrap();

    snip20_migration::ExecuteMsg::RegisterMigrationTokens {
        burn_token: token0.clone().into(),
        mint_token: token1.clone().into(),
        padding: None,
    }
    .test_exec(&migration_contract, &mut chain, admin.clone().into(), &[])
    .unwrap();

    println!("{}", token0.address.clone().to_string());
    match (snip20_migration::QueryMsg::RegistragionStatus {
        token: token0.address.clone().into(),
    }
    .test_query(&migration_contract, &mut chain)
    .unwrap())
    {
        snip20_migration::QueryAnswer::RegistrationStatus { status } => match status {
            Some(tokens) => {
                if (tokens.burn_token.clone() == token0.clone().into()
                    && tokens.mint_token == token1.clone().into())
                {
                    assert!(true);
                } else {
                    assert!(false);
                }
            }
            None => assert!(false),
        },
        _ => assert!(false),
    }

    match (snip20_migration::QueryMsg::Metrics {
        token: token1.address.clone().into(),
    }
    .test_query(&migration_contract, &mut chain)
    .unwrap())
    {
        snip20_migration::QueryAnswer::Metrics { amount_minted } => match amount_minted {
            Some(tokens) => {
                if tokens == Uint128::zero() {
                    assert!(true);
                } else {
                    assert!(false);
                }
            }
            None => assert!(false),
        },
        _ => assert!(false),
    }

    snip20::ExecuteMsg::Send {
        recipient: migration_contract.clone().address.into(),
        recipient_code_hash: None,
        amount: Uint128::new(1000000),
        msg: None,
        memo: None,
        padding: None,
    }
    .test_exec(&token0, &mut chain, admin.clone().into(), &[])
    .unwrap();

    match (snip20::QueryMsg::Balance {
        address: admin.clone().into(),
        key: "vk".into(),
    }
    .test_query(&token1, &mut chain)
    .unwrap())
    {
        snip20::QueryAnswer::Balance { amount } => assert!(amount == Uint128::new(1000000)),
        _ => assert!(false),
    }

    match (snip20_migration::QueryMsg::Metrics {
        token: token1.address.clone().into(),
    }
    .test_query(&migration_contract, &mut chain)
    .unwrap())
    {
        snip20_migration::QueryAnswer::Metrics { amount_minted } => match amount_minted {
            Some(amount) => {
                println!("{}", amount);
                assert!(amount == Uint128::new(1000000));
            }
            None => assert!(false),
        },
        _ => assert!(false),
    }
}
