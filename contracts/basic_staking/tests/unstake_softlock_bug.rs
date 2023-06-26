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
fn unstake_softlock_bug(stake_amount: Uint128, unbond_period: Uint128, reward_amount: Uint128) {
    let mut app = App::default();

    let mut now = 0;
    // init block time for predictable behavior
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(now),
        chain_id: "chain_id".to_string(),
    });

    let viewing_key = "unguessable".to_string();
    let admin_user = Addr::unchecked("admin");
    let staking_user = Addr::unchecked("staker");
    let reward_user = Addr::unchecked("reward_user");

    let extra_amount = Uint128::new(100000000000);

    let token = snip20::InstantiateMsg {
        name: "stake_token".into(),
        admin: Some(admin_user.to_string().clone()),
        symbol: "STKN".into(),
        decimals: 6,
        initial_balances: Some(vec![
            snip20::InitialBalance {
                amount: stake_amount + extra_amount,
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

    // Init Staking
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

    now = 10;
    app.set_block(BlockInfo {
        height: 10,
        time: Timestamp::from_seconds(now),
        chain_id: "chain_id".to_string(),
    });

    let reward_end = now + 100000000;

    // Init Rewards
    snip20::ExecuteMsg::Send {
        recipient: basic_staking.address.to_string().clone(),
        recipient_code_hash: None,
        amount: reward_amount,
        msg: Some(
            to_binary(&basic_staking::Action::Rewards {
                start: Uint128::new(now as u128),
                end: Uint128::new(reward_end as u128),
            })
            .unwrap(),
        ),
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, reward_user.clone(), &[])
    .unwrap();

    // Check reward pool
    match (basic_staking::QueryMsg::RewardPools {})
        .test_query(&basic_staking, &app)
        .unwrap()
    {
        basic_staking::QueryAnswer::RewardPools { rewards } => {
            assert_eq!(rewards[0].amount, reward_amount, "Reward Pool Amount");
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

    // Generate some rewards
    now += reward_end / 4;
    app.set_block(BlockInfo {
        height: 10,
        time: Timestamp::from_seconds(now),
        chain_id: "chain_id".to_string(),
    });

    // Post-staking user balance
    match (basic_staking::QueryMsg::Balance {
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: staking_user.clone().into(),
        },
        unbonding_ids: None,
    })
    .test_query(&basic_staking, &app)
    .unwrap()
    {
        basic_staking::QueryAnswer::Balance {
            staked,
            rewards,
            unbondings,
        } => {
            assert_eq!(staked, stake_amount, "Post-Stake Balance");
            /*
            panic!(
                "REWARDS {}",
                rewards.iter().map(|r| r.amount).sum::<Uint128>()
            );
            */
        }
        _ => {
            panic!("Staking balance query failed");
        }
    };

    // Unbond all funds
    basic_staking::ExecuteMsg::Unbond {
        amount: stake_amount,
        compound: None,
        padding: None,
    }
    .test_exec(&basic_staking, &mut app, staking_user.clone(), &[])
    .unwrap();

    // Check balance reflects unbonding
    match (basic_staking::QueryMsg::Balance {
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: staking_user.clone().into(),
        },
        unbonding_ids: None,
    })
    .test_query(&basic_staking, &app)
    .unwrap()
    {
        basic_staking::QueryAnswer::Balance {
            staked,
            rewards,
            unbondings,
        } => {
            assert_eq!(staked, Uint128::zero(), "Post-Unbond stake amount p");
            assert_eq!(unbondings.len(), 1, "Post unbond unbondings");
            assert_eq!(
                unbondings[0].amount, stake_amount,
                "Post unbond amount unbonding"
            );
        }
        _ => {
            panic!("Staking balance query failed");
        }
    };

    println!("Fast-forward to end of unbonding period");

    now += unbond_period.u128() as u64;
    app.set_block(BlockInfo {
        height: 10,
        time: Timestamp::from_seconds(now),
        chain_id: "chain_id".to_string(),
    });

    // Restake
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

    println!("Checking balance after restake...");
    match (basic_staking::QueryMsg::Balance {
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: staking_user.clone().into(),
        },
        unbonding_ids: None,
    })
    .test_query(&basic_staking, &app)
    .unwrap()
    {
        basic_staking::QueryAnswer::Balance {
            staked,
            rewards,
            unbondings,
        } => {
            assert_eq!(staked, stake_amount, "Re-Staked Balance");
            // assert_eq!(rewards, vec![], "Re-staked Rewards");
            // assert_eq!(unbondings, vec![], "Final Unbondings");
        }
        _ => {
            panic!("Staking balance query failed");
        }
    };
}

macro_rules! unstake_softlock_bug {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    stake_amount,
                    unbond_period,
                    reward_amount,
                ) = $value;
                unstake_softlock_bug(
                    stake_amount,
                    unbond_period,
                    reward_amount,
                )
            }
        )*
    }
}

unstake_softlock_bug! {
    unstake_softlock_bug_0: (
        Uint128::new(100), //   stake_amount
        Uint128::new(100), // unbond_period
        Uint128::new(100000000000), // reward_amount
    ),
}
