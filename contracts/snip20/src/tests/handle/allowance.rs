use shade_protocol::c_std::Addr;
use shade_protocol::utils::{ExecuteCallback, InstantiateCallback, Query, MultiTestable};
use shade_protocol::c_std::Uint128;
//use shade_multi_test::snip20::Snip20;
use shade_protocol::contract_interfaces::snip20::{ExecuteMsg, InitialBalance, QueryAnswer, QueryMsg};
use crate::tests::init_snip20_with_config;

#[test]
fn increase_allowance() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: Addr::from("Sam"),
            amount: (Uint128::new(5000))
        },
        InitialBalance {
            address: Addr::from("Esmail"),
            amount: Uint128::new(1)
        },
    ]), None).unwrap();

    chain.block_mut().time = 0;

    assert!(chain.execute(&ExecuteMsg::IncreaseAllowance {
        spender: Addr::from("Esmail"),
        amount: Uint128::new(1000),
        expiration: Some(1_000_000_000),
        padding: None
    }, MockEnv::new("Sam", snip.clone())).is_ok());

    let answer: QueryAnswer = chain.query(
        snip.address.clone(),
        &QueryMsg::Allowance {
            owner: Addr::from("Sam"),
            spender: Addr::from("Esmail"),
            key: "password".to_string()
        }
    ).unwrap();

    match answer {
        QueryAnswer::Allowance { spender, owner, allowance, expiration} => {
            assert_eq!(allowance, Uint128::new(1000));
        },
        _ => assert!(false)
    }

    assert!(chain.execute(&ExecuteMsg::IncreaseAllowance {
        spender: Addr::from("Esmail"),
        amount: Uint128::new(1000),
        expiration: Some(1_000_000_000),
        padding: None
    }, MockEnv::new("Sam", snip.clone())).is_ok());

    let answer: QueryAnswer = chain.query(
        snip.address.clone(),
        &QueryMsg::Allowance {
            owner: Addr::from("Sam"),
            spender: Addr::from("Esmail"),
            key: "password".to_string()
        }
    ).unwrap();

    match answer {
        QueryAnswer::Allowance { spender, owner, allowance, expiration} => {
            assert_eq!(allowance, Uint128::new(2000));
        },
        _ => assert!(false)
    }
}

#[test]
fn decrease_allowance() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: Addr::from("Sam"),
            amount: (Uint128::new(5000))
        },
        InitialBalance {
            address: Addr::from("Esmail"),
            amount: Uint128::new(1)
        },
    ]), None).unwrap();

    chain.block_mut().time = 0;

    assert!(chain.execute(&ExecuteMsg::IncreaseAllowance {
        spender: Addr::from("Esmail"),
        amount: Uint128::new(1000),
        expiration: Some(1_000_000_000),
        padding: None
    }, MockEnv::new("Sam", snip.clone())).is_ok());

    assert!(chain.execute(&ExecuteMsg::DecreaseAllowance {
        spender: Addr::from("Esmail"),
        amount: Uint128::new(600),
        expiration: Some(1_000_000_000),
        padding: None
    }, MockEnv::new("Sam", snip.clone())).is_ok());

    let answer: QueryAnswer = chain.query(
        snip.address.clone(),
        &QueryMsg::Allowance {
            owner: Addr::from("Sam"),
            spender: Addr::from("Esmail"),
            key: "password".to_string()
        }
    ).unwrap();

    match answer {
        QueryAnswer::Allowance { spender, owner, allowance, expiration} => {
            assert_eq!(allowance, Uint128::new(400));
        },
        _ => assert!(false)
    }
}

#[test]
fn transfer_from() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: Addr::from("Sam"),
            amount: (Uint128::new(5000))
        },
        InitialBalance {
            address: Addr::from("Esmail"),
            amount: Uint128::new(1)
        },
    ]), None).unwrap();

    chain.block_mut().time = 0;

    // Insufficient allowance
    assert!(chain.execute(&ExecuteMsg::TransferFrom {
        owner: Addr::from("Sam"),
        recipient: Addr::from("Eliot"),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());

    assert!(chain.execute(&ExecuteMsg::IncreaseAllowance {
        spender: Addr::from("Esmail"),
        amount: Uint128::new(1000),
        expiration: Some(1_000_000_000),
        padding: None
    }, MockEnv::new("Sam", snip.clone())).is_ok());

    // Transfer more than allowed amount
    assert!(chain.execute(&ExecuteMsg::TransferFrom {
        owner: Addr::from("Sam"),
        recipient: Addr::from("Eliot"),
        amount: Uint128::new(1100),
        memo: None,
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());

    chain.block_mut().time = 1_000_000_010;

    // Transfer expired
    assert!(chain.execute(&ExecuteMsg::TransferFrom {
        owner: Addr::from("Sam"),
        recipient: Addr::from("Eliot"),
        amount: Uint128::new(900),
        memo: None,
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());

    assert!(chain.execute(&ExecuteMsg::IncreaseAllowance {
        spender: Addr::from("Esmail"),
        amount: Uint128::new(1000),
        expiration: None,
        padding: None
    }, MockEnv::new("Sam", snip.clone())).is_ok());

    assert!(chain.execute(&ExecuteMsg::TransferFrom {
        owner: Addr::from("Sam"),
        recipient: Addr::from("Eliot"),
        amount: Uint128::new(900),
        memo: None,
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_ok());

    // Check that allowance gets spent
    assert!(chain.execute(&ExecuteMsg::TransferFrom {
        owner: Addr::from("Sam"),
        recipient: Addr::from("Eliot"),
        amount: Uint128::new(200),
        memo: None,
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());
}

#[test]
fn send_from() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: Addr::from("Sam"),
            amount: (Uint128::new(5000))
        },
        InitialBalance {
            address: Addr::from("Esmail"),
            amount: Uint128::new(1)
        },
    ]), None).unwrap();

    chain.block_mut().time = 0;

    // Insufficient allowance
    assert!(chain.execute(&ExecuteMsg::SendFrom {
        owner: Addr::from("Sam"),
        recipient: Addr::from("Eliot"),
        recipient_code_hash: None,
        amount: Uint128::new(100),
        msg: None,
        memo: None,
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());

    assert!(chain.execute(&ExecuteMsg::IncreaseAllowance {
        spender: Addr::from("Esmail"),
        amount: Uint128::new(1000),
        expiration: Some(1_000_000_000),
        padding: None
    }, MockEnv::new("Sam", snip.clone())).is_ok());

    // Transfer more than allowed amount
    assert!(chain.execute(&ExecuteMsg::SendFrom {
        owner: Addr::from("Sam"),
        recipient: Addr::from("Eliot"),
        recipient_code_hash: None,
        amount: Uint128::new(1100),
        msg: None,
        memo: None,
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());

    chain.block_mut().time = 1_000_000_010;

    // Transfer expired
    assert!(chain.execute(&ExecuteMsg::SendFrom {
        owner: Addr::from("Sam"),
        recipient: Addr::from("Eliot"),
        recipient_code_hash: None,
        amount: Uint128::new(900),
        msg: None,
        memo: None,
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());

    assert!(chain.execute(&ExecuteMsg::IncreaseAllowance {
        spender: Addr::from("Esmail"),
        amount: Uint128::new(1000),
        expiration: None,
        padding: None
    }, MockEnv::new("Sam", snip.clone())).is_ok());

    assert!(chain.execute(&ExecuteMsg::SendFrom {
        owner: Addr::from("Sam"),
        recipient: Addr::from("Eliot"),
        recipient_code_hash: None,
        amount: Uint128::new(900),
        msg: None,
        memo: None,
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_ok());

    // Check that allowance gets spent
    assert!(chain.execute(&ExecuteMsg::SendFrom {
        owner: Addr::from("Sam"),
        recipient: Addr::from("Eliot"),
        recipient_code_hash: None,
        amount: Uint128::new(200),
        msg: None,
        memo: None,
        padding: None
    }, MockEnv::new("Esmail", snip.clone())).is_err());
}
