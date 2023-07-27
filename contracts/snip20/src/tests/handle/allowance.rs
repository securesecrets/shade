use shade_protocol::c_std::{Addr, Timestamp};
use shade_protocol::utils::{ExecuteCallback, InstantiateCallback, Query, MultiTestable};
use shade_protocol::c_std::Uint128;
use shade_protocol::contract_interfaces::snip20::{ExecuteMsg, InitialBalance, QueryAnswer, QueryMsg};
use crate::tests::init_snip20_with_config;

#[test]
fn increase_allowance() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: "sam".into(),
            amount: (Uint128::new(5000))
        },
        InitialBalance {
            address: "esmail".into(),
            amount: Uint128::new(1)
        },
    ]), None).unwrap();

    chain.update_block(|block| block.time = Timestamp::from_seconds(0));

    assert!(ExecuteMsg::IncreaseAllowance {
        spender: "esmail".into(),
        amount: Uint128::new(1000),
        expiration: Some(1_000_000_000),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("sam"), &[]).is_ok());

    let answer: QueryAnswer = QueryMsg::Allowance {
            owner: "sam".into(),
            spender: "esmail".into(),
            key: "password".into()
        }.test_query(&snip, &chain).unwrap();

    match answer {
        QueryAnswer::Allowance { spender, owner, allowance, expiration} => {
            assert_eq!(allowance, Uint128::new(1000));
        },
        _ => assert!(false)
    }

    assert!(ExecuteMsg::IncreaseAllowance {
        spender: "esmail".into(),
        amount: Uint128::new(1000),
        expiration: Some(1_000_000_000),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("sam"), &[]).is_ok());

    let answer: QueryAnswer = QueryMsg::Allowance {
            owner: "sam".into(),
            spender: "esmail".into(),
            key: "password".into()
        }.test_query(&snip, &chain).unwrap();

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
            address: "sam".into(),
            amount: (Uint128::new(5000))
        },
        InitialBalance {
            address: "esmail".into(),
            amount: Uint128::new(1)
        },
    ]), None).unwrap();

    chain.update_block(|block| block.time = Timestamp::from_seconds(10000));

    assert!(ExecuteMsg::IncreaseAllowance {
        spender: "esmail".into(),
        amount: Uint128::new(1000),
        expiration: Some(1_000_000_000),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("sam"), &[]).is_ok());

    assert!(ExecuteMsg::DecreaseAllowance {
        spender: "esmail".into(),
        amount: Uint128::new(600),
        expiration: Some(1_000_000_000),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("sam"), &[]).is_ok());

    let answer: QueryAnswer = QueryMsg::Allowance {
            owner: "sam".into(),
            spender: "esmail".into(),
            key: "password".into()
        }.test_query(&snip, &chain).unwrap();

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
            address: "sam".into(),
            amount: (Uint128::new(5000))
        },
        InitialBalance {
            address: "esmail".into(),
            amount: Uint128::new(1)
        },
    ]), None).unwrap();

    chain.update_block(|block| block.time = Timestamp::from_seconds(0));

    // Insufficient allowance
    assert!(ExecuteMsg::TransferFrom {
        owner: "sam".into(),
        recipient: "eliot".into(),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());

    assert!(ExecuteMsg::IncreaseAllowance {
        spender: "esmail".into(),
        amount: Uint128::new(1000),
        expiration: Some(1_000_000_000),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("sam"), &[]).is_ok());

    // Transfer more than allowed amount
    assert!(ExecuteMsg::TransferFrom {
        owner: "sam".into(),
        recipient: "eliot".into(),
        amount: Uint128::new(1100),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());

    chain.update_block(|block| block.time = Timestamp::from_seconds(1_000_000_010));


    // Transfer expired
    assert!(ExecuteMsg::TransferFrom {
        owner: "sam".into(),
        recipient: "eliot".into(),
        amount: Uint128::new(900),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());

    assert!(ExecuteMsg::IncreaseAllowance {
        spender: "esmail".into(),
        amount: Uint128::new(1000),
        expiration: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("sam"), &[]).is_ok());

    assert!(ExecuteMsg::TransferFrom {
        owner: "sam".into(),
        recipient: "eliot".into(),
        amount: Uint128::new(900),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_ok());

    // Check that allowance gets spent
    assert!(ExecuteMsg::TransferFrom {
        owner: "sam".into(),
        recipient: "eliot".into(),
        amount: Uint128::new(200),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());
}

#[test]
fn send_from() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: "sam".into(),
            amount: (Uint128::new(5000))
        },
        InitialBalance {
            address: "esmail".into(),
            amount: Uint128::new(1)
        },
    ]), None).unwrap();

    chain.update_block(|block| block.time = Timestamp::from_seconds(0));

    // Insufficient allowance
    assert!(ExecuteMsg::SendFrom {
        owner: "sam".into(),
        recipient: "eliot".into(),
        recipient_code_hash: None,
        amount: Uint128::new(100),
        msg: None,
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());

    assert!(ExecuteMsg::IncreaseAllowance {
        spender: "esmail".into(),
        amount: Uint128::new(1000),
        expiration: Some(1_000_000_000),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("sam"), &[]).is_ok());

    // Transfer more than allowed amount
    assert!(ExecuteMsg::SendFrom {
        owner: "sam".into(),
        recipient: "eliot".into(),
        recipient_code_hash: None,
        amount: Uint128::new(1100),
        msg: None,
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());

    chain.update_block(|block| block.time = Timestamp::from_seconds(1_000_000_010));

    // Transfer expired
    assert!(ExecuteMsg::SendFrom {
        owner: "sam".into(),
        recipient: "eliot".into(),
        recipient_code_hash: None,
        amount: Uint128::new(900),
        msg: None,
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());

    assert!(ExecuteMsg::IncreaseAllowance {
        spender: "esmail".into(),
        amount: Uint128::new(1000),
        expiration: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("sam"), &[]).is_ok());

    assert!(ExecuteMsg::SendFrom {
        owner: "sam".into(),
        recipient: "eliot".into(),
        recipient_code_hash: None,
        amount: Uint128::new(900),
        msg: None,
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_ok());

    // Check that allowance gets spent
    assert!(ExecuteMsg::SendFrom {
        owner: "sam".into(),
        recipient: "eliot".into(),
        recipient_code_hash: None,
        amount: Uint128::new(200),
        msg: None,
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("esmail"), &[]).is_err());
}
