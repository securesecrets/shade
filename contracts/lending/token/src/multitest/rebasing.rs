use crate::multitest::suite::SuiteBuilder;

use super::suite::Suite;
use cosmwasm_std::{Decimal, Uint128};
use utils::amount::token_to_base;

fn query_base_total_supply(suite: &Suite) -> Uint128 {
    let info = suite.query_token_info().unwrap();
    token_to_base(info.total_supply, info.multiplier)
}

#[test]
fn queries() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_transferable(lender, Uint128::new(100))
        .build();
    let controller = suite.controller();
    let controller = controller.as_str();

    let basic_mul = suite.query_multiplier().unwrap();

    // Preparation to have anything to query
    suite
        .mint_base(controller, lender, Uint128::new(100))
        .unwrap();

    // Before rebase we have 100 tokens.
    assert_eq!(
        suite.query_base_balance(lender).unwrap(),
        Uint128::new(100u128)
    );
    assert_eq!(query_base_total_supply(&suite), Uint128::new(100u128));

    // Rebase by 1.2. The "displayed" tokens are now at 120. The multiplier is at 1.2.
    suite.rebase(controller, Decimal::percent(120)).unwrap();
    assert_eq!(
        suite.query_multiplier().unwrap(),
        basic_mul * Decimal::percent(120)
    );
    assert_eq!(
        suite.query_base_balance(lender).unwrap(),
        Uint128::new(120u128)
    );
    assert_eq!(query_base_total_supply(&suite), Uint128::new(120u128));

    // Another rebase by 1.2. The "displayed" tokens are now at 144. The multiplier is at 1.44.
    suite.rebase(controller, Decimal::percent(120)).unwrap();
    assert_eq!(
        suite.query_multiplier().unwrap(),
        basic_mul * Decimal::percent(144)
    );
    assert_eq!(
        suite.query_base_balance(lender).unwrap(),
        Uint128::new(144u128)
    );
    assert_eq!(query_base_total_supply(&suite), Uint128::new(144u128));
}

#[test]
fn mint() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_transferable(lender, Uint128::new(100))
        .build();
    let controller = suite.controller();
    let controller = controller.as_str();

    suite
        .mint_base(controller, lender, Uint128::new(100))
        .unwrap();

    let basic_mul = suite.query_multiplier().unwrap();

    // Rebase by 1.25. The "displayed" tokens are now at 125. The multiplier is at 1.25.
    suite.rebase(controller, Decimal::percent(125)).unwrap();
    assert_eq!(
        suite.query_multiplier().unwrap(),
        basic_mul * Decimal::percent(125)
    );
    assert_eq!(
        suite.query_base_balance(lender).unwrap(),
        Uint128::new(125u128)
    );

    // Mint 20 with the multiplier at 1.25. The "displayed" tokens would be 25.
    suite
        .mint_base(controller, lender, Uint128::new(20))
        .unwrap();

    // Reverse the rebase so that the multiplier is back at 1.0
    suite.rebase(controller, Decimal::percent(80)).unwrap();
    assert_eq!(suite.query_multiplier().unwrap(), basic_mul);
    assert_eq!(
        suite.query_base_balance(lender).unwrap(),
        Uint128::new(116u128)
    );
}

#[test]
fn transfer() {
    let lender = "lender";
    let receiver = "receiver";
    let mut suite = SuiteBuilder::new()
        .with_transferable(lender, Uint128::new(100))
        .build();
    let controller = suite.controller();
    let controller = controller.as_str();

    // Preparation to have anything to transfer
    suite
        .mint_base(controller, lender, Uint128::new(100))
        .unwrap();

    // Rebase by 1.20
    suite.rebase(controller, Decimal::percent(120)).unwrap();

    suite
        .transfer_base(lender, receiver, Uint128::new(24))
        .unwrap();

    assert_eq!(
        suite.query_base_balance(lender).unwrap(),
        Uint128::new(96u128)
    );
    assert_eq!(
        suite.query_base_balance(receiver).unwrap(),
        Uint128::new(24u128)
    );
    assert_eq!(
        suite.query_base_balance(controller).unwrap(),
        Uint128::zero()
    );
}

#[test]
fn burn() {
    let mut suite = Suite::new();
    let controller = suite.controller();
    let controller = controller.as_str();

    let basic_mul = suite.query_multiplier().unwrap();

    // Preparation to have anything to burnground
    suite
        .mint_base(controller, controller, Uint128::new(100))
        .unwrap();

    // Rebase by 1.25, the "displayed" tokens are now at 125.
    suite.rebase(controller, Decimal::percent(125)).unwrap();

    suite
        .burn_base(controller, controller, Uint128::new(25))
        .unwrap();
    assert_eq!(
        suite.query_base_balance(controller).unwrap(),
        Uint128::new(100u128)
    );

    // Reverse the rebase so that the multiplier is back at 1.0
    suite.rebase(controller, Decimal::percent(80)).unwrap();
    assert_eq!(
        suite.query_base_balance(controller).unwrap(),
        Uint128::new(80u128)
    );
    assert_eq!(suite.query_multiplier().unwrap(), basic_mul);
}

#[test]
fn multiplier() {
    let mut suite = Suite::new();
    let controller = suite.controller();
    let controller = controller.as_str();

    let basic_mul = suite.query_multiplier().unwrap();

    // Preparation to have anything to check against
    suite
        .mint(controller, controller, Uint128::new(100))
        .unwrap();

    // Rebase by 1.25
    suite.rebase(controller, Decimal::percent(125)).unwrap();

    assert_eq!(
        suite.query_balance(controller).unwrap(),
        Uint128::new(100),
        "balance should stay the same when rebasing"
    );
    assert_eq!(
        suite.query_multiplier().unwrap(),
        basic_mul * Decimal::percent(125),
        "multiplier should change by rebase ratio"
    );
}
