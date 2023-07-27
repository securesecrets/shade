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
fn transfer_stake(stake_amount: Uint128, transfer_amount: Uint128) {
    let mut app = App::default();

    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(0),
        chain_id: "chain_id".to_string(),
    });

    // init block time for predictable behavior
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(0),
        chain_id: "chain_id".to_string(),
    });

    let viewing_key = "unguessable".to_string();
    let admin_user = Addr::unchecked("admin");
    let staking_user = Addr::unchecked("staker");
    let transfer_user = Addr::unchecked("transfer");

    let reward_user = Addr::unchecked("reward_user");
    let reward_amount = Uint128::new(100000000);
    let reward_start = Uint128::new(100);
    let reward_end = Uint128::new(1000);

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
                amount: transfer_amount,
                address: transfer_user.to_string(),
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

    // set transfer_user viewing key
    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, transfer_user.clone(), &[])
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
    // set transfer user VK
    query_auth::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&query_contract, &mut app, transfer_user.clone(), &[])
    .unwrap();

    let basic_staking = basic_staking::InstantiateMsg {
        admin_auth: admin_contract.into(),
        query_auth: query_contract.into(),
        stake_token: token.clone().into(),
        airdrop: None,
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

    // Stake staking user
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

    // Stake transfer user
    snip20::ExecuteMsg::Send {
        recipient: basic_staking.address.to_string().clone(),
        recipient_code_hash: None,
        amount: transfer_amount,
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
    .test_exec(&token, &mut app, transfer_user.clone(), &[])
    .unwrap();

    // Skip to end of rewards
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(reward_end.u128() as u64),
        chain_id: "chain_id".to_string(),
    });

    // Transfer should fail, not on whitelist
    match (basic_staking::ExecuteMsg::TransferStake {
        amount: transfer_amount,
        recipient: staking_user.clone().into(),
        compound: Some(false),
        padding: None,
    }
    .test_exec(&basic_staking, &mut app, transfer_user.clone(), &[]))
    {
        Err(_) => {}
        Ok(_) => {
            panic!("Shouldn't allow transfer if source not on whitelist");
        }
    }

    // Add transfer user to whitelist
    basic_staking::ExecuteMsg::AddTransferWhitelist {
        user: transfer_user.clone().into(),
        padding: None,
    }
    .test_exec(&basic_staking, &mut app, admin_user.clone(), &[])
    .unwrap();

    // Check whitelist
    match (basic_staking::QueryMsg::TransferWhitelist {})
        .test_query(&basic_staking, &app)
        .unwrap()
    {
        basic_staking::QueryAnswer::TransferWhitelist { whitelist } => {
            assert_eq!(
                whitelist,
                vec![transfer_user.clone()],
                "Whitelist contains transfer user"
            );
        }
        _ => {
            panic!("Unexpected transfer whitelist response");
        }
    }

    // check balances to verify rewards claim
    let stake_user_reward = match (basic_staking::QueryMsg::Balance {
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
            assert_eq!(staked, stake_amount, "Pre-Transfer recipient balance");
            rewards[0].amount
        }
        _ => {
            panic!("Staking balance query failed");
        }
    };

    let transfer_user_reward = match (basic_staking::QueryMsg::Balance {
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: transfer_user.clone().into(),
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
            assert_eq!(
                staked, transfer_amount,
                "Pre-Transfer transfer user balance"
            );
            rewards[0].amount
        }
        _ => {
            panic!("Staking balance query failed");
        }
    };

    basic_staking::ExecuteMsg::TransferStake {
        amount: transfer_amount,
        recipient: staking_user.clone().into(),
        compound: Some(false),
        padding: None,
    }
    .test_exec(&basic_staking, &mut app, transfer_user.clone(), &[])
    .unwrap();

    // check staking user balance after transfer
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
            assert_eq!(
                staked,
                stake_amount + transfer_amount,
                "Post-Transfer recipient balance"
            );
            assert_eq!(rewards.len(), 0, "Recipient rewards claimed");
        }
        _ => {
            panic!("Staking balance query failed");
        }
    };
    // Check recipient rewards claimed
    match (snip20::QueryMsg::Balance {
        key: viewing_key.clone(),
        address: staking_user.clone().into(),
    })
    .test_query(&token, &app)
    .unwrap()
    {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, stake_user_reward, "Recipient rewards claimed",);
        }
        _ => {
            panic!("Snip20 balance query failed");
        }
    };

    // check transfer user balance after transfer
    match (basic_staking::QueryMsg::Balance {
        auth: basic_staking::Auth::ViewingKey {
            key: viewing_key.clone(),
            address: transfer_user.clone().into(),
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
            assert_eq!(staked, Uint128::zero(), "Post-Transfer sender balance");
            assert_eq!(rewards.len(), 0, "Sender rewards claimed");
        }
        _ => {
            panic!("Staking balance query failed");
        }
    };

    // Check sender rewards claimed
    match (snip20::QueryMsg::Balance {
        key: viewing_key.clone(),
        address: transfer_user.clone().into(),
    })
    .test_query(&token, &app)
    .unwrap()
    {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, transfer_user_reward, "Sender rewards claimed");
        }
        _ => {
            panic!("Snip20 balance query failed");
        }
    };

    // Remove from whitelist
    basic_staking::ExecuteMsg::RemoveTransferWhitelist {
        user: transfer_user.clone().into(),
        padding: None,
    }
    .test_exec(&basic_staking, &mut app, admin_user.clone(), &[])
    .unwrap();

    // Check whitelist
    match (basic_staking::QueryMsg::TransferWhitelist {})
        .test_query(&basic_staking, &app)
        .unwrap()
    {
        basic_staking::QueryAnswer::TransferWhitelist { whitelist } => {
            assert!(whitelist.is_empty(), "Whitelist empty after removal");
        }
        _ => {
            panic!("Unexpected transfer whitelist response");
        }
    }
}

macro_rules! transfer_stake {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    stake_amount,
                    transfer_amount,
                ) = $value;
                transfer_stake(
                    stake_amount,
                    transfer_amount,
                )
            }
        )*
    }
}

transfer_stake! {
    transfer_stake_0: (
        Uint128::new(1), //   stake_amount
        Uint128::new(1), //   transfer_amount
    ),
    transfer_stake_1: (
        Uint128::new(100),
        Uint128::new(100),
    ),
    transfer_stake_2: (
        Uint128::new(1000),
        Uint128::new(1000),
    ),
    transfer_stake_3: (
        Uint128::new(10),
        Uint128::new(10),
    ),
    transfer_stake_4: (
        Uint128::new(1234567),
        Uint128::new(1234567),
    ),
    transfer_stake_5: (
        Uint128::new(99999999999),
        Uint128::new(99999999999),
    ),
}
