use cosmwasm_std::{Decimal, StdError, Uint128};
use lend_utils::{credit_line::CreditLineValues, token::Token};
use wyndex::factory::PairType;

use super::suite::{SuiteBuilder, COMMON, LENDER};
use crate::{
    error::ContractError,
    multitest::suite::{MARKET_TOKEN, WYND},
};

#[test]
fn deposit_native_works() {
    // Native
    let market_token = Token::Native(MARKET_TOKEN.to_owned());

    let market_coin = market_token.clone().into_coin(100u128);

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_coin])
        .with_market_token(market_token.clone())
        .build();

    // At first, the lender has no c-token, and the contract has no base asset.
    assert_eq!(
        suite.query_contract_asset_balance().unwrap(),
        0,
        "expected the lender to have zero c-token before deposit"
    );
    assert_eq!(suite.query_ctoken_balance(LENDER).unwrap().u128(), 0);

    suite.deposit(LENDER, market_token, 100u128).unwrap();

    // After the deposit, the lender has 100 c-token and the contract has 100 base asset.
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(LENDER).unwrap()),
        100
    );
}

#[test]
fn deposit_cw20_works() {
    // Cw20
    let market_token = Token::Cw20(MARKET_TOKEN.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 100))
        .with_market_token(market_token.clone())
        .build();

    // Recover the address of the created cw20.
    let cw20_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();
    // We have to change the market token with the real address created during suite build.
    let market_token = suite.market_token.clone();

    // Check everything is working as inteded.
    assert_eq!(market_token, cw20_token);

    // At first, the lender has no c-token, and the contract has no base asset.
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 0);
    assert_eq!(suite.query_ctoken_balance(LENDER).unwrap().u128(), 0);

    suite.deposit(LENDER, market_token, 100u128).unwrap();

    // After the deposit, the lender has 100 c-token and the contract has 100 base asset.
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(LENDER).unwrap()),
        100
    );
}

#[test]
fn deposit_multiple_denoms_fails() {
    // Native
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let native = Token::Native("juno".to_string());

    // Coin from token
    let market_coin = market_token.clone().into_coin(100u128);
    let native_coin = native.clone().into_coin(100u128);

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[market_coin, native_coin])
        .with_market_token(market_token.clone())
        .build();

    let err = suite
        .deposit_multiple_native(LENDER, &[(market_token, 100), (native, 100)])
        .unwrap_err();
    assert_eq!(
        ContractError::RequiresExactlyOneCoin {},
        err.downcast().unwrap()
    );
}

#[test]
fn deposit_wrong_native_denom_fails() {
    // Native
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let native = Token::Native("juno".to_string());

    let native_coin = native.clone().into_coin(100u128);

    let mut suite = SuiteBuilder::new()
        .with_funds(LENDER, &[native_coin])
        .with_market_token(market_token.clone())
        .build();

    let err = suite.deposit(LENDER, native, 100u128).unwrap_err();
    assert_eq!(
        ContractError::InvalidDenom(market_token.denom()),
        err.downcast().unwrap()
    );
}

#[test]
fn deposit_wrong_cw20_fails() {
    // Cw20
    let market_token = Token::Cw20(MARKET_TOKEN.to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 100))
        .with_market_token(market_token.clone())
        .build();

    // Recover the address of the created cw20.
    let cw20_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();
    // We have to change the market token with the real address created during suite build.
    let market_token = suite.market_token.clone();

    // Check everything is working as inteded.
    assert_eq!(market_token, cw20_token);

    // At first, the lender has no c-token, and the contract has no base asset.
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 0);
    assert_eq!(suite.query_ctoken_balance(LENDER).unwrap().u128(), 0);

    suite.deposit(LENDER, market_token, 100u128).unwrap();

    // After the deposit, the lender has 100 c-token and the contract has 100 base asset.
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);
    assert_eq!(
        suite.ctokens_to_base(suite.query_ctoken_balance(LENDER).unwrap()),
        100
    );
}

#[test]
fn deposit_nothing_fails() {
    // Native
    let market_token = Token::Native(MARKET_TOKEN.to_owned());

    let mut suite = SuiteBuilder::new().with_market_token(market_token).build();

    let err = suite.deposit_multiple_native(LENDER, &[]).unwrap_err();
    assert_eq!(
        ContractError::RequiresExactlyOneCoin {},
        err.downcast().unwrap()
    );
}

#[test]
fn query_transferable_native_amount() {
    // Native
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let market_coin = market_token.clone().into_coin(1_000_000_000_000_000u128);
    let common_coin = common_token.into_coin(1_000_000_000_000_000u128);

    let mut suite = SuiteBuilder::new()
        .with_market_token(market_token.clone())
        .with_funds(LENDER, &[market_coin.clone()])
        .with_collateral_ratio(Decimal::percent(80))
        .build();

    suite.create_pool_with_liquidity_and_twap_price(common_coin, market_coin, PairType::Xyk {});

    // Set zero credit line in mock
    suite
        .set_credit_line(LENDER, CreditLineValues::zero())
        .unwrap();

    let ctoken = suite.ctoken();
    let resp = suite
        .query_transferable_amount(ctoken.clone(), LENDER)
        .unwrap();
    assert_eq!(Uint128::zero(), resp.transferable);

    // Deposit base asset and mint some C tokens, then query again
    suite.deposit(LENDER, market_token, 100).unwrap();

    // Set appropriate credit line
    suite
        .set_credit_line(
            LENDER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.8 collateral ratio
                credit_line: Uint128::new(80),
                borrow_limit: Uint128::new(80),
                debt: Uint128::zero(),
            },
        )
        .unwrap();

    // Transferable amount is equal to collateral
    let resp = suite
        .query_transferable_amount(ctoken.clone(), LENDER)
        .unwrap();
    assert_eq!(Uint128::new(10_000_000), resp.transferable);

    // Set credit line with debt
    suite
        .set_credit_line(
            LENDER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.8 collateral ratio
                credit_line: Uint128::new(80),
                borrow_limit: Uint128::new(80),
                debt: Uint128::new(50),
            },
        )
        .unwrap();

    // Transferable amount is equal to collateral / (credit_line - debt)
    let resp = suite.query_transferable_amount(ctoken, LENDER).unwrap();
    assert_eq!(Uint128::new(37), resp.transferable);

    let err = suite
        .query_transferable_amount("xtoken", LENDER)
        .unwrap_err();
    assert_eq!(
        StdError::generic_err("Querier contract error: Unrecognised token: xtoken".to_owned()),
        err.downcast().unwrap()
    );
}

#[test]
fn query_transferable_cw20_amount() {
    // Native
    let common_token = Token::Native(COMMON.to_string());
    // Cw20
    let market_token = Token::Cw20(WYND.to_owned());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 100))
        .with_market_token(market_token.clone())
        .with_collateral_ratio(Decimal::percent(80))
        .build();

    // Query market token address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    let market_coin = market_token.clone().into_coin(1_000_000_000_000u128);
    let common_coin = common_token.into_coin(1_000_000_000_000u128);

    // Create cw20 tokens pools.
    suite.create_pool_with_liquidity_and_twap_price(common_coin, market_coin, PairType::Lsd {});

    // Set zero credit line in mock
    suite
        .set_credit_line(LENDER, CreditLineValues::zero())
        .unwrap();

    let ctoken = suite.ctoken();
    let resp = suite
        .query_transferable_amount(ctoken.clone(), LENDER)
        .unwrap();
    assert_eq!(Uint128::zero(), resp.transferable);

    // Deposit base asset and mint some C tokens, then query again
    suite.deposit(LENDER, market_token, 100).unwrap();

    // Set appropriate credit line
    suite
        .set_credit_line(
            LENDER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.8 collateral ratio
                credit_line: Uint128::new(80),
                borrow_limit: Uint128::new(80),
                debt: Uint128::zero(),
            },
        )
        .unwrap();

    // Transferable amount is equal to collateral (100 / (80 - 0))
    let resp = suite
        .query_transferable_amount(ctoken.clone(), LENDER)
        .unwrap();
    assert_eq!(Uint128::new(10_000_000), resp.transferable);

    // Set credit line with debt
    suite
        .set_credit_line(
            LENDER,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.8 collateral ratio
                credit_line: Uint128::new(80),
                borrow_limit: Uint128::new(80),
                debt: Uint128::new(50),
            },
        )
        .unwrap();

    // Transferable amount is equal to collateral / (credit_line - debt)
    let resp = suite.query_transferable_amount(ctoken, LENDER).unwrap();
    assert_eq!(Uint128::new(37), resp.transferable);

    let err = suite
        .query_transferable_amount("xtoken", LENDER)
        .unwrap_err();
    assert_eq!(
        StdError::generic_err("Querier contract error: Unrecognised token: xtoken".to_owned()),
        err.downcast().unwrap()
    );
}

#[test]
fn cannot_deposit_native_over_cap() {
    // Addresses
    let user = "user";
    // Native
    let market_token = Token::Native("base".to_owned());

    let market_coin = market_token.clone().into_coin(100u128);

    let mut suite = SuiteBuilder::new()
        .with_market_token(market_token.clone())
        .with_funds(user, &[market_coin])
        .with_cap(90u128)
        .build();

    // Deposit base asset and mint some C tokens, then query again
    suite.deposit(user, market_token.clone(), 80).unwrap();

    // This one pushes things over the cap.
    let err = suite.deposit(user, market_token, 20).unwrap_err();
    assert_eq!(
        ContractError::DepositOverCap {
            attempted_deposit: Uint128::from(20u128),
            ctoken_base_supply: Uint128::from(80u128),
            cap: Uint128::from(90u128)
        },
        err.downcast().unwrap()
    );
}

#[test]
fn cannot_deposit_cw20_over_cap() {
    // Cw20
    let market_token = Token::Cw20("wynd".to_string());

    let mut suite = SuiteBuilder::new()
        .with_initial_cw20(market_token.denom(), (LENDER, 100))
        .with_market_token(market_token)
        .with_cap(90u128)
        .build();

    // We have to change the market token with the real address created during suite build.
    let market_token = suite.market_token.clone();

    suite.deposit(LENDER, market_token.clone(), 80u128).unwrap();

    // This one pushes things over the cap.
    let err = suite.deposit(LENDER, market_token, 20).unwrap_err();
    assert_eq!(
        ContractError::DepositOverCap {
            attempted_deposit: Uint128::from(20u128),
            ctoken_base_supply: Uint128::from(80u128),
            cap: Uint128::from(90u128)
        },
        err.downcast().unwrap()
    );
}
