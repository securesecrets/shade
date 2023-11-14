use crate::{
    multitest::suite::{BORROWER, LENDER, MARKET_TOKEN},
    state::SECONDS_IN_YEAR,
};

use super::suite::{SuiteBuilder, COMMON};

use cosmwasm_std::{Decimal, StdError, Uint128};
use lending_utils::{credit_line::CreditLineValues, token::Token};
use wyndex::factory::PairType;

#[test]
fn oracle_price_native_not_set() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(1_000u128)])
        .with_market_token(market_token.clone())
        .build();

    suite
        .deposit(LENDER, market_token.clone(), 1_000u128)
        .unwrap();

    let err = suite.query_credit_line(LENDER).unwrap_err();
    assert_eq!(
        StdError::generic_err(format!(
            "Querier contract error: Generic error: \
            Querier contract error: Generic error: \
            There is no info about the contract address of pair {} and {}",
            market_token.denom(),
            common_token.denom()
        )),
        err.downcast().unwrap(),
        "expect to fail since the pool does not exists"
    );
}

#[test]
fn oracle_price_cw20_not_set() {
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

    suite
        .deposit(LENDER, market_token.clone(), 1_000u128)
        .unwrap();

    let err = suite.query_credit_line(LENDER).unwrap_err();
    assert_eq!(
        StdError::generic_err(format!(
            "Querier contract error: Generic error: \
            Querier contract error: Generic error: \
            There is no info about the contract address of pair {} and {}",
            market_token.denom(),
            common_token.denom()
        )),
        err.downcast().unwrap(),
        "expect to fail since the pool does not exists"
    );
}

#[test]
fn zero_credit_line_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let market_coin = market_token.clone().into_coin(100u128);
    let common_coin = common_token.into_coin(100u128);

    let suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(1u128)])
        .with_market_token(market_token)
        .with_pool(1, (common_coin, market_coin))
        .build();

    // No tokens were deposited nor borrowed, so credit line is zero
    let credit_line = suite.query_credit_line(LENDER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues::zero().make_response(suite.common_token())
    );
}

#[test]
fn zero_credit_line_cw20() {
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

    let market_coin = market_token.into_coin(100u128);
    let common_coin = common_token.into_coin(100u128);

    // create cw20 tokens pools
    suite
        .set_pool(&[(1, (common_coin, market_coin))], PairType::Lsd {})
        .unwrap();

    // No tokens were deposited nor borrowed, so credit line is zero
    let credit_line = suite.query_credit_line(LENDER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues::zero().make_response(suite.common_token())
    );
}

#[test]
fn borrower_borrows_native_tokens() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(1_000u128)])
        .with_market_token(market_token.clone())
        // collateral ratio is 0.7
        .with_collateral_ratio(Decimal::percent(70))
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

    // Lender deposits coins
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();
    // Now borrower borrows it
    suite.borrow(BORROWER, 1_000).unwrap();

    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 1_000);

    let credit_line = suite.query_credit_line(BORROWER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            collateral: Uint128::zero(),
            credit_line: Uint128::zero(),
            borrow_limit: Uint128::zero(),
            // 1000 borrowed * 2.0 oracle's price
            debt: Uint128::new(2000),
        }
        .make_response(suite.common_token()),
        "expect to have debt equal to deposit times relative price"
    );
}

#[test]
fn borrower_borrows_cw20_tokens_xyk() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 1_000))
        .with_market_token(market_token.clone())
        .with_collateral_ratio(Decimal::percent(70))
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    // Create an Lsd pool
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    // Lender deposits coins
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();
    // Now borrower borrows it
    suite.borrow(BORROWER, 1000).unwrap();

    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 1000);

    let credit_line = suite.query_credit_line(BORROWER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            collateral: Uint128::zero(),
            credit_line: Uint128::zero(),
            borrow_limit: Uint128::zero(),
            // 1000 borrowed * 2.0 oracle's price
            debt: Uint128::new(2000),
        }
        .make_response(suite.common_token()),
        "expect to have debt equal to deposit times relative price"
    );
}

#[test]
fn borrower_borrows_cw20_tokens_lsd() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 1_000))
        .with_market_token(market_token.clone())
        .with_collateral_ratio(Decimal::percent(70))
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    // Create an Lsd pool. Since it is a Stableswap it have to be balanced.
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Lsd {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    // Lender deposits coins
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();
    // Now borrower borrows it
    suite.borrow(BORROWER, 1000).unwrap();

    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 1000);

    let credit_line = suite.query_credit_line(BORROWER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            collateral: Uint128::zero(),
            credit_line: Uint128::zero(),
            borrow_limit: Uint128::zero(),
            // 1000 borrowed * 1.0 oracle's price
            debt: Uint128::new(1000),
        }
        .make_response(suite.common_token()),
        "expect to have debt equal to deposit times relative price"
    );
}

#[test]
fn lender_deposits_native_tokens() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(1_000u128)])
        .with_market_token(market_token.clone())
        // collateral ratio is 0.7
        .with_collateral_ratio(Decimal::percent(70))
        .build();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market tokens will get
    // debt of 2000 common tokens
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    // Deposit some tokens
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    // After the deposit, the lender has 1000 c-token
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(LENDER).unwrap()),
        1_000
    );

    let credit_line = suite.query_credit_line(LENDER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1000 collateral * 2.0 oracle's price
            collateral: Uint128::new(2000),
            // 1000 collateral * 2.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(1400),
            borrow_limit: Uint128::new(1400),
            // no debt
            debt: Uint128::zero(),
        }
        .make_response(suite.common_token())
    );
}

#[test]
fn lender_deposits_cw20_tokens() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 1_000))
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
        common_token.into_coin(2_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    // Deposit some tokens
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    // After the deposit, the lender has 1000 c-token
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(LENDER).unwrap()),
        1000
    );

    let credit_line = suite.query_credit_line(LENDER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1000 collateral * 2.0 oracle's price
            collateral: Uint128::new(2000),
            // 1000 collateral * 2.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(1400),
            borrow_limit: Uint128::new(1400),
            // no debt
            debt: Uint128::zero(),
        }
        .make_response(suite.common_token())
    );
}

#[test]
fn deposits_and_borrows_native_tokens() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(1_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(1_000u128)])
        .with_market_token(market_token.clone())
        // collateral ratio is 0.7
        .with_collateral_ratio(Decimal::percent(70))
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
    suite.set_high_credit_line(BORROWER).unwrap();

    // Lender deposits coins
    suite
        .deposit(LENDER, market_token.clone(), 1_000u128)
        .unwrap();

    // Now borrower borrows it
    suite.borrow(BORROWER, 1000).unwrap();

    // and deposits all he currently has
    suite.deposit(BORROWER, market_token, 1_100u128).unwrap();

    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 1000);
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(BORROWER).unwrap()),
        1100
    );

    let credit_line = suite.query_credit_line(LENDER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1000 collateral * 2.0 oracle's price
            collateral: Uint128::new(2000),
            // 1000 collateral * 2.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(1400),
            borrow_limit: Uint128::new(1400),
            // no debt
            debt: Uint128::zero(),
        }
        .make_response(suite.common_token())
    );
    let credit_line = suite.query_credit_line(BORROWER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1100 collateral (deposited) * 2.0 oracle's price
            collateral: Uint128::new(2200),
            // 1100 collateral * 2.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(1540),
            borrow_limit: Uint128::new(1540),
            // 1000 borrowed * 2.0 oracle's price
            debt: Uint128::new(2000),
        }
        .make_response(suite.common_token())
    );
}

#[test]
fn deposits_and_borrows_cw20_tokens() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 1_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 1_000))
        .with_market_token(market_token.clone())
        .with_collateral_ratio(Decimal::percent(70))
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
    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(2_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();
    suite.set_high_credit_line(BORROWER).unwrap();

    // Lender deposits coins
    suite
        .deposit(LENDER, market_token.clone(), 1_000u128)
        .unwrap();

    // Now borrower borrows it
    suite.borrow(BORROWER, 1000).unwrap();

    // and deposits all he currently has
    suite.deposit(BORROWER, market_token, 1_100u128).unwrap();

    assert_eq!(suite.query_debt(BORROWER).unwrap().u128(), 1000);
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(BORROWER).unwrap()),
        1100
    );

    let credit_line = suite.query_credit_line(LENDER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1000 collateral * 2.0 oracle's price
            collateral: Uint128::new(2000),
            // 1000 collateral * 2.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(1400),
            borrow_limit: Uint128::new(1400),
            // no debt
            debt: Uint128::zero(),
        }
        .make_response(suite.common_token())
    );
    let credit_line = suite.query_credit_line(BORROWER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1100 collateral (deposited) * 2.0 oracle's price
            collateral: Uint128::new(2200),
            // 1100 collateral * 2.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(1540),
            borrow_limit: Uint128::new(1540),
            // 1000 borrowed * 2.0 oracle's price
            debt: Uint128::new(2000),
        }
        .make_response(suite.common_token())
    );
}

#[test]
fn deposits_and_borrows_native_tokens_market_common_matches_denoms() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(1_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(100u128)])
        .with_market_token(market_token.clone())
        // collateral ratio is 0.7
        .with_collateral_ratio(Decimal::percent(70))
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    suite
        .deposit(LENDER, market_token.clone(), 1_000u128)
        .unwrap();
    suite.borrow(BORROWER, 1_000).unwrap();

    suite.deposit(BORROWER, market_token, 1_100u128).unwrap();

    let credit_line = suite.query_credit_line(LENDER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1000 collateral * 1.0 oracle's price (no common_token denom)
            collateral: Uint128::new(1_000),
            // 1000 collateral * 1.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(700),
            borrow_limit: Uint128::new(700),
            // no debt
            debt: Uint128::zero(),
        }
        .make_response(suite.common_token())
    );
    let credit_line = suite.query_credit_line(BORROWER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1100 collateral (deposited) * 1.0 oracle's price
            collateral: Uint128::new(1100),
            // 1100 collateral * 1.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(770),
            borrow_limit: Uint128::new(770),
            // 1000 borrowed * 1.0 oracle's price
            debt: Uint128::new(1000),
        }
        .make_response(suite.common_token())
    );
}

#[test]
fn deposits_and_borrows_cw20_tokens_market_common_matches_denoms() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 1_000))
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
        PairType::Lsd {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    suite
        .deposit(LENDER, market_token.clone(), 1_000u128)
        .unwrap();
    suite.borrow(BORROWER, 1000).unwrap();

    suite.deposit(BORROWER, market_token, 1_100u128).unwrap();

    let credit_line = suite.query_credit_line(LENDER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1000 collateral * 1.0 oracle's price (no common_token denom)
            collateral: Uint128::new(1_000),
            // 1000 collateral * 0.5 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(700),
            borrow_limit: Uint128::new(700),
            // no debt
            debt: Uint128::zero(),
        }
        .make_response(suite.common_token())
    );
    let credit_line = suite.query_credit_line(BORROWER).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1100 collateral (deposited) * 1.0 oracle's price
            collateral: Uint128::new(1_100),
            // 1100 collateral * 1.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(770),
            borrow_limit: Uint128::new(770),
            // 1000 borrowed * 1.0 oracle's price
            debt: Uint128::new(1_000),
        }
        .make_response(suite.common_token())
    );
}

#[test]
fn query_credit_line_with_uncharged_interest_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(5_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(500u128)])
        .with_charge_period((SECONDS_IN_YEAR) as u64)
        .with_interest(10, 0)
        .with_reserve_factor(0)
        .with_market_token(market_token.clone())
        // collateral ratio is 0.7
        .with_collateral_ratio(Decimal::percent(70))
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();
    suite.set_high_credit_line(LENDER).unwrap();

    suite.deposit(LENDER, market_token, 2_000u128).unwrap();
    suite.borrow(BORROWER, 1_000).unwrap();

    suite.assert_debt(BORROWER, 1_000u128);
    suite.assert_collateral(LENDER, 2_000u128);

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // we want to make sure the queries return the amount with interest charged
    // even if there was no call to `charge_interest`

    suite.assert_debt(BORROWER, 1_100u128);
    suite.assert_collateral(LENDER, 2_100u128);
}

#[test]
fn query_credit_line_with_uncharged_interest_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 5_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 500))
        .with_charge_period((SECONDS_IN_YEAR) as u64)
        .with_interest(10, 0)
        .with_reserve_factor(0)
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
        PairType::Lsd {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();
    suite.set_high_credit_line(LENDER).unwrap();

    suite.deposit(LENDER, market_token, 2_000u128).unwrap();
    suite.borrow(BORROWER, 1_000).unwrap();

    suite.assert_debt(BORROWER, 1_000u128);
    suite.assert_collateral(LENDER, 2_000u128);

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // we want to make sure the queries return the amount with interest charged
    // even if there was no call to `charge_interest`

    suite.assert_debt(BORROWER, 1_100u128);
    suite.assert_collateral(LENDER, 2_100u128);
}
