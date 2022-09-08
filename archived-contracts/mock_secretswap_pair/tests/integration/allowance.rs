use mock_adapter;
use shade_multi_test::{
    interfaces,
    multi::{
        admin::init_admin_auth,
        mock_adapter::MockAdapter,
        snip20::Snip20,
        treasury::Treasury,
    },
};
use shade_protocol::{
    c_std::{
        coins,
        from_binary,
        to_binary,
        Addr,
        Binary,
        BlockInfo,
        Coin,
        Decimal,
        Env,
        StdError,
        StdResult,
        Timestamp,
        Uint128,
        Validator,
    },
    contract_interfaces::{
        dao::{
            adapter,
            treasury,
            treasury::{Allowance, AllowanceType, RunLevel},
        },
        snip20,
    },
    multi_test::{App, BankSudo, StakingSudo, SudoMsg},
    utils::{
        asset::Contract,
        cycle::{parse_utc_datetime, Cycle},
        storage::plus::period_storage::Period,
        ExecuteCallback,
        InstantiateCallback,
        MultiTestable,
        Query,
    },
};

fn allowance_cycle(
    deposit: Uint128,
    removed: Uint128,
    expected: Uint128,
    allowance: Uint128,
    allow_type: AllowanceType,
    cycle: Cycle,
    start: String,
    not_refreshed: String,
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
        allowance: treasury::Allowance {
            //nick: "Mid-Stakes-Manager".to_string(),
            spender: spender.clone(),
            allowance_type: allow_type,
            cycle,
            amount: allowance,
            // 100% (adapter balance will 2x before unbond)
            tolerance: Uint128::zero(),
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

    // Check treasury allowance
    match (treasury::QueryMsg::Allowance {
        asset: token.address.to_string().clone(),
        spender: spender.to_string().clone(),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Allowance { amount } => {
            assert_eq!(amount, expected, "Initial Allowance");
        }
        _ => panic!("query failed"),
    };

    // Send out of treasury to reduce allowance
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

    // Send back to treasury to maintain balance/expected
    snip20::ExecuteMsg::Send {
        recipient: treasury.address.to_string().clone(),
        recipient_code_hash: None,
        amount: removed,
        memo: None,
        msg: None,
        padding: None,
    }
    .test_exec(&token, &mut app, spender.clone(), &[])
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
            assert_eq!(amount, expected - removed, "Allowance after use");
        }
        _ => panic!("query failed"),
    };

    // Update treasury
    treasury::ExecuteMsg::Update {
        asset: token.address.to_string().clone(),
    }
    .test_exec(&treasury, &mut app, admin.clone(), &[])
    .unwrap();

    //TODO override env.block.time
    let not_refreshed = parse_utc_datetime(&not_refreshed).unwrap();
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(not_refreshed.timestamp() as u64),
        chain_id: "chain_id".to_string(),
    });

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
            assert_eq!(amount, expected - removed, "Allowance not refreshed");
        }
        _ => panic!("query failed"),
    };

    //TODO override env.block.time
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

    // Check treasury allowance
    match (treasury::QueryMsg::Allowance {
        asset: token.address.to_string().clone(),
        spender: spender.to_string().clone(),
    }
    .test_query(&treasury, &app)
    .unwrap())
    {
        treasury::QueryAnswer::Allowance { amount } => {
            assert_eq!(amount, expected, "Allowance refreshed");
        }
        _ => panic!("query failed"),
    };
}

macro_rules! allowance_cycle_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    deposit,
                    removed,
                    expected,
                    allowance,
                    allow_type,
                    cycle,
                    start,
                    not_refreshed,
                    refreshed,
                ) = $value;
                allowance_cycle(
                    deposit,
                    removed,
                    expected,
                    allowance,
                    allow_type,
                    cycle,
                    start.to_string(),
                    not_refreshed.to_string(),
                    refreshed.to_string(),
                );
            }
        )*
    }
}

allowance_cycle_tests! {
    portion_daily_1: (
        Uint128::new(100), // deposit
        Uint128::new(100), // removed
        Uint128::new(100), // expected
        Uint128::new(1 * 10u128.pow(18)), // allowance
        AllowanceType::Portion,
        Cycle::Daily { days: Uint128::new(1) },
        "1995-11-13T00:00:00.00Z",
        "1995-11-13T12:00:00.00Z",
        "1995-11-14T00:00:00.00Z",
    ),
    portion_monthly_1: (
        Uint128::new(100), // deposit
        Uint128::new(100), // removed
        Uint128::new(100), // expected
        Uint128::new(1 * 10u128.pow(18)), // allowance
        AllowanceType::Portion,
        Cycle::Monthly { months: Uint128::new(1) },
        "1995-11-13T00:00:00.00Z",
        "1995-11-13T12:00:00.00Z",
        "1995-12-13T00:00:00.00Z",
    ),
}
