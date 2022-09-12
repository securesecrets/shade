use mock_adapter;
use shade_multi_test::{
    interfaces,
    multi::{
        admin::init_admin_auth,
        mock_adapter::MockAdapter,
        snip20::Snip20,
        treasury::Treasury,
        treasury_manager::TreasuryManager,
    },
};
use shade_protocol::{
    c_std::{
        coins,
        from_binary,
        to_binary,
        Addr,
        Binary,
        Coin,
        Decimal,
        Env,
        StdError,
        StdResult,
        Uint128,
        Validator,
    },
    multi_test::{App, BankSudo, StakingSudo, SudoMsg},
};
use shade_protocol::{
    contract_interfaces::{
        dao::{
            adapter,
            manager,
            //mock_adapter,
            treasury,
            treasury::{Allowance, AllowanceType, RunLevel},
            treasury_manager::{self, Allocation, AllocationType},
        },
        snip20,
    },
    utils::{
        asset::Contract,
        cycle::{utc_from_timestamp, Cycle},
        storage::plus::period_storage::Period,
        ExecuteCallback,
        InstantiateCallback,
        MultiTestable,
        Query,
    },
};

use serde_json;

// Add other adapters here as they come
fn bonded_adapter_int(
    deposit: Uint128,
    allowance: Uint128,
    expected_allowance: Uint128,
    alloc_type: AllocationType,
    alloc_amount: Uint128,
    rewards: Uint128,
    // expected balances
    pre_rewards: (Uint128, Uint128, Uint128),
    post_rewards: (Uint128, Uint128, Uint128),
) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
    let user = Addr::unchecked("user");
    //let validator = Addr::unchecked("validator");
    let admin_auth = init_admin_auth(&mut app, &admin);

    let viewing_key = "viewing_key".to_string();

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

    let treasury = treasury::InstantiateMsg {
        admin_auth: admin_auth.clone().into(),
        viewing_key: viewing_key.clone(),
        multisig: admin.to_string().clone(),
    }
    .test_init(Treasury::default(), &mut app, admin.clone(), "treasury", &[
    ])
    .unwrap();

    let manager = treasury_manager::InstantiateMsg {
        admin_auth: admin_auth.clone().into(),
        treasury: treasury.address.to_string(),
        viewing_key: viewing_key.clone(),
    }
    .test_init(
        TreasuryManager::default(),
        &mut app,
        admin.clone(),
        "manager",
        &[],
    )
    .unwrap();

    let adapter = mock_adapter::contract::Config {
        owner: manager.address.clone(),
        instant: false,
        token: token.clone().into(),
    }
    .test_init(
        MockAdapter::default(),
        &mut app,
        admin.clone(),
        "mock_adapter",
        &[],
    )
    .unwrap();

    /*
    app.sudo(SudoMsg::Staking(StakingSudo::AddValidator {
        validator: validator.to_string().clone(),
    }))
    .unwrap();
    */

    // Set admin viewing key
    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

    // Register treasury assets
    treasury::ExecuteMsg::RegisterAsset {
        contract: token.clone().into(),
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    // Register manager assets
    treasury_manager::ExecuteMsg::RegisterAsset {
        contract: token.clone().into(),
    }
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Register manager w/ treasury
    treasury::ExecuteMsg::RegisterManager {
        contract: manager.clone().into(),
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    // treasury allowance to manager
    treasury::ExecuteMsg::Allowance {
        asset: token.address.to_string().clone(),
        allowance: treasury::Allowance {
            //nick: "Mid-Stakes-Manager".to_string(),
            spender: manager.address.clone(),
            allowance_type: AllowanceType::Portion,
            cycle: Cycle::Constant,
            amount: allowance,
            // 100% (adapter balance will 2x before unbond)
            tolerance: Uint128::zero(),
        },
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    // Allocate to mock_adapter from manager
    treasury_manager::ExecuteMsg::Allocate {
        asset: token.address.to_string().clone(),
        allocation: Allocation {
            nick: Some("mock_adapter".to_string()),
            contract: Contract {
                address: adapter.address.clone(),
                code_hash: adapter.code_hash.clone(),
            },
            alloc_type,
            amount: alloc_amount,
            tolerance: Uint128::zero(),
        },
    }
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    let init_coin = Coin {
        denom: "uscrt".into(),
        amount: deposit + rewards,
    };
    app.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &admin.clone(), vec![init_coin.clone()])
            .unwrap();
    });

    // Wrap L1
    snip20::ExecuteMsg::Deposit { padding: None }
        .test_exec(&token, &mut app, admin.clone(), &vec![init_coin])
        .unwrap();

    // Deposit funds into treasury
    snip20::ExecuteMsg::Send {
        recipient: treasury.address.to_string().clone(),
        recipient_code_hash: None,
        amount: Uint128::new(deposit.u128()),
        msg: None,
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

    // Update treasury
    println!("UPDATE TREASURY");
    treasury::ExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    /*
    // Check Metrics
    match (treasury::QueryMsg::Metrics {
        date: metric_key(utc_from_timestamp(app.block_info().time)),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Metrics { metrics } => {
            for m in metrics {
                println!("{}", serde_json::to_string(&m).unwrap());
            }
            //assert!(metrics.len() != 0, "Treasury Metrics");
        }
        _ => panic!("query failed"),
    };
    */

    // Check treasury allowance to manager
    match (treasury::QueryMsg::Allowance {
        asset: token.address.to_string().clone(),
        spender: manager.address.to_string().clone(),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Allowance { amount } => {
            assert_eq!(amount, expected_allowance, "Treasury->Manager Allowance");
        }
        _ => panic!("query failed"),
    };

    // Update manager
    manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    })
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Update Adapter
    adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    })
    .test_exec(&adapter, &mut app, admin.clone(), &[])
    .unwrap();

    // Treasury reserves check
    match (treasury::QueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, pre_rewards.0, "Treasury Reserves");
        }
        _ => panic!("Query Failed"),
    };

    // Manager reserves
    match (manager::QueryMsg::Manager(manager::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
        holder: treasury.address.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap())
    {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, pre_rewards.1, "Manager Reserves");
        }
        _ => panic!("Query Failed"),
    };

    // Adapter reserves should be 0 (all staked)
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &app)
    .unwrap())
    {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "Bonded Adapter Reserves");
        }
        _ => panic!("Query Failed"),
    };

    // Adapter balance check
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &app)
    .unwrap())
    {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, pre_rewards.2, "Adapter Balance");
        }
        _ => panic!("Query Failed"),
    };

    // Add Rewards
    snip20::ExecuteMsg::Send {
        recipient: adapter.address.to_string().clone(),
        recipient_code_hash: None,
        amount: rewards,
        msg: None,
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

    // Adapter Balance
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &app)
    .unwrap())
    {
        adapter::QueryAnswer::Balance { amount } => {
            println!("L352 scrt bal {}", amount);
            assert_eq!(
                amount,
                pre_rewards.2 + rewards,
                "Adapter Balance Post-Rewards Pre-update"
            );
        }
        _ => panic!("Query Failed"),
    };

    for _ in 0..2 {
        // Update Adapter
        /*adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Update {
            asset: token.address.to_string().clone(),
        })
        .test_exec(&adapter, &mut app, admin.clone(), &[])
        .unwrap();*/

        // Update manager
        manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Update {
            asset: token.address.to_string().clone(),
        })
        .test_exec(&manager, &mut app, admin.clone(), &[])
        .unwrap();

        // Update treasury
        treasury::ExecuteMsg::Update {
            asset: token.address.to_string().clone(),
        }
        .test_exec(&treasury, &mut app, admin.clone(), &[])
        .unwrap();
    }

    // Adapter Balance
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &app)
    .unwrap())
    {
        adapter::QueryAnswer::Balance { amount } => {
            println!("L394 balance {}", amount);
            assert_eq!(
                amount,
                pre_rewards.2 + rewards,
                "Adapter Reserves Post-Rewards Post-Update"
            );
        }
        _ => panic!("Query Failed"),
    };

    // Adapter unbondable check
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbondable {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &mut app)
    .unwrap())
    {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, post_rewards.2, "Adapter Unbondable");
        }
        _ => panic!("Query Failed"),
    };

    // Manager unbondable check
    match (manager::QueryMsg::Manager(manager::SubQueryMsg::Unbondable {
        asset: token.address.to_string().clone(),
        holder: treasury.address.to_string().clone(),
    })
    .test_query(&manager, &mut app)
    .unwrap())
    {
        manager::QueryAnswer::Unbondable { amount } => {
            assert_eq!(
                amount,
                post_rewards.1 + post_rewards.2,
                "Manager Unbondable"
            );
        }
        _ => panic!("Query Failed"),
    };

    /*
    // Treasury unbondable check
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbondable {
        asset: token.address.to_string().clone(),
    })
    .test_query(&treasury, &mut app)
    .unwrap())
    {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(
                amount,
                post_rewards.1 + post_rewards.2,
                "Treasury Unbondable"
            );
        }
        _ => panic!("Query Failed"),
    };
    */

    // Unbond all w/ treasury
    manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Unbond {
        amount: post_rewards.1 + post_rewards.2,
        asset: token.address.to_string().clone(),
    })
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // adapter balance
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &mut app)
    .unwrap())
    {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "Adapter Reserves Pre-fastforward");
        }
        _ => panic!("Query Failed"),
    };

    // adapter unbonding
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbonding {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &mut app)
    .unwrap())
    {
        adapter::QueryAnswer::Unbonding { amount } => {
            assert_eq!(
                amount,
                pre_rewards.2 + rewards,
                "Adapter Unbonding Pre-fastforward"
            );
        }
        _ => panic!("Query Failed"),
    };

    // adapter claimable
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &mut app)
    .unwrap())
    {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Adapter Claimable Pre-fastforward");
        }
        _ => panic!("Query Failed"),
    };

    // Manager Claimable
    match (manager::QueryMsg::Manager(manager::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
        holder: treasury.address.to_string().clone(),
    })
    .test_query(&manager, &mut app)
    .unwrap())
    {
        manager::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Manager Claimable Pre-fastforward");
        }
        _ => panic!("Query Failed"),
    };

    // Manager Unbonding
    match (manager::QueryMsg::Manager(manager::SubQueryMsg::Unbonding {
        asset: token.address.to_string().clone(),
        holder: treasury.address.to_string().clone(),
    })
    .test_query(&manager, &mut app)
    .unwrap())
    {
        manager::QueryAnswer::Unbonding { amount } => {
            assert_eq!(
                amount,
                pre_rewards.2 + rewards,
                "Manager Unbonding Pre-fastforward"
            );
        }
        _ => panic!("Query Failed"),
    };

    mock_adapter::contract::ExecuteMsg::CompleteUnbonding {}
        .test_exec(&adapter, &mut app, admin.clone(), &[])
        .unwrap();

    // adapter unbonding
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbonding {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &mut app)
    .unwrap())
    {
        adapter::QueryAnswer::Unbonding { amount } => {
            assert_eq!(
                amount,
                Uint128::zero(),
                "Adapter Unbonding Post-fastforward"
            );
        }
        _ => panic!("Query Failed"),
    };

    // adapter claimable
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &mut app)
    .unwrap())
    {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(
                amount,
                pre_rewards.2 + rewards,
                "Adapter Claimable Post-fastforward"
            );
        }
        _ => panic!("Query Failed"),
    };

    // Claim Treasury Manager
    manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Claim {
        asset: token.address.to_string().clone(),
    })
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    /*
    // Claim Treasury
    adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Claim {
        asset: token.address.to_string().clone(),
    })
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();
    */

    /*
    // Treasury reserves check
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    })
    .test_query(&treasury, &mut app))
    .unwrap()
    {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, deposit + rewards, "Treasury Reserves Post-Claim");
        }
        _ => panic!("Bad Reserves Query Response"),
    };
    */

    /*
    // Manager balance check
    match (manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
        holder: treasury.address.to_string().clone(),
    })
    .test_query(&manager, &mut app)
    .unwrap())
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit + rewards, "Manager Balance Post Claim");
        }
        _ => panic!("Query Failed"),
    };
    */

    // Adapter balance
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &mut app)
    .unwrap())
    {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "Adapter Balance Post Claim ");
        }
        _ => panic!("Query Failed"),
    };

    // Treasury balance check
    match (treasury::QueryMsg::Balance {
        asset: token.address.to_string().clone(),
    }
    .test_query(&treasury, &mut app)
    .unwrap())
    {
        treasury::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit + rewards, "Treasury Balance Post Claim");
        }
        _ => panic!("Query Failed"),
    };

    // Adapter reserves
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &mut app)
    .unwrap())
    {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "Adapter Reserves Post Unbond");
        }
        _ => panic!("Query Failed"),
    };

    // Manager unbonding check
    match (manager::QueryMsg::Manager(manager::SubQueryMsg::Unbonding {
        asset: token.address.to_string().clone(),
        holder: treasury.address.to_string().clone(),
    })
    .test_query(&manager, &mut app)
    .unwrap())
    {
        manager::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, Uint128::zero(), "Manager Unbonding Post-Claim");
        }
        _ => panic!("Query Failed"),
    };

    // Manager balance check
    match (manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
        holder: treasury.address.to_string().clone(),
    })
    .test_query(&manager, &mut app)
    .unwrap())
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "Manager Balance Post-Claim");
        }
        _ => panic!("Query Failed"),
    };

    // Manager reserves check
    match (manager::QueryMsg::Manager(manager::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
        holder: treasury.address.to_string().clone(),
    })
    .test_query(&manager, &mut app)
    .unwrap())
    {
        manager::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "Manager Reserves Post-Unbond");
        }
        _ => panic!("Query Failed"),
    };

    // Treasury reserves check
    match (treasury::QueryMsg::Reserves {
        asset: token.address.to_string().clone(),
    }
    .test_query(&treasury, &mut app)
    .unwrap())
    {
        treasury::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, deposit + rewards, "Treasury Reserves Post-Unbond");
        }
        _ => panic!("Query Failed"),
    };

    // Treasury balance check
    match (treasury::QueryMsg::Balance {
        asset: token.address.to_string().clone(),
    }
    .test_query(&treasury, &mut app)
    .unwrap())
    {
        treasury::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit + rewards, "Treasury Balance Post-Unbond");
        }
        _ => panic!("Query Failed"),
    };

    // Migration
    println!("Setting migration runlevel");
    treasury::ExecuteMsg::SetRunLevel {
        run_level: RunLevel::Migrating,
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    //Update
    treasury::ExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    /*// Check Metrics
    match (treasury::QueryMsg::Metrics {
        date: None, //Some(utc_from_timestamp(app.block_info().time).to_rfc3339()),
        period: Period::Hour,
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Metrics { metrics } => {
            for m in metrics.clone() {
                println!("{}", serde_json::to_string(&m).unwrap());
            }
        }
        _ => panic!("query failed"),
    };*/

    match (snip20::QueryMsg::Balance {
        address: admin.to_string().clone(),
        key: viewing_key.clone(),
    })
    .test_query(&token, &app)
    .unwrap()
    {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit + rewards, "post-migration full unbond");
        }
        _ => {}
    };
}

macro_rules! bonded_adapter_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    deposit,
                    allowance,
                    expected_allowance,
                    alloc_type,
                    alloc_amount,
                    rewards,
                    pre_rewards,
                    post_rewards,
                ) = $value;
                bonded_adapter_int(
                    deposit,
                    allowance,
                    expected_allowance,
                    alloc_type,
                    alloc_amount,
                    rewards,
                    pre_rewards,
                    post_rewards,
                );
            }
        )*
    }
}

bonded_adapter_tests! {
    portion_with_rewards_0: (
        Uint128::new(100), // deposit
        Uint128::new(1 * 10u128.pow(18)), // manager allowance 100%
        Uint128::new(100), // expected manager allowance
        AllocationType::Portion,
        Uint128::new(1 * 10u128.pow(18)), // allocate 100%
        Uint128::new(100), // rewards
        // pre-rewards
        (
            Uint128::new(0), // treasury 10
            Uint128::new(0), // manager 0
            Uint128::new(100), // mock_adapter 90
        ),
        //post-rewards
        (
            Uint128::new(0), // treasury 10
            Uint128::new(0), // manager 0
            Uint128::new(200), // mock_adapter 90
        ),
    ),
    portion_with_rewards_1: (
        Uint128::new(1000), // deposit
        Uint128::new(5 * 10u128.pow(17)), // %50 manager allowance
        Uint128::new(500), // expected manager allowance
        AllocationType::Portion,
        Uint128::new(1 * 10u128.pow(18)), // 100% allocate
        Uint128::new(10), // rewards
        (
            Uint128::new(500), // treasury 55 (manager won't pull unused allowance
            Uint128::new(0), // manager 0
            Uint128::new(500), // mock_adapter
        ),
        (
            Uint128::new(505),
            Uint128::new(0),
            Uint128::new(505),
        ),
    ),
    /*
    // TODO: this needs separate test logic bc of update
    amount_with_rewards_0: (
        Uint128::new(1_000_000), // deposit
        Uint128::new(5 * 10u128.pow(17)), // %50 manager allowance
        Uint128::new(500_000), // expected manager allowance
        AllocationType::Amount,
        Uint128::new(500_000), // .5 tkn (all) allocate
        Uint128::new(500), // rewards
        (
            Uint128::new(500_000), // treasury
            Uint128::new(0), // manager 0
            Uint128::new(500_000), // mock_adapter
        ),
        (
            Uint128::new(500_250),
            Uint128::new(250),
            Uint128::new(500_000),
        ),
    ),
    */
}
