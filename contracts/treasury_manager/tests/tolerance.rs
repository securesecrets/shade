use mock_adapter;
use shade_multi_test::{
    interfaces,
    multi::{
        admin::init_admin_auth,
        mock_adapter::MockAdapter,
        snip20::Snip20,
        //treasury::Treasury,
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
    contract_interfaces::{
        dao::{
            adapter,
            manager,
            treasury_manager::{self, Allocation, AllocationType},
        },
        snip20,
    },
    multi_test::{App, BankSudo, StakingSudo, SudoMsg},
    utils::{
        asset::Contract,
        cycle::Cycle,
        ExecuteCallback,
        InstantiateCallback,
        MultiTestable,
        Query,
    },
};

fn overfunded_tolerance(
    deposit: Uint128,
    added: Uint128,
    tolerance: Uint128,
    allocation: Uint128,
    alloc_type: AllocationType,
    expected: Uint128,
) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
    let spender = Addr::unchecked("spender");
    let treasury = Addr::unchecked("treasury");
    let user = Addr::unchecked("user");
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
        allocation: Allocation {
            nick: Some("Manager".to_string()),
            contract: adapter.clone().into(),
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
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &app)
    .unwrap())
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
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
    })
    .test_query(&adapter, &app)
    .unwrap())
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
                    added,
                    tolerance,
                    allocation,
                    alloc_type,
                    expected,
                ) = $value;
                overfunded_tolerance(
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

overfunded_tolerance_tests! {
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
    //TODO decrease tests
}
