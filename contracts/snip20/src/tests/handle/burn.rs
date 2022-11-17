use shade_protocol::c_std::{Addr, Timestamp};
use shade_protocol::utils::{ExecuteCallback, Query, MultiTestable};
use shade_protocol::c_std::Uint128;
use shade_protocol::contract_interfaces::snip20::{ExecuteMsg, InitConfig, InitialBalance};
use shade_protocol::contract_interfaces::snip20::batch::BurnFromAction;
use shade_protocol::contract_interfaces::snip20::manager::{Balance, TotalSupply};
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};
use crate::tests::init_snip20_with_config;

#[test]
fn burn() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: "finger".into(),
            amount: (Uint128::new(5000))
        },
    ]), Some(InitConfig {
        public_total_supply: None,
        enable_deposit: None,
        enable_redeem: None,
        enable_mint: None,
        enable_burn: Some(true),
        enable_transfer: None
    })).unwrap();

    chain.update_block(|block| block.time = Timestamp::from_seconds(0));

    // Insufficient tokens
    assert!(ExecuteMsg::Burn {
        amount: Uint128::new(8000),
        padding: None,
        memo: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("finger"), &[]).is_err());

    // Burn some
    assert!(ExecuteMsg::Burn {
        amount: Uint128::new(4000),
        padding: None,
        memo: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("finger"), &[]).is_ok());

    // Check that tokens were spend
    chain.deps(&snip.address, |storage| {
        assert_eq!(Balance::load(
            storage,
            Addr::unchecked("finger")).unwrap().0, Uint128::new(1000)
        );
        assert_eq!(TotalSupply::load(storage).unwrap().0, Uint128::new(1000)
        );
    });

}

#[test]
fn burn_from() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: "sam".into(),
            amount: (Uint128::new(5000))
        },
        InitialBalance {
            address: "esmail".into(),
            amount: Uint128::new(1)
        },
    ]), Some(InitConfig {
        public_total_supply: None,
        enable_deposit: None,
        enable_redeem: None,
        enable_mint: None,
        enable_burn: Some(true),
        enable_transfer: None
    })).unwrap();

    chain.update_block(|block| block.time = Timestamp::from_seconds(0));

    // Insufficient allowance
    assert!(ExecuteMsg::BurnFrom {
        owner: "sam".into(),
        amount: Uint128::new(1000),
        padding: None,
        memo: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());

    assert!(ExecuteMsg::IncreaseAllowance {
        spender: "esmail".into(),
        amount: Uint128::new(700),
        expiration: Some(1_000_000_000),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("sam"), &[]).is_ok());

    // Transfer more than allowed amount
    assert!(ExecuteMsg::BurnFrom {
        owner: "sam".into(),
        amount: Uint128::new(1000),
        padding: None,
        memo: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());

    chain.update_block(|block| block.time = Timestamp::from_seconds(1_000_000_010));
    // Transfer expired
    assert!(ExecuteMsg::BurnFrom {
        owner: "sam".into(),
        amount: Uint128::new(1000),
        padding: None,
        memo: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());

    assert!(ExecuteMsg::IncreaseAllowance {
        spender: "esmail".into(),
        amount: Uint128::new(1000),
        expiration: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("sam"), &[]).is_ok());

    assert!(ExecuteMsg::BurnFrom {
        owner: "sam".into(),
        amount: Uint128::new(800),
        padding: None,
        memo: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_ok());

    // Check that allowance gets spent
    assert!(ExecuteMsg::BurnFrom {
        owner: "sam".into(),
        amount: Uint128::new(300),
        padding: None,
        memo: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());
}

#[test]
fn batch_burn_from() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: "eliot".into(),
            amount: (Uint128::new(5000))
        },
        InitialBalance{
            address: "alderson".into(),
            amount: (Uint128::new(5000))
        },
        InitialBalance{
            address: "sam".into(),
            amount: (Uint128::new(5000))
        },
        InitialBalance {
            address: "esmail".into(),
            amount: Uint128::new(1)
        },
    ]), Some(InitConfig {
        public_total_supply: None,
        enable_deposit: None,
        enable_redeem: None,
        enable_mint: None,
        enable_burn: Some(true),
        enable_transfer: None
    })).unwrap();

    chain.update_block(|block| block.time = Timestamp::from_seconds(0));

    let granters = vec!["eliot", "alderson", "sam"];

    let batch: Vec<_> = granters.iter().map(|name| {
        BurnFromAction {
            owner: (*name).to_string(),
            amount: Uint128::new(800),
            memo: None
        }
    }).collect();

    // Insufficient allowance
    assert!(ExecuteMsg::BatchBurnFrom {
        actions: batch.clone(),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());

    for granter in granters.iter() {
        assert!(ExecuteMsg::IncreaseAllowance {
            spender: "esmail".into(),
            amount: Uint128::new(700),
            expiration: Some(1_000_000_000),
            padding: None
        }.test_exec(&snip, &mut chain, Addr::unchecked(*granter), &[]).is_ok());
    }

    // Transfer more than allowed amount
    assert!(ExecuteMsg::BatchBurnFrom {
        actions: batch.clone(),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());

    chain.update_block(|block| block.time = Timestamp::from_seconds(1_000_000_010));

    // Transfer expired
    assert!(ExecuteMsg::BatchBurnFrom {
        actions: batch.clone(),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());

    for granter in granters.iter() {
        assert!(ExecuteMsg::IncreaseAllowance {
            spender: "esmail".into(),
            amount: Uint128::new(1000),
            expiration: None,
            padding: None
        }.test_exec(&snip, &mut chain, Addr::unchecked(*granter), &[]).is_ok());
    }

    assert!(ExecuteMsg::BatchBurnFrom {
        actions: batch.clone(),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_ok());

    // Check that allowance gets spent
    assert!(ExecuteMsg::BatchBurnFrom {
        actions: batch.clone(),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());
}