use mock_adapter;
use shade_multi_test::multi::{
    admin::init_admin_auth,
    mock_adapter::MockAdapter,
    snip20::Snip20,
    treasury_manager::TreasuryManager,
};
use shade_protocol::{
    c_std::{to_binary, Addr, Uint128},
    contract_interfaces::{
        dao::{
            adapter,
            manager,
            treasury_manager::{self, AllocationType, RawAllocation},
        },
        snip20,
    },
    multi_test::App,
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

fn underfunded_tolerance(
    deposit: Uint128,
    added: Uint128,
    tolerance: Uint128,
    allocation: Uint128,
    alloc_type: AllocationType,
    expected: Uint128,
) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
    let _spender = Addr::unchecked("spender");
    let treasury = Addr::unchecked("treasury");
    let _user = Addr::unchecked("user");
    //let validator = Addr::unchecked("validator");
    let admin_auth = init_admin_auth(&mut app, &admin);

    let viewing_key = "viewing_key".to_string();

    let token = snip20::InstantiateMsg {
        name: "token".into(),
        admin: Some("admin".into()),
        symbol: "TKN".into(),
        decimals: 6,
        initial_balances: Some(vec![snip20::InitialBalance {
            address: admin.to_string().clone(),
            amount: deposit + added,
        }]),
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

    let manager = treasury_manager::InstantiateMsg {
        admin_auth: admin_auth.clone().into(),
        viewing_key: viewing_key.clone(),
        treasury: treasury.to_string().clone(),
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
        instant: true,
        token: token.clone().into(),
    }
    .test_init(
        MockAdapter::default(),
        &mut app,
        admin.clone(),
        "adapter",
        &[],
    )
    .unwrap();

    // Set admin viewing key
    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

    // Register treasury assets
    treasury_manager::ExecuteMsg::RegisterAsset {
        contract: token.clone().into(),
    }
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // treasury allocation to spender
    treasury_manager::ExecuteMsg::Allocate {
        asset: token.address.to_string().clone(),
        allocation: RawAllocation {
            nick: Some("Manager".to_string()),
            contract: RawContract::from(adapter.clone()),
            alloc_type,
            amount: allocation,
            // 100% (adapter balance will 2x before unbond)
            tolerance,
        },
    }
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Deposit funds into treasury
    snip20::ExecuteMsg::Send {
        recipient: manager.address.to_string().clone(),
        recipient_code_hash: None,
        amount: deposit,
        msg: None,
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

    // Update manager
    manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    })
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Check adapter balance
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Adapter Balance");
        }
        _ => panic!("query failed"),
    };

    // Additional funds into manager
    snip20::ExecuteMsg::Send {
        recipient: manager.address.to_string().clone(),
        recipient_code_hash: None,
        amount: added,
        msg: None,
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

    // Update manager
    manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    })
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Check adapter balance
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, expected, "Final Adapter Balance");
        }
        _ => panic!("query failed"),
    };
}

macro_rules! underfunded_tolerance_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    deposit,
                    added,
                    tolerance,
                    allocation,
                    alloc_type,
                    expected,
                ) = $value;
                underfunded_tolerance(
                    deposit,
                    added,
                    tolerance,
                    allocation,
                    alloc_type,
                    expected,
                );
            }
        )*
    }
}

underfunded_tolerance_tests! {
    tolerance_portion_90_no_increase: (
        Uint128::new(100), // deposit
        Uint128::new(50), // added
        Uint128::new(9 * 10u128.pow(17)), // tolerance
        Uint128::new(1 * 10u128.pow(18)), // allocation
        AllocationType::Portion,
        Uint128::new(100), // expected
    ),
    tolerance_portion_90_will_increase: (
        Uint128::new(100), // deposit
        Uint128::new(1000), // added
        Uint128::new(9 * 10u128.pow(17)), // tolerance
        Uint128::new(1 * 10u128.pow(18)), // allowance
        AllocationType::Portion,
        Uint128::new(1100), // expected
    ),
    tolerance_amount_10_no_increase: (
        Uint128::new(100), // deposit
        Uint128::new(5), // added
        Uint128::new(1 * 10u128.pow(17)), // tolerance
        Uint128::new(105), //allowance
        AllocationType::Amount,
        Uint128::new(100), // expected
    ),
}

fn overfunded_tolerance(
    deposit: Uint128,
    tolerance: Uint128,
    allocation: Uint128,
    reduced: Uint128,
    alloc_type: AllocationType,
    expected: Uint128,
) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
    let _spender = Addr::unchecked("spender");
    let treasury = Addr::unchecked("treasury");
    let _user = Addr::unchecked("user");
    //let validator = Addr::unchecked("validator");
    let admin_auth = init_admin_auth(&mut app, &admin);

    let viewing_key = "viewing_key".to_string();

    let token = snip20::InstantiateMsg {
        name: "token".into(),
        admin: Some("admin".into()),
        symbol: "TKN".into(),
        decimals: 6,
        initial_balances: Some(vec![snip20::InitialBalance {
            address: admin.to_string().clone(),
            amount: deposit,
        }]),
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

    let manager = treasury_manager::InstantiateMsg {
        admin_auth: admin_auth.clone().into(),
        viewing_key: viewing_key.clone(),
        treasury: treasury.to_string().clone(),
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
        instant: true,
        token: token.clone().into(),
    }
    .test_init(
        MockAdapter::default(),
        &mut app,
        admin.clone(),
        "adapter",
        &[],
    )
    .unwrap();

    // Set admin viewing key
    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

    // Register treasury assets
    treasury_manager::ExecuteMsg::RegisterAsset {
        contract: token.clone().into(),
    }
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // treasury allocation to spender
    treasury_manager::ExecuteMsg::Allocate {
        asset: token.address.to_string().clone(),
        allocation: RawAllocation {
            nick: Some("Manager".to_string()),
            contract: RawContract::from(adapter.clone()),
            alloc_type: alloc_type.clone(),
            amount: allocation,
            // 100% (adapter balance will 2x before unbond)
            tolerance,
        },
    }
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Deposit funds into treasury
    snip20::ExecuteMsg::Send {
        recipient: manager.address.to_string().clone(),
        recipient_code_hash: None,
        amount: deposit,
        msg: None,
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

    // Update manager
    manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    })
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Check adapter balance
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Adapter Balance");
        }
        _ => panic!("query failed"),
    };

    // reduce allocation
    treasury_manager::ExecuteMsg::Allocate {
        asset: token.address.to_string().clone(),
        allocation: RawAllocation {
            nick: Some("Manager".to_string()),
            contract: RawContract::from(adapter.clone()),
            alloc_type,
            amount: reduced,
            // 100% (adapter balance will 2x before unbond)
            tolerance,
        },
    }
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Update manager
    manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    })
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Check adapter balance
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, expected, "Final Adapter Balance");
        }
        _ => panic!("query failed"),
    };
}

macro_rules! overfunded_tolerance_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    deposit,
                    tolerance,
                    allocation,
                    reduced,
                    alloc_type,
                    expected,
                ) = $value;
                overfunded_tolerance(
                    deposit,
                    tolerance,
                    allocation,
                    reduced,
                    alloc_type,
                    expected,
                );
            }
        )*
    }
}

overfunded_tolerance_tests! {
    over_portion_tolerance_10_no_decrease: (
        Uint128::new(100), // deposit
        Uint128::new(1 * 10u128.pow(17)), // tolerance
        Uint128::new(1 * 10u128.pow(18)), // allocation
        Uint128::new(95 * 10u128.pow(16)), // reduced allocation
        AllocationType::Portion,
        Uint128::new(100), // expected
    ),
    over_portion_tolerance_10_will_decrease: (
        Uint128::new(100), // deposit
        Uint128::new(1 * 10u128.pow(17)), // tolerance
        Uint128::new(1 * 10u128.pow(18)), // allocation
        Uint128::new(1 * 10u128.pow(17)), // reduced allocation
        AllocationType::Portion,
        Uint128::new(10), // expected
    ),
    over_amount_tolerance_10_no_decrease: (
        Uint128::new(100), // deposit
        Uint128::new(1 * 10u128.pow(17)), // tolerance
        Uint128::new(100), // allocation
        Uint128::new(95), // reduced allocation
        AllocationType::Amount,
        Uint128::new(100), // expected
    ),
    over_amount_tolerance_10_will_decrease: (
        Uint128::new(100), // deposit
        Uint128::new(1 * 10u128.pow(17)), // tolerance
        Uint128::new(100), // allocation
        Uint128::new(80), // reduced allocation
        AllocationType::Amount,
        Uint128::new(80), // expected
    ),
}
