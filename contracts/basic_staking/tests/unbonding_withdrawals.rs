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
fn unbonding_withdrawals(
    stake_amount: Uint128,
    unbond_period: Uint128,
    unbonding_amounts: Vec<Uint128>,
    withdraw_order: Vec<usize>,
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
        initial_balances: Some(vec![snip20::InitialBalance {
            amount: stake_amount,
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

    let mut now = 50;
    let mut unbonded = Uint128::zero();

    // Setup timestamp
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(now),
        chain_id: "chain_id".to_string(),
    });

    // Perform each unbonding
    for unbond_amount in unbonding_amounts.iter() {
        // Unbond
        basic_staking::ExecuteMsg::Unbond {
            amount: unbond_amount.clone(),
            compound: None,
            padding: None,
        }
        .test_exec(&basic_staking, &mut app, staking_user.clone(), &[])
        .unwrap();
    }

    // Check that unbonding list is as expected
    let unbonding_ids = match (basic_staking::QueryMsg::Balance {
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
                unbondings
                    .iter()
                    .map(|u| u.amount)
                    .collect::<Vec<Uint128>>(),
                unbonding_amounts,
                "unbondings order as expectedt"
            );
            unbondings.iter().map(|u| u.id).collect::<Vec<Uint128>>()
        }
        _ => {
            panic!("Balance query failed");
            vec![]
        }
    };

    println!("Fast-forward to end of unbonding period");
    app.set_block(BlockInfo {
        height: 10,
        time: Timestamp::from_seconds(now + unbond_period.u128() as u64),
        chain_id: "chain_id".to_string(),
    });

    let mut withdrawn_ids = vec![];

    for i in withdraw_order.into_iter() {
        // Withdraw
        basic_staking::ExecuteMsg::Withdraw {
            ids: Some(vec![unbonding_ids[i]]),
            padding: None,
        }
        .test_exec(&basic_staking, &mut app, staking_user.clone(), &[])
        .unwrap();

        withdrawn_ids.push(unbonding_ids[i]);

        let expected_unbonding_ids = unbonding_ids
            .clone()
            .into_iter()
            .filter(|id| !withdrawn_ids.contains(id))
            .collect::<Vec<Uint128>>();

        // Check that unbonding list is as expected
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
                    unbondings.iter().map(|u| u.id).collect::<Vec<Uint128>>(),
                    expected_unbonding_ids,
                    "unbondings order as expected after withdrawing",
                );
            }
            _ => {
                panic!("Balance query failed");
            }
        };
    }

    // Check snip20 received by user
    match (snip20::QueryMsg::Balance {
        key: viewing_key.clone(),
        address: staking_user.clone().into(),
    })
    .test_query(&token, &app)
    .unwrap()
    {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, stake_amount, "Final user balance",);
        }
        _ => {
            panic!("Snip20 balance query failed");
        }
    };
}

macro_rules! unbonding_withdrawals {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    stake_amount,
                    unbond_period,
                    unbonding_amounts,
                    withdraw_order,
                ) = $value;
                unbonding_withdrawals(
                    stake_amount,
                    unbond_period,
                    unbonding_amounts,
                    withdraw_order,
                )
            }
        )*
    }
}

unbonding_withdrawals! {
    unbonding_withdrawals_0: (
        Uint128::new(100), //   stake_amount
        Uint128::new(100), // unbond_period
        vec![
            Uint128::new(10),
            Uint128::new(50),
            Uint128::new(17),
            Uint128::new(23),
        ],
        vec![0, 2, 1, 3],
    ),
    unbonding_withdrawals_1: (
        Uint128::new(1000), //   stake_amount
        Uint128::new(100), // unbond_period
        vec![
            Uint128::new(10),
            Uint128::new(50),
            Uint128::new(17),
            Uint128::new(23),
            Uint128::new(100),
            Uint128::new(700),
            Uint128::new(50),
            Uint128::new(50),
        ],
        vec![7, 5, 6, 2, 4, 3, 1, 0],
    ),
}
