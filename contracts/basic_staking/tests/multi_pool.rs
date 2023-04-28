/*
use shade_protocol::c_std::{to_binary, Addr, BlockInfo, Timestamp, Uint128};

use shade_protocol::{
    contract_interfaces::{basic_staking, query_auth, snip20},
    multi_test::App,
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

use shade_multi_test::multi::{
    admin::{init_admin_auth, Admin},
    basic_staking::BasicStaking,
    query_auth::QueryAuth,
    snip20::Snip20,
};

// Add other adapters here as they come
fn multi_staker_multi_pool(
    stake_amounts: Vec<Uint128>,
    unbond_period: Uint128,
    rewards: Vec<(Uint128, Uint128, Uint128)>,
    expected_rewards: Vec<Uint128>,
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
    let reward_user = Addr::unchecked("reward_user");
    let staking_users = stake_amounts
        .iter()
        .enumerate()
        .map(|(i, x)| Addr::unchecked(format!("staker-{}", i)))
        .collect::<Vec<Addr>>();

    let mut initial_balances = staking_users
        .iter()
        .zip(stake_amounts.clone().into_iter())
        .map(|(user, amount)| snip20::InitialBalance {
            amount,
            address: user.to_string(),
        })
        .collect::<Vec<snip20::InitialBalance>>();

    initial_balances.push(snip20::InitialBalance {
        amount: reward_amount,
        address: reward_user.to_string(),
    });

    let rewards_by_start = rewards.iter().sort_by(|a, b| a.1.cmp(b.1)).collect();
    let rewards_by_end = rewards.iter().sort_by(|a, b| a.2.cmp(b.2)).collect();

    let mut rewards_by_time = vec![];
    for i in rewards.len() * 2 {
        rewards_by_time.push((rewards_by_start.first().1 <= rewards_by_end.first().2 ? rewards_by_start : rewards_by_end).pop_front());
    }

    let token = snip20::InstantiateMsg {
        name: "stake_token".into(),
        admin: Some(admin_user.to_string().clone()),
        symbol: "STKN".into(),
        decimals: 6,
        initial_balances: Some(initial_balances),
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

    // set staking_users viewing key
    for user in staking_users.clone().into_iter() {
        snip20::ExecuteMsg::SetViewingKey {
            key: viewing_key.clone(),
            padding: None,
        }
        .test_exec(&token, &mut app, user.clone(), &[])
        .unwrap();

        // set staking user VK
        query_auth::ExecuteMsg::SetViewingKey {
            key: viewing_key.clone(),
            padding: None,
        }
        .test_exec(&query_contract, &mut app, user.clone(), &[])
        .unwrap();
    }

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

    for (user, stake_amount) in staking_users.iter().zip(stake_amounts.clone().into_iter()) {
        // Pre-staking user balance
        match (basic_staking::QueryMsg::Balance {
            auth: basic_staking::Auth::ViewingKey {
                key: viewing_key.clone(),
                address: user.clone().into(),
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
        .test_exec(&token, &mut app, user.clone(), &[])
        .unwrap();

        // Post-staking user balance
        match (basic_staking::QueryMsg::Balance {
            auth: basic_staking::Auth::ViewingKey {
                key: viewing_key.clone(),
                address: user.clone().into(),
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
    }

    // Init Rewards
    for (reward_amount, reward_start, reward_end) in rewards.iter() {
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
        match (basic_staking::QueryMsg::Balance {
            auth: basic_staking::Auth::ViewingKey {
                key: viewing_key.clone(),
                address: reward_user.clone().into(),
            },
        })
        .test_query(&basic_staking, &app)
        .unwrap()
        {
            basic_staking::QueryAnswer::Balance { amount } => {
                assert_eq!(amount, Uint128::zero(), "Reward User Stake Balance");
            }
            _ => {
                panic!("Staking balance query failed");
            }
        };

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
    }

    // Move forward to reward start
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(reward_start.u128() as u64),
        chain_id: "chain_id".to_string(),
    });

    for user in staking_users.iter() {
        // No rewards should be pending
        match (basic_staking::QueryMsg::Rewards {
            auth: basic_staking::Auth::ViewingKey {
                key: viewing_key.clone(),
                address: user.clone().into(),
            },
        })
        .test_query(&basic_staking, &app)
        .unwrap()
        {
            basic_staking::QueryAnswer::Rewards { amount } => {
                assert_eq!(amount, Uint128::zero(), "Rewards claimable at beginning");
            }
            _ => {
                panic!("Staking rewards query failed");
            }
        };
    }

    let reward_duration = reward_end - reward_start;

    // Move to middle of reward period
    println!("Fast-forward to reward middle");
    app.set_block(BlockInfo {
        height: 2,
        time: Timestamp::from_seconds((reward_start.u128() + reward_duration.u128() / 2) as u64),
        chain_id: "chain_id".to_string(),
    });

    for (user, reward) in staking_users
        .iter()
        .zip(expected_rewards.clone().into_iter())
    {
        // Half-ish rewards should be pending
        match (basic_staking::QueryMsg::Rewards {
            auth: basic_staking::Auth::ViewingKey {
                key: viewing_key.clone(),
                address: user.clone().into(),
            },
        })
        .test_query(&basic_staking, &app)
        .unwrap()
        {
            basic_staking::QueryAnswer::Rewards { amount } => {
                let expected = reward / Uint128::new(2);
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
    }

    // Move to end of rewards
    println!("Fast-forward to reward end");
    app.set_block(BlockInfo {
        height: 3,
        time: Timestamp::from_seconds(reward_end.u128() as u64),
        chain_id: "chain_id".to_string(),
    });

    for ((user, amount), reward) in staking_users
        .iter()
        .zip(stake_amounts.clone().into_iter())
        .zip(expected_rewards.clone().into_iter())
    {
        // All rewards should be pending
        match (basic_staking::QueryMsg::Rewards {
            auth: basic_staking::Auth::ViewingKey {
                key: viewing_key.clone(),
                address: user.clone().into(),
            },
        })
        .test_query(&basic_staking, &app)
        .unwrap()
        {
            basic_staking::QueryAnswer::Rewards { amount } => {
                assert!(
                    amount >= reward - Uint128::one() && amount <= reward,
                    "Rewards claimable at the end within error of 1 unit token {} != {}",
                    amount,
                    reward,
                );
            }
            _ => {
                panic!("Staking rewards query failed");
            }
        };

        // Claim rewards
        basic_staking::ExecuteMsg::Claim {}
            .test_exec(&basic_staking, &mut app, user.clone(), &[])
            .unwrap();

        // Check rewards were claimed
        match (snip20::QueryMsg::Balance {
            key: viewing_key.clone(),
            address: user.clone().into(),
        })
        .test_query(&token, &app)
        .unwrap()
        {
            snip20::QueryAnswer::Balance { amount } => {
                assert!(
                    amount >= reward - Uint128::one() && amount <= reward,
                    "Rewards claimed at the end within error of 1 unit token {} != {}",
                    amount,
                    reward,
                );
            }
            _ => {
                panic!("Snip20 balance query failed");
            }
        };

        // Unbond
        basic_staking::ExecuteMsg::Unbond { amount }
            .test_exec(&basic_staking, &mut app, user.clone(), &[])
            .unwrap();

        // All funds should be unbonding
        match (basic_staking::QueryMsg::Unbonding {
            auth: basic_staking::Auth::ViewingKey {
                key: viewing_key.clone(),
                address: user.clone().into(),
            },
        })
        .test_query(&basic_staking, &app)
        .unwrap()
        {
            basic_staking::QueryAnswer::Unbonding { unbondings } => {
                assert_eq!(unbondings.len(), 1, "1 unbonding");
                assert_eq!(unbondings[0].amount, amount, "Unbonding full amount");
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
    }

    println!("Fast-forward to end of unbonding period");
    app.set_block(BlockInfo {
        height: 10,
        time: Timestamp::from_seconds((reward_end + unbond_period).u128() as u64),
        chain_id: "chain_id".to_string(),
    });

    for ((user, stake_amount), reward) in staking_users
        .iter()
        .zip(stake_amounts.clone().into_iter())
        .zip(expected_rewards.clone().into_iter())
    {
        // Withdraw unbonding
        basic_staking::ExecuteMsg::Withdraw {}
            .test_exec(&basic_staking, &mut app, user.clone(), &[])
            .unwrap();

        // Check unbonding withdrawn
        match (snip20::QueryMsg::Balance {
            key: viewing_key.clone(),
            address: user.clone().into(),
        })
        .test_query(&token, &app)
        .unwrap()
        {
            snip20::QueryAnswer::Balance { amount } => {
                let expected = stake_amount + reward;
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
}

macro_rules! multi_staker_multi_pool {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    stake_amounts,
                    unbond_period,
                    rewards,
                    expected_rewards,
                ) = $value;
                multi_staker_multi_pool(
                    stake_amounts,
                    unbond_period,
                    rewards,
                    expected_rewards,
                )
            }
        )*
    }
}

multi_staker_multi_pool! {
    multi_staker_multi_pool_0: (
        vec![           //   stake_amount
            Uint128::new(1),
            Uint128::new(10),
        ],
        Uint128::new(100), // unbond_period
        vec![
            (
                Uint128::new(100), // reward_amount
                Uint128::new(0), //   reward_start (0-*)
                Uint128::new(100), // reward_end
            ),
        ],
        vec![               // expected rewards
            Uint128::new(10),
            Uint128::new(90),
        ],
    ),
}
*/
