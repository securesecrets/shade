use shade_protocol::c_std::{Coin, HumanAddr};
use shade_protocol::fadroma::ensemble::MockEnv;
use shade_protocol::math_compat::Uint128;
use shade_protocol::contract_interfaces::snip20::{HandleMsg, InitConfig};
use shade_protocol::contract_interfaces::snip20::manager::{Balance, TotalSupply};
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};
use crate::tests::init_snip20_with_config;

#[test]
fn deposit() {
    let (mut chain, snip) = init_snip20_with_config(None, Some(InitConfig{
        public_total_supply: None,
        enable_deposit: Some(true),
        enable_redeem: Some(true),
        enable_mint: None,
        enable_burn: None,
        enable_transfer: None
    })).unwrap();

    let scrt_coin = Coin {
        denom: "uscrt".to_string(),
        amount: shade_protocol::c_std::Uint128(1000)
    };

    let not_coin = Coin {
        denom: "token".to_string(),
        amount: shade_protocol::c_std::Uint128(1000)
    };

    chain.add_funds(HumanAddr::from("Marco"), vec![
        scrt_coin.clone(), not_coin.clone()]);

    // Deposit
    let mut env = MockEnv::new("Marco", snip.clone()).sent_funds(vec![not_coin]);
    assert!(chain.execute(&HandleMsg::Deposit {
        padding: None
    }, env).is_err());

    let mut env = MockEnv::new("Marco", snip.clone()).sent_funds(vec![scrt_coin]);
    assert!(chain.execute(&HandleMsg::Deposit {
        padding: None
    }, env).is_ok());

    // Check that internal states were updated accordingly
    chain.deps(snip.address, |deps| {
        assert_eq!(Balance::load(
            &deps.storage,
            HumanAddr::from("Marco")).unwrap().0, Uint128::new(1000)
        );
        assert_eq!(TotalSupply::load(&deps.storage).unwrap().0, Uint128::new(1000)
        );
    });
}

#[test]
fn redeem() {
    let (mut chain, snip) = init_snip20_with_config(None, Some(InitConfig{
        public_total_supply: None,
        enable_deposit: Some(true),
        enable_redeem: Some(true),
        enable_mint: None,
        enable_burn: None,
        enable_transfer: None
    })).unwrap();

    let scrt_coin = Coin {
        denom: "uscrt".to_string(),
        amount: shade_protocol::c_std::Uint128(1000)
    };

    chain.add_funds(HumanAddr::from("Marco"), vec![
        scrt_coin.clone()]);

    // Deposit
    let mut env = MockEnv::new("Marco", snip.clone()).sent_funds(vec![scrt_coin]);
    assert!(chain.execute(&HandleMsg::Deposit {
        padding: None
    }, env).is_ok());

    // Redeem
    assert!(chain.execute(&HandleMsg::Redeem {
        amount: Uint128::new(10000),
        denom: None,
        padding: None
    }, MockEnv::new("Marco", snip.clone())).is_err());

    assert!(chain.execute(&HandleMsg::Redeem {
        amount: Uint128::new(500),
        denom: None,
        padding: None
    }, MockEnv::new("Marco", snip.clone())).is_ok());
    
    // Check that internal states were updated accordingly
    chain.deps(snip.address, |deps| {
        assert_eq!(Balance::load(
            &deps.storage,
            HumanAddr::from("Marco")).unwrap().0, Uint128::new(500)
        );
        assert_eq!(TotalSupply::load(&deps.storage).unwrap().0, Uint128::new(500)
        );
        let balance = chain.balances(HumanAddr::from("Marco")).unwrap().get("uscrt").unwrap();
        assert_eq!(balance, &shade_protocol::c_std::Uint128(500));
    });
}