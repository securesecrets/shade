use shade_protocol::c_std::{Coin, Addr};
use shade_protocol::fadroma::ensemble::MockEnv;
use shade_protocol::c_std::Uint128;
use shade_protocol::contract_interfaces::snip20::{ExecuteMsg, InitConfig, InitialBalance};
use shade_protocol::contract_interfaces::snip20::manager::{ContractStatusLevel, HashedKey, Key, ReceiverHash};
use shade_protocol::utils::storage::plus::MapStorage;
use crate::tests::init_snip20_with_config;

pub mod transfer;
pub mod wrap;
pub mod mint;
pub mod burn;
pub mod allowance;

#[test]
fn register_receive() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();

    assert!(chain.execute(&ExecuteMsg::RegisterReceive {
        code_hash: "some_hash".to_string(),
        padding: None
    }, MockEnv::new("contract", snip.clone())).is_ok());

    chain.deps(snip.address, |borrowed_chain| {
        let hash = ReceiverHash::load(&borrowed_chain.storage, Addr::from("contract")).unwrap();
        assert_eq!(hash.0, "some_hash".to_string());
    }).unwrap();
}

#[test]
fn create_viewing_key() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();

    assert!(chain.execute(&ExecuteMsg::CreateViewingKey {
        entropy: "some_entropy".to_string(),
        padding: None
    }, MockEnv::new("Sam", snip.clone())).is_ok());

    chain.deps(snip.address, |borrowed_chain| {
        assert!(HashedKey::
        may_load(&borrowed_chain.storage, Addr::from("Sam"))
            .unwrap().is_some());
    }).unwrap();
}

#[test]
fn set_viewing_key() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();

    assert!(chain.execute(&ExecuteMsg::SetViewingKey {
        key: "some_key".to_string(),
        padding: None
    }, MockEnv::new("Sam", snip.clone())).is_ok());

    chain.deps(snip.address, |borrowed_chain| {
        assert!(Key::verify(
            &borrowed_chain.storage,
            Addr::from("Sam"),
            "some_key".to_string()
        ).unwrap());
    }).unwrap();
}

#[test]
fn change_admin() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();

    assert!(chain.execute(&ExecuteMsg::ChangeAdmin {
        address: Addr::from("NewAdmin"),
        padding: None
    }, MockEnv::new("NotAdmin", snip.clone())).is_err());

    assert!(chain.execute(&ExecuteMsg::ChangeAdmin {
        address: Addr::from("NewAdmin"),
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    assert!(chain.execute(&ExecuteMsg::ChangeAdmin {
        address: Addr::from("OtherAdmin"),
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_err());
}

#[test]
fn set_contract_status() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();

    assert!(chain.execute(&ExecuteMsg::SetContractStatus {
        level: ContractStatusLevel::StopAll,
        padding: None
    }, MockEnv::new("notAdmin", snip.clone())).is_err());

    chain.deps(snip.address.clone(), |deps| {
        assert_eq!(ContractStatusLevel::load(&deps.storage).unwrap(), ContractStatusLevel::NormalRun);
    });

    assert!(chain.execute(&ExecuteMsg::SetContractStatus {
        level: ContractStatusLevel::StopAll,
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    chain.deps(snip.address, |deps| {
        assert_eq!(ContractStatusLevel::load(&deps.storage).unwrap(), ContractStatusLevel::StopAll);
    });
}

#[test]
fn contract_status_stop_all() {
    let (mut chain, snip) = init_snip20_with_config(None, Some(InitConfig {
        public_total_supply: None,
        enable_deposit: Some(true),
        enable_redeem: Some(true),
        enable_mint: None,
        enable_burn: None,
        enable_transfer: None
    })).unwrap();

    let scrt_coin = Coin {
        denom: "uscrt".to_string(),
        amount: Uint128::new(1000)
    };

    chain.add_funds(Addr::from("Bob"), vec![
        scrt_coin.clone()]);

    // Deposit
    let mut env = MockEnv::new("Bob", snip.clone()).sent_funds(vec![scrt_coin]);
    assert!(chain.execute(&ExecuteMsg::Deposit {
        padding: None
    }, env).is_ok());

    assert!(chain.execute(&ExecuteMsg::SetContractStatus {
        level: ContractStatusLevel::StopAll,
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    assert!(chain.execute(&ExecuteMsg::Transfer {
        recipient: Addr::from("Dylan"),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }, MockEnv::new("Bob", snip.clone())).is_err());

    assert!(chain.execute(&ExecuteMsg::Redeem {
        amount: Uint128::new(100),
        denom: None,
        padding: None
    }, MockEnv::new("Bob", snip.clone())).is_err());

    assert!(chain.execute(&ExecuteMsg::SetContractStatus {
        level: ContractStatusLevel::NormalRun,
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    assert!(chain.execute(&ExecuteMsg::Transfer {
        recipient: Addr::from("Dylan"),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }, MockEnv::new("Bob", snip.clone())).is_ok());

    assert!(chain.execute(&ExecuteMsg::Redeem {
        amount: Uint128::new(100),
        denom: None,
        padding: None
    }, MockEnv::new("Bob", snip.clone())).is_ok());
}

#[test]
fn contract_status_stop_all_but_redeem() {
    let (mut chain, snip) = init_snip20_with_config(None, Some(InitConfig {
        public_total_supply: None,
        enable_deposit: Some(true),
        enable_redeem: Some(true),
        enable_mint: None,
        enable_burn: None,
        enable_transfer: None
    })).unwrap();

    let scrt_coin = Coin {
        denom: "uscrt".to_string(),
        amount: Uint128::new(1000)
    };

    chain.add_funds(Addr::from("Bob"), vec![
        scrt_coin.clone()]);

    // Deposit
    let mut env = MockEnv::new("Bob", snip.clone()).sent_funds(vec![scrt_coin]);
    assert!(chain.execute(&ExecuteMsg::Deposit {
        padding: None
    }, env).is_ok());

    assert!(chain.execute(&ExecuteMsg::SetContractStatus {
        level: ContractStatusLevel::StopAllButRedeems,
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    assert!(chain.execute(&ExecuteMsg::Transfer {
        recipient: Addr::from("Dylan"),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }, MockEnv::new("Bob", snip.clone())).is_err());

    assert!(chain.execute(&ExecuteMsg::Redeem {
        amount: Uint128::new(100),
        denom: None,
        padding: None
    }, MockEnv::new("Bob", snip.clone())).is_ok());

    assert!(chain.execute(&ExecuteMsg::SetContractStatus {
        level: ContractStatusLevel::NormalRun,
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    assert!(chain.execute(&ExecuteMsg::Transfer {
        recipient: Addr::from("Dylan"),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }, MockEnv::new("Bob", snip.clone())).is_ok());

    assert!(chain.execute(&ExecuteMsg::Redeem {
        amount: Uint128::new(100),
        denom: None,
        padding: None
    }, MockEnv::new("Bob", snip.clone())).is_ok());
}