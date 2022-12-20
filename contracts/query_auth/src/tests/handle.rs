use crate::tests::{get_config, init_contract, validate_permit, validate_vk};
use shade_protocol::{
    c_std::{from_binary, Addr},
    contract_interfaces::{query_auth, query_auth::ContractStatus},
    utils::{asset::Contract, ExecuteCallback},
};

#[test]
fn set_admin() {
    let (mut chain, auth) = init_contract().unwrap();

    let msg = query_auth::ExecuteMsg::SetAdminAuth {
        admin: Contract {
            address: Addr::unchecked("some_addr"),
            code_hash: "some_hash".to_string(),
        },
        padding: None,
    };

    assert!(
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("not_admin"), &[])
            .is_err()
    );

    assert!(
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
            .is_ok()
    );

    match get_config(&chain, &auth) {
        Ok((admin, _)) => assert_eq!(admin.address, Addr::unchecked("some_addr")),
        Err(_) => assert!(false),
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
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("not_admin"), &[])
            .is_err()
    );

    assert!(
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
            .is_ok()
    );

    match get_config(&chain, &auth) {
        Ok((_, state)) => assert_eq!(state, ContractStatus::DisableAll),
        Err(_) => assert!(false),
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
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
            .is_ok()
    );

    let msg = query_auth::ExecuteMsg::BlockPermitKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("user"), &[])
            .is_err()
    );

    let msg = query_auth::ExecuteMsg::SetViewingKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("user"), &[])
            .is_ok()
    );

    let msg = query_auth::ExecuteMsg::CreateViewingKey {
        entropy: "random".to_string(),
        padding: None,
    };

    assert!(
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("user"), &[])
            .is_ok()
    );

    assert!(validate_permit(&chain, &auth).is_err());

    assert!(validate_vk(&chain, &auth, "user", "key").is_ok());
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
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
            .is_ok()
    );

    let msg = query_auth::ExecuteMsg::BlockPermitKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
            .is_ok()
    );

    let msg = query_auth::ExecuteMsg::SetViewingKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
            .is_err()
    );

    let msg = query_auth::ExecuteMsg::CreateViewingKey {
        entropy: "random".to_string(),
        padding: None,
    };

    assert!(
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
            .is_err()
    );

    assert!(validate_permit(&chain, &auth).is_ok());

    assert!(validate_vk(&chain, &auth, "user", "key").is_err());
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
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
            .is_ok()
    );

    let msg = query_auth::ExecuteMsg::BlockPermitKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
            .is_err()
    );

    let msg = query_auth::ExecuteMsg::SetViewingKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
            .is_err()
    );

    let msg = query_auth::ExecuteMsg::CreateViewingKey {
        entropy: "random".to_string(),
        padding: None,
    };

    assert!(
        &msg.test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
            .is_err()
    );

    assert!(validate_permit(&chain, &auth).is_err());

    assert!(validate_vk(&chain, &auth, "user", "key").is_err());
}

#[test]
fn set_vk() {
    let (mut chain, auth) = init_contract().unwrap();

    assert!(
        query_auth::ExecuteMsg::SetViewingKey {
            key: "password".to_string(),
            padding: None
        }
        .test_exec(&auth, &mut chain, Addr::unchecked("user"), &[])
        .is_ok()
    );
}

#[test]
fn create_vk() {
    let (mut chain, auth) = init_contract().unwrap();

    let data = query_auth::ExecuteMsg::CreateViewingKey {
        entropy: "blah".to_string(),
        padding: None,
    }
    .test_exec(&auth, &mut chain, Addr::unchecked("user"), &[])
    .unwrap()
    .data
    .unwrap();

    let msg: query_auth::ExecuteAnswer = from_binary(&data).unwrap();

    let key = match msg {
        query_auth::ExecuteAnswer::CreateViewingKey { key, .. } => key,
        _ => {
            assert!(false);
            "doesnt_work".to_string()
        }
    };

    assert!(validate_vk(&chain, &auth, "user", &key).unwrap());
}

#[test]
fn block_permit_key() {
    let (mut chain, auth) = init_contract().unwrap();

    let msg = query_auth::ExecuteMsg::BlockPermitKey {
        key: "key".to_string(),
        padding: None,
    };

    assert!(
        msg.test_exec(
            &auth,
            &mut chain,
            Addr::unchecked("secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq"),
            &[]
        )
        .is_ok()
    );

    assert!(validate_permit(&chain, &auth).unwrap().1);
}
