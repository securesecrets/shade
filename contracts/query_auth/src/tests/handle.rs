use cosmwasm_std::HumanAddr;
use fadroma::ensemble::MockEnv;
use crate::tests::init_contract;
use shade_protocol::contract_interfaces::query_auth;
use shade_protocol::contract_interfaces::query_auth::ContractStatus;

#[test]
fn set_admin() {
    let (mut chain, auth) = init_contract().unwrap();

    let msg = query_auth::HandleMsg::SetAdmin {
        admin: HumanAddr::from("other_admin"),
        padding: None
    };

    assert!(chain.execute(&msg, MockEnv::new("not_admin", auth.clone())).is_err());

    assert!(chain.execute(&msg, MockEnv::new("admin", auth.clone())).is_ok());

    let query: query_auth::QueryAnswer = chain.query(
        auth.address,
        &query_auth::QueryMsg::Config {}
    ).unwrap();

    match query {
        query_auth::QueryAnswer::Config { admin, .. } => {
            assert_eq!(admin, HumanAddr::from("other_admin"));
        }
        _ => assert!(false)
    };
}

#[test]
fn set_runstate() {
    let (mut chain, auth) = init_contract().unwrap();

    let msg = query_auth::HandleMsg::SetRunState {
        state: ContractStatus::DisableAll,
        padding: None
    };

    assert!(chain.execute(&msg, MockEnv::new("not_admin", auth.clone())).is_err());

    assert!(chain.execute(&msg, MockEnv::new("admin", auth.clone())).is_ok());

    let query: query_auth::QueryAnswer = chain.query(
        auth.address,
        &query_auth::QueryMsg::Config {}
    ).unwrap();

    match query {
        query_auth::QueryAnswer::Config { state, .. } => {
            assert_eq!(state, ContractStatus::DisableAll);
        }
        _ => assert!(false)
    };
}

#[test]
fn runstate_limitations() {
    todo!()
}

#[test]
fn set_vk() {
    let (mut chain, auth) = init_contract().unwrap();

    assert!(chain.execute(&query_auth::HandleMsg::SetViewingKey {
        key: "password".to_string(),
        padding: None
    }, MockEnv::new("user", auth)).is_ok());
}

#[test]
fn create_vk() {
    let (mut chain, auth) = init_contract().unwrap();

    assert!(chain.execute(&query_auth::HandleMsg::CreateViewingKey {
        entropy: "randomness".to_string(),
        padding: None
    }, MockEnv::new("user", auth)).is_ok());
}

#[test]
fn block_permit_key() {
    todo!()
}