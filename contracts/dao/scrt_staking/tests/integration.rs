use shade_multi_test::multi::admin::init_admin_auth;
use shade_protocol::c_std::{to_binary, Addr, Coin, Delegation, Uint128};

use shade_protocol::{
    contract_interfaces::{
        dao::{adapter, scrt_staking},
        snip20,
    },
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

use shade_multi_test::multi::{scrt_staking::ScrtStaking, snip20::Snip20};
use shade_protocol::multi_test::{App, StakingSudo, SudoMsg};

// Add other adapters here as they come
fn basic_scrt_staking_integration(
    deposit: Uint128,
    rewards: Uint128,
    expected_scrt_staking: Uint128,
) {
    let mut app = App::default();

    let viewing_key = "unguessable".to_string();
    let admin = Addr::unchecked("admin");
    let validator = Addr::unchecked("validator");
    let admin_auth = init_admin_auth(&mut app, &admin);
    let token = snip20::InstantiateMsg {
        name: "secretSCRT".into(),
        admin: Some("admin".into()),
        symbol: "SSCRT".into(),
        decimals: 6,
        initial_balances: None,
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
    .test_init(Snip20::default(), &mut app, admin.clone(), "token", &[])
    .unwrap();

    let scrt_staking = scrt_staking::InstantiateMsg {
        admin_auth: admin_auth.into(),
        owner: admin.clone().into(),
        sscrt: token.clone().into(),
        validator_bounds: None,
        viewing_key: viewing_key.clone(),
    }
    .test_init(
        ScrtStaking::default(),
        &mut app,
        admin.clone(),
        "scrt_staking",
        &[],
    )
    .unwrap();
    println!("SCRT STAKING ADDR {}", scrt_staking.address);

    app.sudo(SudoMsg::Staking(StakingSudo::AddValidator {
        validator: validator.to_string().clone(),
    }))
    .unwrap();

    // set admin owner key
    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

    if !deposit.is_zero() {
        let deposit_coin = Coin {
            denom: "uscrt".into(),
            amount: deposit,
        };
        app.init_modules(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &admin.clone(), vec![deposit_coin.clone()])
                .unwrap();
        });

        // Wrap L1 into tokens
        snip20::ExecuteMsg::Deposit { padding: None }
            .test_exec(&token, &mut app, admin.clone(), &vec![deposit_coin])
            .unwrap();

        // Deposit funds in scrt staking
        snip20::ExecuteMsg::Send {
            recipient: scrt_staking.address.to_string().clone(),
            recipient_code_hash: None,
            amount: deposit,
            msg: None,
            memo: None,
            padding: None,
        }
        .test_exec(&token, &mut app, admin.clone(), &[])
        .unwrap();

        // Delegations
        let delegations: Vec<Delegation> = scrt_staking::QueryMsg::Delegations {}
            .test_query(&scrt_staking, &app)
            .unwrap();
        assert!(
            !delegations.is_empty(),
            "empty delegations! {}",
            delegations.len()
        );
    }

    // reserves should be 0 (all staked)
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "Reserves Pre-Rewards");
        }
        _ => panic!("Query failed"),
    };

    // Balance
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Balance Pre-Rewards");
        }
        _ => panic!("Query failed"),
    };

    // Rewards
    let cur_rewards: Uint128 = scrt_staking::QueryMsg::Rewards {}
        .test_query(&scrt_staking, &app)
        .unwrap();
    assert_eq!(cur_rewards, Uint128::zero(), "Rewards Pre-add");

    //ensemble.add_rewards(rewards);
    app.sudo(SudoMsg::Staking(StakingSudo::AddRewards {
        amount: Coin {
            amount: rewards,
            denom: "uscrt".into(),
        },
    }))
    .unwrap();
    /*
    let block = app.block_info();
    app.init_modules(|router, api, storage| {
        router.staking.add_rewards(
            api,
            storage,
            router,
            &block,
            Coin { amount: rewards, denom: "uscrt".into() },
        ).unwrap();
    });
    */

    // Rewards
    let cur_rewards: Uint128 = scrt_staking::QueryMsg::Rewards {}
        .test_query(&scrt_staking, &app)
        .unwrap();

    if deposit.is_zero() {
        assert_eq!(cur_rewards, Uint128::zero(), "Rewards Post-add");
    } else {
        assert_eq!(cur_rewards, rewards, "Rewards Post-add");
    }

    // reserves should be rewards
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Reserves { amount } => {
            if deposit.is_zero() {
                assert_eq!(amount, Uint128::zero(), "Reserves Post-Rewards");
            } else {
                assert_eq!(amount, rewards, "Reserves Post-Rewards");
            }
        }
        _ => panic!("Query failed"),
    };

    // Balance
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Balance { amount } => {
            if deposit.is_zero() {
                assert_eq!(amount, Uint128::zero(), "Balance Post-Rewards");
            } else {
                assert_eq!(amount, deposit + rewards, "Balance Post-Rewards");
            }
        }
        _ => panic!("Query failed"),
    };

    // Update SCRT Staking
    adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Update {
        asset: token.address.to_string().clone().to_string(),
    })
    .test_exec(&scrt_staking, &mut app, admin.clone(), &[])
    .unwrap();

    // reserves/rewards should be staked
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "Reserves Post-Update");
        }
        _ => panic!("Query failed"),
    };

    // Balance
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, expected_scrt_staking, "Balance Post-Update");
        }
        _ => panic!("Query failed"),
    };

    // Claimable
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Claimable Pre-Unbond");
        }
        _ => panic!("Query failed"),
    };

    // Unbondable
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbondable {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, expected_scrt_staking, "Unbondable Pre-Unbond");
        }
        _ => panic!("Query failed"),
    };

    println!("SCRT STAKING ADDR {}", scrt_staking.address);
    // Unbond all
    adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Unbond {
        amount: expected_scrt_staking,
        asset: token.address.to_string().clone().to_string(),
    })
    .test_exec(&scrt_staking, &mut app, admin.clone(), &[])
    .unwrap();
    println!("SCRT STAKING ADDR {}", scrt_staking.address);

    // Unbonding
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbonding {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, expected_scrt_staking, "Unbonding Pre fast forward");
        }
        _ => panic!("Query failed"),
    };

    // Claimable
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Claimable Pre unbond fast forward");
        }
        _ => panic!("Query failed"),
    };

    app.sudo(SudoMsg::Staking(StakingSudo::FastForwardUndelegate {}))
        .unwrap();

    // Claimable
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Claimable { amount } => {
            if deposit.is_zero() {
                assert_eq!(amount, Uint128::zero(), "Claimable post fast forward");
            } else {
                assert_eq!(amount, deposit + rewards, "Claimable post fast forward");
            }
        }
        _ => panic!("Query failed"),
    };

    // Claim
    adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Claim {
        asset: token.address.to_string().clone().to_string(),
    })
    .test_exec(&scrt_staking, &mut app, admin.clone(), &[])
    .unwrap();

    // Reserves
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "Reserves Post Claim");
        }
        _ => panic!("Query failed"),
    };

    // Balance
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "Balance Post Claim");
        }
        _ => panic!("Query failed"),
    };

    // ensure wrapped tokens were returned
    match (snip20::QueryMsg::Balance {
        address: admin.to_string().clone(),
        key: viewing_key.clone(),
    })
    .test_query(&token, &app)
    .unwrap()
    {
        snip20::QueryAnswer::Balance { amount } => {
            if deposit.is_zero() {
                assert_eq!(amount.u128(), 0u128, "Final User balance");
            } else {
                assert_eq!(
                    amount.u128(),
                    deposit.u128() + rewards.u128(),
                    "Final user balance"
                );
            }
        }
        _ => {
            panic!("snip20 balance query failed");
        }
    };
}

macro_rules! basic_scrt_staking_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    deposit,
                    rewards,
                    expected_scrt_staking,
                ) = $value;
                basic_scrt_staking_integration(deposit, rewards, expected_scrt_staking);
            }
        )*
    }
}

basic_scrt_staking_tests! {
    basic_scrt_staking_0: (
        Uint128::new(100), // deposit
        Uint128::new(0),   // rewards
        Uint128::new(100), // balance
    ),
    basic_scrt_staking_1: (
        Uint128::new(100), // deposit
        Uint128::new(50),   // rewards
        Uint128::new(150), // balance
    ),
    basic_scrt_staking_no_deposit: (
        Uint128::new(0), // deposit
        Uint128::new(1000),   // rewards
        Uint128::new(0), // balance
    ),
}
