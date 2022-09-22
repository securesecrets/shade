use crate::tests::{admin_only_governance, get_contract};
use shade_protocol::{
    c_std::Addr,
    contract_interfaces::governance,
    utils::{asset::Contract, ExecuteCallback},
};

#[test]
fn add_contract() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    governance::ExecuteMsg::AddContract {
        name: "Contract".to_string(),
        metadata: "some description".to_string(),
        contract: Contract {
            address: Addr::unchecked("contract"),
            code_hash: "hash".to_string(),
        },
        assemblies: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    let contracts = get_contract(&mut chain, &gov, 0, 1).unwrap();

    assert_eq!(contracts.len(), 2);
}
#[test]
fn unauthorised_add_contract() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(
        governance::ExecuteMsg::AddContract {
            name: "Contract".to_string(),
            metadata: "some description".to_string(),
            contract: Contract {
                address: Addr::unchecked("contract"),
                code_hash: "hash".to_string(),
            },
            assemblies: None,
            padding: None,
        }
        .test_exec(
            // Sender is self
            &gov,
            &mut chain,
            Addr::unchecked("random"),
            &[]
        )
        .is_err()
    );
}
#[test]
fn set_contract() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    governance::ExecuteMsg::AddContract {
        name: "Contract".to_string(),
        metadata: "some description".to_string(),
        contract: Contract {
            address: Addr::unchecked("contract"),
            code_hash: "hash".to_string(),
        },
        assemblies: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    let old_contract = get_contract(&mut chain, &gov, 1, 1).unwrap()[0].clone();

    governance::ExecuteMsg::SetContract {
        id: 1,
        name: Some("New name".to_string()),
        metadata: Some("New desc".to_string()),
        contract: Some(Contract {
            address: Addr::unchecked("new contract"),
            code_hash: "other hash".to_string(),
        }),
        disable_assemblies: false,
        assemblies: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    let new_contract = get_contract(&mut chain, &gov, 1, 1).unwrap()[0].clone();

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

    governance::ExecuteMsg::AddContract {
        name: "Contract".to_string(),
        metadata: "some description".to_string(),
        contract: Contract {
            address: Addr::unchecked("contract"),
            code_hash: "hash".to_string(),
        },
        assemblies: Some(vec![0]),
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    let old_contract = get_contract(&mut chain, &gov, 1, 1).unwrap()[0].clone();

    governance::ExecuteMsg::SetContract {
        id: 1,
        name: Some("New name".to_string()),
        metadata: Some("New desc".to_string()),
        contract: Some(Contract {
            address: Addr::unchecked("new contract"),
            code_hash: "other hash".to_string(),
        }),
        disable_assemblies: true,
        assemblies: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    let new_contract = get_contract(&mut chain, &gov, 1, 1).unwrap()[0].clone();

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

    governance::ExecuteMsg::AddContract {
        name: "Contract".to_string(),
        metadata: "some description".to_string(),
        contract: Contract {
            address: Addr::unchecked("contract"),
            code_hash: "hash".to_string(),
        },
        assemblies: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    let old_contract = get_contract(&mut chain, &gov, 1, 1).unwrap()[0].clone();

    governance::ExecuteMsg::SetContract {
        id: 1,
        name: Some("New name".to_string()),
        metadata: Some("New desc".to_string()),
        contract: Some(Contract {
            address: Addr::unchecked("new contract"),
            code_hash: "other hash".to_string(),
        }),
        disable_assemblies: false,
        assemblies: Some(vec![0]),
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    let new_contract = get_contract(&mut chain, &gov, 1, 1).unwrap()[0].clone();

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

    assert!(
        governance::ExecuteMsg::SetContract {
            id: 1,
            name: Some("New name".to_string()),
            metadata: Some("New desc".to_string()),
            contract: Some(Contract {
                address: Addr::unchecked("new contract"),
                code_hash: "other hash".to_string(),
            }),
            disable_assemblies: false,
            assemblies: None,
            padding: None,
        }
        .test_exec(
            // Sender is self
            &gov,
            &mut chain,
            Addr::unchecked("random"),
            &[]
        )
        .is_err()
    );
}
#[test]
fn add_contract_assemblies() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    governance::ExecuteMsg::AddContract {
        name: "Contract".to_string(),
        metadata: "some description".to_string(),
        contract: Contract {
            address: Addr::unchecked("contract"),
            code_hash: "hash".to_string(),
        },
        assemblies: Some(vec![0]),
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    let old_contract = get_contract(&mut chain, &gov, 1, 1).unwrap()[0].clone();

    governance::ExecuteMsg::AddContractAssemblies {
        id: 1,
        assemblies: vec![1],
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    let new_contract = get_contract(&mut chain, &gov, 1, 1).unwrap()[0].clone();

    assert_ne!(old_contract.assemblies, new_contract.assemblies);
}
