use cosmwasm_std::{Decimal, Uint128};
use lending_utils::{credit_line::CreditLineValues, token::Token};
use wyndex::factory::PairType;

use super::suite::{SuiteBuilder, BORROWER, COMMON, LENDER, MARKET_TOKEN};
use crate::error::ContractError;
use lend_token::error::ContractError as TokenContractError;

#[test]
fn withdraw_native_works() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(100u128)])
        .with_market_token(market_token.clone())
        .build();

    // Create a cw20 token pool.
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(LENDER).unwrap();

    // seposit some tokens so we have something to withdraw
    suite.deposit(LENDER, market_token, 100u128).unwrap();

    // the lender should be able to withdraw 40 tokens
    suite.withdraw(LENDER, 40).unwrap();

    assert_eq!(
        suite
            .query_asset_balance(LENDER, MARKET_TOKEN.to_owned())
            .unwrap(),
        40,
        "expected the lender to have 40 tokens after withdrawal"
    );
    assert_eq!(
        suite.query_contract_asset_balance().unwrap(),
        60,
        "expected the contract to have 60 base assets after withdrawal"
    );
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(LENDER).unwrap()),
        60,
        "expected the lender to have 60 base assets after withdrawal"
    );
}

#[test]
fn withdraw_cw20_works() {
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
        PairType::Lsd {},
    );

    suite.set_high_credit_line(LENDER).unwrap();

    // deposit some tokens so we have something to withdraw
    suite
        .deposit(LENDER, market_token.clone(), 100u128)
        .unwrap();

    // lender should be able to withdraw 40 tokens
    suite.withdraw(LENDER, 40).unwrap();

    assert_eq!(
        suite
            .query_cw20_balance(LENDER, market_token.denom())
            .unwrap(),
        40,
        "expected the lender to have 40 tokens after withdrawal"
    );
    assert_eq!(
        suite.query_contract_asset_balance().unwrap(),
        60,
        "expected the contract to have 60 base assets after withdrawal"
    );
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(LENDER).unwrap()),
        60,
        "expected the lender to have 60 base assets after withdrawal"
    );
}

#[test]
fn withdraw_native_overflow_is_handled() {
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

    suite.set_high_credit_line(LENDER).unwrap();

    suite.deposit(LENDER, market_token, 100u128).unwrap();

    // try to withdraw more base asset than we have deposited - should fail and not
    // affect any balances
    let err = suite.withdraw(LENDER, 150).unwrap_err();
    assert_eq!(
        TokenContractError::InsufficientTokens {
            available: Uint128::new(10_000_000),
            needed: Uint128::new(15_000_000)
        },
        err.downcast().unwrap()
    );

    assert_eq!(
        suite
            .query_asset_balance(LENDER, MARKET_TOKEN.to_owned())
            .unwrap(),
        0,
        "expected the lender to have 0 tokens after error in withdrawal"
    );
    assert_eq!(
        suite.query_contract_asset_balance().unwrap(),
        100,
        "expected the contract to have 100 base assets after error in withdrawal"
    );
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(LENDER).unwrap()),
        100,
        "expected the lender to have 100 base assets after erorr in withdrawal"
    );
}

#[test]
fn withdraw_cw20_overflow_is_handled() {
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

    suite.set_high_credit_line(LENDER).unwrap();

    // deposit some tokens so we have something to withdraw
    suite.deposit(LENDER, market_token, 100u128).unwrap();

    // try to withdraw more base asset than we have deposited - should fail and not
    // affect any balances
    let err = suite.withdraw(LENDER, 150).unwrap_err();
    assert_eq!(
        TokenContractError::InsufficientTokens {
            available: Uint128::new(10_000_000),
            needed: Uint128::new(15_000_000)
        },
        err.downcast().unwrap()
    );

    assert_eq!(
        suite
            .query_asset_balance(LENDER, MARKET_TOKEN.to_owned())
            .unwrap(),
        0,
        "expected the lender to have 0 tokens after error in withdrawal"
    );
    assert_eq!(
        suite.query_contract_asset_balance().unwrap(),
        100,
        "expected the contract to have 100 base assets after error in withdrawal"
    );
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(LENDER).unwrap()),
        100,
        "expected the lender to have 100 base assets after erorr in withdrawal"
    );
}

#[test]
fn cant_withdraw_native_with_debt_higher_then_credit_line() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(100u128)])
        .with_collateral_ratio(Decimal::percent(70))
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(LENDER, market_token, 100u128).unwrap();

    // Set debt higher then credit line
    suite
        .set_credit_line(
            LENDER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(70),
                borrow_limit: Uint128::new(70),
                debt: Uint128::new(200),
            },
        )
        .unwrap();

    let err = suite.withdraw(LENDER, 1).unwrap_err();
    assert_eq!(
        ContractError::CannotWithdraw {
            amount: Uint128::new(1),
            account: LENDER.to_owned()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn cant_withdraw_cw20_with_debt_higher_then_credit_line() {
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
        PairType::Lsd {},
    );

    // deposit some tokens so we have something to withdraw
    suite.deposit(LENDER, market_token, 100u128).unwrap();

    // Set debt higher then credit line
    suite
        .set_credit_line(
            LENDER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(70),
                borrow_limit: Uint128::new(70),
                debt: Uint128::new(200),
            },
        )
        .unwrap();

    let err = suite.withdraw(LENDER, 1).unwrap_err();
    assert_eq!(
        ContractError::CannotWithdraw {
            amount: Uint128::new(1),
            account: LENDER.to_owned()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn can_withdraw_native_up_to_credit_line() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(100u128)])
        .with_collateral_ratio(Decimal::percent(70))
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(LENDER, market_token, 100u128).unwrap();

    // Set appropriate credit line and collateral
    suite
        .set_credit_line(
            LENDER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(70),
                borrow_limit: Uint128::new(70),
                debt: Uint128::zero(),
            },
        )
        .unwrap();

    // withdraw more then credit line is
    suite.withdraw(LENDER, 90).unwrap();
    assert_eq!(
        suite
            .query_asset_balance(LENDER, MARKET_TOKEN.to_owned())
            .unwrap(),
        90
    );

    // withdrawing another 20 dollars (10 over limit) will fail
    // adjust mock's response
    suite
        .set_credit_line(
            LENDER,
            CreditLineValues {
                collateral: Uint128::new(10),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(7),
                borrow_limit: Uint128::new(7),
                debt: Uint128::zero(),
            },
        )
        .unwrap();
    let err = suite.withdraw(LENDER, 2_000_000).unwrap_err();
    assert_eq!(
        ContractError::CannotWithdraw {
            amount: Uint128::new(2_000_000),
            account: LENDER.to_owned()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn can_withdraw_cw20_up_to_credit_line() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 100))
        .with_collateral_ratio(Decimal::percent(70))
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

    suite
        .deposit(LENDER, market_token.clone(), 100u128)
        .unwrap();

    // Set appropriate credit line and collateral
    suite
        .set_credit_line(
            LENDER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(70),
                borrow_limit: Uint128::new(70),
                debt: Uint128::zero(),
            },
        )
        .unwrap();

    // withdraw more then credit line is
    suite.withdraw(LENDER, 90).unwrap();
    assert_eq!(
        suite
            .query_cw20_balance(LENDER, market_token.denom())
            .unwrap(),
        90
    );

    // withdrawing another 20 dollars (10 over limit) will fail
    // adjust mock's response
    suite
        .set_credit_line(
            LENDER,
            CreditLineValues {
                collateral: Uint128::new(10),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(7),
                borrow_limit: Uint128::new(7),
                debt: Uint128::zero(),
            },
        )
        .unwrap();
    let err = suite.withdraw(LENDER, 2_000_000).unwrap_err();
    assert_eq!(
        ContractError::CannotWithdraw {
            amount: Uint128::new(2_000_000),
            account: LENDER.to_owned()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn query_native_withdrawable_when_only_lending() {
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

    suite.set_high_credit_line(LENDER).unwrap();

    suite.deposit(LENDER, market_token, 100u128).unwrap();

    suite.assert_withdrawable(LENDER, 100u128);

    suite.attempt_withdraw_max(LENDER).unwrap();
}

#[test]
fn query_cw20_withdrawable_when_only_lending() {
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

    suite.set_high_credit_line(LENDER).unwrap();

    // deposit some tokens so we have something to withdraw
    suite.deposit(LENDER, market_token, 100u128).unwrap();

    suite.assert_withdrawable(LENDER, 100u128);

    suite.attempt_withdraw_max(LENDER).unwrap();
}

#[test]
fn query_native_withdrawable_up_to_borrow_limit() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(100u128)])
        .with_collateral_ratio(Decimal::percent(50))
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(LENDER, market_token, 100u128).unwrap();
    suite
        .set_credit_line(
            LENDER,
            CreditLineValues {
                collateral: Uint128::new(100),
                credit_line: Uint128::new(50),
                borrow_limit: Uint128::new(50),
                debt: Uint128::new(40),
            },
        )
        .unwrap();

    suite.assert_withdrawable(LENDER, 20u128);

    suite.attempt_withdraw_max(LENDER).unwrap();
}

#[test]
fn query_cw20_withdrawable_up_to_borrow_limit() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 100))
        .with_collateral_ratio(Decimal::percent(50))
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

    suite.deposit(LENDER, market_token, 100u128).unwrap();

    suite
        .set_credit_line(
            LENDER,
            CreditLineValues {
                collateral: Uint128::new(100),
                credit_line: Uint128::new(50),
                borrow_limit: Uint128::new(50),
                debt: Uint128::new(40),
            },
        )
        .unwrap();

    suite.assert_withdrawable(LENDER, 20u128);

    suite.attempt_withdraw_max(LENDER).unwrap();
}

#[test]
fn query_native_withdrawable_not_enough_liquid() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(100u128)])
        .with_collateral_ratio(Decimal::percent(50))
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();
    suite.set_high_credit_line(LENDER).unwrap();

    suite.deposit(LENDER, market_token, 100u128).unwrap();
    suite.borrow(BORROWER, 40).unwrap();

    // Technically, the lender is allowed to withdraw the whole 100 tokens, but
    // the contract doesn't have enough liquidity to cover that!
    suite.assert_withdrawable(LENDER, 60u128);
    suite.attempt_withdraw_max(LENDER).unwrap();
}

#[test]
fn query_cw20_withdrawable_not_enough_liquid() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 100))
        .with_collateral_ratio(Decimal::percent(50))
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
    suite.set_high_credit_line(LENDER).unwrap();

    suite.deposit(LENDER, market_token, 100u128).unwrap();
    suite.borrow(BORROWER, 40).unwrap();

    // Technically, the lender is allowed to withdraw the whole 100 tokens, but
    // the contract doesn't have enough liquidity to cover that!
    suite.assert_withdrawable(LENDER, 60u128);
    suite.attempt_withdraw_max(LENDER).unwrap();
}
