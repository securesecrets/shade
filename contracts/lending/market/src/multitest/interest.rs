use super::suite::{SuiteBuilder, COMMON};

use cosmwasm_std::{Decimal, Timestamp};
use lend_utils::token::Token;
use wyndex::factory::PairType;

use crate::msg::InterestResponse;
use crate::multitest::suite::{BORROWER, LENDER, MARKET_TOKEN};
use crate::state::SECONDS_IN_YEAR;

const YEAR: u64 = (SECONDS_IN_YEAR) as u64;
const QUARTER: u64 = YEAR / 4;

#[test]
fn query_interest_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_token.clone().into_coin(150u128)])
        .with_market_token(market_token.clone())
        .with_collateral_ratio(Decimal::percent(70))
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    // At first, the lender has no c-token, and the contract has no base asset.
    assert_eq!(suite.query_ctoken_balance(LENDER).unwrap().u128(), 0);
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 0);

    // And, we are at base interest, with no utilisation
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            interest: Decimal::percent(3),
            utilisation: Decimal::zero(),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    // Deposit some tokens
    suite
        .deposit(LENDER, market_token.clone(), 100u128)
        .unwrap();

    // After the deposit, the lender has 100 c-token and the contract has 100 base asset.
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(LENDER).unwrap()),
        100
    );
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);

    // We still are at base interest, with no utilisation
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            interest: Decimal::percent(3),
            utilisation: Decimal::zero(),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    // Borrow some tokens
    suite.borrow(BORROWER, 10).unwrap();

    // Now utilisation is 10% (10/100),
    // The interest changed according to the linear formula: 3% + 20% * 10% = 3% + 2% = 5%.
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(10),
            interest: Decimal::percent(3) + Decimal::percent(2),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    suite
        .repay(BORROWER, market_token.clone().into_coin(5u128))
        .unwrap();

    // Utilisation is now 5% ((10-5)/100).
    // The interest changed according to the linear formula: 3% + 20% * 5% = 3% + 1% = 4%.
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(5),
            interest: Decimal::percent(3) + Decimal::percent(1),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    // Lend some more
    suite.deposit(LENDER, market_token, 50u128).unwrap();

    // Utilisation is now ~3.33% ((10-5)/(100+50)).
    // The interest changed according to the linear formula: 3% + 20% * 3.33% = 3% + 0.67% = 3.67%.
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::from_ratio(1u8, 30u8),
            interest: Decimal::percent(3) + Decimal::from_ratio(1u8, 150u8),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );
}

#[test]
fn query_interest_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 150))
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

    // At first, the lender has no c-token, and the contract has no base asset.
    assert_eq!(suite.query_ctoken_balance(LENDER).unwrap().u128(), 0);
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 0);

    // And, we are at base interest, with no utilisation
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            interest: Decimal::percent(3),
            utilisation: Decimal::zero(),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    // Deposit some tokens
    suite
        .deposit(LENDER, market_token.clone(), 100u128)
        .unwrap();

    // After the deposit, the lender has 100 c-token and the contract has 100 base asset.
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(LENDER).unwrap()),
        100
    );
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);

    // We still are at base interest, with no utilisation
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            interest: Decimal::percent(3),
            utilisation: Decimal::zero(),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    // Borrow some tokens
    suite.borrow(BORROWER, 10).unwrap();

    // Now utilisation is 10% (10/100),
    // The interest changed according to the linear formula: 3% + 20% * 10% = 3% + 2% = 5%.
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(10),
            interest: Decimal::percent(3) + Decimal::percent(2),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    suite
        .repay(BORROWER, market_token.clone().into_coin(5u128))
        .unwrap();

    // Utilisation is now 5% ((10-5)/100).
    // The interest changed according to the linear formula: 3% + 20% * 5% = 3% + 1% = 4%.
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(5),
            interest: Decimal::percent(3) + Decimal::percent(1),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    // Lend some more
    suite.deposit(LENDER, market_token, 50u128).unwrap();

    // Utilisation is now ~3.33% ((10-5)/(100+50)).
    // The interest changed according to the linear formula: 3% + 20% * 3.33% = 3% + 0.67% = 3.67%.
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::from_ratio(1u8, 30u8),
            interest: Decimal::percent(3) + Decimal::from_ratio(1u8, 150u8),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );
}

#[test]
fn charge_interest_borrow_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(YEAR)
        .with_funds(LENDER, &[market_token.clone().into_coin(2_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(500u128)])
        .with_interest(4, 20)
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    // Deposit some tokens
    suite
        .deposit(LENDER, market_token.clone(), 2_000u128)
        .unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 1600).unwrap();

    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(80),
            interest: Decimal::percent(20),
            charge_period: Timestamp::from_seconds(YEAR),
        },
        resp
    );

    suite.advance_seconds(YEAR);

    // Repay some tokens
    // interests are 20%
    // that means debt 1600 + 320
    // repay 800 -> 1120 left debt
    suite
        .repay(BORROWER, market_token.clone().into_coin(800u128))
        .unwrap();

    assert_eq!(suite.query_total_debt().unwrap().total.u128(), 1120);
    suite.advance_seconds(YEAR);

    // Repay some tokens
    // Utilisation is 48.3%
    // interests are 13.66%
    // debt 1120 + 13.66% - 800 = 472.992
    suite
        .repay(BORROWER, market_token.clone().into_coin(800u128))
        .unwrap();

    assert_eq!(suite.query_total_debt().unwrap().total.u128(), 472);

    // Repay the rest of debt (borrower had extra 500 tokens)
    // since we overpay a bit, this should leave no debt for the borrower
    suite
        .repay(BORROWER, market_token.into_coin(474u128))
        .unwrap();
    assert_eq!(suite.query_total_debt().unwrap().total.u128(), 0);
}

#[test]
fn charge_interest_borrow_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(YEAR)
        .with_initial_cw20(market_token.denom(), (LENDER, 2_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 500))
        .with_interest(4, 20)
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

    // Deposit some tokens
    suite
        .deposit(LENDER, market_token.clone(), 2_000u128)
        .unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 1_600).unwrap();

    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(80),
            interest: Decimal::percent(20),
            charge_period: Timestamp::from_seconds(YEAR),
        },
        resp
    );

    suite.advance_seconds(YEAR);

    // Repay some tokens
    // interests are 20%
    // that means debt 1600 + 320
    // repay 800 -> 1120 left debt
    suite
        .repay(BORROWER, market_token.clone().into_coin(800u128))
        .unwrap();

    assert_eq!(suite.query_total_debt().unwrap().total.u128(), 1120);
    suite.advance_seconds(YEAR);

    // Repay some tokens
    // Utilisation is 48.3%
    // interests are 13.66%
    // debt 1120 + 13.66% - 800 = 472.992
    suite
        .repay(BORROWER, market_token.clone().into_coin(800u128))
        .unwrap();

    assert_eq!(suite.query_total_debt().unwrap().total.u128(), 472);

    // Repay the rest of debt (borrower had extra 500 tokens)
    // since we overpay a bit, this should leave no debt for the borrower
    suite
        .repay(BORROWER, market_token.into_coin(474u128))
        .unwrap();
    assert_eq!(suite.query_total_debt().unwrap().total.u128(), 0);
}

#[test]
fn charge_interest_deposit_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(YEAR)
        .with_funds(LENDER, &[market_token.clone().into_coin(4_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(2_300u128)])
        .with_interest(4, 20)
        .with_market_token(market_token.clone())
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

    suite.advance_seconds(YEAR);

    // Deposit some tokens
    // interest is 20% (4% base + 20% slope * 80% utilization)
    // that means ctoken 2000 + 1600*20% = 2320
    // deposit 1000 -> 3320 left debt
    suite
        .deposit(LENDER, market_token.clone(), 1_000u128)
        .unwrap();

    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        3319u128
    );

    suite.advance_seconds(YEAR);

    // Repay some tokens
    // Now utilisation is 57.85%,
    // interest rate 15.57%
    // amount of debt - 1600 + 20% interests = 1920
    // 1920 * 15.57% = 298.94 ctokens interests are made
    // ctokens should go up to 3618.14
    // 3618.14 + 1000 = 4618.14 ctokens
    suite
        .deposit(LENDER, market_token.clone(), 1_000u128)
        .unwrap();

    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        4618u128
    );

    // Borrower pays all of his debt
    suite
        .repay(BORROWER, market_token.clone().into_coin(2_219u128))
        .unwrap();
    assert_eq!(suite.query_total_debt().unwrap().total.u128(), 0);

    // ...which allows to withdraw all tokens with interests
    suite.withdraw(LENDER, 4616).unwrap();
    assert_eq!(
        suite
            .query_asset_balance(LENDER, market_token.denom())
            .unwrap(),
        4616
    );
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        // TODO: rounding error
        2u128
    );
}

#[test]
fn charge_interest_deposit_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(YEAR)
        .with_initial_cw20(market_token.denom(), (LENDER, 4_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 2_300))
        .with_interest(4, 20)
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

    // Deposit some tokens
    suite
        .deposit(LENDER, market_token.clone(), 2_000u128)
        .unwrap();

    // Borrow some tokens
    suite.borrow(BORROWER, 1600).unwrap();

    suite.advance_seconds(YEAR);

    // Deposit some tokens
    // interest is 20% (4% base + 20% slope * 80% utilization)
    // that means ctoken 2000 + 1600*20% = 2320
    // deposit 1000 -> 3320 left debt
    suite
        .deposit(LENDER, market_token.clone(), 1_000u128)
        .unwrap();

    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        3319u128
    );

    suite.advance_seconds(YEAR);

    // Repay some tokens
    // Now utilisation is 57.85%,
    // interest rate 15.57%
    // amount of debt - 1600 + 20% interests = 1920
    // 1920 * 15.57% = 298.94 ctokens interests are made
    // ctokens should go up to 3618.14
    // 3618.14 + 1000 = 4618.14 ctokens
    suite
        .deposit(LENDER, market_token.clone(), 1_000u128)
        .unwrap();

    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        4618u128
    );

    // Borrower pays all of his debt
    suite
        .repay(BORROWER, market_token.clone().into_coin(2_219u128))
        .unwrap();
    assert_eq!(suite.query_total_debt().unwrap().total.u128(), 0);

    // ...which allows to withdraw all tokens with interests
    suite.withdraw(LENDER, 4616).unwrap();
    assert_eq!(
        suite
            .query_cw20_balance(LENDER, market_token.denom())
            .unwrap(),
        4616
    );
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_info().unwrap().total_supply),
        // TODO: rounding error
        2u128
    );
}

#[test]
fn query_native_balance_with_uncharged_interest() {
    // We want to make sure if we query for balance with interest that hasn't been charged yet,
    // the query will display the value with interest included.

    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(YEAR)
        .with_funds(LENDER, &[market_token.clone().into_coin(2_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(500u128)])
        .with_interest(10, 20)
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();

    suite.deposit(LENDER, market_token, 2_000u128).unwrap();

    suite.borrow(BORROWER, 500).unwrap();

    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(25),
            interest: Decimal::percent(15),
            charge_period: Timestamp::from_seconds(YEAR),
        },
        resp
    );

    suite.assert_ctoken_balance(LENDER, 2000u128);
    suite.assert_debt_balance(BORROWER, 500u128);

    suite.advance_seconds(YEAR);

    suite.assert_ctoken_balance(LENDER, 2075u128);
    suite.assert_debt_balance(BORROWER, 575u128);
}

#[test]
fn query_cw20_balance_with_uncharged_interest() {
    // We want to make sure if we query for balance with interest that hasn't been charged yet,
    // the query will display the value with interest included.

    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(YEAR)
        .with_initial_cw20(market_token.denom(), (LENDER, 2_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 500))
        .with_market_token(market_token.clone())
        .with_interest(10, 20)
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

    suite.deposit(LENDER, market_token, 2_000u128).unwrap();

    suite.borrow(BORROWER, 500).unwrap();

    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(25),
            interest: Decimal::percent(15),
            charge_period: Timestamp::from_seconds(YEAR),
        },
        resp
    );

    suite.assert_ctoken_balance(LENDER, 2000u128);
    suite.assert_debt_balance(BORROWER, 500u128);

    suite.advance_seconds(YEAR);

    suite.assert_ctoken_balance(LENDER, 2075u128);
    suite.assert_debt_balance(BORROWER, 575u128);
}

#[test]
fn compounding_interest_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(YEAR)
        .with_funds(LENDER, &[market_token.clone().into_coin(5_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(500u128)])
        .with_charge_period(QUARTER)
        .with_interest(40, 0) // 40% annual, 10% quarterly
        .with_reserve_factor(15)
        .with_market_token(market_token.clone())
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

    suite.assert_debt_balance("borrower", 1000u128);

    // We're charging interest every quarter.
    // After three quarters pass, the result should be:
    // 1000 * 110% * 110% * 110% = 1331
    suite.advance_seconds(QUARTER * 3);
    suite.assert_debt_balance("borrower", 1331u128);
}

#[test]
fn compounding_interest_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 5_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 500))
        .with_market_token(market_token.clone())
        .with_charge_period(QUARTER)
        .with_interest(40, 0) // 40% annual, 10% quarterly
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

    suite.assert_debt_balance("borrower", 1000u128);

    // We're charging interest every quarter.
    // After three quarters pass, the result should be:
    // 1000 * 110% * 110% * 110% = 1331
    suite.advance_seconds(QUARTER * 3);
    suite.assert_debt_balance("borrower", 1331u128);
}

#[test]
fn compounding_interest_native_charge_triggered_every_epoch() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(YEAR)
        .with_funds(LENDER, &[market_token.clone().into_coin(5_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(500u128)])
        .with_charge_period(QUARTER)
        .with_interest(40, 0) // 40% annual, 10% quarterly
        .with_reserve_factor(15)
        .with_market_token(market_token.clone())
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.set_high_credit_line(BORROWER).unwrap();
    suite.set_high_credit_line(LENDER).unwrap();

    suite
        .deposit(LENDER, market_token.clone(), 2_000u128)
        .unwrap();
    suite.borrow(BORROWER, 1000).unwrap();

    suite.assert_debt_balance("borrower", 1000u128);

    for _ in 0..3 {
        suite.advance_seconds(QUARTER);
        // Just to trigger an interest charge
        suite.deposit(LENDER, market_token.clone(), 2u128).unwrap();
    }

    // We're charging interest every quarter.
    // After three quarters pass, the result should be:
    // 1000 * 110% * 110% * 110% = 1331
    suite.assert_debt_balance("borrower", 1331u128);
}

#[test]
fn compounding_interest_cw20_charge_triggered_every_epoch() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 5_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 500))
        .with_market_token(market_token.clone())
        .with_charge_period(QUARTER)
        .with_interest(40, 0) // 40% annual, 10% quarterly
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

    suite
        .deposit(LENDER, market_token.clone(), 2_000u128)
        .unwrap();
    suite.borrow(BORROWER, 1000).unwrap();

    suite.assert_debt_balance("borrower", 1000u128);

    for _ in 0..3 {
        suite.advance_seconds(QUARTER);
        // Just to trigger an interest charge
        suite.deposit(LENDER, market_token.clone(), 2u128).unwrap();
    }

    // We're charging interest every quarter.
    // After three quarters pass, the result should be:
    // 1000 * 110% * 110% * 110% = 1331
    suite.assert_debt_balance("borrower", 1331u128);
}

#[test]
fn query_last_charged_native_with_uncharged_interest() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_charge_period(YEAR)
        .with_funds(LENDER, &[market_token.clone().into_coin(5_000u128)])
        .with_funds(BORROWER, &[market_token.clone().into_coin(500u128)])
        .with_charge_period(QUARTER)
        .with_interest(40, 0) // 40% annual, 10% quarterly
        .with_reserve_factor(15)
        .with_market_token(market_token.clone())
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

    let next_epoch = suite.query_config().unwrap().last_charged + YEAR;

    suite.advance_seconds(YEAR + 123);

    // we want to make sure the query returns the timestamp as if interest was already charged for this epoch
    // even if there was no call to `charge_interest`

    assert_eq!(next_epoch, suite.query_config().unwrap().last_charged);
}

#[test]
fn query_last_charged_cw20_with_uncharged_interest() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 5_000))
        .with_initial_cw20(market_token.denom(), (BORROWER, 500))
        .with_market_token(market_token.clone())
        .with_charge_period(QUARTER)
        .with_interest(40, 0) // 40% annual, 10% quarterly
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

    let next_epoch = suite.query_config().unwrap().last_charged + YEAR;

    suite.advance_seconds(YEAR + 123);

    // we want to make sure the query returns the timestamp as if interest was already charged for this epoch
    // even if there was no call to `charge_interest`

    assert_eq!(next_epoch, suite.query_config().unwrap().last_charged);
}
