use super::suite::{SuiteBuilder, COMMON};
use crate::{
    multitest::suite::{BORROWER, LENDER, MARKET_TOKEN},
    state::SECONDS_IN_YEAR,
};

use cosmwasm_std::{Decimal, Uint128};

use lending_utils::{
    assert_approx_eq,
    interest::{Interest, ValidatedInterest},
    token::Token,
};
use wyndex::factory::PairType;

#[test]
fn adjust_collateral_ratio() {
    let mut suite = SuiteBuilder::new()
        .with_collateral_ratio(Decimal::percent(15))
        .build();

    suite.sudo_adjust_collateral_ratio(30).unwrap();

    assert_eq!(
        Decimal::percent(30),
        suite.query_config().unwrap().collateral_ratio
    );
}

#[test]
fn adjust_reserve_factor_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period((SECONDS_IN_YEAR) as u64)
        .with_market_token(market_token.clone())
        .with_funds(LENDER, &[market_token.clone().into_coin(4_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(2_300u128)])
        .with_interest(4, 20)
        .with_reserve_factor(10)
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();
    suite.set_high_credit_line(LENDER).unwrap();

    // Deposit some tokens
    suite
        .deposit(LENDER, market_token.clone(), 2_000u128)
        .unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 1600).unwrap();

    // Point of test - change reserve factor
    suite.sudo_adjust_reserve_factor(30).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // interests are 20% (4% base + 20% slope * 80% utilization)
    // bMul (debt_ratio) = 20% after full year
    // charged interests = 20% * 1600 = 320
    // reserve = 30% * charged interests = 96
    assert_eq!(suite.query_reserve().unwrap(), Uint128::new(96));
    // deposit some tokens to trigger charging
    // liquid assets = 400
    // ctokens supplied = 1600 + 400 - 96 = 1904
    // cMul (ctoken_ratio) = borrowed * bMul / cMul = 1600 * 0.2 / 1904 ~= 0.168
    // that means ctokens 2000 * 1.163 = 2336
    // deposit 1000 -> 3336 left ctokens + 96 interest -> 3432
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();
    assert_eq!(
        Decimal::percent(30),
        suite.query_config().unwrap().reserve_factor
    );

    assert_eq!(suite.query_reserve().unwrap(), Uint128::zero());
    assert_approx_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        3432u128,
        Decimal::permille(1),
    );
}

#[test]
fn adjust_reserve_factor_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(SECONDS_IN_YEAR as u64)
        .with_initial_cw20(market_token.denom(), (LENDER, 4_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 2_300))
        .with_market_token(market_token.clone())
        .with_interest(4, 20)
        .with_reserve_factor(10)
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

    // Deposit some tokens
    suite
        .deposit(LENDER, market_token.clone(), 2_000u128)
        .unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 1600).unwrap();

    // Point of test - change reserve factor
    suite.sudo_adjust_reserve_factor(30).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // interests are 20% (4% base + 20% slope * 80% utilization)
    // bMul (debt_ratio) = 20% after full year
    // charged interests = 20% * 1600 = 320
    // reserve = 30% * charged interests = 96
    assert_eq!(suite.query_reserve().unwrap(), Uint128::new(96));
    // deposit some tokens to trigger charging
    // liquid assets = 400
    // ctokens supplied = 1600 + 400 - 96 = 1904
    // cMul (ctoken_ratio) = borrowed * bMul / cMul = 1600 * 0.2 / 1904 ~= 0.168
    // that means ctokens 2000 * 1.163 = 2336
    // deposit 1000 -> 3336 left ctokens + 96 interest -> 3432
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();
    assert_eq!(
        Decimal::percent(30),
        suite.query_config().unwrap().reserve_factor
    );

    assert_eq!(suite.query_reserve().unwrap(), Uint128::zero());
    assert_approx_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        3432u128,
        Decimal::permille(1),
    );
}

#[test]
fn adjust_price_oracle() {
    let mut suite = SuiteBuilder::new().build();

    let new_oracle = "some new oracle";
    suite.sudo_adjust_price_oracle(new_oracle).unwrap();

    assert_eq!(new_oracle, suite.query_config().unwrap().price_oracle);
}

#[test]
fn adjust_market_cap() {
    let mut suite = SuiteBuilder::new().with_cap(Uint128::new(100)).build();

    let new_cap = Some(Uint128::new(333));
    suite.sudo_adjust_market_cap(new_cap).unwrap();

    assert_eq!(new_cap, suite.query_config().unwrap().market_cap);

    let new_cap = None;
    suite.sudo_adjust_market_cap(new_cap).unwrap();

    assert_eq!(new_cap, suite.query_config().unwrap().market_cap);
}

#[test]
fn adjust_interest_rates_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(SECONDS_IN_YEAR as u64)
        .with_market_token(market_token.clone())
        .with_funds(LENDER, &[market_token.clone().into_coin(4_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(2_300u128)])
        .with_interest(4, 20)
        .with_reserve_factor(0)
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();
    suite.set_high_credit_line(LENDER).unwrap();

    // Deposit some tokens
    suite
        .deposit(LENDER, market_token.clone(), 2_000u128)
        .unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 1000).unwrap();

    let new_interests = Interest::Linear {
        base: Decimal::percent(5),
        slope: Decimal::percent(50),
    };
    suite
        .sudo_adjust_interest_rates(new_interests.clone())
        .unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // interests are 30% (5% base + 50% slope * 50% utilization)
    // bMul (debt_ratio) = 30% after full year
    // charged interests = 30% * 1000 = 300
    // liquid assets = 1000
    // ctokens supplied = 1000 + 1000 = 2000
    // cMul (ctoken_ratio) = borrowed * bMul / cMul = 1000 * 0.3 / 2000 ~= 0.15
    // that means ctokens 2000 * 1.15 = 2300
    // deposit 1000 -> 3300 left ctokens

    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    assert_eq!(
        ValidatedInterest::unchecked(new_interests),
        suite.query_config().unwrap().rates
    );

    // TODO: Rounding issue
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        3299u128
    );
}

#[test]
fn adjust_interest_rates_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(SECONDS_IN_YEAR as u64)
        .with_market_token(market_token.clone())
        .with_initial_cw20(market_token.denom(), (LENDER, 4_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 2_300))
        .with_interest(4, 20)
        .with_reserve_factor(0)
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

    // Deposit some tokens
    suite
        .deposit(LENDER, market_token.clone(), 2_000u128)
        .unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 1000).unwrap();

    let new_interests = Interest::Linear {
        base: Decimal::percent(5),
        slope: Decimal::percent(50),
    };
    suite
        .sudo_adjust_interest_rates(new_interests.clone())
        .unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // interests are 30% (5% base + 50% slope * 50% utilization)
    // bMul (debt_ratio) = 30% after full year
    // charged interests = 30% * 1000 = 300
    // liquid assets = 1000
    // ctokens supplied = 1000 + 1000 = 2000
    // cMul (ctoken_ratio) = borrowed * bMul / cMul = 1000 * 0.3 / 2000 ~= 0.15
    // that means ctokens 2000 * 1.15 = 2300
    // deposit 1000 -> 3300 left ctokens

    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    assert_eq!(
        ValidatedInterest::unchecked(new_interests),
        suite.query_config().unwrap().rates
    );

    // TODO: Rounding issue
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        3299u128
    );
}
