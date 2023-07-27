use shade_protocol::c_std::{
    coins,
    from_binary,
    to_binary,
    Binary,
    Env,
    DepsMut,
    Addr,
    Response,
    StdError,
    StdResult,
};

use shade_protocol::c_std::Uint128;
use shade_protocol::{
    contract_interfaces::{
        snip20,
        mint::liability_mint,
    },
    utils::{
        MultiTestable,
        InstantiateCallback,
        ExecuteCallback,
        Query,
        asset::Contract,
        price::{normalize_price, translate_price},
    },
};

use shade_protocol::multi_test::{ App };

use shade_multi_test::multi::{
    snip20::Snip20,
    liability_mint::LiabilityMint,
};

fn test_liabilities(
    mint_amount: Uint128,
    limit: Uint128,
    payback: Uint128,
    expected_balance: Uint128,
    expected_liabilities: Uint128,
) {
    let mut app = App::default();

    let admin = Addr::unchecked("admin");
    let viewing_key = "viewing_key".to_string();

    let token = snip20::InstantiateMsg {
        name: "token".into(),
        admin: Some(admin.clone().into()),
        symbol: "TKN".into(),
        decimals: 6,
        initial_balances: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: None,
            enable_redeem: None,
            enable_mint: Some(true),
            enable_burn: Some(true),
            enable_transfer: Some(true),
        }),
        query_auth: None,
    }.test_init(Snip20::default(), &mut app, admin.clone(), "token", &[]).unwrap();

    let liab_mint = liability_mint::InstantiateMsg {
        admin: Some(admin.clone()),
        token: Contract {
            address: token.address.clone(),
            code_hash: token.code_hash.clone(),
        },
        limit,
    }.test_init(LiabilityMint::default(), &mut app, admin.clone(), "liability_mint", &[]).unwrap();

    // Setup liability minting
    &snip20::ExecuteMsg::AddMinters {
        minters: vec![liab_mint.address.to_string().clone()],
        padding: None,
    }.test_exec(&token, &mut app, admin.clone(), &[]).unwrap();

    // add user to whitelist
    &liability_mint::ExecuteMsg::AddWhitelist {
        address: admin.clone(),
    }.test_exec(&liab_mint, &mut app, admin.clone(), &[]).unwrap();

    // Mint funds
    &liability_mint::ExecuteMsg::Mint {
        amount: mint_amount,
    }.test_exec(&liab_mint, &mut app, admin.clone(), &[]).unwrap();

    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }.test_exec(&token, &mut app, admin.clone(), &[]).unwrap();

    // Check total supply
    match (snip20::QueryMsg::TokenInfo { }).test_query(&token, &app).unwrap() {
        snip20::QueryAnswer::TokenInfo { name, symbol, decimals, total_supply } => {
            assert_eq!(total_supply.unwrap(), mint_amount, "total supply {} less than mint amount {}", total_supply.unwrap(), mint_amount);
        },
        _ => { panic!("Query failed"); },
    }
    // Check user balance
    match (snip20::QueryMsg::Balance {
        address: admin.to_string().clone(),
        key: viewing_key.clone(),
    }).test_query(&token, &app).unwrap() {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, mint_amount, "amount minted")
        },
        _ => { panic!("Query failed"); },
    }

    // Check liabilities
    match (liability_mint::QueryMsg::Liabilities {
    }).test_query(&liab_mint, &app).unwrap() {
        liability_mint::QueryAnswer::Liabilities { outstanding, limit } => {
            assert_eq!(outstanding, mint_amount, "liabilities before payback")
        },
        _ => { panic!("Query failed"); },
    }

    // Payback
    snip20::ExecuteMsg::Send {
        recipient: liab_mint.address.to_string().clone(),
        recipient_code_hash: None,
        amount: payback,
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&token, &mut app, admin.clone(), &[]).unwrap();
    
    // Check total supply
    match (snip20::QueryMsg::TokenInfo { }).test_query(&token, &app).unwrap() {
        snip20::QueryAnswer::TokenInfo { name, symbol, decimals, total_supply } => {
            assert_eq!(total_supply.unwrap(), mint_amount - payback, "total supply {} should be mint amount - payback {}", total_supply.unwrap(), mint_amount - payback);
        },
        _ => { panic!("Query failed"); },
    }
    // Check user balance
    match (snip20::QueryMsg::Balance {
        address: admin.to_string().clone(),
        key: viewing_key.clone(),
    }).test_query(&token, &app).unwrap() {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, mint_amount - payback, "user balance after payback")
        },
        _ => { panic!("Query failed"); },
    }

    // Check liabilities
    match (liability_mint::QueryMsg::Liabilities {
    }).test_query(&liab_mint, &app).unwrap() {
        liability_mint::QueryAnswer::Liabilities { outstanding, limit } => {
            assert_eq!(outstanding, mint_amount - payback, "liabilities after payback")
        },
        _ => { panic!("Query failed"); },
    }
}

macro_rules! liability_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (mint_amount, limit, payback, expected_balance, expected_liabilities) = $value;
                test_liabilities(mint_amount, limit, payback, expected_balance, expected_liabilities);
            }
        )*
    }
}
liability_tests! {
    liability_half_payback: (
        Uint128::new(1_000_000), // mint amount
        Uint128::new(1_000_000), // limit
        Uint128::new(  500_000), // payback
        Uint128::new(  500_000), // end balance
        Uint128::new(  500_000), // end liabilities
    ),
    liability_full_payback: (
        Uint128::new(1_000_000), // mint amount
        Uint128::new(1_000_000), // limit
        Uint128::new(1_000_000), // payback
        Uint128::new(0), // end balance
        Uint128::new(0), // end liabilities
    ),
    liability_no_payback: (
        Uint128::new(1_000_000), // mint amount
        Uint128::new(1_000_000), // limit
        Uint128::new(0), // payback
        Uint128::new(1_000_000), // end balance
        Uint128::new(1_000_000), // end liabilities
    ),
}
