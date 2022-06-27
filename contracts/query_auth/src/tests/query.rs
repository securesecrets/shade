use crate::{
    tests::{get_permit, init_contract},
};
use cosmwasm_std::{testing::*, HumanAddr, StdResult};
use fadroma::ensemble::MockEnv;
use shade_protocol::contract_interfaces::{
    query_auth,
    query_auth::ContractStatus,
};

#[test]
fn get_config() {
    let (chain, auth) = init_contract().unwrap();

    let query: query_auth::QueryAnswer = chain
        .query(auth.address, &query_auth::QueryMsg::Config {})
        .unwrap();

    match query {
        query_auth::QueryAnswer::Config { admin, state } => {
            assert_eq!(admin.address, HumanAddr::from("admin_contract"));
            assert_eq!(state, ContractStatus::Default);
        }
        _ => assert!(false),
    };
}

#[test]
fn validate_vk() {
    let (mut chain, auth) = init_contract().unwrap();

    let query: query_auth::QueryAnswer = chain
        .query(
            auth.address.clone(),
            &query_auth::QueryMsg::ValidateViewingKey {
                user: HumanAddr::from("user"),
                key: "password".to_string(),
            },
        )
        .unwrap();

    match query {
        query_auth::QueryAnswer::ValidateViewingKey { is_valid } => {
            assert!(!is_valid)
        }
        _ => assert!(false),
    };

    assert!(
        chain
            .execute(
                &query_auth::HandleMsg::SetViewingKey {
                    key: "password".to_string(),
                    padding: None
                },
                MockEnv::new("user", auth.clone())
            )
            .is_ok()
    );

    let query: query_auth::QueryAnswer = chain
        .query(
            auth.address.clone(),
            &query_auth::QueryMsg::ValidateViewingKey {
                user: HumanAddr::from("user"),
                key: "not_password".to_string(),
            },
        )
        .unwrap();

    match query {
        query_auth::QueryAnswer::ValidateViewingKey { is_valid } => {
            assert!(!is_valid);
        }
        _ => assert!(false),
    };

    let query: query_auth::QueryAnswer = chain
        .query(auth.address, &query_auth::QueryMsg::ValidateViewingKey {
            user: HumanAddr::from("user"),
            key: "password".to_string(),
        })
        .unwrap();

    match query {
        query_auth::QueryAnswer::ValidateViewingKey { is_valid } => {
            assert!(is_valid)
        }
        _ => assert!(false),
    };
}

#[test]
fn validate_permit() {
    let mut permit = get_permit();

    let deps = mock_dependencies(20, &[]);

    // Confirm that the permit is valid
    assert!(permit.clone().validate(&deps.api, None).is_ok());

    let (chain, auth) = init_contract().unwrap();

    let query: query_auth::QueryAnswer = chain
        .query(auth.address.clone(), &query_auth::QueryMsg::ValidatePermit {
            permit: permit.clone(),
        })
        .unwrap();

    match query {
        query_auth::QueryAnswer::ValidatePermit { user, is_revoked } => {
            assert!(!is_revoked);
            assert_eq!(
                user,
                HumanAddr::from("secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq")
            )
        }
        _ => assert!(false),
    };

    permit.params.ver = 2;

    let query: StdResult<query_auth::QueryAnswer> = chain
        .query(auth.address, &query_auth::QueryMsg::ValidatePermit {
            permit: permit,
        });

    assert!(query.is_err());
}
