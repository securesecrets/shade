use shade_protocol::c_std::{Coin, Addr, Uint128};
use shade_protocol::contract_interfaces::snip20::{ExecuteMsg, InitialBalance, QueryAnswer, QueryMsg};
use shade_protocol::contract_interfaces::snip20::transaction_history::{RichTx, TxAction};
use shade_protocol::utils::{ExecuteCallback, InstantiateCallback, Query, MultiTestable};
use crate::tests::{create_vk, init_snip20_with_config};

#[test]
fn allowance_vk() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();

    let saul = Addr::unchecked("Saul");
    let goodman = Addr::unchecked("Goodman");

    create_vk(&mut chain, &snip, "Saul", None).unwrap();

    ExecuteMsg::IncreaseAllowance {
        spender: goodman.clone(),
        amount: Uint128::new(100),
        expiration: None,
        padding: None
    }.test_exec(&snip, &mut chain, saul.clone(), &[]).unwrap();
    
    let answer: QueryAnswer = QueryMsg::Allowance {
        owner: saul.clone(),
        spender: goodman.clone(),
        key: "password".to_string()
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
        address: Addr::unchecked("Robinson"),
        amount: Uint128::new(1500)
    }]), None).unwrap();

    let answer: QueryAnswer = QueryMsg::Balance {
        address: Addr::unchecked("Robinson"),
        key: "password".to_string()
    }.test_query(&snip, &chain).unwrap();

    match answer {
        QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::new(1500));
        },
        _ => assert!(false)
    }
}

// y

#[test]
fn transaction_history() {
    let setsuna = Addr::unchecked("Setsuna");
    let stratos = Addr::unchecked("Stratos");
    let smirnoff = Addr::unchecked("Smirnoff");
    let felt = Addr::unchecked("Felt");
    let tieria = Addr::unchecked("Tieria");

    let (mut chain, snip) = init_snip20_with_config(Some(vec![InitialBalance {
        address: setsuna.clone(),
        amount: Uint128::new(1500)
    }]), None).unwrap();

    ExecuteMsg::Transfer {
        recipient: stratos.clone(),
        amount: Uint128::new(200),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, setsuna.clone(), &[]).unwrap();

    ExecuteMsg::Send {
        recipient: smirnoff.clone(),
        recipient_code_hash: None,
        amount: Uint128::new(140),
        msg: None,
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, setsuna.clone(), &[]).unwrap();

    ExecuteMsg::Transfer {
        recipient: felt.clone(),
        amount: Uint128::new(300),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, setsuna.clone(), &[]).unwrap();

    ExecuteMsg::Transfer {
        recipient: tieria.clone(),
        amount: Uint128::new(540),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, setsuna.clone(), &[]).unwrap();

    let answer: QueryAnswer = QueryMsg::TransactionHistory {
        address: setsuna.clone(),
        key: "password".to_string(),
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
                denom: "TKN".to_string(),
                amount: Uint128::new(1500)
            });

            assert_eq!(txs[1].id, 2);
            assert_eq!(txs[1].action, TxAction::Transfer {
                from: setsuna.clone(),
                sender: setsuna.clone(),
                recipient: stratos.clone()
            });
            assert_eq!(txs[1].coins, Coin {
                denom: "TKN".to_string(),
                amount: Uint128::new(200)
            });

            assert_eq!(txs[2].id, 3);
            assert_eq!(txs[2].action, TxAction::Transfer {
                from: setsuna.clone(),
                sender: setsuna.clone(),
                recipient: smirnoff.clone()
            });
            assert_eq!(txs[2].coins, Coin {
                denom: "TKN".to_string(),
                amount: Uint128::new(140)
            });

            assert_eq!(txs[3].id, 4);
            assert_eq!(txs[3].action, TxAction::Transfer {
                from: setsuna.clone(),
                sender: setsuna.clone(),
                recipient: felt.clone()
            });
            assert_eq!(txs[3].coins, Coin {
                denom: "TKN".to_string(),
                amount: Uint128::new(300)
            });

            assert_eq!(txs[4].id, 5);
            assert_eq!(txs[4].action, TxAction::Transfer {
                from: setsuna.clone(),
                sender: setsuna.clone(),
                recipient: tieria.clone()
            });
            assert_eq!(txs[4].coins, Coin {
                denom: "TKN".to_string(),
                amount: Uint128::new(540)
            });

        },
        _ => assert!(false)
    }
}