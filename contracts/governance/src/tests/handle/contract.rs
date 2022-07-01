use crate::tests::{admin_only_governance, get_contract};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::HumanAddr;
use fadroma::ensemble::MockEnv;
use shade_protocol::{contract_interfaces::governance, utils::asset::Contract};

#[test]
fn add_contract() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::AddContract {
                name: "Contract".to_string(),
                metadata: "some description".to_string(),
                contract: Contract {
                    address: HumanAddr::from("contract"),
                    code_hash: "hash".to_string(),
                },
                assemblies: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let contracts = get_contract(&mut chain, &gov, Uint128::zero(), Uint128::new(1)).unwrap();

    assert_eq!(contracts.len(), 2);
}
#[test]
fn unauthorised_add_contract() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::AddContract {
                name: "Contract".to_string(),
                metadata: "some description".to_string(),
                contract: Contract {
                    address: HumanAddr::from("contract"),
                    code_hash: "hash".to_string(),
                },
                assemblies: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("random"),
                gov.clone(),
            ),
        )
        .is_err();
}
#[test]
fn set_contract() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::AddContract {
                name: "Contract".to_string(),
                metadata: "some description".to_string(),
                contract: Contract {
                    address: HumanAddr::from("contract"),
                    code_hash: "hash".to_string(),
                },
                assemblies: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let old_contract =
        get_contract(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    chain
        .execute(
            &governance::HandleMsg::SetContract {
                id: Uint128::new(1),
                name: Some("New name".to_string()),
                metadata: Some("New desc".to_string()),
                contract: Some(Contract {
                    address: HumanAddr::from("new contract"),
                    code_hash: "other hash".to_string(),
                }),
                disable_assemblies: false,
                assemblies: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let new_contract =
        get_contract(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    assert_ne!(old_contract.name, new_contract.name);
    assert_ne!(old_contract.metadata, new_contract.metadata);
    assert_ne!(old_contract.contract.address, new_contract.contract.address);
    assert_ne!(
        old_contract.contract.code_hash,
        new_contract.contract.code_hash
    );
}

#[test]
fn disable_contract_assemblies() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::AddContract {
                name: "Contract".to_string(),
                metadata: "some description".to_string(),
                contract: Contract {
                    address: HumanAddr::from("contract"),
                    code_hash: "hash".to_string(),
                },
                assemblies: Some(vec![Uint128::zero()]),
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let old_contract =
        get_contract(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    chain
        .execute(
            &governance::HandleMsg::SetContract {
                id: Uint128::new(1),
                name: Some("New name".to_string()),
                metadata: Some("New desc".to_string()),
                contract: Some(Contract {
                    address: HumanAddr::from("new contract"),
                    code_hash: "other hash".to_string(),
                }),
                disable_assemblies: true,
                assemblies: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let new_contract =
        get_contract(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    assert_ne!(old_contract.name, new_contract.name);
    assert_ne!(old_contract.metadata, new_contract.metadata);
    assert_ne!(old_contract.contract.address, new_contract.contract.address);
    assert_ne!(
        old_contract.contract.code_hash,
        new_contract.contract.code_hash
    );
    assert_ne!(old_contract.assemblies, new_contract.assemblies);
}

#[test]
fn enable_contract_assemblies() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::AddContract {
                name: "Contract".to_string(),
                metadata: "some description".to_string(),
                contract: Contract {
                    address: HumanAddr::from("contract"),
                    code_hash: "hash".to_string(),
                },
                assemblies: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let old_contract =
        get_contract(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    chain
        .execute(
            &governance::HandleMsg::SetContract {
                id: Uint128::new(1),
                name: Some("New name".to_string()),
                metadata: Some("New desc".to_string()),
                contract: Some(Contract {
                    address: HumanAddr::from("new contract"),
                    code_hash: "other hash".to_string(),
                }),
                disable_assemblies: false,
                assemblies: Some(vec![Uint128::zero()]),
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let new_contract =
        get_contract(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    assert_ne!(old_contract.name, new_contract.name);
    assert_ne!(old_contract.metadata, new_contract.metadata);
    assert_ne!(old_contract.contract.address, new_contract.contract.address);
    assert_ne!(
        old_contract.contract.code_hash,
        new_contract.contract.code_hash
    );
    assert_ne!(old_contract.assemblies, new_contract.assemblies);
}

#[test]
fn unauthorised_set_contract() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::SetContract {
                id: Uint128::new(1),
                name: Some("New name".to_string()),
                metadata: Some("New desc".to_string()),
                contract: Some(Contract {
                    address: HumanAddr::from("new contract"),
                    code_hash: "other hash".to_string(),
                }),
                disable_assemblies: false,
                assemblies: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("random"),
                gov.clone(),
            ),
        )
        .is_err();
}
#[test]
fn add_contract_assemblies() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::AddContract {
                name: "Contract".to_string(),
                metadata: "some description".to_string(),
                contract: Contract {
                    address: HumanAddr::from("contract"),
                    code_hash: "hash".to_string(),
                },
                assemblies: Some(vec![Uint128::zero()]),
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let old_contract =
        get_contract(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    chain
        .execute(
            &governance::HandleMsg::AddContractAssemblies {
                id: Uint128::new(1),
                assemblies: vec![Uint128::new(1)],
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let new_contract =
        get_contract(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    assert_ne!(old_contract.assemblies, new_contract.assemblies);
}
