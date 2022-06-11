use cosmwasm_std::{Coin, HumanAddr};
use fadroma_ensemble::MockEnv;
use cosmwasm_math_compat::Uint128;
use shade_protocol::contract_interfaces::snip20::{HandleMsg, InitialBalance, QueryAnswer, QueryMsg};
use shade_protocol::contract_interfaces::snip20::transaction_history::{RichTx, TxAction};
use crate::tests::{create_vk, init_snip20_with_config};

#[test]
fn allowance_vk() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();

    create_vk(&mut chain, &snip, "Saul", None).unwrap();

    chain.execute(&HandleMsg::IncreaseAllowance {
        spender: HumanAddr::from("Goodman"),
        amount: Uint128::new(100),
        expiration: None,
        padding: None
    }, MockEnv::new("Saul", snip.clone())).unwrap();

    let answer: QueryAnswer = chain.query(
        snip.address.clone(),
        &QueryMsg::Allowance {
            owner: HumanAddr::from("Saul"),
            spender: HumanAddr::from("Goodman"),
            key: "password".to_string()
        }
    ).unwrap();

    match answer {
        QueryAnswer::Allowance { spender, owner, allowance, expiration} => {
            assert_eq!(owner, HumanAddr::from("Saul"));
            assert_eq!(spender, HumanAddr::from("Goodman"));
            assert_eq!(allowance, Uint128::new(100));
            assert_eq!(expiration, None);
        },
        _ => assert!(false)
    }
}

#[test]
fn balance_vk() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![InitialBalance {
        address: HumanAddr::from("Robinson"),
        amount: Uint128::new(1500)
    }]), None).unwrap();

    let answer: QueryAnswer = chain.query(
        snip.address.clone(),
        &QueryMsg::Balance {
            address: HumanAddr::from("Robinson"),
            key: "password".to_string()
        }
    ).unwrap();

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
    let (mut chain, snip) = init_snip20_with_config(Some(vec![InitialBalance {
        address: HumanAddr::from("Setsuna"),
        amount: Uint128::new(1500)
    }]), None).unwrap();

    chain.execute(&HandleMsg::Transfer {
        recipient: HumanAddr::from("Stratos"),
        amount: Uint128::new(200),
        memo: None,
        padding: None
    }, MockEnv::new("Setsuna", snip.clone())).unwrap();

    chain.execute(&HandleMsg::Send {
        recipient: HumanAddr::from("Smirnoff"),
        recipient_code_hash: None,
        amount: Uint128::new(140),
        msg: None,
        memo: None,
        padding: None
    }, MockEnv::new("Setsuna", snip.clone())).unwrap();

    chain.execute(&HandleMsg::Transfer {
        recipient: HumanAddr::from("Felt"),
        amount: Uint128::new(300),
        memo: None,
        padding: None
    }, MockEnv::new("Setsuna", snip.clone())).unwrap();

    chain.execute(&HandleMsg::Transfer {
        recipient: HumanAddr::from("Tieria"),
        amount: Uint128::new(540),
        memo: None,
        padding: None
    }, MockEnv::new("Setsuna", snip.clone())).unwrap();

    let answer: QueryAnswer = chain.query(
        snip.address.clone(),
        &QueryMsg::TransactionHistory {
            address: HumanAddr::from("Setsuna"),
            key: "password".to_string(),
            page: None,
            page_size: 10
        }
    ).unwrap();

    match answer {
        QueryAnswer::TransactionHistory { txs, .. } => {
            assert_eq!(txs.len(), 5);

            assert_eq!(txs[0].id, 1);
            assert_eq!(txs[0].action, TxAction::Mint {
                minter: HumanAddr::from("admin"),
                recipient: HumanAddr::from("Setsuna")
            });
            assert_eq!(txs[0].coins, Coin {
                denom: "TKN".to_string(),
                amount: cosmwasm_std::Uint128(1500)
            });

            assert_eq!(txs[1].id, 2);
            assert_eq!(txs[1].action, TxAction::Transfer {
                from: HumanAddr::from("Setsuna"),
                sender: HumanAddr::from("Setsuna"),
                recipient: HumanAddr::from("Stratos")
            });
            assert_eq!(txs[1].coins, Coin {
                denom: "TKN".to_string(),
                amount: cosmwasm_std::Uint128(200)
            });

            assert_eq!(txs[2].id, 3);
            assert_eq!(txs[2].action, TxAction::Transfer {
                from: HumanAddr::from("Setsuna"),
                sender: HumanAddr::from("Setsuna"),
                recipient: HumanAddr::from("Smirnoff")
            });
            assert_eq!(txs[2].coins, Coin {
                denom: "TKN".to_string(),
                amount: cosmwasm_std::Uint128(140)
            });

            assert_eq!(txs[3].id, 4);
            assert_eq!(txs[3].action, TxAction::Transfer {
                from: HumanAddr::from("Setsuna"),
                sender: HumanAddr::from("Setsuna"),
                recipient: HumanAddr::from("Felt")
            });
            assert_eq!(txs[3].coins, Coin {
                denom: "TKN".to_string(),
                amount: cosmwasm_std::Uint128(300)
            });

            assert_eq!(txs[4].id, 5);
            assert_eq!(txs[4].action, TxAction::Transfer {
                from: HumanAddr::from("Setsuna"),
                sender: HumanAddr::from("Setsuna"),
                recipient: HumanAddr::from("Tieria")
            });
            assert_eq!(txs[4].coins, Coin {
                denom: "TKN".to_string(),
                amount: cosmwasm_std::Uint128(540)
            });

        },
        _ => assert!(false)
    }
}