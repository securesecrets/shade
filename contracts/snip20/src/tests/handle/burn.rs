use shade_protocol::c_std::Addr;
use shade_protocol::fadroma::ensemble::MockEnv;
use shade_protocol::c_std::Uint128;
use shade_protocol::contract_interfaces::snip20::{HandleMsg, InitConfig, InitialBalance};
use shade_protocol::contract_interfaces::snip20::batch::BurnFromAction;
use shade_protocol::contract_interfaces::snip20::manager::{Balance, TotalSupply};
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};
use crate::tests::init_snip20_with_config;

#[test]
fn burn() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: Addr::from("Finger"),
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

    chain.block_mut().time = 0;

    // Insufficient tokens
    assert!(chain.execute(&HandleMsg::Burn {
        amount: Uint128::new(8000),
        padding: None,
        memo: None
    }, MockEnv::new("Finger", snip.clone())).is_err());

    // Burn some
    assert!(chain.execute(&HandleMsg::Burn {
        amount: Uint128::new(4000),
        padding: None,
        memo: None
    }, MockEnv::new("Finger", snip.clone())).is_ok());

    // Check that tokens were spend
    chain.deps(snip.address, |deps| {
        assert_eq!(Balance::load(
            &deps.storage,
            Addr::from("Finger")).unwrap().0, Uint128::new(1000)
        );
        assert_eq!(TotalSupply::load(&deps.storage).unwrap().0, Uint128::new(1000)
        );
    });

}

#[test]
fn burn_from() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: Addr::from("Sam"),
            amount: (Uint128::new(5000))
        },
        InitialBalance {
            address: Addr::from("Esmail"),
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

    chain.block_mut().time = 0;

    // Insufficient allowance
    assert!(chain.execute(&HandleMsg::BurnFrom {
        owner: Addr::from("Sam"),
        amount: Uint128::new(1000),
        padding: None,
        memo: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());

    assert!(chain.execute(&HandleMsg::IncreaseAllowance {
        spender: Addr::from("Esmail"),
        amount: Uint128::new(700),
        expiration: Some(1_000_000_000),
        padding: None
    }, MockEnv::new("Sam", snip.clone())).is_ok());

    // Transfer more than allowed amount
    assert!(chain.execute(&HandleMsg::BurnFrom {
        owner: Addr::from("Sam"),
        amount: Uint128::new(1000),
        padding: None,
        memo: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());

    chain.block_mut().time = 1_000_000_010;

    // Transfer expired
    assert!(chain.execute(&HandleMsg::BurnFrom {
        owner: Addr::from("Sam"),
        amount: Uint128::new(1000),
        padding: None,
        memo: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());

    assert!(chain.execute(&HandleMsg::IncreaseAllowance {
        spender: Addr::from("Esmail"),
        amount: Uint128::new(1000),
        expiration: None,
        padding: None
    }, MockEnv::new("Sam", snip.clone())).is_ok());

    assert!(chain.execute(&HandleMsg::BurnFrom {
        owner: Addr::from("Sam"),
        amount: Uint128::new(800),
        padding: None,
        memo: None
    }, MockEnv::new("Esmail", snip.clone())).is_ok());

    // Check that allowance gets spent
    assert!(chain.execute(&HandleMsg::BurnFrom {
        owner: Addr::from("Sam"),
        amount: Uint128::new(300),
        padding: None,
        memo: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());
}

#[test]
fn batch_burn_from() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: Addr::from("Eliot"),
            amount: (Uint128::new(5000))
        },
        InitialBalance{
            address: Addr::from("Alderson"),
            amount: (Uint128::new(5000))
        },
        InitialBalance{
            address: Addr::from("Sam"),
            amount: (Uint128::new(5000))
        },
        InitialBalance {
            address: Addr::from("Esmail"),
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

    chain.block_mut().time = 0;

    let granters = vec!["Eliot", "Alderson", "Sam"];

    let batch: Vec<_> = granters.iter().map(|name| {
        BurnFromAction {
            owner: Addr::from(*name),
            amount: Uint128::new(800),
            memo: None
        }
    }).collect();

    // Insufficient allowance
    assert!(chain.execute(&HandleMsg::BatchBurnFrom {
        actions: batch.clone(),
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());

    for granter in granters.iter() {
        assert!(chain.execute(&HandleMsg::IncreaseAllowance {
            spender: Addr::from("Esmail"),
            amount: Uint128::new(700),
            expiration: Some(1_000_000_000),
            padding: None
        }, MockEnv::new(*granter, snip.clone())).is_ok());
    }

    // Transfer more than allowed amount
    assert!(chain.execute(&HandleMsg::BatchBurnFrom {
        actions: batch.clone(),
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());

    chain.block_mut().time = 1_000_000_010;

    // Transfer expired
    assert!(chain.execute(&HandleMsg::BatchBurnFrom {
        actions: batch.clone(),
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());

    for granter in granters.iter() {
        assert!(chain.execute(&HandleMsg::IncreaseAllowance {
            spender: Addr::from("Esmail"),
            amount: Uint128::new(1000),
            expiration: None,
            padding: None
        }, MockEnv::new(*granter, snip.clone())).is_ok());
    }

    assert!(chain.execute(&HandleMsg::BatchBurnFrom {
        actions: batch.clone(),
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_ok());

    // Check that allowance gets spent
    assert!(chain.execute(&HandleMsg::BatchBurnFrom {
        actions: batch.clone(),
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());
}