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

//use serde_json;

// Add other adapters here as they come
fn treasury_tolerance_test(
    deposit: Uint128,
    added: Uint128,
    tolerance: Uint128,

    allowance: Uint128,
    allow_type: AllowanceType,

    expected: Uint128,
) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
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
            allowance_type: allow_type,
            cycle: Cycle::Constant,
            amount: allowance,
            // 100% (adapter balance will 2x before unbond)
            tolerance,
        },
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    // Deposit funds into treasury
    snip20::ExecuteMsg::Send {
        recipient: treasury.address.to_string().clone(),
        recipient_code_hash: None,
        amount: deposit,
        msg: None,
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

    // Update treasury
    treasury::ExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    // Check treasury allowance to manager
    match (treasury::QueryMsg::Allowance {
        asset: token.address.to_string().clone(),
        spender: manager.address.to_string().clone(),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Allowance { amount } => {
            assert_eq!(amount, deposit, "Initial Treasury->Manager Allowance");
        }
        _ => panic!("query failed"),
    };

    // Additional funds into treasury
    snip20::ExecuteMsg::Send {
        recipient: treasury.address.to_string().clone(),
        recipient_code_hash: None,
        amount: added,
        msg: None,
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, admin.clone(), &[])
    .unwrap();

    // Update treasury
    treasury::ExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    // Check treasury allowance to manager
    match (treasury::QueryMsg::Allowance {
        asset: token.address.to_string().clone(),
        spender: manager.address.to_string().clone(),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Allowance { amount } => {
            assert_eq!(amount, expected, "Final Treasury->Manager Allowance");
        }
        _ => panic!("query failed"),
    };

    /*
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
    */
}

macro_rules! treasury_tolerance_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    deposit,
                    added,
                    tolerance,
                    allowance,
                    allow_type,
                    expected,
                ) = $value;
                treasury_tolerance_test(
                    deposit,
                    added,
                    tolerance,
                    allowance,
                    allow_type,
                    expected,
                );
            }
        )*
    }
}

treasury_tolerance_tests! {
    tolerance_portion_90_no_increase: (
        Uint128::new(100), // deposit
        Uint128::new(50), // added
        Uint128::new(9 * 10u128.pow(17)), // tolerance
        Uint128::new(1 * 10u128.pow(18)), // allowance
        AllowanceType::Portion,
        Uint128::new(100), // expected
    ),
    tolerance_portion_90_will_increase: (
        Uint128::new(100), // deposit
        Uint128::new(200), // added
        Uint128::new(9 * 10u128.pow(17)), // tolerance
        Uint128::new(1 * 10u128.pow(18)), // allowance
        AllowanceType::Portion,
        Uint128::new(300), // expected
    ),
    tolerance_amount_90_no_increase: (
        Uint128::new(100), // deposit
        Uint128::new(50), // added
        Uint128::new(9 * 10u128.pow(17)), // tolerance
        Uint128::new(100), //allowance
        AllowanceType::Amount,
        Uint128::new(100), // expected
    ),
    /*
     * TODO needs the fixes for not exceeding balance w/ allowance
    tolerance_amount_90_will_increase: (
        Uint128::new(100), // deposit
        Uint128::new(200), // added
        Uint128::new(9 * 10u128.pow(17)), // tolerance
        Uint128::new(300), //allowance
        AllowanceType::Amount,
        Uint128::new(300), // expected
    ),
    */
}
