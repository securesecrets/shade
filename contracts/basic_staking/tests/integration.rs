use shade_protocol::c_std::{
    to_binary,
    Addr,
    BlockInfo,
    Coin,
    Decimal,
    Delegation,
    Timestamp,
    Uint128,
    Validator,
};

use shade_protocol::{
    contract_interfaces::{admin, basic_staking, query_auth, snip20},
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

use shade_multi_test::multi::{
    admin::{init_admin_auth, Admin},
    basic_staking::BasicStaking,
    query_auth::QueryAuth,
    snip20::Snip20,
};
use shade_protocol::multi_test::{App, StakingSudo, SudoMsg};

// Add other adapters here as they come
fn single_stake_with_rewards(
    stake_amount: Uint128,
    reward_amount: Uint128,
    reward_start: Uint128,
    reward_end: Uint128,
) {
    let mut app = App::default();

    // init block time for predictable behavior
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(0),
        chain_id: "chain_id".to_string(),
    });

    let viewing_key = "unguessable".to_string();
    let admin_user = Addr::unchecked("admin");
    let staking_user = Addr::unchecked("staker");

    let token = snip20::InstantiateMsg {
        name: "stake_token".into(),
        admin: Some(admin_user.to_string().clone()),
        symbol: "STKN".into(),
        decimals: 6,
        initial_balances: Some(vec![snip20::InitialBalance {
            amount: stake_amount + reward_amount,
            address: staking_user.to_string(),
        }]),
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

    // set staking_user viewing key
    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, staking_user.clone(), &[])
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

    // set staking user VK
    query_auth::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&query_contract, &mut app, staking_user.clone(), &[])
    .unwrap();

    let basic_staking = basic_staking::InstantiateMsg {
        admin_auth: admin_contract.into(),
        query_auth: query_contract.into(),
        stake_token: token.clone().into(),
        unbond_period: Uint128::new(100),
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
    println!("BASIC STAKING {}", basic_staking.address);

    // Pre-staking user balance
    match (basic_staking::QueryMsg::Balance {
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: staking_user.clone().into(),
        },
    })
    .test_query(&basic_staking, &app)
    .unwrap()
    {
        basic_staking::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "Pre-Stake Balance");
        }
        _ => {
            panic!("Staking balance query failed");
        }
    };

    // Stake funds
    snip20::ExecuteMsg::Send {
        recipient: basic_staking.address.to_string().clone(),
        recipient_code_hash: None,
        amount: stake_amount,
        msg: Some(to_binary(&basic_staking::Action::Stake {}).unwrap()),
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, staking_user.clone(), &[])
    .unwrap();

    // Post-staking user balance
    match (basic_staking::QueryMsg::Balance {
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: staking_user.clone().into(),
        },
    })
    .test_query(&basic_staking, &app)
    .unwrap()
    {
        basic_staking::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, stake_amount, "Post-Stake Balance");
        }
        _ => {
            panic!("Staking balance query failed");
        }
    };

    // Init Rewards
    snip20::ExecuteMsg::Send {
        recipient: basic_staking.address.to_string().clone(),
        recipient_code_hash: None,
        amount: reward_amount,
        msg: Some(
            to_binary(&basic_staking::Action::Rewards {
                start: reward_start,
                end: reward_end,
            })
            .unwrap(),
        ),
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, staking_user.clone(), &[])
    .unwrap();

    // Check reward pool
    match (basic_staking::QueryMsg::RewardPool {})
        .test_query(&basic_staking, &app)
        .unwrap()
    {
        basic_staking::QueryAnswer::RewardPool { rewards } => {
            assert_eq!(rewards[0].amount, reward_amount, "Reward Pool Amount");
            assert_eq!(rewards[0].start, reward_start, "Reward Pool Start");
            assert_eq!(rewards[0].end, reward_end, "Reward Pool End");
        }
        _ => {
            panic!("Staking balance query failed");
        }
    };

    // Move forward to reward start
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(reward_start.u128() as u64),
        chain_id: "chain_id".to_string(),
    });

    // let reward_duration = reward_end - reward_start;
    // Total Steps + 2  to ensure we claim at least 1x after rewards period is over
    // let step_size = (reward_duration / claim_steps).u128();
    // let rewards_per_step = reward_amount / claim_steps;

    // Move to end of rewards
    let set_end: u64 = reward_end.u128() as u64; // + 100000000;
    println!("Fast-forward to end {}", set_end);
    app.set_block(BlockInfo {
        height: 10,
        time: Timestamp::from_seconds(set_end),
        chain_id: "chain_id".to_string(),
    });

    // All rewards should be pending
    match (basic_staking::QueryMsg::Rewards {
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: staking_user.clone().into(),
        },
    })
    .test_query(&basic_staking, &app)
    .unwrap()
    {
        basic_staking::QueryAnswer::Rewards { amount } => {
            assert_eq!(amount, reward_amount, "Rewards claimable at end");
        }
        _ => {
            panic!("Staking balance query failed");
        }
    };
}

macro_rules! single_stake_with_rewards {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    stake_amount,
                    reward_amount,
                    reward_start,
                    reward_end,
                ) = $value;
                single_stake_with_rewards(
                    stake_amount,
                    reward_amount,
                    reward_start,
                    reward_end,
                )
            }
        )*
    }
}

single_stake_with_rewards! {
    single_stake_with_rewards_0: (
        Uint128::new(1), //   stake_amount
        Uint128::new(100), // reward_amount
        Uint128::new(0), //   reward_start (0-*)
        Uint128::new(100), // reward_end
    ),
    single_stake_with_rewards_1: (
        Uint128::new(100),
        Uint128::new(1000),
        Uint128::new(0),
        Uint128::new(100),
    ),
    single_stake_with_rewards_2: (
        Uint128::new(1000),
        Uint128::new(300),
        Uint128::new(0),
        Uint128::new(100),
    ),
    single_stake_with_rewards_3: (
        Uint128::new(10),
        Uint128::new(50000),
        Uint128::new(0),
        Uint128::new(2500000),
    ),
    /*
    // fails bc 1 unit is un rewarded (499 < 500)
    single_stake_with_rewards_broken_0: (
        Uint128::new(1234567),
        Uint128::new(500),
        Uint128::new(0),
        Uint128::new(10000),
    ),
    // fails bc timeframe is so small
    single_stake_with_rewards_broken_1: (
        Uint128::new(99999999999),
        Uint128::new(8192),
        Uint128::new(20),
        Uint128::new(80),
    ),
    */
}
