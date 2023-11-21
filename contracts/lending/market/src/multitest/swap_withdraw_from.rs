use shade_protocol::c_std::Uint128;
use lending_utils::{coin::coin_native, token::Token};
use wyndex::factory::PairType;

use super::suite::{SuiteBuilder, COMMON};
use crate::{
    error::ContractError,
    multitest::suite::{MARKET_TOKEN, USDC, USER},
};

#[test]
fn sender_not_credit_agency() {
    let mut suite = SuiteBuilder::new().build();

    let err = suite
        .swap_withdraw_from(
            "any sender",
            "account",
            Uint128::zero(),
            coin_native(100, "denom"),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::RequiresCreditAgency {},
        err.downcast().unwrap()
    );
}

#[test]
fn two_denoms() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let usdc_token = Token::Native(USDC.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_market_token(market_token.clone())
        .with_funds(USER, &[market_token.clone().into_coin(5_000_000u128)])
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.clone().into_coin(1_000_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000_000u128),
        usdc_token.into_coin(1_000_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(USER, market_token, 5_000_000u128).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 5_000_000);

    let ca = suite.credit_agency();
    // Buy 4.5M USDC, using maximally 5M MARKET_TOKEN tokens for that
    suite
        .swap_withdraw_from(
            ca.clone(),
            USER,
            Uint128::new(5_000_000),
            coin_native(4_500_000, USDC),
        )
        .unwrap();

    let market_balance = suite.query_contract_asset_balance().unwrap();

    // Excluding swap fees, amount left on contract should be less or equal to 0.5M tokens
    assert_eq!(
        market_balance, 455_000,
        "ensure remaining market tokens are 5_000_000 - 4_545_000 (the estimate) * 1.01",
    );

    // Check USDC transferred to the agency.
    let usdc_balance = suite.query_asset_balance(&ca, USDC.to_owned()).unwrap();
    assert_eq!(usdc_balance, 4_500_000);
}

#[test]
fn two_denoms_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let usdc_token = Token::Native(USDC.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_market_token(market_token.clone())
        .with_initial_cw20(market_token.denom(), (USER, 5_000_000))
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.clone().into_coin(1_000_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000_000u128),
        usdc_token.into_coin(1_000_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(USER, market_token, 5_000_000u128).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 5_000_000);

    let ca = suite.credit_agency();
    // Buy 4.5M USDC, using maximally 5M MARKET_TOKEN tokens for that
    suite
        .swap_withdraw_from(
            ca.clone(),
            USER,
            Uint128::new(5_000_000),
            coin_native(4_500_000, USDC),
        )
        .unwrap();

    let market_balance = suite.query_contract_asset_balance().unwrap();

    // Excluding swap fees, amount left on contract should be less or equal to 0.5M tokens
    assert_eq!(
        market_balance, 455_000,
        "ensure remaining market tokens are 5_000_000 - 4_545_000 (the estimate) * 1.01",
    );

    // Check USDC transferred to the agency.
    let usdc_balance = suite.query_asset_balance(&ca, USDC.to_owned()).unwrap();
    assert_eq!(usdc_balance, 4_500_000);
}

#[test]
fn sell_limit_lesser_then_required() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let usdc_token = Token::Native(USDC.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_market_token(market_token.clone())
        .with_funds(USER, &[market_token.clone().into_coin(5_000_000u128)])
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.clone().into_coin(1_000_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000_000u128),
        usdc_token.clone().into_coin(1_000_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(USER, market_token, 5_000_000u128).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 5_000_000);

    let ca = suite.credit_agency();
    // Since price ratio is 1:1, sell limit == buy will fail because of fees
    suite
        .swap_withdraw_from(
            ca,
            USER,
            Uint128::new(4_500_000),
            coin_native(4_500_000, usdc_token.denom()),
        )
        .unwrap_err();
    // TODO: How to assert querier error?
}

#[test]
fn same_denom() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());

    let mut suite = SuiteBuilder::new()
        .with_market_token(market_token.clone())
        .with_funds(USER, &[market_token.clone().into_coin(5_000_000u128)])
        .build();

    suite
        .deposit(USER, market_token.clone(), 5_000_000u128)
        .unwrap();

    let ca = suite.credit_agency();
    suite
        .swap_withdraw_from(
            ca,
            USER,
            Uint128::new(4_500_000),
            coin_native(4_500_000, market_token.denom()),
        )
        .unwrap();

    // Excluding swap fees, amount left on contract should be equal to 0.5M tokens,
    // becase no fees are included
    assert!(matches!(
        suite.query_contract_asset_balance().unwrap(),
        500_000
    ));
}

#[test]
fn buy_common_denom_native() {
    let market_token = Token::Native(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_market_token(market_token.clone())
        .with_funds(USER, &[market_token.clone().into_coin(5_000_000u128)])
        .build();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(USER, market_token, 5_000_000u128).unwrap();

    let ca = suite.credit_agency();
    suite
        .swap_withdraw_from(
            ca.clone(),
            USER,
            Uint128::new(5_000_000),
            coin_native(4_500_000, COMMON),
        )
        .unwrap();

    // Excluding swap fees, amount left on contract should be less or equal to 0.5M tokens
    // Similar as in two_denoms testcase, but here estimate goes through only one LP so fee
    // is twice smaller
    let market_balance = suite.query_contract_asset_balance().unwrap();

    // Excluding swap fees, amount left on contract should be less or equal to 0.5M tokens
    assert_eq!(
        market_balance, 455_000,
        "ensure remaining market tokens are 5_000_000 - 4_545_000 (the estimate) * 1.01",
    );

    // Check USDC transferred to the agency.
    let common_balance = suite.query_asset_balance(&ca, COMMON.to_owned()).unwrap();
    assert_eq!(common_balance, 4_500_000);
}

#[test]
fn buy_common_denom_with_cw20() {
    let market_token = Token::Cw20(MARKET_TOKEN.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_market_token(market_token.clone())
        .with_initial_cw20(market_token.denom(), (USER, 5_000_000))
        .build();

    // recover the cw20 address
    let market_token = suite
        .starting_cw20
        .get(&market_token.denom())
        .unwrap()
        .clone();

    suite.create_pool_with_liquidity_and_twap_price(
        common_token.into_coin(1_000_000_000_000_000u128),
        market_token.clone().into_coin(1_000_000_000_000_000u128),
        PairType::Xyk {},
    );

    suite.deposit(USER, market_token, 5_000_000u128).unwrap();

    let ca = suite.credit_agency();
    suite
        .swap_withdraw_from(
            ca.clone(),
            USER,
            Uint128::new(5_000_000),
            coin_native(4_500_000, COMMON),
        )
        .unwrap();

    // Excluding swap fees, amount left on contract should be less or equal to 0.5M tokens
    // Similar as in two_denoms testcase, but here estimate goes through only one LP so fee
    // is twice smaller
    let market_balance = suite.query_contract_asset_balance().unwrap();

    // Excluding swap fees, amount left on contract should be less or equal to 0.5M tokens
    assert_eq!(
        market_balance, 455_000,
        "ensure remaining market tokens are 5_000_000 - 4_545_000 (the estimate) * 1.01",
    );

    // Check USDC transferred to the agency.
    let common_balance = suite.query_asset_balance(&ca, COMMON.to_owned()).unwrap();
    assert_eq!(common_balance, 4_500_000);
}

#[test]
fn market_uses_common_token() {
    let market_token = Token::Native(COMMON.to_owned());
    let common_token = Token::Native(COMMON.to_string());

    let mut suite = SuiteBuilder::new()
        .with_market_token(common_token.clone())
        .with_funds(USER, &[common_token.clone().into_coin(5_000_000u128)])
        .build();

    suite.deposit(USER, common_token, 5_000_000u128).unwrap();

    let ca = suite.credit_agency();
    suite
        .swap_withdraw_from(
            ca.clone(),
            USER,
            Uint128::new(5_000_000),
            coin_native(4_500_000, market_token.denom()),
        )
        .unwrap();

    // Excluding swap fees, amount left on contract should be less or equal to 0.5M tokens
    // Similar as in buy_common_denom testcase, but here estimate goes through only one LP so fee
    // is twice smaller
    assert!(
        matches!(suite.query_contract_asset_balance().unwrap(), x if x > 485_000 && x <= 500_000)
    );

    // Check USDC transferred to the agency.
    let common_balance = suite.query_asset_balance(&ca, COMMON.to_owned()).unwrap();
    assert_eq!(common_balance, 4_500_000);
}
