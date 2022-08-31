use shade_multi_test::{
    interfaces,
    multi::{admin::init_admin_auth, snip20::Snip20, treasury_manager::TreasuryManager},
};
use shade_protocol::{
    c_std::{
        coins,
        from_binary,
        to_binary,
        Addr,
        Binary,
        Coin,
        ContractInfo,
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
        asset::{Contract, RawContract},
        cycle::{utc_from_timestamp, Cycle},
        storage::plus::period_storage::Period,
        ExecuteCallback,
        InstantiateCallback,
        MultiTestable,
        Query,
    },
};

// Add other adapters here as they come
fn batch_balance_test(balances: Vec<Uint128>) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
    let user = Addr::unchecked("user");
    let admin_auth = init_admin_auth(&mut app, &admin);

    let viewing_key = "viewing_key".to_string();

    let manager = treasury_manager::InstantiateMsg {
        admin_auth: admin_auth.clone().into(),
        viewing_key: viewing_key.clone(),
        treasury: admin.to_string().clone(),
    }
    .test_init(
        TreasuryManager::default(),
        &mut app,
        admin.clone(),
        "treasury",
        &[],
    )
    .unwrap();

    let mut tokens = vec![];

    for bal in balances.clone() {
        let token = snip20::InstantiateMsg {
            name: "token".into(),
            admin: Some("admin".into()),
            symbol: "TKN".into(),
            decimals: 6,
            initial_balances: Some(vec![snip20::InitialBalance {
                address: admin.to_string().clone(),
                amount: bal.clone(),
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
        .test_init(
            Snip20::default(),
            &mut app,
            admin.clone(),
            &bal.to_string(),
            &[],
        )
        .unwrap();

        treasury_manager::ExecuteMsg::RegisterAsset {
            contract: token.clone().into(),
        }
        .test_exec(&manager, &mut app, admin.clone(), &[])
        .unwrap();

        // Deposit funds as treasury
        snip20::ExecuteMsg::Send {
            recipient: manager.address.to_string().clone(),
            recipient_code_hash: None,
            amount: bal,
            msg: None,
            memo: None,
            padding: None,
        }
        .test_exec(&token, &mut app, admin.clone(), &[])
        .unwrap();

        tokens.push(token);
    }

    // Treasury Balances
    match (manager::QueryMsg::Manager(manager::SubQueryMsg::BatchBalance {
        assets: tokens
            .iter()
            .map(|t| t.address.to_string().clone())
            .collect(),
        holder: admin.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap())
    {
        manager::QueryAnswer::BatchBalance { amounts } => {
            assert!(amounts == balances, "Reported balances match inputs");
        }
        _ => {
            panic!("Failed to query batch balances");
        }
    }
}

macro_rules! batch_balance_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                batch_balance_test($value.into_iter().map(|a| Uint128::new(a as u128)).collect());
            }
        )*
    }
}

batch_balance_tests! {
    batch_balances_0: vec![10, 23840, 8402840, 123456, 0],
}
