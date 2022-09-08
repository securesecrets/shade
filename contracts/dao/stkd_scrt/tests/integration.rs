/*use shade_multi_test::multi::admin::init_admin_auth;
use shade_protocol::c_std::{to_binary, Addr, Coin, Decimal, Delegation, Uint128, Validator};

use shade_protocol::{
    contract_interfaces::{
        dao::{adapter, stkd_scrt},
        snip20,
    },
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

use shade_multi_test::multi::{snip20::Snip20, stkd_scrt::StkdScrt};
use shade_protocol::multi_test::{App, StakingSudo, SudoMsg};

fn bonded_adapter_test(deposit: Uint128, rewards: Uint128, reserves: Uint128, balance: Uint128) {
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

    let stkd_scrt = stkd_scrt::InstantiateMsg {
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
        "stkd_scrt",
        &[],
    )
    .unwrap();

    app.sudo(SudoMsg::Staking(StakingSudo::AddValidator {
        validator: validator.to_string().clone(),
    }))
    .unwrap();

    //TODO deploy staking_derivatives

    // set admin owner key
    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

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

    // Send funds to adapter
    snip20::ExecuteMsg::Send {
        recipient: stkd_scrt.address.to_string().clone(),
        recipient_code_hash: None,
        amount: deposit,
        msg: None,
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

    // reserves
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    })
    .test_query(&stkd_scrt, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, reserves, "Reserves Pre-Rewards");
        }
        _ => panic!("Query failed"),
    };

    // Balance
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&stkd_scrt, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, balance, "Balance Pre-Rewards");
        }
        _ => panic!("Query failed"),
    };

    // Rewards
    let cur_rewards: Uint128 = stkd_scrt::QueryMsg::Rewards {}
        .test_query(&stkd_scrt, &app)
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

    // Reserves
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    })
    .test_query(&stkd_scrt, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, reserves, "Reserves Post-Rewards");
        }
        _ => panic!("Query failed"),
    };

    // Balance
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&stkd_scrt, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit + rewards, "Balance Post-Rewards");
        }
        _ => panic!("Query failed"),
    };

    // Claimable
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
    })
    .test_query(&stkd_scrt, &app)
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
    .test_query(&stkd_scrt, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, balance, "Unbondable Pre-Unbond");
        }
        _ => panic!("Query failed"),
    };

    // Unbond all
    adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Unbond {
        amount: balance,
        asset: token.address.to_string().clone().to_string(),
    })
    .test_exec(&stkd_scrt, &mut app, admin.clone(), &[])
    .unwrap();
    println!("SCRT STAKING ADDR {}", stkd_scrt.address);

    // Unbonding
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbonding {
        asset: token.address.to_string().clone(),
    })
    .test_query(&stkd_scrt, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, deposit, "Unbonding Pre fast forward");
        }
        _ => panic!("Query failed"),
    };

    // Claimable
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
    })
    .test_query(&stkd_scrt, &app)
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
    .test_query(&stkd_scrt, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, deposit, "Claimable post fast forward");
        }
        _ => panic!("Query failed"),
    };

    // Claim
    adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Claim {
        asset: token.address.to_string().clone().to_string(),
    })
    .test_exec(&stkd_scrt, &mut app, admin.clone(), &[])
    .unwrap();

    // Reserves
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    })
    .test_query(&stkd_scrt, &app)
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
    .test_query(&stkd_scrt, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "Balance Post Claim");
        }
        _ => panic!("Query failed"),
    };

    // Unbonding
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbonding {
        asset: token.address.to_string().clone(),
    })
    .test_query(&stkd_scrt, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, Uint128::zero(), "Unbonding Post Claim");
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
            assert_eq!(amount, deposit + rewards, "Final User balance");
        }
        _ => {
            panic!("snip20 balance query failed");
        }
    };
}

macro_rules! basic_stkd_scrt_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    deposit,
                    rewards,
                    expected_stkd_scrt,
                ) = $value;
                basic_stkd_scrt_integration(deposit, rewards, expected_stkd_scrt);
            }
        )*
    }
}

basic_stkd_scrt_tests! {
    basic_stkd_scrt_0: (
        Uint128::new(100), // deposit
        Uint128::new(10),   // rewards
        Uint128::new(100), // reserves
        Uint128::new(100), // balance
    ),
}*/
