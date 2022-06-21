use cosmwasm_std::{Binary, from_binary, HumanAddr, Uint128};
use fadroma::ensemble::MockEnv;
use crate::tests::init_contract;
use cosmwasm_std::testing::*;
use crate::contract::{init, query};
use shade_protocol::contract_interfaces::query_auth;
use shade_protocol::contract_interfaces::query_auth::{ContractStatus, PermitData, QueryPermit};
use query_authentication::transaction::{PubKey, PermitSignature};

#[test]
fn get_config() {
    let (mut chain, auth) = init_contract().unwrap();

    let query: query_auth::QueryAnswer = chain.query(
        auth.address,
        &query_auth::QueryMsg::Config {}
    ).unwrap();

    match query {
        query_auth::QueryAnswer::Config { admin, state } => {
            assert_eq!(admin, HumanAddr::from("admin"));
            assert_eq!(state, ContractStatus::Default);
        }
        _ => assert!(false)
    };
}

#[test]
fn validate_vk() {
    let (mut chain, auth) = init_contract().unwrap();

    let query: query_auth::QueryAnswer = chain.query(
        auth.address.clone(),
        &query_auth::QueryMsg::ValidateViewingKey {
            user: HumanAddr::from("user"),
            key: "password".to_string()
        }
    ).unwrap();

    match query {
        query_auth::QueryAnswer::ValidateViewingKey { is_valid } => {
            assert!(!is_valid)
        }
        _ => assert!(false)
    };

    assert!(chain.execute(&query_auth::HandleMsg::SetViewingKey {
        key: "password".to_string(),
        padding: None
    }, MockEnv::new("user", auth.clone())).is_ok());

    let query: query_auth::QueryAnswer = chain.query(
        auth.address.clone(),
        &query_auth::QueryMsg::ValidateViewingKey {
            user: HumanAddr::from("user"),
            key: "not_password".to_string()
        }
    ).unwrap();

    match query {
        query_auth::QueryAnswer::ValidateViewingKey { is_valid } => {
            assert!(!is_valid);
        }
        _ => assert!(false)
    };

    let query: query_auth::QueryAnswer = chain.query(
        auth.address,
        &query_auth::QueryMsg::ValidateViewingKey {
            user: HumanAddr::from("user"),
            key: "password".to_string()
        }
    ).unwrap();

    match query {
        query_auth::QueryAnswer::ValidateViewingKey { is_valid } => {
            assert!(is_valid)
        }
        _ => assert!(false)
    };
}

#[test]
fn validate_permit() {
    let permit = QueryPermit {
        params: PermitData {
            key: "key".to_string(),
            data: Binary::from_base64("c29tZSBzdHJpbmc=").unwrap()
        },
        signature: PermitSignature {
            pub_key: PubKey::new(
                Binary::from_base64(
                    "A9NjbriiP7OXCpoTov9ox/35+h5k0y1K0qCY/B09YzAP"
                ).unwrap()
            ),
            signature: Binary::from_base64(
                "XRzykrPmMs0ZhksNXX+eU0TM21fYBZXZogr5wYZGGy11t2ntfySuQNQJEw6D4QKvPsiU9gYMsQ259dOzMZNAEg=="
            ).unwrap()
        },
        account_number: None,
        chain_id: Some(String::from("chain")),
        sequence: None,
        memo: None
    };

    // Confirm that the permit is valid
    assert!(permit.clone().validate(None).is_ok());

    let (mut chain, auth) = init_contract().unwrap();

    let query: query_auth::QueryAnswer = chain.query(
        auth.address,
        &query_auth::QueryMsg::ValidatePermit {
            permit
        }
    ).unwrap();

    match query {
        query_auth::QueryAnswer::ValidatePermit { user, is_revoked } => {
            assert!(!is_revoked);
            assert_eq!(user, HumanAddr::from("secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq"))
        }
        _ => assert!(false)
    };
}