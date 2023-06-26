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

// Add other adapters here as they come
fn stake_0_total_bug(
    stake_amount: Uint128,
    unbond_period: Uint128,
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
    let reward_user = Addr::unchecked("reward_user");

    let token = snip20::InstantiateMsg {
        name: "stake_token".into(),
        admin: Some(admin_user.to_string().clone()),
        symbol: "STKN".into(),
        decimals: 6,
        initial_balances: Some(vec![
            snip20::InitialBalance {
                amount: stake_amount,
                address: staking_user.to_string(),
            },
            snip20::InitialBalance {
                amount: reward_amount,
                address: reward_user.to_string(),
            },
        ]),
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

    // set reward user VK
    query_auth::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&query_contract, &mut app, reward_user.clone(), &[])
    .unwrap();

    let basic_staking = basic_staking::InstantiateMsg {
        admin_auth: admin_contract.into(),
        query_auth: query_contract.into(),
        airdrop: None,
        stake_token: token.clone().into(),
        unbond_period,
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
    println!("BASIC STAKING {}", basic_staking.address);

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
    .test_exec(&token, &mut app, reward_user.clone(), &[])
    .unwrap();

    // reward user has no stake
    match (basic_staking::QueryMsg::Staked {
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: reward_user.clone().into(),
        },
    })
    .test_query(&basic_staking, &app)
    .unwrap()
    {
        basic_staking::QueryAnswer::Staked { amount } => {
            assert_eq!(amount, Uint128::zero(), "Reward User Stake Balance");
        }
        _ => {
            panic!("Staking balance query failed");
        }
    };

    // Check reward pool
    match (basic_staking::QueryMsg::RewardPools {})
        .test_query(&basic_staking, &app)
        .unwrap()
    {
        basic_staking::QueryAnswer::RewardPools { rewards } => {
            assert_eq!(rewards[0].amount, reward_amount, "Reward Pool Amount");
            assert_eq!(rewards[0].start, reward_start, "Reward Pool Start");
            assert_eq!(rewards[0].end, reward_end, "Reward Pool End");
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
        msg: Some(
            to_binary(&basic_staking::Action::Stake {
                compound: None,
                airdrop_task: None,
            })
            .unwrap(),
        ),
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, staking_user.clone(), &[])
    .unwrap();

    // Post-staking user balance
    match (basic_staking::QueryMsg::Staked {
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: staking_user.clone().into(),
        },
    })
    .test_query(&basic_staking, &app)
    .unwrap()
    {
        basic_staking::QueryAnswer::Staked { amount } => {
            assert_eq!(amount, stake_amount, "Post-Stake Balance");
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

    // No rewards should be pending
    match (basic_staking::QueryMsg::Rewards {
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: staking_user.clone().into(),
        },
    })
    .test_query(&basic_staking, &app)
    .unwrap()
    {
        basic_staking::QueryAnswer::Rewards { rewards } => {
            assert_eq!(rewards.len(), 1, "Rewards length at beginning");
            assert_eq!(
                rewards[0].amount,
                Uint128::zero(),
                "Rewards claimable at beginning"
            );
        }
        _ => {
            panic!("Staking rewards query failed");
        }
    };

    let reward_duration = reward_end - reward_start;

    // Move to middle of reward period
    println!("Fast-forward to reward middle");
    app.set_block(BlockInfo {
        height: 2,
        time: Timestamp::from_seconds((reward_start.u128() + reward_duration.u128() / 2) as u64),
        chain_id: "chain_id".to_string(),
    });

    // Half-ish rewards should be pending
    match (basic_staking::QueryMsg::Rewards {
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: staking_user.clone().into(),
        },
    })
    .test_query(&basic_staking, &app)
    .unwrap()
    {
        basic_staking::QueryAnswer::Rewards { rewards } => {
            assert_eq!(rewards.len(), 1, "rewards length in the middle");
            let amount = rewards[0].amount;
            let expected = reward_amount / Uint128::new(2);
            assert!(
                amount >= expected - Uint128::one() && amount <= expected,
                "Rewards claimable in the middle within error of 1 unit token {} != {}",
                amount,
                expected
            );
        }
        _ => {
            panic!("Staking rewards query failed");
        }
    };

    // Move to end of rewards
    println!("Fast-forward to reward end");
    app.set_block(BlockInfo {
        height: 3,
        time: Timestamp::from_seconds(reward_end.u128() as u64),
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
        basic_staking::QueryAnswer::Rewards { rewards } => {
            assert_eq!(rewards.len(), 1, "rewards length at end");
            let amount = rewards[0].amount;
            assert!(
                amount >= reward_amount - Uint128::one() && amount <= reward_amount,
                "Rewards claimable at the end within error of 1 unit token {} != {}",
                amount,
                reward_amount,
            );
        }
        _ => {
            panic!("Staking rewards query failed");
        }
    };

    // Claim rewards
    basic_staking::ExecuteMsg::Claim { padding: None }
        .test_exec(&basic_staking, &mut app, staking_user.clone(), &[])
        .unwrap();

    // Check rewards were claimed
    match (snip20::QueryMsg::Balance {
        key: viewing_key.clone(),
        address: staking_user.clone().into(),
    })
    .test_query(&token, &app)
    .unwrap()
    {
        snip20::QueryAnswer::Balance { amount } => {
            assert!(
                amount >= reward_amount - Uint128::one() && amount <= reward_amount,
                "Rewards claimed at the end within error of 1 unit token {} != {}",
                amount,
                reward_amount,
            );
        }
        _ => {
            panic!("Snip20 balance query failed");
        }
    };

    // Unbond
    basic_staking::ExecuteMsg::Unbond {
        amount: stake_amount,
        compound: None,
        padding: None,
    }
    .test_exec(&basic_staking, &mut app, staking_user.clone(), &[])
    .unwrap();

    // All rewards should be pending
    match (basic_staking::QueryMsg::Unbonding {
        ids: None,
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: staking_user.clone().into(),
        },
    })
    .test_query(&basic_staking, &app)
    .unwrap()
    {
        basic_staking::QueryAnswer::Unbonding { unbondings } => {
            assert_eq!(unbondings.len(), 1, "1 unbonding");
            assert_eq!(unbondings[0].amount, stake_amount, "Unbonding full amount");
            assert_eq!(
                unbondings[0].complete,
                reward_end + unbond_period,
                "Unbonding complete expectedt"
            );
        }
        _ => {
            panic!("Staking unbonding query failed");
        }
    };
    println!("Fast-forward to end of unbonding period");
    app.set_block(BlockInfo {
        height: 10,
        time: Timestamp::from_seconds((reward_end + unbond_period).u128() as u64),
        chain_id: "chain_id".to_string(),
    });

    basic_staking::ExecuteMsg::Withdraw {
        ids: None,
        padding: None,
    }
    .test_exec(&basic_staking, &mut app, staking_user.clone(), &[])
    .unwrap();

    // Check unbonding withdrawn
    match (snip20::QueryMsg::Balance {
        key: viewing_key.clone(),
        address: staking_user.clone().into(),
    })
    .test_query(&token, &app)
    .unwrap()
    {
        snip20::QueryAnswer::Balance { amount } => {
            let expected = stake_amount + reward_amount;
            assert!(
                amount >= expected - Uint128::one() && amount <= expected,
                "Final user balance within error of 1 unit token {} != {}",
                amount,
                expected,
            );
        }
        _ => {
            panic!("Snip20 balance query failed");
        }
    };
}

macro_rules! stake_0_total_bug {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    stake_amount,
                    unbond_period,
                    reward_amount,
                    reward_start,
                    reward_end,
                ) = $value;
                stake_0_total_bug(
                    stake_amount,
                    unbond_period,
                    reward_amount,
                    reward_start,
                    reward_end,
                )
            }
        )*
    }
}

stake_0_total_bug! {
    stake_0_total_bug_0: (
        Uint128::new(1), //   stake_amount
        Uint128::new(100), // unbond_period
        Uint128::new(100), // reward_amount
        Uint128::new(0), //   reward_start (0-*)
        Uint128::new(100), // reward_end
    ),
}
