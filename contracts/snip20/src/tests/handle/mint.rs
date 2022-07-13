use shade_protocol::c_std::Addr;
use shade_protocol::fadroma::ensemble::MockEnv;
use shade_protocol::c_std::Uint128;
use shade_protocol::contract_interfaces::snip20::{HandleMsg, InitConfig};
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

    assert!(chain.execute(&HandleMsg::Mint {
        recipient: Addr::from("Jimmy"),
        amount: Uint128::new(1000),
        memo: None,
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_err());

    assert!(chain.execute(&HandleMsg::AddMinters {
        minters: vec![Addr::from("admin")],
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    assert!(chain.execute(&HandleMsg::Mint {
        recipient: Addr::from("Jimmy"),
        amount: Uint128::new(1500),
        memo: None,
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    chain.deps(snip.address, |deps| {
        assert_eq!(Balance::load(
            &deps.storage,
            Addr::from("Jimmy")).unwrap().0, Uint128::new(1500)
        );
        assert_eq!(TotalSupply::load(&deps.storage).unwrap().0, Uint128::new(1500)
        );
    });
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

    assert!(chain.execute(&HandleMsg::SetMinters {
        minters: vec![Addr::from("admin")],
        padding: None
    }, MockEnv::new("notAdmin", snip.clone())).is_err());

    assert!(chain.execute(&HandleMsg::SetMinters {
        minters: vec![Addr::from("admin")],
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    chain.deps(snip.address.clone(), |deps| {
        assert_eq!(Minters::load(&deps.storage).unwrap().0, vec![Addr::from("admin")]);
    });

    assert!(chain.execute(&HandleMsg::SetMinters {
        minters: vec![Addr::from("other_address"), Addr::from("some_other")],
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    chain.deps(snip.address, |deps| {
        assert_eq!(Minters::load(&deps.storage).unwrap().0,
                   vec![Addr::from("other_address"), Addr::from("some_other")]);
    });
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

    assert!(chain.execute(&HandleMsg::AddMinters {
        minters: vec![Addr::from("admin")],
        padding: None
    }, MockEnv::new("notAdmin", snip.clone())).is_err());

    assert!(chain.execute(&HandleMsg::AddMinters {
        minters: vec![Addr::from("admin")],
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    chain.deps(snip.address.clone(), |deps| {
        assert_eq!(Minters::load(&deps.storage).unwrap().0, vec![Addr::from("admin")]);
    });

    assert!(chain.execute(&HandleMsg::AddMinters {
        minters: vec![Addr::from("other_address"), Addr::from("some_other")],
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    chain.deps(snip.address, |deps| {
        assert_eq!(Minters::load(&deps.storage).unwrap().0,
                   vec![
                       Addr::from("admin"),
                       Addr::from("other_address"),
                       Addr::from("some_other")
                   ]);
    });
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

    assert!(chain.execute(&HandleMsg::AddMinters {
        minters: vec![Addr::from("other_address"), Addr::from("some_other")],
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    assert!(chain.execute(&HandleMsg::RemoveMinters {
        minters: vec![Addr::from("other_address")],
        padding: None
    }, MockEnv::new("admin", snip.clone())).is_ok());

    chain.deps(snip.address, |deps| {
        assert_eq!(Minters::load(&deps.storage).unwrap().0,
                   vec![
                       Addr::from("some_other")
                   ]);
    });
}