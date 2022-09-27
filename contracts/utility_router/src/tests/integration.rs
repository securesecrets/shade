use std::ops::Add;

use crate::{query, tests::*};
use shade_protocol::{
    c_std::{Addr, StdError},
    contract_interfaces::{admin, utility_router},
    utility_router::*,
    utils::{ExecuteCallback, Query},
    Contract,
};

#[test]
fn maintenance_mode_fail() {
    let (mut chain, router, admin) = init_contract().unwrap();
    let treasury = Contract {
        address: Addr::unchecked("treasury"),
        code_hash: "treasury_code_hash".to_string(),
    };

    set_contract(
        &mut chain,
        &router,
        UtilityKey::Treasury,
        treasury.clone(),
        Addr::unchecked("admin"),
    )
    .unwrap();

    // Should work to start
    assert_eq!(
        get_contract(&chain, &router, UtilityKey::Treasury).unwrap(),
        treasury
    );
    assert_eq!(
        get_address(&chain, &router, UtilityKey::Treasury).unwrap(),
        treasury.address
    );

    // Set to maintenance mode
    set_status(
        &mut chain,
        &router,
        RouterStatus::UnderMaintenance,
        Addr::unchecked("admin"),
    )
    .unwrap();

    // Queries should fail
    assert!(get_contract(&chain, &router, UtilityKey::Treasury).is_err());
    assert!(get_address(&chain, &router, UtilityKey::Treasury).is_err());
}

#[test]
fn contract_address_keys_tests() {
    let (mut chain, router, admin) = init_contract().unwrap();

    let contract_keys = vec![
        UtilityKey::Treasury,
        UtilityKey::QueryAuth,
        UtilityKey::OracleRouter,
    ];

    let contracts = vec![
        Contract {
            address: Addr::unchecked("treasury"),
            code_hash: "treasury_code_hash".to_string(),
        },
        Contract {
            address: Addr::unchecked("query_auth"),
            code_hash: "query_auth_code_hash".to_string(),
        },
        Contract {
            address: Addr::unchecked("oracle_router"),
            code_hash: "oracle_router_code_hash".to_string(),
        },
    ];

    let address_keys = vec![UtilityKey::Multisig];

    let addresses = vec![Addr::unchecked("multisig")];

    let mut all_keys = contract_keys.clone();
    all_keys.append(&mut address_keys.clone());

    let mut all_addresses: Vec<Addr> = contracts.iter().map(|c| c.address.clone()).collect();
    all_addresses.append(&mut addresses.clone());

    // Set contracts
    for (key, contract) in contract_keys.clone().iter().zip(contracts.iter()) {
        set_contract(
            &mut chain,
            &router,
            key.clone(),
            contract.clone(),
            Addr::unchecked("admin"),
        )
        .unwrap();
    }

    // Set Addresses
    for (key, address) in address_keys.iter().zip(addresses.iter()) {
        set_address(
            &mut chain,
            &router,
            key.clone(),
            address.clone(),
            Addr::unchecked("admin"),
        )
        .unwrap();
    }

    // Check individual contracts
    for (key, contract) in contract_keys.clone().iter().zip(contracts.iter()) {
        assert_eq!(
            get_contract(&chain, &router, key.clone()).unwrap(),
            contract.clone()
        );
    }

    // Check individual addresses
    for (key, address) in all_keys.iter().zip(all_addresses.iter()) {
        assert_eq!(
            get_address(&chain, &router, key.clone()).unwrap(),
            address.clone()
        );
    }

    // Bulk Contracts
    assert_eq!(
        get_contracts(&chain, &router, contract_keys.clone()).unwrap(),
        contracts
    );

    // Maintains order
    assert_eq!(
        get_contracts(
            &chain,
            &router,
            contract_keys.clone().into_iter().rev().collect()
        )
        .unwrap(),
        contracts.into_iter().rev().collect::<Vec<Contract>>()
    );

    // Bulk addresses
    assert_eq!(
        get_addresses(&chain, &router, all_keys.clone()).unwrap(),
        all_addresses,
    );

    // Maintains order
    assert_eq!(
        get_addresses(
            &chain,
            &router,
            all_keys.clone().into_iter().rev().collect()
        )
        .unwrap(),
        all_addresses.into_iter().rev().collect::<Vec<Addr>>(),
    );

    // Query all keys right amount
    assert_eq!(
        get_keys(&chain, &router, 0, all_keys.len()).unwrap(),
        all_keys
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<String>>(),
    );

    // Query all keys excessive amount
    assert_eq!(
        get_keys(&chain, &router, 0, 100).unwrap(),
        all_keys
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<String>>(),
    );
    assert_eq!(
        get_keys(&chain, &router, 2, 100).unwrap(),
        all_keys[2..]
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<String>>(),
    );

    // Page keys
    // First half
    assert_eq!(
        get_keys(&chain, &router, 0, all_keys.len() / 2).unwrap(),
        all_keys[..all_keys.len() / 2]
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<String>>(),
    );

    // Second half
    let start = all_keys.len() / 2;
    let limit = all_keys.len() / 2;
    assert_eq!(
        get_keys(&chain, &router, start, limit).unwrap(),
        all_keys[start..start + limit]
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<String>>(),
    );
}
