use crate::tests::{get_permit, init_contract};
use shade_protocol::c_std::{from_binary, Addr};
use shade_protocol::fadroma::ensemble::MockEnv;
use shade_protocol::{
    contract_interfaces::{query_auth, query_auth::ContractStatus},
};
use shade_protocol::utils::asset::Contract;

#[test]
fn set_admin() {
    let (mut chain, auth) = init_contract().unwrap();

    let msg = query_auth::ExecuteMsg::SetAdminAuth {
        admin: Contract {
            address: Addr::from("some_addr"),
            code_hash: "some_hash".to_string()
        },
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("not_admin", auth.clone()))
            .is_err()
    );

    assert!(
        chain
            .execute(&msg, MockEnv::new("admin", auth.clone()))
            .is_ok()
    );

    let query: query_auth::QueryAnswer = chain
        .query(auth.address, &query_auth::QueryMsg::Config {})
        .unwrap();

    match query {
        query_auth::QueryAnswer::Config { admin, .. } => {
            assert_eq!(admin.address, Addr::from("some_addr"));
        }
        _ => assert!(false),
    };
}

#[test]
fn set_runstate() {
    let (mut chain, auth) = init_contract().unwrap();

    let msg = query_auth::ExecuteMsg::SetRunState {
        state: ContractStatus::DisableAll,
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("not_admin", auth.clone()))
            .is_err()
    );

    assert!(
        chain
            .execute(&msg, MockEnv::new("admin", auth.clone()))
            .is_ok()
    );

    let query: query_auth::QueryAnswer = chain
        .query(auth.address, &query_auth::QueryMsg::Config {})
        .unwrap();

    match query {
        query_auth::QueryAnswer::Config { state, .. } => {
            assert_eq!(state, ContractStatus::DisableAll);
        }
        _ => assert!(false),
    };
}

#[test]
fn runstate_block_permits() {
    let (mut chain, auth) = init_contract().unwrap();

    // Validate permits

    let msg = query_auth::ExecuteMsg::SetRunState {
        state: ContractStatus::DisablePermit,
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("admin", auth.clone()))
            .is_ok()
    );

    let msg = query_auth::ExecuteMsg::BlockPermitKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("user", auth.clone()))
            .is_err()
    );

    let msg = query_auth::ExecuteMsg::SetViewingKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("user", auth.clone()))
            .is_ok()
    );

    let msg = query_auth::ExecuteMsg::CreateViewingKey {
        entropy: "random".to_string(),
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("user", auth.clone()))
            .is_ok()
    );

    let res: Result<query_auth::QueryAnswer, shade_protocol::c_std::StdError> = chain.query(
        auth.address.clone(),
        &query_auth::QueryMsg::ValidatePermit {
            permit: get_permit(),
        },
    );

    assert!(res.is_err());

    let res: Result<query_auth::QueryAnswer, shade_protocol::c_std::StdError> = chain.query(
        auth.address.clone(),
        &query_auth::QueryMsg::ValidateViewingKey {
            user: Addr::from("user"),
            key: "key".to_string(),
        },
    );

    assert!(res.is_ok());
}

#[test]
fn runstate_block_vks() {
    let (mut chain, auth) = init_contract().unwrap();

    // Validate permits

    let msg = query_auth::ExecuteMsg::SetRunState {
        state: ContractStatus::DisableVK,
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("admin", auth.clone()))
            .is_ok()
    );

    let msg = query_auth::ExecuteMsg::BlockPermitKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("user", auth.clone()))
            .is_ok()
    );

    let msg = query_auth::ExecuteMsg::SetViewingKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("user", auth.clone()))
            .is_err()
    );

    let msg = query_auth::ExecuteMsg::CreateViewingKey {
        entropy: "random".to_string(),
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("user", auth.clone()))
            .is_err()
    );

    let res: Result<query_auth::QueryAnswer, shade_protocol::c_std::StdError> = chain.query(
        auth.address.clone(),
        &query_auth::QueryMsg::ValidatePermit {
            permit: get_permit(),
        },
    );

    assert!(res.is_ok());

    let res: Result<query_auth::QueryAnswer, shade_protocol::c_std::StdError> = chain.query(
        auth.address.clone(),
        &query_auth::QueryMsg::ValidateViewingKey {
            user: Addr::from("user"),
            key: "key".to_string(),
        },
    );

    assert!(res.is_err());
}

#[test]
fn runstate_block_all() {
    let (mut chain, auth) = init_contract().unwrap();

    // Validate permits

    let msg = query_auth::ExecuteMsg::SetRunState {
        state: ContractStatus::DisableAll,
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("admin", auth.clone()))
            .is_ok()
    );

    let msg = query_auth::ExecuteMsg::BlockPermitKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("user", auth.clone()))
            .is_err()
    );

    let msg = query_auth::ExecuteMsg::SetViewingKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("user", auth.clone()))
            .is_err()
    );

    let msg = query_auth::ExecuteMsg::CreateViewingKey {
        entropy: "random".to_string(),
        padding: None,
    };

    assert!(
        chain
            .execute(&msg, MockEnv::new("user", auth.clone()))
            .is_err()
    );

    let res: Result<query_auth::QueryAnswer, shade_protocol::c_std::StdError> = chain.query(
        auth.address.clone(),
        &query_auth::QueryMsg::ValidatePermit {
            permit: get_permit(),
        },
    );

    assert!(res.is_err());

    let res: Result<query_auth::QueryAnswer, shade_protocol::c_std::StdError> = chain.query(
        auth.address.clone(),
        &query_auth::QueryMsg::ValidateViewingKey {
            user: Addr::from("user"),
            key: "key".to_string(),
        },
    );

    assert!(res.is_err());
}

#[test]
fn set_vk() {
    let (mut chain, auth) = init_contract().unwrap();

    assert!(
        chain
            .execute(
                &query_auth::ExecuteMsg::SetViewingKey {
                    key: "password".to_string(),
                    padding: None
                },
                MockEnv::new("user", auth)
            )
            .is_ok()
    );
}

#[test]
fn create_vk() {
    let (mut chain, auth) = init_contract().unwrap();

    let data = chain
        .execute(
            &query_auth::ExecuteMsg::CreateViewingKey {
                entropy: "randomness".to_string(),
                padding: None,
            },
            MockEnv::new("user", auth.clone()),
        )
        .unwrap()
        .response
        .data
        .unwrap();

    let msg: query_auth::HandleAnswer = from_binary(&data).unwrap();

    let key = match msg {
        query_auth::HandleAnswer::CreateViewingKey { key, .. } => key,
        _ => {
            assert!(false);
            "doesnt_work".to_string()
        }
    };

    let query: query_auth::QueryAnswer = chain
        .query(
            auth.address.clone(),
            &query_auth::QueryMsg::ValidateViewingKey {
                user: Addr::from("user"),
                key,
            },
        )
        .unwrap();

    match query {
        query_auth::QueryAnswer::ValidateViewingKey { is_valid } => {
            assert!(is_valid);
        }
        _ => assert!(false),
    };
}

#[test]
fn block_permit_key() {
    let (mut chain, auth) = init_contract().unwrap();

    let msg = query_auth::ExecuteMsg::BlockPermitKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        chain
            .execute(
                &msg,
                MockEnv::new(
                    "secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq",
                    auth.clone()
                )
            )
            .is_ok()
    );

    let permit = get_permit();

    let query: query_auth::QueryAnswer = chain
        .query(auth.address, &query_auth::QueryMsg::ValidatePermit {
            permit,
        })
        .unwrap();

    match query {
        query_auth::QueryAnswer::ValidatePermit { user: _, is_revoked } => {
            assert!(is_revoked);
        }
        _ => assert!(false),
    };
}
