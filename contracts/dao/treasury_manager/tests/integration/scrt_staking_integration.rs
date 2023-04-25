/*
use shade_protocol::{
    c_std::{to_binary, Addr, Coin, Decimal, Uint128, Validator},
    contract_interfaces::{
        dao::{
            adapter,
            manager,
            scrt_staking,
            treasury_manager::{self, Allocation, AllocationType},
        },
        snip20,
    },
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

use shade_multi_test::multi::{
    admin::init_admin_auth,
    scrt_staking::ScrtStaking,
    snip20::Snip20,
    treasury_manager::TreasuryManager,
};
use shade_protocol::multi_test::{App, BankSudo, StakingSudo, SudoMsg};

/* No adapters configured
 * All assets will sit on manager unused as "reserves"
 * No need to "claim" as "unbond" will send up to "reserves"
 */
fn single_holder_scrt_staking_adapter(
    deposit: Uint128,
    alloc_type: AllocationType,
    alloc_amount: Uint128,
    rewards: Uint128,
    expected_scrt_staking: Uint128,
    expected_manager_holder: Uint128,
    expected_manager_treasury: Uint128,
    unbond_amount: Uint128,
) {
    let mut app = App::default();
    let viewing_key = "unguessable".to_string();

    let admin = Addr::unchecked("admin");
    let holder = Addr::unchecked("holder");
    let treasury = Addr::unchecked("treasury");
    let validator = Addr::unchecked("validator");
    let admin_auth = init_admin_auth(&mut app, &admin);

    app.sudo(SudoMsg::Staking(StakingSudo::AddValidator {
        validator: validator.to_string().clone(),
    }))
    .unwrap();

    let token = snip20::InstantiateMsg {
        name: "secretSCRT".into(),
        admin: Some("admin".into()),
        symbol: "SSCRT".into(),
        decimals: 6,
        initial_balances: None,
        prng_seed: to_binary("").ok().unwrap(),
        query_auth: None,
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(false),
            enable_burn: Some(false),
            enable_transfer: Some(true),
        }),
    }
    .test_init(Snip20::default(), &mut app, admin.clone(), "token", &[])
    .unwrap();

    let manager = treasury_manager::InstantiateMsg {
        admin_auth: admin_auth.clone().into(),
        treasury: treasury.clone().into(),
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

    let scrt_staking = scrt_staking::InstantiateMsg {
        admin_auth: admin_auth.into(),
        owner: manager.address.to_string().clone().into(),
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
    println!("scrt staking {}", scrt_staking.address.clone());

    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, holder.clone(), &[])
    .unwrap();

    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, treasury.clone(), &[])
    .unwrap();

    // Register manager assets
    treasury_manager::ExecuteMsg::RegisterAsset {
        contract: token.clone().into(),
    }
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Add 'holder' as holder
    treasury_manager::ExecuteMsg::AddHolder {
        holder: holder.to_string().clone().into(),
    }
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Allocate to scrt_staking from manager
    treasury_manager::ExecuteMsg::Allocate {
        asset: token.address.to_string().clone(),
        allocation: Allocation {
            nick: Some("scrt_staking".to_string()),
            contract: Contract {
                address: scrt_staking.address.clone(),
                code_hash: scrt_staking.code_hash.clone(),
            },
            alloc_type,
            amount: alloc_amount,
            tolerance: Uint128::zero(),
        },
    }
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    let deposit_coin = Coin {
        denom: "uscrt".into(),
        amount: deposit,
    };
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: holder.to_string().clone(),
        amount: vec![deposit_coin.clone()],
    }))
    .unwrap();

    assert!(deposit_coin.amount > Uint128::zero());

    // Wrap L1
    &snip20::ExecuteMsg::Deposit { padding: None }
        .test_exec(&token, &mut app, holder.clone(), &[deposit_coin])
        .unwrap();

    // Deposit funds into manager
    println!("deposit to manager");
    snip20::ExecuteMsg::Send {
        recipient: manager.address.to_string().clone(),
        recipient_code_hash: None,
        amount: deposit,
        msg: None,
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, holder.clone(), &[])
    .unwrap();

    // Update manager
    manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    })
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Balance Checks

    // manager reported holder balance
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond Manager Holder Balance");
        }
        _ => assert!(false),
    };

    // manager reported treasury balance
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
        holder: treasury.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(
                amount,
                Uint128::zero(),
                "Pre-unbond Manager Treasury Balance"
            );
        }
        _ => assert!(false),
    };

    // scrt staking balance
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(
                amount, expected_scrt_staking,
                "Pre-unbond scrt staking balance"
            );
        }
        _ => assert!(false),
    };

    // manager unbondable
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Unbondable {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond unbondable");
        }
        _ => assert!(false),
    };

    let mut reserves = Uint128::zero();

    // Reserves
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Reserves { amount } => {
            reserves = amount;
            assert_eq!(amount, expected_manager_holder, "Pre-unbond reserves");
        }
        _ => assert!(false),
    };

    // Claimable
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Pre-unbond claimable");
        }
        _ => assert!(false),
    };

    // Add Rewards
    app.sudo(SudoMsg::Staking(StakingSudo::AddRewards {
        amount: Coin {
            amount: rewards,
            denom: "uscrt".into(),
        },
    }))
    .unwrap();

    // Update scrt staking to claim & restake rewards
    adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    })
    .test_exec(&scrt_staking, &mut app, admin.clone(), &[])
    .unwrap();

    // Update manager to detect & rebalance after gainz
    manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    })
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // holder unbond from manager
    println!("manager unbond {}", unbond_amount);
    manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Unbond {
        asset: token.address.to_string().clone(),
        amount: unbond_amount,
    })
    .test_exec(&manager, &mut app, holder.clone(), &[])
    .unwrap();

    // scrt staking Unbondable
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbondable {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(
                amount,
                deposit - unbond_amount,
                "Post-unbond scrt staking unbondable"
            );
        }
        _ => assert!(false),
    };

    // manager Unbondable
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Unbondable {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Unbondable { amount } => {
            assert_eq!(
                amount,
                deposit - unbond_amount,
                "Post-unbond manager holder unbondable"
            );
        }
        _ => assert!(false),
    };

    // Unbonding
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Unbonding {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Unbonding { amount } => {
            assert_eq!(
                amount,
                unbond_amount - reserves,
                "Post-unbond manager unbonding"
            );
        }
        _ => assert!(false),
    };

    // Manager Claimable
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Pre-fastforward manager claimable");
        }
        _ => assert!(false),
    };

    app.sudo(SudoMsg::Staking(StakingSudo::FastForwardUndelegate {}))
        .unwrap();

    // Scrt Staking Claimable
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
    })
    .test_query(&scrt_staking, &app)
    .unwrap()
    {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(
                amount,
                unbond_amount - reserves + rewards,
                "Post-fastforward scrt staking claimable"
            );
        }
        _ => assert!(false),
    };

    // Manager Claimable
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Claimable { amount } => {
            assert_eq!(
                amount,
                unbond_amount - reserves,
                "Post-fastforward manager claimable"
            );
        }
        _ => assert!(false),
    };

    // Claim
    manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Claim {
        asset: token.address.to_string().clone(),
    })
    .test_exec(&manager, &mut app, holder.clone(), &[])
    .unwrap();

    // Manager Holder Balance
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit - unbond_amount);
        }
        _ => {
            assert!(false);
        }
    };

    // Manager Treasury Balance
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
        holder: treasury.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, expected_manager_treasury);
        }
        _ => assert!(false),
    };

    // user received unbonded
    match (snip20::QueryMsg::Balance {
        address: holder.to_string().clone(),
        key: viewing_key.clone(),
    })
    .test_query(&token, &app)
    .unwrap()
    {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(
                amount.u128(),
                unbond_amount.u128(),
                "Post-claim holder snip20 balance"
            );
        }
        _ => {
            assert!(false);
        }
    };

    /*
    // treasury received gainz
    match (snip20::QueryMsg::Balance {
        address: treasury.to_string().clone(),
        key: viewing_key.clone(),
    }).test_query(&token, &app).unwrap() {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, expected_manager_treasury, "treasury snip20 balance");
        },
        _ => assert!(false),
    };
    */
}

macro_rules! single_holder_scrt_staking_adapter_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (deposit, alloc_type, alloc_amount, rewards,
                     expected_scrt_staking, expected_manager_holder, expected_manager_treasury, unbond_amount) = $value;
                single_holder_scrt_staking_adapter(deposit, alloc_type, alloc_amount, rewards, expected_scrt_staking, expected_manager_holder, expected_manager_treasury, unbond_amount);
            }
        )*
    }
}

single_holder_scrt_staking_adapter_tests! {
    single_holder_scrt_staking_portion: (
        // 100
        Uint128::new(100_000_000),
        // % 50 alloc
        AllocationType::Portion,
        Uint128::new(5u128 * 10u128.pow(17)),
        // 0 rewards
        Uint128::zero(),
        // 50/50
        Uint128::new(50_000_000),
        Uint128::new(50_000_000),
        Uint128::zero(),
        // unbond 75
        Uint128::new(75_000_000),
    ),
    single_holder_scrt_staking_amount: (
        // 100
        Uint128::new(100_000_000),
        // 50 alloc
        AllocationType::Amount,
        Uint128::new(50_000_000),
        // 0 rewards
        Uint128::zero(),
        // 50/50
        Uint128::new(50_000_000),
        Uint128::new(50_000_000),
        Uint128::zero(),
        // unbond 75
        Uint128::new(75_000_000),
    ),
    single_holder_scrt_staking_amount_rewards: (
        // 100
        Uint128::new(100_000_000),
        // 50 alloc
        AllocationType::Amount,
        Uint128::new(50_000_000),
        // 0 rewards
        Uint128::new(100_000_000),
        // 50/50
        Uint128::new(50_000_000),
        Uint128::new(50_000_000),
        Uint128::new(100_000_000),
        // unbond 75
        Uint128::new(75_000_000),
    ),
}
*/
