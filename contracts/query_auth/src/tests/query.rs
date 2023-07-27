use crate::tests::{get_config, get_permit, init_contract, validate_permit, validate_vk};
use shade_protocol::{
    c_std::{Addr},
    contract_interfaces::{query_auth, query_auth::ContractStatus},
    utils::{ExecuteCallback},
};

#[test]
fn config() {
    let (chain, auth) = init_contract().unwrap();

    let (_admin, state) = get_config(&chain, &auth).unwrap();

    assert_eq!(state, ContractStatus::Default);
}

#[test]
fn vk_validation() {
    let (mut chain, auth) = init_contract().unwrap();

    assert!(!validate_vk(&chain, &auth, "user", "password").unwrap());

    assert!(
        query_auth::ExecuteMsg::SetViewingKey {
            key: "password".to_string(),
            padding: None
        }
        .test_exec(&auth, &mut chain, Addr::unchecked("user"), &[])
        .is_ok()
    );

    assert!(!validate_vk(&chain, &auth, "user", "not_password").unwrap());

    assert!(validate_vk(&chain, &auth, "user", "password").unwrap());
}

#[test]
fn permit_validation() {
    let _permit = get_permit();

    let (chain, auth) = init_contract().unwrap();

    let (user, is_revoked) = validate_permit(&chain, &auth).unwrap();

    assert!(!is_revoked);
    assert_eq!(
        user,
        Addr::unchecked("secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq")
    );
}
