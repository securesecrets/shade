use shade_protocol::c_std::Addr;
use shade_protocol::utils::{ExecuteCallback, Query, MultiTestable};
use shade_protocol::c_std::Uint128;
use shade_protocol::contract_interfaces::snip20::{ExecuteMsg, InitConfig};
use shade_protocol::contract_interfaces::snip20::manager::{Balance, Minters, TotalSupply};
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};
use crate::tests::init_snip20_with_config;

#[test]
fn mint() {
    let (mut chain, snip) = init_snip20_with_config(None, Some(InitConfig {
        public_total_supply: None,
        enable_deposit: None,
        enable_redeem: None,
        enable_mint: Some(true),
        enable_burn: None,
        enable_transfer: None
    })).unwrap();

    assert!(ExecuteMsg::Mint {
        recipient: "jimmy".into(),
        amount: Uint128::new(1000),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_err());

    assert!(ExecuteMsg::AddMinters {
        minters: vec!["admin".into()],
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    assert!(ExecuteMsg::Mint {
        recipient: "jimmy".into(),
        amount: Uint128::new(1500),
        memo: None,
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    chain.deps(&snip.address, |storage| {
        assert_eq!(Balance::load(
            storage,
            Addr::unchecked("jimmy")).unwrap().0, Uint128::new(1500)
        );
        assert_eq!(TotalSupply::load(storage).unwrap().0, Uint128::new(1500)
        );
    }).unwrap();
}

#[test]
fn set_minters() {
    let (mut chain, snip) = init_snip20_with_config(None, Some(InitConfig {
        public_total_supply: None,
        enable_deposit: None,
        enable_redeem: None,
        enable_mint: Some(true),
        enable_burn: None,
        enable_transfer: None
    })).unwrap();

    assert!(ExecuteMsg::SetMinters {
        minters: vec!["admin".into()],
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("notadmin"), &[]).is_err());

    assert!(ExecuteMsg::SetMinters {
        minters: vec!["admin".into()],
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    chain.deps(&snip.address.clone(), |storage| {
        assert_eq!(Minters::load(storage).unwrap().0, vec![Addr::unchecked("admin")]);
    }).unwrap();

    assert!(ExecuteMsg::SetMinters {
        minters: vec!["other_address".into(), "some_other".into()],
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    chain.deps(&snip.address, |storage| {
        assert_eq!(Minters::load(storage).unwrap().0,
                   vec![Addr::unchecked("other_address"), Addr::unchecked("some_other")]);
    }).unwrap();
}

#[test]
fn add_minters() {
    let (mut chain, snip) = init_snip20_with_config(None, Some(InitConfig {
        public_total_supply: None,
        enable_deposit: None,
        enable_redeem: None,
        enable_mint: Some(true),
        enable_burn: None,
        enable_transfer: None
    })).unwrap();

    assert!(ExecuteMsg::AddMinters {
        minters: vec!["admin".into()],
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("notadmin"), &[]).is_err());

    assert!(ExecuteMsg::AddMinters {
        minters: vec!["admin".into()],
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    chain.deps(&snip.address.clone(), |storage| {
        assert_eq!(Minters::load(storage).unwrap().0, vec![Addr::unchecked("admin")]);
    }).unwrap();

    assert!(ExecuteMsg::AddMinters {
        minters: vec!["other_address".into(), "some_other".into()],
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    chain.deps(&snip.address, |storage| {
        assert_eq!(Minters::load(storage).unwrap().0,
                   vec![
                       Addr::unchecked("admin"),
                       Addr::unchecked("other_address"),
                       Addr::unchecked("some_other")
                   ]);
    }).unwrap();
}

#[test]
fn remove_minters() {
    let (mut chain, snip) = init_snip20_with_config(None, Some(InitConfig {
        public_total_supply: None,
        enable_deposit: None,
        enable_redeem: None,
        enable_mint: Some(true),
        enable_burn: None,
        enable_transfer: None
    })).unwrap();

    assert!(ExecuteMsg::AddMinters {
        minters: vec!["other_address".into(), "some_other".into()],
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    assert!(ExecuteMsg::RemoveMinters {
        minters: vec!["other_address".into()],
        padding: None
    }.test_exec(&snip, &mut chain, Addr::unchecked("admin"), &[]).is_ok());

    chain.deps(&snip.address, |storage| {
        assert_eq!(Minters::load(storage).unwrap().0,
                   vec![
                       Addr::unchecked("some_other")
                   ]);
    }).unwrap();
}