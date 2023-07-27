use shade_protocol::c_std::Addr;
use shade_protocol::utils::{ExecuteCallback, Query};
use shade_protocol::c_std::Uint128;
use shade_protocol::contract_interfaces::snip20::{ExecuteMsg, InitialBalance, QueryMsg, QueryAnswer};
use crate::tests::init_snip20_with_config;

#[test]
fn total_supply_overflow() {
    assert!(init_snip20_with_config(Some(vec![
        InitialBalance{
            address: "john".into(),
            amount: Uint128::MAX
        }
    ]), None).is_ok());

    assert!(init_snip20_with_config(Some(vec![
        InitialBalance{
            address: "john".into(),
            amount: (Uint128::MAX - Uint128::new(1))
        },
        InitialBalance {
            address: "salchi".into(),
            amount: Uint128::new(1)
        },
        InitialBalance {
            address: "chonn".into(),
            amount: Uint128::new(1)
        }
    ]), None).is_err());
}

#[test]
fn transfer() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: "bob".into(),
            amount: (Uint128::new(1000))
        },
        InitialBalance {
            address: "dylan".into(),
            amount: Uint128::new(1000)
        },
    ]), None).unwrap();

    assert!(ExecuteMsg::Transfer {
        recipient: "dylan".into(),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[]).is_ok());

    {
        let answer: QueryAnswer = QueryMsg::Balance {
                address: "bob".into(),
                key: "password".into()
            }.test_query(&snip, &chain).unwrap();

        match answer {
            QueryAnswer::Balance {amount} => assert_eq!(amount, Uint128::new(900)),
            _ => assert!(false)
        }

        let answer: QueryAnswer = QueryMsg::Balance {
                address: "dylan".into(),
                key: "password".into()
            }.test_query(&snip, &chain).unwrap();

        match answer {
            QueryAnswer::Balance {amount} => assert_eq!(amount, Uint128::new(1100)),
            _ => assert!(false)
        }
    }

    assert!(ExecuteMsg::Transfer {
        recipient: "dylan".into(),
        amount: Uint128::new(1000),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[]).is_err());
}

#[test]
fn send() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: "bob".into(),
            amount: (Uint128::new(1000))
        },
        InitialBalance {
            address: "dylan".into(),
            amount: Uint128::new(1000)
        },
    ]), None).unwrap();

    assert!(ExecuteMsg::Send {
        recipient: "dylan".into(),
        amount: Uint128::new(100),
        recipient_code_hash: None,
        memo: None,
        padding: None,
        msg: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[]).is_ok());

    {
        let answer: QueryAnswer = QueryMsg::Balance {
                address: "bob".into(),
                key: "password".into()
            }.test_query(&snip, &chain).unwrap();

        match answer {
            QueryAnswer::Balance {amount} => assert_eq!(amount, Uint128::new(900)),
            _ => assert!(false)
        }

        let answer: QueryAnswer = QueryMsg::Balance {
                address: "dylan".into(),
                key: "password".into()
            }.test_query(&snip, &chain).unwrap();

        match answer {
            QueryAnswer::Balance {amount} => assert_eq!(amount, Uint128::new(1100)),
            _ => assert!(false)
        }
    }

    assert!(ExecuteMsg::Send {
        recipient: "dylan".into(),
        amount: Uint128::new(1000),
        recipient_code_hash: None,
        memo: None,
        padding: None,
        msg: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[]).is_err());
}