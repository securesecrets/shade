use shade_protocol::c_std::{Coin, Addr};
use shade_protocol::utils::{ExecuteCallback, Query, MultiTestable};
use shade_protocol::c_std::Uint128;
use shade_protocol::contract_interfaces::snip20::{ExecuteMsg, InitConfig};
use shade_protocol::contract_interfaces::snip20::manager::{Balance, TotalSupply};
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};
use crate::tests::init_snip20_with_config;

#[test]
fn deposit() {
    let (mut chain, snip20) = init_snip20_with_config(None, Some(InitConfig{
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

    let not_coin = Coin {
        denom: "token".into(),
        amount: Uint128::new(1000)
    };

    // chain.add_funds(Addr::unchecked("marco"), vec![
    //     scrt_coin.clone(), not_coin.clone()]);
    chain.init_modules(|router, _, storage| {
        router.bank.init_balance(storage, &Addr::unchecked("marco"), vec![scrt_coin.clone(), not_coin.clone()]).unwrap();
    });

    // Deposit
    assert!(ExecuteMsg::Deposit {
        padding: None
    }.test_exec(&snip20, &mut chain, Addr::unchecked("marco"), &vec![not_coin]).is_err());

    assert!(ExecuteMsg::Deposit {
        padding: None
    }.test_exec(&snip20, &mut chain, Addr::unchecked("marco"), &vec![scrt_coin]).is_ok());

    // Check that internal states were updated accordingly
    chain.deps(&snip20.address, |storage| {
        assert_eq!(Balance::load(
            storage,
            Addr::unchecked("marco")).unwrap().0, Uint128::new(1000)
        );
        assert_eq!(TotalSupply::load(storage).unwrap().0, Uint128::new(1000)
        );
    }).unwrap();
}

#[test]
fn redeem() {
    let (mut chain, snip20) = init_snip20_with_config(None, Some(InitConfig{
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
        router.bank.init_balance(storage, &Addr::unchecked("marco"), vec![scrt_coin.clone()]).unwrap();
    });
    

    // Deposit
    assert!(ExecuteMsg::Deposit {
        padding: None
    }.test_exec(&snip20, &mut chain, Addr::unchecked("marco"), &vec![scrt_coin]).is_ok());

    // Redeem
    assert!(ExecuteMsg::Redeem {
        amount: Uint128::new(10000),
        denom: None,
        padding: None
    }.test_exec(&snip20, &mut chain, Addr::unchecked("marco"), &[]).is_err());

    assert!(ExecuteMsg::Redeem {
        amount: Uint128::new(500),
        denom: None,
        padding: None
    }.test_exec(&snip20, &mut chain, Addr::unchecked("marco"), &[]).is_ok());
    
    // Check that internal states were updated accordingly
    chain.deps(&snip20.address, |storage| {
        assert_eq!(Balance::load(
            storage,
            Addr::unchecked("marco")).unwrap().0, Uint128::new(500)
        );
        assert_eq!(TotalSupply::load(storage).unwrap().0, Uint128::new(500)
        );
    }).unwrap();

    let balance = chain.wrap().query_balance("marco", "uscrt").unwrap();
    assert_eq!(balance.amount, Uint128::new(500));
}