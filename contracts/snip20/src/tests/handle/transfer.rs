use cosmwasm_std::HumanAddr;
use fadroma_ensemble::MockEnv;
use cosmwasm_math_compat::Uint128;
use shade_protocol::contract_interfaces::snip20::{HandleMsg, InitialBalance, QueryMsg, QueryAnswer};
use shade_protocol::contract_interfaces::snip20::manager::Balance;
use crate::tests::init_snip20_with_config;

#[test]
fn total_supply_overflow() {
    assert!(init_snip20_with_config(Some(vec![
        InitialBalance{
            address: HumanAddr::from("John"),
            amount: Uint128::MAX
        }
    ]), None).is_ok());

    assert!(init_snip20_with_config(Some(vec![
        InitialBalance{
            address: HumanAddr::from("John"),
            amount: (Uint128::MAX - Uint128::new(1))
        },
        InitialBalance {
            address: HumanAddr::from("Salchi"),
            amount: Uint128::new(1)
        },
        InitialBalance {
            address: HumanAddr::from("Chonn"),
            amount: Uint128::new(1)
        }
    ]), None).is_err());
}

#[test]
fn transfer() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: HumanAddr::from("Bob"),
            amount: (Uint128::new(1000))
        },
        InitialBalance {
            address: HumanAddr::from("Dylan"),
            amount: Uint128::new(1000)
        },
    ]), None).unwrap();

    assert!(chain.execute(&HandleMsg::Transfer {
        recipient: HumanAddr::from("Dylan"),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }, MockEnv::new("Bob", snip.clone())).is_ok());

    {
        let answer: QueryAnswer = chain.query(
            snip.address.clone(),
            &QueryMsg::Balance {
                address: HumanAddr::from("Bob"),
                key: "password".to_string()
            }
        ).unwrap();

        match answer {
            QueryAnswer::Balance {amount} => assert_eq!(amount, Uint128::new(900)),
            _ => assert!(false)
        }

        let answer: QueryAnswer = chain.query(
            snip.address.clone(),
            &QueryMsg::Balance {
                address: HumanAddr::from("Dylan"),
                key: "password".to_string()
            }
        ).unwrap();

        match answer {
            QueryAnswer::Balance {amount} => assert_eq!(amount, Uint128::new(1100)),
            _ => assert!(false)
        }
    }

    assert!(chain.execute(&HandleMsg::Transfer {
        recipient: HumanAddr::from("Dylan"),
        amount: Uint128::new(1000),
        memo: None,
        padding: None
    }, MockEnv::new("Bob", snip.clone())).is_err());
}

#[test]
fn send() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: HumanAddr::from("Bob"),
            amount: (Uint128::new(1000))
        },
        InitialBalance {
            address: HumanAddr::from("Dylan"),
            amount: Uint128::new(1000)
        },
    ]), None).unwrap();

    assert!(chain.execute(&HandleMsg::Send {
        recipient: HumanAddr::from("Dylan"),
        amount: Uint128::new(100),
        recipient_code_hash: None,
        memo: None,
        padding: None,
        msg: None
    }, MockEnv::new("Bob", snip.clone())).is_ok());

    {
        let answer: QueryAnswer = chain.query(
            snip.address.clone(),
            &QueryMsg::Balance {
                address: HumanAddr::from("Bob"),
                key: "password".to_string()
            }
        ).unwrap();

        match answer {
            QueryAnswer::Balance {amount} => assert_eq!(amount, Uint128::new(900)),
            _ => assert!(false)
        }

        let answer: QueryAnswer = chain.query(
            snip.address.clone(),
            &QueryMsg::Balance {
                address: HumanAddr::from("Dylan"),
                key: "password".to_string()
            }
        ).unwrap();

        match answer {
            QueryAnswer::Balance {amount} => assert_eq!(amount, Uint128::new(1100)),
            _ => assert!(false)
        }
    }

    assert!(chain.execute(&HandleMsg::Send {
        recipient: HumanAddr::from("Dylan"),
        amount: Uint128::new(1000),
        recipient_code_hash: None,
        memo: None,
        padding: None,
        msg: None
    }, MockEnv::new("Bob", snip.clone())).is_err());
}