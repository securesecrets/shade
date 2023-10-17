use cosmwasm_std::{Decimal, Uint128};
use utils::{credit_line::CreditLineValues, token::Token};
use wyndex::factory::PairType;

use super::suite::{SuiteBuilder, COMMON, USER};
use crate::{
    error::ContractError,
    multitest::suite::{BORROWER, LENDER, MARKET_TOKEN, OWNER},
};

// Each logic testeted in this file is composed of two tests, one for native coins and one for
// cw20 tokens.

#[test]
fn borrow_native_works() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_contract_funds(market_token.clone().into_coin(150u128))
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    // At first, the borrower has no debt, and the contract has some base assets
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 150);
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 0);

    // Borrow some tokens
    suite.borrow(BORROWER, 100).unwrap();

    assert_eq!(suite.query_contract_asset_balance().unwrap(), 50);
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 100);
}

#[test]
fn borrow_cw20_works_lsd() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 100))
        .with_market_token(market_token.clone())
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    // Create a cw20 token pool.
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Lsd {},
    );

    suite
        .mint_cw20_to_market(market_token.denom(), 150u128)
        .unwrap();
    suite.set_high_credit_line(BORROWER).unwrap();

    // At first, the borrower has no debt, and the contract has some base assets
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 150);
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 0);

    // Borrow some tokens
    suite.borrow(BORROWER, 100).unwrap();

    assert_eq!(suite.query_contract_asset_balance().unwrap(), 50);
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 100);
}

#[test]
fn borrow_native_and_repay() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_contract_funds(market_token.clone().into_coin(150u128))
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 100).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 50);

    // Repay all borrowed tokens
    suite
        .repay(BORROWER, market_token.into_coin(100u128))
        .unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 150);
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 0);
}

#[test]
fn borrow_cw20_and_repay() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 100))
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
        PairType::Xyk {},
    );

    suite
        .mint_cw20_to_market(market_token.denom(), 150u128)
        .unwrap();

    suite.set_high_credit_line(BORROWER).unwrap();
    // Borrow some tokens
    suite.borrow(BORROWER, 100).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 50);

    // Repay all borrowed tokens
    suite
        .repay(BORROWER, market_token.into_coin(100u128))
        .unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 150);
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 0);
}

#[test]
fn cant_borrow_native_with_debt_higher_then_credit_line() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(BORROWER, &[market_token.clone().into_coin(100u128)])
        .with_market_token(market_token.clone())
        .with_collateral_ratio(Decimal::percent(70))
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(BORROWER, market_token, 100u128).unwrap();

    // Set debt higher then credit line
    suite
        .set_credit_line(
            BORROWER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(70),
                borrow_limit: Uint128::new(70),
                debt: Uint128::new(200),
            },
        )
        .unwrap();

    let err = suite.borrow(BORROWER, 1).unwrap_err();
    assert_eq!(
        ContractError::CannotBorrow {
            amount: Uint128::new(1),
            account: BORROWER.to_owned()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn cant_borrow_cw20_with_debt_higher_then_credit_line() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (BORROWER, 100))
        .with_market_token(market_token.clone())
        .with_collateral_ratio(Decimal::percent(70))
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    // create cw20 tokens pools
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(BORROWER, market_token, 100u128).unwrap();

    // Set debt higher then credit line
    suite
        .set_credit_line(
            BORROWER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(70),
                borrow_limit: Uint128::new(70),
                debt: Uint128::new(200),
            },
        )
        .unwrap();

    let err = suite.borrow(BORROWER, 1).unwrap_err();
    assert_eq!(
        ContractError::CannotBorrow {
            amount: Uint128::new(1),
            account: BORROWER.to_owned()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn cant_borrow_native_more_then_credit_line() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(BORROWER, &[market_token.clone().into_coin(100u128)])
        .with_market_token(market_token.clone())
        .with_collateral_ratio(Decimal::percent(70))
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(BORROWER, market_token, 100u128).unwrap();

    // Set appropriate collateral and credit line without debt
    suite
        .set_credit_line(
            BORROWER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(70),
                borrow_limit: Uint128::new(70),
                debt: Uint128::zero(),
            },
        )
        .unwrap();

    let err = suite.borrow(BORROWER, 80).unwrap_err();
    assert_eq!(
        ContractError::CannotBorrow {
            amount: Uint128::new(80),
            account: BORROWER.to_owned()
        },
        err.downcast().unwrap()
    );

    // Borrowing smaller amount then credit line is fine
    suite.borrow(BORROWER, 60).unwrap();
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 60);
}

#[test]
fn cant_borrow_cw20_more_then_credit_line() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (BORROWER, 100))
        .with_market_token(market_token.clone())
        .with_collateral_ratio(Decimal::percent(70))
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
        PairType::Xyk {},
    );

    suite.deposit(BORROWER, market_token, 100u128).unwrap();

    // Set appropriate collateral and credit line without debt
    suite
        .set_credit_line(
            BORROWER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(70),
                borrow_limit: Uint128::new(70),
                debt: Uint128::zero(),
            },
        )
        .unwrap();

    let err = suite.borrow(BORROWER, 80).unwrap_err();
    assert_eq!(
        ContractError::CannotBorrow {
            amount: Uint128::new(80),
            account: BORROWER.to_owned()
        },
        err.downcast().unwrap()
    );

    // Borrowing smaller amount then credit line is fine
    suite.borrow(BORROWER, 60).unwrap();
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 60);
}

#[test]
fn cant_borrow_native_more_than_borrow_limit() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(BORROWER, &[market_token.clone().into_coin(100u128)])
        .with_market_token(market_token.clone())
        .with_collateral_ratio(Decimal::percent(70))
        .with_borrow_limit_ratio(Decimal::percent(90))
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(BORROWER, market_token, 100u128).unwrap();

    // Set appropriate collateral and credit line without debt
    suite
        .set_credit_line(
            BORROWER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(70),
                // 70 * 0.9 borrow limit ratio
                borrow_limit: Uint128::new(63),
                debt: Uint128::zero(),
            },
        )
        .unwrap();

    let err = suite.borrow(BORROWER, 70).unwrap_err();
    assert_eq!(
        ContractError::CannotBorrow {
            amount: Uint128::new(70),
            account: BORROWER.to_owned()
        },
        err.downcast().unwrap()
    );

    // Borrowing smaller amount then borrow limit is fine
    suite.assert_borrowable(BORROWER, 63u128);
    suite.borrow(BORROWER, 63).unwrap();
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 63);
}

#[test]
fn cant_borrow_cw20_more_than_borrow_limit() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (BORROWER, 100))
        .with_market_token(market_token.clone())
        .with_collateral_ratio(Decimal::percent(70))
        .with_borrow_limit_ratio(Decimal::percent(90))
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
        PairType::Xyk {},
    );

    suite.deposit(BORROWER, market_token, 100u128).unwrap();

    // Set appropriate collateral and credit line without debt
    suite
        .set_credit_line(
            BORROWER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(70),
                // 70 * 0.9 borrow limit ratio
                borrow_limit: Uint128::new(63),
                debt: Uint128::zero(),
            },
        )
        .unwrap();

    let err = suite.borrow(BORROWER, 70).unwrap_err();
    assert_eq!(
        ContractError::CannotBorrow {
            amount: Uint128::new(70),
            account: BORROWER.to_owned()
        },
        err.downcast().unwrap()
    );

    // Borrowing smaller amount then borrow limit is fine
    suite.assert_borrowable(BORROWER, 63u128);
    suite.borrow(BORROWER, 63).unwrap();
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 63);
}

#[test]
fn repay_small_native_amounts() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_contract_funds(market_token.clone().into_coin(100u128))
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 100).unwrap();

    // Repay some borrowed tokens
    suite
        .repay(BORROWER, market_token.clone().into_coin(33u128))
        .unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 33);
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 67);
    suite
        .repay(BORROWER, market_token.into_coin(67u128))
        .unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 0);
}

#[test]
fn repay_small_cw20_amounts() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        // We need a user with initial tokens to create the cw20 contract
        .with_initial_cw20(market_token.denom(), (OWNER, 100))
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
        PairType::Xyk {},
    );

    suite
        .mint_cw20_to_market(market_token.denom(), 100u128)
        .unwrap();

    suite.set_high_credit_line(BORROWER).unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 100).unwrap();

    // Repay some borrowed tokens
    suite
        .repay(BORROWER, market_token.clone().into_coin(33u128))
        .unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 33);
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 67);
    suite
        .repay(BORROWER, market_token.into_coin(67u128))
        .unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 0);
}

#[test]
fn overpay_repay_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(BORROWER, &[market_token.clone().into_coin(50u128)])
        .with_contract_funds(market_token.clone().into_coin(100u128))
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 100).unwrap();

    // Overpay borrowed tokens - 120 instead of 100
    suite
        .repay(BORROWER, market_token.clone().into_coin(120u128))
        .unwrap();

    // Contract will still have only initial 100 tokens, since it sends
    // surplus back to borrower
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);
    // No more debt
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 0);
    // Initial amount - surplus was returned
    assert_eq!(
        suite
            .query_asset_balance(BORROWER, market_token.denom())
            .unwrap(),
        50
    );
}

#[test]
fn overpay_repay_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (BORROWER, 50))
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
        PairType::Xyk {},
    );

    suite
        .mint_cw20_to_market(market_token.denom(), 100u128)
        .unwrap();

    suite.set_high_credit_line(BORROWER).unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 100).unwrap();

    // Overpay borrowed tokens - 120 instead of 100
    suite
        .repay(BORROWER, market_token.clone().into_coin(120u128))
        .unwrap();

    // Contract will still have only initial 100 tokens, since it sends
    // surplus back to borrower
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);
    // No more debt
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 0);
    // Initial amount - surplus was returned
    assert_eq!(
        suite
            .query_cw20_balance(BORROWER, market_token.denom())
            .unwrap(),
        50
    );
}

#[test]
fn repay_to_no_agency() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(100u128)])
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    // Only credit agency can send RepayTo message.
    let err = suite
        .repay_to(LENDER, USER, market_token.into_coin(1u128))
        .unwrap_err();
    assert_eq!(
        ContractError::RequiresCreditAgency {},
        err.downcast().unwrap()
    );
}

#[test]
fn repay_to_invalid_denom() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_agency_funds(common_token.clone().into_coin(100u128))
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.clone().into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    // Only credit agency can send RepayTo message.
    let err = suite
        .repay_to(&suite.credit_agency(), USER, common_token.into_coin(1u128))
        .unwrap_err();
    assert_eq!(
        ContractError::InvalidDenom(market_token.denom()),
        err.downcast().unwrap()
    );
}

#[test]
fn repay_to_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let market_coin = market_token.clone().into_coin(100u128);
    let mut suite = SuiteBuilder::new()
        .with_agency_funds(market_coin.clone())
        .with_contract_funds(market_token.clone().into_coin(150u128))
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 100).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 50);

    // Repay all borrowed tokens
    suite
        .repay_to(&suite.credit_agency(), BORROWER, market_coin)
        .unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 150);
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 0);
}

#[test]
fn repay_to_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        // we need at least one initial balance to create a cw20
        .with_initial_cw20(market_token.denom(), (OWNER, 1))
        .with_market_token(market_token.clone())
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    // mint cw20 tokens to agency so it can repay to debt
    suite
        .mint_cw20_to_agency(market_token.denom(), 100u128)
        .unwrap();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    // mint cw20 tokens for the market so we don't have to deposit from another address
    suite
        .mint_cw20_to_market(market_token.denom(), 150u128)
        .unwrap();

    suite.set_high_credit_line(BORROWER).unwrap();

    // borrow some tokens
    suite.borrow(BORROWER, 100).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 50);

    // Repay all borrowed tokens
    suite
        .repay_to(
            &suite.credit_agency(),
            BORROWER,
            market_token.into_coin(100u128),
        )
        .unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 150);
    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 0);
}

#[test]
fn repay_to_amount_higher_than_debt_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_agency_funds(market_token.clone().into_coin(150u128))
        .with_contract_funds(market_token.clone().into_coin(150u128))
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 100).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 50);

    let err = suite
        .repay_to(
            &suite.credit_agency(),
            BORROWER,
            market_token.into_coin(150u128),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::LiquidationInsufficientDebt {
            account: BORROWER.to_owned(),
            debt: Uint128::from(100u128)
        },
        err.downcast().unwrap(),
        "expected to fail since repay tokens amount higher than debt"
    );
}

#[test]
fn repay_to_amount_higher_than_debt_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        // we need at least one initial balance to create a cw20
        .with_initial_cw20(market_token.denom(), (OWNER, 1))
        .with_market_token(market_token.clone())
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    // mint cw20 tokens to agency so it can repay to debt
    suite
        .mint_cw20_to_agency(market_token.denom(), 150u128)
        .unwrap();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    // mint cw20 tokens for the market so we don't have to deposit from another address
    suite
        .mint_cw20_to_market(market_token.denom(), 150u128)
        .unwrap();

    suite.set_high_credit_line(BORROWER).unwrap();

    // borrow some tokens
    suite.borrow(BORROWER, 100).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 50);

    // Repay all borrowed tokens
    let err = suite
        .repay_to(
            &suite.credit_agency(),
            BORROWER,
            market_token.into_coin(150u128),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::LiquidationInsufficientDebt {
            account: BORROWER.to_owned(),
            debt: Uint128::from(100u128)
        },
        err.downcast().unwrap(),
        "expected to fail since repay tokens amount higher than debt"
    );
}

#[test]
fn query_borrowable_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(100u128)])
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    // Set arbitrary market/common exchange ratio and credit line (not part of this test)
    suite.set_high_credit_line(LENDER).unwrap();
    suite
        .set_credit_line(
            BORROWER,
            CreditLineValues {
                collateral: Uint128::new(100),
                credit_line: Uint128::new(50),
                borrow_limit: Uint128::new(50),
                debt: Uint128::new(10),
            },
        )
        .unwrap();

    // Deposit some tokens so we have something to borrow.
    suite.deposit(LENDER, market_token, 100u128).unwrap();

    // Only 40 tokens can be borrowed due to credit health
    // (credit_line - debt)
    suite.assert_borrowable(BORROWER, 40u128);
    suite.attempt_borrow_max(BORROWER).unwrap();
}

#[test]
fn query_borrowable_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 100))
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
        PairType::Xyk {},
    );

    // Set arbitrary market/common exchange ratio and credit line (not part of this test)
    suite.set_high_credit_line(LENDER).unwrap();
    suite
        .set_credit_line(
            BORROWER,
            CreditLineValues {
                collateral: Uint128::new(100),
                credit_line: Uint128::new(50),
                borrow_limit: Uint128::new(50),
                debt: Uint128::new(10),
            },
        )
        .unwrap();

    // Deposit some tokens so we have something to borrow.
    suite.deposit(LENDER, market_token, 100u128).unwrap();

    // Only 40 tokens can be borrowed due to credit health
    // (credit_line - debt)
    suite.assert_borrowable(BORROWER, 40u128);
    suite.attempt_borrow_max(BORROWER).unwrap();
}

#[test]
fn query_borrowable_native_with_limited_liquidity() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(20u128)])
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    // Set arbitrary market/common exchange ratio and credit line (not part of this test)
    suite.set_high_credit_line(LENDER).unwrap();
    suite.set_high_credit_line(BORROWER).unwrap();

    // Deposit some tokens so we have something to borrow.
    suite.deposit(LENDER, market_token, 20u128).unwrap();

    // Borrower has a high credit line, but there's only 20 tokens liquid
    // in the market.
    suite.assert_borrowable(BORROWER, 20u128);
    suite.attempt_borrow_max(BORROWER).unwrap();
}

#[test]
fn query_borrowable_cw20_with_limited_liquidity() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 20))
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
        PairType::Xyk {},
    );

    // Set arbitrary market/common exchange ratio and credit line (not part of this test)
    suite.set_high_credit_line(LENDER).unwrap();
    suite.set_high_credit_line(BORROWER).unwrap();

    // Deposit some tokens so we have something to borrow.
    suite.deposit(LENDER, market_token, 20u128).unwrap();

    // Borrower has a high credit line, but there's only 20 tokens liquid
    // in the market.
    suite.assert_borrowable(BORROWER, 20u128);
    suite.attempt_borrow_max(BORROWER).unwrap();
}
