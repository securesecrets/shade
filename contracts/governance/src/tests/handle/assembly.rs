use crate::tests::{admin_only_governance, get_assemblies};
use shade_protocol::{c_std::Addr, contract_interfaces::governance, utils::ExecuteCallback};

#[test]
fn add_assembly() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    governance::ExecuteMsg::AddAssembly {
        name: "Other assembly".to_string(),
        metadata: "some data".to_string(),
        members: vec![],
        profile: 1,
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

    let assemblies = get_assemblies(&mut chain, &gov, 0, 2).unwrap();

    assert_eq!(assemblies.len(), 3);
}

#[test]
fn unauthorised_add_assembly() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(
        governance::ExecuteMsg::AddAssembly {
            name: "Other assembly".to_string(),
            metadata: "some data".to_string(),
            members: vec![],
            profile: 1,
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
    )
}

#[test]
fn set_assembly() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let old_assembly = get_assemblies(&mut chain, &gov, 1, 2).unwrap()[0].clone();

    governance::ExecuteMsg::SetAssembly {
        id: 1,
        name: Some("Random name".to_string()),
        metadata: Some("data".to_string()),
        members: None,
        profile: None,
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

    let new_assembly = get_assemblies(&mut chain, &gov, 1, 2).unwrap()[0].clone();

    assert_ne!(new_assembly.name, old_assembly.name);
    assert_ne!(new_assembly.metadata, old_assembly.metadata);
    assert_eq!(new_assembly.members, old_assembly.members);
    assert_eq!(new_assembly.profile, old_assembly.profile);
}

#[test]
fn unauthorised_set_assembly() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(
        governance::ExecuteMsg::SetAssembly {
            id: 1,
            name: Some("Random name".to_string()),
            metadata: Some("data".to_string()),
            members: None,
            profile: None,
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
    )
}
