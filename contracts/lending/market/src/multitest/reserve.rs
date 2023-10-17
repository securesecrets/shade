use super::suite::{SuiteBuilder, COMMON};

use cosmwasm_std::{Decimal, Uint128};
use utils::token::Token;
use wyndex::factory::PairType;

use crate::multitest::suite::{BORROWER, GOVERNANCE, LENDER, MARKET_TOKEN};
use crate::state::SECONDS_IN_YEAR;
use utils::assert_approx_eq;

#[test]
fn after_full_year_native() {
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

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // Deposit some tokens
    // interests are 20% (4% base + 20% slope * 80% utilization)
    // supplied (ctokens) = 2000
    // borrowed (debt) = 1600
    // bMul (debt_ratio) = 20% after full year
    // charged interests = 20% * 1600 = 320
    // reserve = 10% * charged interests = 32
    // liquid assets = 400
    // ctokens supplied = 1600 + 400 - 32 = 1968
    // lMul (ctoken_ratio) = borrowed * bMul / lMul = 1600 * 0.2 / 1968 ~= 0.162
    // that means ctokens 2000 * 1.163 = 2324
    // reserve is minted straight away to the governance contract increasing supply
    // deposit 1000 -> 3324 left ctokens + 32 interest -> 3357
    // Reserve should be zero after this
    // Deposit some tokens
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        3357u128
    );
    let gov_balance = suite.ctokens_to_base(suite.query_ctoken_balance(GOVERNANCE).unwrap());
    // Reserve is now depleted
    assert_eq!(suite.query_reserve().unwrap(), Uint128::zero());
    // Gov contract has received interest
    // Note we are receiving all but one of the charged interest tokens.
    assert_eq!(gov_balance, 31);
}

#[test]
fn after_full_year_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period((SECONDS_IN_YEAR) as u64)
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

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // Deposit some tokens
    // interests are 20% (4% base + 20% slope * 80% utilization)
    // supplied (ctokens) = 2000
    // borrowed (debt) = 1600
    // bMul (debt_ratio) = 20% after full year
    // charged interests = 20% * 1600 = 320
    // reserve = 10% * charged interests = 32
    // liquid assets = 400
    // ctokens supplied = 1600 + 400 - 32 = 1968
    // lMul (ctoken_ratio) = borrowed * bMul / lMul = 1600 * 0.2 / 1968 ~= 0.162
    // that means ctokens 2000 * 1.163 = 2324
    // reserve is minted straight away to the governance contract increasing supply
    // deposit 1000 -> 3324 left ctokens + 32 interest -> 3357
    // Reserve should be zero after this
    // Deposit some tokens
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        3357u128
    );
    let gov_balance = suite.ctokens_to_base(suite.query_ctoken_balance(GOVERNANCE).unwrap());
    // Reserve is now depleted
    assert_eq!(suite.query_reserve().unwrap(), Uint128::zero());
    // Gov contract has received interest
    // Note we are receiving all but one of the charged interest tokens.
    assert_eq!(gov_balance, 31);
}

#[test]
fn after_half_year_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period((SECONDS_IN_YEAR / 2) as u64)
        .with_market_token(market_token.clone())
        .with_funds(LENDER, &[market_token.clone().into_coin(5_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(500u128)])
        .with_interest(4, 20)
        .with_reserve_factor(20)
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
        .deposit(LENDER, market_token.clone(), 4_000u128)
        .unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 3000).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR / 2) as u64);

    // Deposit some tokens
    // interests are 19% (4% base + 20% slope * 75% utilization)
    // supplied (ctokens) = 4000
    // borrwed (debt) = 3000
    // bMul (debt_ratio) = 9.5% after half year
    // charged interests = 9.5% * 3000 = 285
    // reserve = 20% * charged interests = 57
    // liquid assets = 1000
    // ctokens supplied = 3000 + 1000 - 57 = 3943
    // lMul (ctoken_ratio) = borrowed * bMul / lMul = 3000 * 0.095 / 3400 ~= 0.072
    // that means ctokens 4000 * 1.072 = 4288
    // deposit 1000 -> 5288 left debt
    // reserve is minted straight away to the governance contract increasing supply
    // deposit 1000 -> 5288 left debt + 57 interest -> 5346
    // Reserve should be zero after this
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    assert_approx_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        5346u128,
        Decimal::permille(1),
    );

    let gov_balance = suite.ctokens_to_base(suite.query_ctoken_balance(GOVERNANCE).unwrap());
    // Reserve is now depleted
    assert_eq!(suite.query_reserve().unwrap(), Uint128::zero());
    // Gov contract has received interest
    assert_eq!(gov_balance, 56);
}

#[test]
fn after_half_year_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period((SECONDS_IN_YEAR / 2) as u64)
        .with_initial_cw20(market_token.denom(), (LENDER, 5_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 500))
        .with_market_token(market_token.clone())
        .with_interest(4, 20)
        .with_reserve_factor(20)
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
        .deposit(LENDER, market_token.clone(), 4_000u128)
        .unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 3000).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR / 2) as u64);

    // Deposit some tokens
    // interests are 19% (4% base + 20% slope * 75% utilization)
    // supplied (ctokens) = 4000
    // borrwed (debt) = 3000
    // bMul (debt_ratio) = 9.5% after half year
    // charged interests = 9.5% * 3000 = 285
    // reserve = 20% * charged interests = 57
    // liquid assets = 1000
    // ctokens supplied = 3000 + 1000 - 57 = 3943
    // lMul (ctoken_ratio) = borrowed * bMul / lMul = 3000 * 0.095 / 3400 ~= 0.072
    // that means ctokens 4000 * 1.072 = 4288
    // deposit 1000 -> 5288 left debt
    // reserve is minted straight away to the governance contract increasing supply
    // deposit 1000 -> 5288 left debt + 57 interest -> 5346
    // Reserve should be zero after this
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    assert_approx_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        5346u128,
        Decimal::permille(1),
    );

    let gov_balance = suite.ctokens_to_base(suite.query_ctoken_balance(GOVERNANCE).unwrap());
    // Reserve is now depleted
    assert_eq!(suite.query_reserve().unwrap(), Uint128::zero());
    // Gov contract has received interest
    assert_eq!(gov_balance, 56);
}

#[test]
fn charged_couple_times_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period((SECONDS_IN_YEAR / 4) as u64)
        .with_market_token(market_token.clone())
        .with_funds(LENDER, &[market_token.clone().into_coin(5_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(500u128)])
        .with_interest(4, 20)
        .with_reserve_factor(15)
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
    suite.borrow(BORROWER, 1200).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR / 4) as u64);

    // Deposit some tokens
    // interests are 16%(4% base + 20% slope * 60% utilization)
    // supplied (ctokens) = 2000
    // borrwed (debt) = 1200
    // bMul (debt_ratio) = 4% after 3 months
    // charged interests = 4% * 1200 = 48
    // reserve = 15% * charged interests = 7.2 ~= 7
    // liquid assets = 800
    // ctokens supplied = 1200 + 800 - 7 = 1993
    // lMul (ctoken_ratio) = borrowed * bMul / lMul = 1200 * 0.04 / 1993 ~= 0.024
    // that means ctokens 2000 * 1.024 = 2048
    // deposit 1000 -> 3048 left debt
    // reserve is minted straight away to the governance contract increasing supply
    // deposit 1000 -> 3048 left debt + 7 interest -> 3055
    // Reserve should be zero after this
    suite
        .deposit(LENDER, market_token.clone(), 1_000u128)
        .unwrap();

    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        3055u128
    );

    let gov_balance = suite.ctokens_to_base(suite.query_ctoken_balance(GOVERNANCE).unwrap());
    // Reserve is now depleted
    assert_eq!(suite.query_reserve().unwrap(), Uint128::zero());
    // Gov contract has received interest
    assert_eq!(gov_balance, 6);

    suite.borrow(BORROWER, 800).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR / 4) as u64);

    // Deposit some tokens
    // interests are 17.4%(4% base + 20% slope * 67% utilization)
    // supplied (ctokens) = 3047
    // borrwed (debt) = 2047 (1200 * 1.04 + 1000)
    // bMul (debt_ratio) = 4.35% after 6 months
    // charged interets = 4.35% * 2047 = 89.0445 ~= 89
    // reserve = 15% * charged interests = 7 + (15% * 89) = 20
    // liquid assets =  3047 + 7 (old reserve) - 2047 (borrowed) = 1007
    // ctokens supplied = 2047 + 1007 - 20 = 3034
    // lMul (ctoken_ratio) = borrowed * bMul / lMul = 2047 * 0.0435 / 3034 ~= 0.029
    // that means ctokens 3047 * 1.029 = 3136
    // deposit 1000 -> 4136 left ctokens + 20 interest 4156
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    assert_approx_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        4156u128,
        Decimal::permille(1),
    );

    let gov_balance = suite.ctokens_to_base(suite.query_ctoken_balance(GOVERNANCE).unwrap());
    // Reserve is now depleted
    assert_eq!(suite.query_reserve().unwrap(), Uint128::zero());
    // Gov contract has received interest
    assert_eq!(gov_balance, 20);
}

#[test]
fn charged_couple_times_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period((SECONDS_IN_YEAR / 4) as u64)
        .with_initial_cw20(market_token.denom(), (LENDER, 5_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 500))
        .with_market_token(market_token.clone())
        .with_interest(4, 20)
        .with_reserve_factor(15)
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
    suite.borrow(BORROWER, 1200).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR / 4) as u64);

    // Deposit some tokens
    // interests are 16%(4% base + 20% slope * 60% utilization)
    // supplied (ctokens) = 2000
    // borrwed (debt) = 1200
    // bMul (debt_ratio) = 4% after 3 months
    // charged interests = 4% * 1200 = 48
    // reserve = 15% * charged interests = 7.2 ~= 7
    // liquid assets = 800
    // ctokens supplied = 1200 + 800 - 7 = 1993
    // lMul (ctoken_ratio) = borrowed * bMul / lMul = 1200 * 0.04 / 1993 ~= 0.024
    // that means ctokens 2000 * 1.024 = 2048
    // deposit 1000 -> 3048 left debt
    // reserve is minted straight away to the governance contract increasing supply
    // deposit 1000 -> 3048 left debt + 7 interest -> 3055
    // Reserve should be zero after this
    suite
        .deposit(LENDER, market_token.clone(), 1_000u128)
        .unwrap();

    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        3055u128
    );

    let gov_balance = suite.ctokens_to_base(suite.query_ctoken_balance(GOVERNANCE).unwrap());
    // Reserve is now depleted
    assert_eq!(suite.query_reserve().unwrap(), Uint128::zero());
    // Gov contract has received interest
    assert_eq!(gov_balance, 6);

    suite.borrow(BORROWER, 800).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR / 4) as u64);

    // Deposit some tokens
    // interests are 17.4%(4% base + 20% slope * 67% utilization)
    // supplied (ctokens) = 3047
    // borrwed (debt) = 2047 (1200 * 1.04 + 1000)
    // bMul (debt_ratio) = 4.35% after 6 months
    // charged interets = 4.35% * 2047 = 89.0445 ~= 89
    // reserve = 15% * charged interests = 7 + (15% * 89) = 20
    // liquid assets =  3047 + 7 (old reserve) - 2047 (borrowed) = 1007
    // ctokens supplied = 2047 + 1007 - 20 = 3034
    // lMul (ctoken_ratio) = borrowed * bMul / lMul = 2047 * 0.0435 / 3034 ~= 0.029
    // that means ctokens 3047 * 1.029 = 3136
    // deposit 1000 -> 4136 left ctokens + 20 interest 4156
    suite.deposit(LENDER, market_token, 1_000u128).unwrap();

    assert_approx_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        4156u128,
        Decimal::permille(1),
    );

    let gov_balance = suite.ctokens_to_base(suite.query_ctoken_balance(GOVERNANCE).unwrap());
    // Reserve is now depleted
    assert_eq!(suite.query_reserve().unwrap(), Uint128::zero());
    // Gov contract has received interest
    assert_eq!(gov_balance, 20);
}

#[test]
fn query_reserve_with_uncharged_interest_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(SECONDS_IN_YEAR as u64)
        .with_market_token(market_token.clone())
        .with_funds(LENDER, &[market_token.clone().into_coin(5_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(500u128)])
        .with_interest(10, 0)
        .with_reserve_factor(15)
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();
    suite.set_high_credit_line(LENDER).unwrap();

    suite.deposit(LENDER, market_token, 2_000u128).unwrap();

    suite.borrow(BORROWER, 1000).unwrap();

    assert_eq!(Uint128::zero(), suite.query_reserve().unwrap());

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    assert_eq!(15, suite.query_reserve().unwrap().u128());
}

#[test]
fn query_reserve_with_uncharged_interest_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(SECONDS_IN_YEAR as u64)
        .with_initial_cw20(market_token.denom(), (LENDER, 5_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 500))
        .with_market_token(market_token.clone())
        .with_interest(10, 0)
        .with_reserve_factor(15)
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

    suite.borrow(BORROWER, 1000).unwrap();

    assert_eq!(Uint128::zero(), suite.query_reserve().unwrap());

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    assert_eq!(15, suite.query_reserve().unwrap().u128());
}
