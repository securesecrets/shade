use shade_protocol::c_std::{Coin, Addr};
use shade_protocol::c_std::Uint128;
use shade_protocol::utils::{ExecuteCallback};
use shade_protocol::contract_interfaces::snip20::{ExecuteMsg, InitConfig};
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
    
    assert!(ExecuteMsg::RegisterReceive {
        code_hash: "some_hash".into(),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("contract"), &[]).is_ok());

    chain.deps(&snip.address, |borrowed_chain| {
        let hash = ReceiverHash::load(borrowed_chain, Addr::unchecked("contract")).unwrap();
        assert_eq!(hash.0, "some_hash".to_string());
    }).unwrap();
}

#[test]
fn create_viewing_key() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();

    assert!(ExecuteMsg::CreateViewingKey {
        entropy: "some_entropy".into(),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("sam"), &[]).is_ok());

    chain.deps(&snip.address, |borrowed_chain| {
        assert!(HashedKey::
        may_load(borrowed_chain, Addr::unchecked("sam"))
            .unwrap().is_some());
    }).unwrap();
}

#[test]
fn set_viewing_key() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();

    assert!(ExecuteMsg::SetViewingKey {
        key: "some_key".into(),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("sam"), &[]).is_ok());

    chain.deps(&snip.address, |borrowed_chain| {
        assert!(Key::verify(
            borrowed_chain,
            Addr::unchecked("sam"),
            "some_key".into()
        ).unwrap());
    }).unwrap();
}

#[test]
fn change_admin() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();

    assert!(ExecuteMsg::ChangeAdmin {
        address: "newadmin".into(),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("notadmin"), &[]).is_err());

    assert!(ExecuteMsg::ChangeAdmin {
        address: "newadmin".into(),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    assert!(ExecuteMsg::ChangeAdmin {
        address: "otheradmin".into(),
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_err());
}

#[test]
fn set_contract_status() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();

    assert!(ExecuteMsg::SetContractStatus {
        level: ContractStatusLevel::StopAll,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("notadmin"), &[]).is_err());

    chain.deps(&snip.address.clone(), |storage| {
        assert_eq!(ContractStatusLevel::load(storage).unwrap(), ContractStatusLevel::NormalRun);
    }).unwrap();

    assert!(ExecuteMsg::SetContractStatus {
        level: ContractStatusLevel::StopAll,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    chain.deps(&snip.address, |storage| {
        assert_eq!(ContractStatusLevel::load(storage).unwrap(), ContractStatusLevel::StopAll);
    }).unwrap();
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
        denom: "uscrt".into(),
        amount: Uint128::new(1000)
    };

    chain.init_modules(|router, _, storage| {
        router.bank.init_balance(storage, &Addr::unchecked("bob"), vec![scrt_coin.clone()]).unwrap();
    });

    // Deposit
    assert!(ExecuteMsg::Deposit {
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[scrt_coin]).is_ok());

    assert!(ExecuteMsg::SetContractStatus {
        level: ContractStatusLevel::StopAll,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    assert!(ExecuteMsg::Transfer {
        recipient: "dylan".into(),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[]).is_err());

    assert!(ExecuteMsg::Redeem {
        amount: Uint128::new(100),
        denom: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[]).is_err());

    assert!(ExecuteMsg::SetContractStatus { 
        level: ContractStatusLevel::NormalRun, 
        padding: None 
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    assert!(ExecuteMsg::Transfer {
        recipient: "dylan".into(),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[]).is_ok());

    assert!(ExecuteMsg::Redeem {
        amount: Uint128::new(100),
        denom: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[]).is_ok());
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
        denom: "uscrt".into(),
        amount: Uint128::new(1000)
    };

    chain.init_modules(|router, _, storage| {
        router.bank.init_balance(storage, &Addr::unchecked("bob"), vec![scrt_coin.clone()]).unwrap();
    });

    // Deposit
    assert!(ExecuteMsg::Deposit {
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[scrt_coin]).is_ok());

    assert!(ExecuteMsg::SetContractStatus {
        level: ContractStatusLevel::StopAllButRedeems,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    assert!(ExecuteMsg::Transfer {
        recipient: "dylan".into(),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[]).is_err());

    assert!(ExecuteMsg::Redeem {
        amount: Uint128::new(100),
        denom: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[]).is_ok());

    assert!(ExecuteMsg::SetContractStatus {
        level: ContractStatusLevel::NormalRun,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    assert!(ExecuteMsg::Transfer {
        recipient: "dylan".into(),
        amount: Uint128::new(100),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[]).is_ok());

    assert!(ExecuteMsg::Redeem {
        amount: Uint128::new(100),
        denom: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("bob"), &[]).is_ok());
}