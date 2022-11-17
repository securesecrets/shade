use shade_protocol::c_std::{Coin, Addr, Uint128};
use shade_protocol::contract_interfaces::snip20::{ExecuteMsg, InitialBalance, QueryAnswer, QueryMsg};
use shade_protocol::contract_interfaces::snip20::transaction_history::{RichTx, TxAction};
use shade_protocol::query_auth;
use shade_protocol::utils::{ExecuteCallback, InstantiateCallback, Query, MultiTestable};
use crate::tests::{create_vk, init_snip20_with_auth, init_snip20_with_config};

#[test]
fn allowance_vk() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();

    let saul = Addr::unchecked("saul");
    let goodman = Addr::unchecked("goodman");

    create_vk(&mut chain, &snip, "saul", None).unwrap();

    ExecuteMsg::IncreaseAllowance {
        spender: goodman.clone().into_string(),
        amount: Uint128::new(100),
        expiration: None,
        padding: None
    }.test_exec(&snip, &mut chain, saul.clone(), &[]).unwrap();
    
    let answer: QueryAnswer = QueryMsg::Allowance {
        owner: saul.clone().into(),
        spender: goodman.clone().into(),
        key: "password".into()
    }.test_query(&snip, &chain).unwrap();

    match answer {
        QueryAnswer::Allowance { spender, owner, allowance, expiration} => {
            assert_eq!(owner, saul);
            assert_eq!(spender, goodman);
            assert_eq!(allowance, Uint128::new(100));
            assert_eq!(expiration, None);
        },
        _ => assert!(false)
    }
}

#[test]
fn allowance_auth_vk() {
    let (mut chain, snip, auth) = init_snip20_with_auth(None, None, true).unwrap();

    let saul = Addr::unchecked("saul");
    let goodman = Addr::unchecked("goodman");

    query_auth::ExecuteMsg::SetViewingKey {
        key: "password".into(),
        padding: None,
    }.test_exec(&auth.unwrap(), &mut chain, saul.clone(), &[]).unwrap();

    ExecuteMsg::IncreaseAllowance {
        spender: goodman.clone().into_string(),
        amount: Uint128::new(100),
        expiration: None,
        padding: None
    }.test_exec(&snip, &mut chain, saul.clone(), &[]).unwrap();

    let answer: QueryAnswer = QueryMsg::Allowance {
        owner: saul.clone().into(),
        spender: goodman.clone().into(),
        key: "password".into()
    }.test_query(&snip, &chain).unwrap();

    match answer {
        QueryAnswer::Allowance { spender, owner, allowance, expiration} => {
            assert_eq!(owner, saul);
            assert_eq!(spender, goodman);
            assert_eq!(allowance, Uint128::new(100));
            assert_eq!(expiration, None);
        },
        _ => assert!(false)
    }
}

#[test]
fn balance_vk() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![InitialBalance {
        address: "robinson".into(),
        amount: Uint128::new(1500)
    }]), None).unwrap();

    let answer: QueryAnswer = QueryMsg::Balance {
        address: "robinson".into(),
        key: "password".into()
    }.test_query(&snip, &chain).unwrap();

    match answer {
        QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::new(1500));
        },
        _ => assert!(false)
    }
}

#[test]
fn transaction_history() {
    let setsuna = Addr::unchecked("setsuna");
    let stratos = Addr::unchecked("stratos");
    let smirnoff = Addr::unchecked("smirnoff");
    let felt = Addr::unchecked("felt");
    let tieria = Addr::unchecked("tieria");

    let (mut chain, snip) = init_snip20_with_config(Some(vec![InitialBalance {
        address: setsuna.clone().into_string(),
        amount: Uint128::new(1500)
    }]), None).unwrap();

    ExecuteMsg::Transfer {
        recipient: stratos.clone().into_string(),
        amount: Uint128::new(200),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("setsuna"), &[]).unwrap();

    ExecuteMsg::Send {
        recipient: smirnoff.clone().into_string(),
        recipient_code_hash: None,
        amount: Uint128::new(140),
        msg: None,
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("setsuna"), &[]).unwrap();

    ExecuteMsg::Transfer {
        recipient: felt.clone().into_string(),
        amount: Uint128::new(300),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("setsuna"), &[]).unwrap();

    ExecuteMsg::Transfer {
        recipient: tieria.clone().into_string(),
        amount: Uint128::new(540),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("setsuna"), &[]).unwrap();

    let answer: QueryAnswer = QueryMsg::TransactionHistory {
        address: setsuna.clone().into(),
        key: "password".into(),
        page: None,
        page_size: 10
    }.test_query(&snip, &chain).unwrap();

    match answer {
        QueryAnswer::TransactionHistory { txs, .. } => {
            assert_eq!(txs.len(), 5);

            assert_eq!(txs[0].id, 1);
            assert_eq!(txs[0].action, TxAction::Mint {
                minter: Addr::unchecked("admin"),
                recipient: setsuna.clone()
            });
            assert_eq!(txs[0].coins, Coin {
                denom: "TKN".into(),
                amount: Uint128::new(1500)
            });

            assert_eq!(txs[1].id, 2);
            assert_eq!(txs[1].action, TxAction::Transfer {
                from: setsuna.clone(),
                sender: setsuna.clone(),
                recipient: stratos.clone()
            });
            assert_eq!(txs[1].coins, Coin {
                denom: "TKN".into(),
                amount: Uint128::new(200)
            });

            assert_eq!(txs[2].id, 3);
            assert_eq!(txs[2].action, TxAction::Transfer {
                from: setsuna.clone(),
                sender: setsuna.clone(),
                recipient: smirnoff.clone()
            });
            assert_eq!(txs[2].coins, Coin {
                denom: "TKN".into(),
                amount: Uint128::new(140)
            });

            assert_eq!(txs[3].id, 4);
            assert_eq!(txs[3].action, TxAction::Transfer {
                from: setsuna.clone(),
                sender: setsuna.clone(),
                recipient: felt.clone()
            });
            assert_eq!(txs[3].coins, Coin {
                denom: "TKN".into(),
                amount: Uint128::new(300)
            });

            assert_eq!(txs[4].id, 5);
            assert_eq!(txs[4].action, TxAction::Transfer {
                from: setsuna.clone(),
                sender: setsuna.clone(),
                recipient: tieria.clone()
            });
            assert_eq!(txs[4].coins, Coin {
                denom: "TKN".into(),
                amount: Uint128::new(540)
            });

        },
        _ => assert!(false)
    }
}