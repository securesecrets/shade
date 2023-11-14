use cosmwasm_std::Decimal;
use lending_utils::token::Token;
use wyndex::factory::PairType;

use crate::multitest::suite::{BORROWER, LENDER, MARKET_TOKEN};

use super::suite::{SuiteBuilder, COMMON};

#[test]
fn nothing_on_market_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_market_token(market_token.clone())
        .build();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market tokens will get
    // debt of 2000 common tokens
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        "0.030454529542178457".parse::<Decimal>().unwrap()
    );
    assert_eq!(apy.lender, Decimal::zero());
}

#[test]
fn nothing_on_market_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 1))
        .with_market_token(market_token.clone())
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market tokens will get
    // debt of 2000 common tokens
    // create cw20 tokens pools
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        "0.030454529542178457".parse::<Decimal>().unwrap()
    );
    assert_eq!(apy.lender, Decimal::zero());
}

#[test]
fn nothing_borrowed_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(1_000u128)])
        .with_market_token(market_token.clone())
        .build();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market tokens will get
    // debt of 2000 common tokens
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        "0.030454529542178457".parse::<Decimal>().unwrap()
    );
    assert_eq!(apy.lender, Decimal::zero());
}

#[test]
fn nothing_borrowed_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 1_000))
        .with_market_token(market_token.clone())
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market tokens will get
    // debt of 2000 common tokens
    // create cw20 tokens pools
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        "0.030454529542178457".parse::<Decimal>().unwrap()
    );
    assert_eq!(apy.lender, Decimal::zero());
}

#[test]
fn half_borrowed_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(1_000u128)])
        .with_market_token(market_token.clone())
        .build();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market tokens will get
    // debt of 2000 common tokens
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    suite.borrow(BORROWER, 500).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        "0.138828291780615352".parse::<Decimal>().unwrap()
    );
    assert_eq!(
        apy.lender,
        "0.069414145890307676".parse::<Decimal>().unwrap()
    );
}

#[test]
fn half_borrowed_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 1_000))
        .with_market_token(market_token.clone())
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market tokens will get
    // debt of 2000 common tokens
    // create cw20 tokens pools
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    suite.borrow(BORROWER, 500).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        "0.138828291780615352".parse::<Decimal>().unwrap()
    );
    assert_eq!(
        apy.lender,
        "0.069414145890307676".parse::<Decimal>().unwrap()
    );
}

#[test]
fn whole_borrowed_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(1_000u128)])
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    suite.borrow(BORROWER, 1_000).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        "0.258599693244403384".parse::<Decimal>().unwrap()
    );
    assert_eq!(
        apy.lender,
        "0.258599693244403384".parse::<Decimal>().unwrap()
    );
}

#[test]
fn whole_borrowed_cw20_xyk() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 1_000))
        .with_market_token(market_token.clone())
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market tokens will get
    // debt of 2000 common tokens
    // create cw20 tokens pools
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    suite.borrow(BORROWER, 1_000).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        "0.258599693244403384".parse::<Decimal>().unwrap()
    );
    assert_eq!(
        apy.lender,
        "0.258599693244403384".parse::<Decimal>().unwrap()
    );
}

#[test]
fn whole_borrowed_cw20_lsd() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 1_000))
        .with_market_token(market_token.clone())
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Lsd {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    suite.borrow(BORROWER, 1_000).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        "0.258599693244403384".parse::<Decimal>().unwrap()
    );
    assert_eq!(
        apy.lender,
        "0.258599693244403384".parse::<Decimal>().unwrap()
    );
}

#[test]
fn with_reserve_factor_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(1_000u128)])
        .with_market_token(market_token.clone())
        .with_reserve_factor(20)
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Lsd {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    suite.borrow(BORROWER, 500).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        "0.138828291780615352".parse::<Decimal>().unwrap()
    );
    assert_eq!(
        apy.lender,
        "0.05553131671224614".parse::<Decimal>().unwrap()
    );
}

#[test]
fn with_reserve_factor_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 1_000))
        .with_reserve_factor(20)
        .with_market_token(market_token.clone())
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Lsd {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    suite.borrow(BORROWER, 500).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        "0.138828291780615352".parse::<Decimal>().unwrap()
    );
    assert_eq!(
        apy.lender,
        "0.05553131671224614".parse::<Decimal>().unwrap()
    );
}
