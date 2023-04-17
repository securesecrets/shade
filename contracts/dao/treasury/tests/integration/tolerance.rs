use shade_multi_test::multi::{admin::init_admin_auth, snip20::Snip20, treasury::Treasury};
use shade_protocol::{
    c_std::{to_binary, Addr, Uint128},
    contract_interfaces::{
        dao::{treasury, treasury::AllowanceType},
        snip20,
    },
    multi_test::App,
    utils::{cycle::Cycle, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

fn underfunded_tolerance(
    deposit: Uint128,
    added: Uint128,
    tolerance: Uint128,

    allowance: Uint128,
    allow_type: AllowanceType,

    expected: Uint128,
) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
    let spender = Addr::unchecked("spender");
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

    let treasury = treasury::InstantiateMsg {
        admin_auth: admin_auth.clone().into(),
        viewing_key: viewing_key.clone(),
        multisig: admin.to_string().clone(),
    }
    .test_init(Treasury::default(), &mut app, admin.clone(), "treasury", &[
    ])
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

    // treasury allowance to spender
    treasury::ExecuteMsg::Allowance {
        asset: token.address.to_string().clone(),
        allowance: treasury::RawAllowance {
            //nick: "Mid-Stakes-Manager".to_string(),
            spender: spender.clone().to_string(),
            allowance_type: allow_type,
            cycle: Cycle::Constant,
            amount: allowance,
            // 100% (adapter balance will 2x before unbond)
            tolerance,
        },
        refresh_now: true,
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

    // Check treasury allowance
    match (treasury::QueryMsg::Allowance {
        asset: token.address.to_string().clone(),
        spender: spender.to_string().clone(),
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

    // Check treasury allowance
    match (treasury::QueryMsg::Allowance {
        asset: token.address.to_string().clone(),
        spender: spender.to_string().clone(),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Allowance { amount } => {
            assert_eq!(amount, expected, "Final Treasury->Manager Allowance");
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
                    allowance,
                    allow_type,
                    expected,
                ) = $value;
                underfunded_tolerance(
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

underfunded_tolerance_tests! {
    portion_tolerance_90_no_increase: (
        Uint128::new(100), // deposit
        Uint128::new(50), // added
        Uint128::new(9 * 10u128.pow(17)), // tolerance
        Uint128::new(1 * 10u128.pow(18)), // allowance
        AllowanceType::Portion,
        Uint128::new(100), // expected
    ),
    portion_tolerance_90_will_increase: (
        Uint128::new(100), // deposit
        Uint128::new(1000), // added
        Uint128::new(9 * 10u128.pow(17)), // tolerance
        Uint128::new(1 * 10u128.pow(18)), // allowance
        AllowanceType::Portion,
        Uint128::new(1100), // expected
    ),
    amount_tolerance_10_no_increase: (
        Uint128::new(500), // deposit
        Uint128::new(20), // added
        Uint128::new(9 * 10u128.pow(17)), // tolerance
        Uint128::new(520), //allowance
        AllowanceType::Amount,
        Uint128::new(500), // expected
    ),
    amount_tolerance_10_will_increase: (
        Uint128::new(500), // deposit
        Uint128::new(1000), // added
        Uint128::new(1 * 10u128.pow(17)), // tolerance
        Uint128::new(1500), //allowance
        AllowanceType::Amount,
        Uint128::new(1500), // expected
    ),
}

fn overfunded_tolerance(
    deposit: Uint128,
    tolerance: Uint128,
    allowance: Uint128,
    reduced: Uint128,
    allow_type: AllowanceType,
    expected: Uint128,
) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
    let spender = Addr::unchecked("spender");
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

    let treasury = treasury::InstantiateMsg {
        admin_auth: admin_auth.clone().into(),
        viewing_key: viewing_key.clone(),
        multisig: admin.to_string().clone(),
    }
    .test_init(Treasury::default(), &mut app, admin.clone(), "treasury", &[
    ])
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

    // treasury allowance to spender
    treasury::ExecuteMsg::Allowance {
        asset: token.address.to_string().clone(),
        allowance: treasury::RawAllowance {
            //nick: "Mid-Stakes-Manager".to_string(),
            spender: spender.clone().to_string(),
            allowance_type: allow_type.clone(),
            cycle: Cycle::Constant,
            amount: allowance,
            // 100% (adapter balance will 2x before unbond)
            tolerance,
        },
        refresh_now: true,
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

    // Check treasury allowance
    match (treasury::QueryMsg::Allowance {
        asset: token.address.to_string().clone(),
        spender: spender.to_string().clone(),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Allowance { amount } => {
            println!("INITIAL {}", amount);
            assert_eq!(amount, deposit, "Initial Treasury->Manager Allowance");
        }
        _ => panic!("query failed"),
    };

    // Reduce allowance to simulate overfunding
    treasury::ExecuteMsg::Allowance {
        asset: token.address.to_string().clone(),
        allowance: treasury::RawAllowance {
            spender: spender.clone().to_string(),
            allowance_type: allow_type,
            cycle: Cycle::Constant,
            amount: reduced,
            tolerance,
        },
        refresh_now: true,
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    // Update treasury
    treasury::ExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    // Check treasury allowance
    match (treasury::QueryMsg::Allowance {
        asset: token.address.to_string().clone(),
        spender: spender.to_string().clone(),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Allowance { amount } => {
            assert_eq!(amount, expected, "Final Treasury->Manager Allowance");
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
                    allowance,
                    reduced,
                    allow_type,
                    expected,
                ) = $value;
                overfunded_tolerance(
                    deposit,
                    tolerance,
                    allowance,
                    reduced,
                    allow_type,
                    expected,
                );
            }
        )*
    }
}

overfunded_tolerance_tests! {
    portion_tolerance_10_no_decrease: (
        Uint128::new(1000), // deposit
        Uint128::new(1 * 10u128.pow(17)), // tolerance
        Uint128::new(1 * 10u128.pow(18)), // allowance
        Uint128::new(99 * 10u128.pow(16)), // reduced_allowance
        AllowanceType::Portion,
        Uint128::new(1000), // expected
    ),
    portion_tolerance_10_will_decrease: (
        Uint128::new(1000), // deposit
        Uint128::new(1 * 10u128.pow(17)), // tolerance
        Uint128::new(1 * 10u128.pow(18)), // allowance
        Uint128::new(5 * 10u128.pow(17)), // reduced_allowance
        AllowanceType::Portion,
        Uint128::new(500), // expected
    ),
    amount_tolerance_10_no_decrease: (
        Uint128::new(500), // deposit
        Uint128::new(1 * 10u128.pow(17)), // tolerance
        Uint128::new(500), //allowance
        Uint128::new(460), // reduced allowance
        AllowanceType::Amount,
        Uint128::new(500), // expected
    ),
    amount_tolerance_10_will_decrease: (
        Uint128::new(500), // deposit
        Uint128::new(1 * 10u128.pow(17)), // tolerance
        Uint128::new(500), //allowance
        Uint128::new(400), // reduced allowance
        AllowanceType::Amount,
        Uint128::new(400), // expected
    ),
}
