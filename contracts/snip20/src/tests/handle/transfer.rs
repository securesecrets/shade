use shade_protocol::c_std::Addr;
use shade_protocol::fadroma::ensemble::MockEnv;
use shade_protocol::c_std::Uint128;
use shade_protocol::contract_interfaces::snip20::{ExecuteMsg, InitialBalance, QueryMsg, QueryAnswer};
use shade_protocol::contract_interfaces::snip20::manager::Balance;
use crate::tests::init_snip20_with_config;

#[test]
fn total_supply_overflow() {
    assert!(init_snip20_with_config(Some(vec![
        InitialBalance{
            address: Addr::from("John"),
            amount: Uint128::MAX
        }
    ]), None).is_ok());

    assert!(init_snip20_with_config(Some(vec![
        InitialBalance{
            address: Addr::from("John"),
            amount: (Uint128::MAX - Uint128::new(1))
        },
        InitialBalance {
            address: Addr::from("Salchi"),
            amount: Uint128::new(1)
        },
        InitialBalance {
            address: Addr::from("Chonn"),
            amount: Uint128::new(1)
        }
    ]), None).is_err());
}

#[test]
fn transfer() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: Addr::from("Bob"),
            amount: (Uint128::new(1000))
        },
        InitialBalance {
            address: Addr::from("Dylan"),
            amount: Uint128::new(1000)
        },
    ]), None).unwrap();

    assert!(chain.execute(&ExecuteMsg::Transfer {
        recipient: Addr::from("Dylan"),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }, MockEnv::new("Bob", snip.clone())).is_ok());

    {
        let answer: QueryAnswer = chain.query(
            snip.address.clone(),
            &QueryMsg::Balance {
                address: Addr::from("Bob"),
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
                address: Addr::from("Dylan"),
                key: "password".to_string()
            }
        ).unwrap();

        match answer {
            QueryAnswer::Balance {amount} => assert_eq!(amount, Uint128::new(1100)),
            _ => assert!(false)
        }
    }

    assert!(chain.execute(&ExecuteMsg::Transfer {
        recipient: Addr::from("Dylan"),
        amount: Uint128::new(1000),
        memo: None,
        padding: None
    }, MockEnv::new("Bob", snip.clone())).is_err());
}

#[test]
fn send() {
    let (mut chain, snip) = init_snip20_with_config(Some(vec![
        InitialBalance{
            address: Addr::from("Bob"),
            amount: (Uint128::new(1000))
        },
        InitialBalance {
            address: Addr::from("Dylan"),
            amount: Uint128::new(1000)
        },
    ]), None).unwrap();

    assert!(chain.execute(&ExecuteMsg::Send {
        recipient: Addr::from("Dylan"),
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
                address: Addr::from("Bob"),
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
                address: Addr::from("Dylan"),
                key: "password".to_string()
            }
        ).unwrap();

        match answer {
            QueryAnswer::Balance {amount} => assert_eq!(amount, Uint128::new(1100)),
            _ => assert!(false)
        }
    }

    assert!(chain.execute(&ExecuteMsg::Send {
        recipient: Addr::from("Dylan"),
        amount: Uint128::new(1000),
        recipient_code_hash: None,
        memo: None,
        padding: None,
        msg: None
    }, MockEnv::new("Bob", snip.clone())).is_err());
}