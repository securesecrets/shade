use shade_multi_test::multi::{admin::init_admin_auth, snip20::Snip20, treasury::Treasury};
use shade_protocol::{
    c_std::{to_binary, Addr, BlockInfo, Timestamp, Uint128},
    contract_interfaces::{
        dao::{treasury, treasury::AllowanceType},
        snip20,
    },
    multi_test::App,
    utils::{
        cycle::{parse_utc_datetime, Cycle},
        ExecuteCallback,
        InstantiateCallback,
        MultiTestable,
        Query,
    },
};

fn allowance_cycle(
    deposit: Uint128,
    removed: Uint128,
    start_expected: Uint128,
    start_allowance: Uint128,
    start_allow_type: AllowanceType,
    start_cycle: Cycle,
    updated_expected: Uint128,
    updated_allowance: Uint128,
    updated_allow_type: AllowanceType,
    updated_cycle: Cycle,
    start: String,
    refreshed: String,
) {
    let mut app = App::default();

    let start = parse_utc_datetime(&start).unwrap();
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(start.timestamp() as u64),
        chain_id: "chain_id".to_string(),
    });

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
            amount: deposit + deposit,
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

    // treasury starting allowance to spender
    treasury::ExecuteMsg::Allowance {
        asset: token.address.to_string().clone(),
        allowance: treasury::RawAllowance {
            //nick: "Mid-Stakes-Manager".to_string(),
            spender: spender.clone().to_string(),
            allowance_type: start_allow_type,
            cycle: start_cycle,
            amount: start_allowance,
            // 100% (adapter balance will 2x before unbond)
            tolerance: Uint128::zero(),
        },
        refresh_now: false,
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
            assert_eq!(amount, start_expected, "Initial Allowance");
        }
        _ => panic!("query failed"),
    };

    // Send out of treasury to reduce allowance (user using funds)
    snip20::ExecuteMsg::SendFrom {
        recipient: spender.to_string().clone(), //treasury.address.to_string().clone(),
        recipient_code_hash: None,
        owner: treasury.address.to_string(),
        amount: removed,
        memo: None,
        msg: None,
        padding: None,
    }
    .test_exec(&token, &mut app, spender.clone(), &[])
    .unwrap();

    // Refill treasury balance
    snip20::ExecuteMsg::Send {
        recipient: treasury.address.to_string().clone(),
        recipient_code_hash: None,
        amount: removed,
        memo: None,
        msg: None,
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

    // Check treasury allowance reflects used funds
    match (treasury::QueryMsg::Allowance {
        asset: token.address.to_string().clone(),
        spender: spender.to_string().clone(),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Allowance { amount } => {
            assert_eq!(amount, start_expected - removed, "Allowance after use");
        }
        _ => panic!("query failed"),
    };

    // Update allowance to spender
    treasury::ExecuteMsg::Allowance {
        asset: token.address.to_string().clone(),
        allowance: treasury::RawAllowance {
            //nick: "Mid-Stakes-Manager".to_string(),
            spender: spender.clone().to_string(),
            allowance_type: updated_allow_type,
            cycle: updated_cycle,
            amount: updated_allowance,
            // 100% (adapter balance will 2x before unbond)
            tolerance: Uint128::zero(),
        },
        refresh_now: false,
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    // Update treasury
    treasury::ExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    // Check allowance hasn't changed
    match (treasury::QueryMsg::Allowance {
        asset: token.address.to_string().clone(),
        spender: spender.to_string().clone(),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Allowance { amount } => {
            assert_eq!(
                amount,
                start_expected - removed,
                "Allowance after update, not refreshed"
            );
        }
        _ => panic!("query failed"),
    };

    let refreshed = parse_utc_datetime(&refreshed).unwrap();
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(refreshed.timestamp() as u64),
        chain_id: "chain_id".to_string(),
    });

    // Update treasury
    treasury::ExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    // Check treasury updated to new allowance settings
    match (treasury::QueryMsg::Allowance {
        asset: token.address.to_string().clone(),
        spender: spender.to_string().clone(),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Allowance { amount } => {
            assert_eq!(
                amount, updated_expected,
                "Allowance refreshed to updated amount"
            );
        }
        _ => panic!("query failed"),
    };
}

macro_rules! allowance_delay_update_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    deposit,
                    removed,
                    start_expected,
                    start_allowance,
                    start_allow_type,
                    start_cycle,
                    updated_expected,
                    updated_allowance,
                    updated_allow_type,
                    updated_cycle,
                    start,
                    refreshed,
                ) = $value;
                allowance_cycle(
                    deposit,
                    removed,
                    start_expected,
                    start_allowance,
                    start_allow_type,
                    start_cycle,
                    updated_expected,
                    updated_allowance,
                    updated_allow_type,
                    updated_cycle,
                    start.to_string(),
                    refreshed.to_string(),
                );
            }
        )*
    }
}

allowance_delay_update_tests! {
    portion_seconds_30: (
        Uint128::new(100), // deposit
        Uint128::new(100), // used
        Uint128::new(100), // start expected
        Uint128::new(1 * 10u128.pow(18)), // start allowance
        AllowanceType::Portion,
        Cycle::Seconds { seconds: Uint128::new(30) },

        Uint128::new(90), // updated expected
        Uint128::new(9 * 10u128.pow(17)), // updated allowance
        AllowanceType::Portion,
        Cycle::Seconds { seconds: Uint128::new(30) },
        "1995-11-13T00:00:00.00Z",
        "1995-11-13T00:00:30.00Z",
    ),
    amount_seconds_30: (
        Uint128::new(100), // deposit
        Uint128::new(100), // used
        Uint128::new(100), // start expected
        Uint128::new(100), // start allowance
        AllowanceType::Amount,
        Cycle::Seconds { seconds: Uint128::new(30) },

        Uint128::new(90), // updated expected
        Uint128::new(9 * 10u128.pow(17)), // updated allowance
        AllowanceType::Portion,
        Cycle::Seconds { seconds: Uint128::new(30) },
        "1995-11-13T00:00:00.00Z",
        "1995-11-13T00:00:30.00Z",
    ),
}
