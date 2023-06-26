use shade_protocol::c_std::{to_binary, Addr, BlockInfo, Timestamp, Uint128};

use shade_protocol::{
    contract_interfaces::{basic_staking, query_auth, snip20},
    multi_test::App,
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

use shade_multi_test::multi::{
    admin::{init_admin_auth, Admin},
    basic_staking::BasicStaking,
    query_auth::QueryAuth,
    snip20::Snip20,
};

#[test]
fn update_config() {
    let mut app = App::default();

    // init block time for predictable behavior
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(0),
        chain_id: "chain_id".to_string(),
    });

    let viewing_key = "unguessable".to_string();
    let admin_user = Addr::unchecked("admin");

    let token = snip20::InstantiateMsg {
        name: "stake_token".into(),
        admin: Some(admin_user.to_string().clone()),
        symbol: "STKN".into(),
        decimals: 6,
        initial_balances: Some(vec![]),
        query_auth: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(false),
            enable_redeem: Some(false),
            enable_mint: Some(false),
            enable_burn: Some(false),
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

    let admin_contract = init_admin_auth(&mut app, &admin_user);

    let query_contract = query_auth::InstantiateMsg {
        admin_auth: admin_contract.clone().into(),
        prng_seed: to_binary("").ok().unwrap(),
    }
    .test_init(
        QueryAuth::default(),
        &mut app,
        admin_user.clone(),
        "query_auth",
        &[],
    )
    .unwrap();

    let basic_staking = basic_staking::InstantiateMsg {
        admin_auth: admin_contract.clone().into(),
        query_auth: query_contract.clone().into(),
        airdrop: None,
        stake_token: token.clone().into(),
        unbond_period: Uint128::zero(),
        max_user_pools: Uint128::one(),
        viewing_key: viewing_key.clone(),
    }
    .test_init(
        BasicStaking::default(),
        &mut app,
        admin_user.clone(),
        "basic_staking",
        &[],
    )
    .unwrap();

    let mut config_match = match (basic_staking::QueryMsg::Config {})
        .test_query(&basic_staking, &app)
        .unwrap()
    {
        basic_staking::QueryAnswer::Config { config } => config,
        _ => panic!("Config query failed"),
    };

    config_match.admin_auth = query_contract.clone().into();
    config_match.query_auth = admin_contract.clone().into();
    config_match.airdrop = Some(admin_contract.clone().into());
    config_match.unbond_period = Uint128::new(100);
    config_match.max_user_pools = Uint128::new(10);

    // update config fields
    basic_staking::ExecuteMsg::UpdateConfig {
        admin_auth: Some(config_match.admin_auth.clone().into()),
        query_auth: Some(config_match.query_auth.clone().into()),
        airdrop: Some(RawContract {
            address: admin_contract.address.to_string(),
            code_hash: admin_contract.code_hash,
        }),
        unbond_period: Some(config_match.unbond_period.clone()),
        max_user_pools: Some(config_match.max_user_pools.clone()),
        padding: None,
    }
    .test_exec(&basic_staking, &mut app, admin_user.clone(), &[])
    .unwrap();

    match (basic_staking::QueryMsg::Config {})
        .test_query(&basic_staking, &app)
        .unwrap()
    {
        basic_staking::QueryAnswer::Config { config } => {
            assert_eq!(config_match.clone(), config);
        }
        _ => panic!("Config query failed"),
    }
}
