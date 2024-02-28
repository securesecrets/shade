use shade_protocol::c_std::{
    to_binary, Addr, BlockInfo, ContractInfo, Decimal, Timestamp, Uint128,
};

use shade_protocol::{
    credit_agency, lend_market, lend_token,
    multi_test::{App, Executor},
    query_auth, snip20,
    utils::{
        asset::{Contract, RawContract},
        ExecuteCallback, InstantiateCallback, MultiTestable, Query,
    },
};

use shade_multi_test::multi::{
    admin::{init_admin_auth, Admin},
    credit_agency::CreditAgency,
    lend_market::LendMarket,
    lend_token::LendToken,
    query_auth::QueryAuth,
    snip20::Snip20,
};

use shade_protocol::lending_utils::{coin::Coin, interest::Interest, token::Token, Authentication};

// Add other adapters here as they come
fn instantiate_test(lend_amount: Uint128, borrow_amount: Uint128) {
    let mut app = App::default();

    let lend_token_info = app.store_code(LendToken::default().contract());
    let lend_market_info = app.store_code(LendMarket::default().contract());
    let credit_agency_info = app.store_code(CreditAgency::default().contract());

    let viewing_key = "unguessable".to_string();
    let admin_user = Addr::unchecked("admin");
    let borrow_user = Addr::unchecked("borrow_user");
    let lending_user = Addr::unchecked("lending_user");

    let common_token = snip20::InstantiateMsg {
        name: "common_token".into(),
        admin: Some(admin_user.to_string().clone()),
        symbol: "CTKN".into(),
        decimals: 6,
        initial_balances: None,
        query_auth: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(false),
            enable_redeem: Some(false),
            enable_mint: Some(false),
            enable_burn: Some(false),
            enable_transfer: Some(true),
        }),
    }
    .test_init(
        Snip20::default(),
        &mut app,
        admin_user.clone(),
        "common_token",
        &[],
    )
    .unwrap();

    let market_token = snip20::InstantiateMsg {
        name: "market_token".into(),
        admin: Some(admin_user.to_string().clone()),
        symbol: "STKN".into(),
        decimals: 6,
        initial_balances: Some(vec![snip20::InitialBalance {
            amount: lend_amount,
            address: lending_user.to_string(),
        }]),
        query_auth: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(false),
            enable_redeem: Some(false),
            enable_mint: Some(false),
            enable_burn: Some(false),
            enable_transfer: Some(true),
        }),
    }
    .test_init(
        Snip20::default(),
        &mut app,
        admin_user.clone(),
        "market_token",
        &[],
    )
    .unwrap();

    // set user viewing keys
    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&common_token, &mut app, borrow_user.clone(), &[])
    .unwrap();

    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&common_token, &mut app, lending_user.clone(), &[])
    .unwrap();

    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&market_token, &mut app, borrow_user.clone(), &[])
    .unwrap();

    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&market_token, &mut app, lending_user.clone(), &[])
    .unwrap();

    let admin_contract = init_admin_auth(&mut app, &admin_user);

    let query_contract = query_auth::InstantiateMsg {
        admin_auth: admin_contract.clone().into(),
        prng_seed: to_binary("").ok().unwrap(),
    }
    .test_init(
        QueryAuth::default(),
        &mut app,
        admin_user.clone(),
        "query_auth",
        &[],
    )
    .unwrap();

    // set user VK in query auth
    query_auth::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&query_contract, &mut app, lending_user.clone(), &[])
    .unwrap();

    query_auth::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&query_contract, &mut app, borrow_user.clone(), &[])
    .unwrap();

    println!("AGENCY INIT");
    let credit_agency = credit_agency::InstantiateMsg {
        gov_contract: Contract {
            address: admin_user.clone(),
            code_hash: "".to_string(),
        },
        query_auth: query_contract.into(),
        lend_market_id: lend_market_info.code_id,
        lend_market_code_hash: lend_market_info.code_hash,
        market_viewing_key: viewing_key.clone(),
        ctoken_token_id: lend_token_info.code_id,
        ctoken_code_hash: lend_token_info.code_hash,
        reward_token: Token::new_cw20(ContractInfo {
            address: common_token.address.clone(),
            code_hash: common_token.code_hash.clone(),
        }),
        common_token: Token::new_cw20(ContractInfo {
            address: common_token.address.clone(),
            code_hash: common_token.code_hash.clone(),
        }),
        liquidation_price: Decimal::raw(0_92_000_000_00),
        liquidation_threshold: Decimal::raw(0_02_000_000_00),
        borrow_limit_ratio: Decimal::raw(0_01_000_000_00),
        default_estimate_multiplier: Decimal::one(),
    }
    .test_init(
        CreditAgency::default(),
        &mut app,
        admin_user.clone(),
        "basic_staking",
        &[],
    )
    .unwrap();

    credit_agency::ExecuteMsg::CreateMarket(credit_agency::MarketConfig {
        name: "Market 0".to_string(),
        symbol: "M0".to_string(),
        decimals: 6,
        market_token: Token::Cw20(ContractInfo {
            address: market_token.address.clone(),
            code_hash: market_token.code_hash.clone(),
        }),
        market_cap: None,
        interest_rate: Interest::Linear {
            base: Decimal::raw(0),
            slope: Decimal::raw(0),
        },
        interest_charge_period: 3600,
        collateral_ratio: Decimal::raw(0),
        price_oracle: Contract::default(),
        reserve_factor: Decimal::raw(0),
    })
    .test_exec(&credit_agency, &mut app, admin_user.clone(), &[]);

    panic!("END");
}

macro_rules! instantiate_test {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    lend_amount,
                    borrow_amount,
                    ) = $value;
                instantiate_test(lend_amount, borrow_amount)
            }
        )*
    }
}

instantiate_test! {
    instantiate_test_0: (
        Uint128::new(2),
        Uint128::new(1),
    ),
}
