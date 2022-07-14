use crate::tests::{admin_only_governance, get_assemblies};
use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::Addr;
use shade_protocol::fadroma::ensemble::MockEnv;
use shade_protocol::contract_interfaces::governance;

#[test]
fn add_assembly() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::ExecuteMsg::AddAssembly {
                name: "Other assembly".to_string(),
                metadata: "some data".to_string(),
                members: vec![],
                profile: Uint128::new(1),
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let assemblies = get_assemblies(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap();

    assert_eq!(assemblies.len(), 3);
}

#[test]
fn unauthorised_add_assembly() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::ExecuteMsg::AddAssembly {
                name: "Other assembly".to_string(),
                metadata: "some data".to_string(),
                members: vec![],
                profile: Uint128::new(1),
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                Addr::from("random"),
                gov.clone(),
            ),
        )
        .is_err();
}

#[test]
fn set_assembly() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let old_assembly =
        get_assemblies(&mut chain, &gov, Uint128::new(1), Uint128::new(2)).unwrap()[0].clone();

    chain
        .execute(
            &governance::ExecuteMsg::SetAssembly {
                id: Uint128::new(1),
                name: Some("Random name".to_string()),
                metadata: Some("data".to_string()),
                members: None,
                profile: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let new_assembly =
        get_assemblies(&mut chain, &gov, Uint128::new(1), Uint128::new(2)).unwrap()[0].clone();

    assert_ne!(new_assembly.name, old_assembly.name);
    assert_ne!(new_assembly.metadata, old_assembly.metadata);
    assert_eq!(new_assembly.members, old_assembly.members);
    assert_eq!(new_assembly.profile, old_assembly.profile);
}

#[test]
fn unauthorised_set_assembly() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::ExecuteMsg::SetAssembly {
                id: Uint128::new(1),
                name: Some("Random name".to_string()),
                metadata: Some("data".to_string()),
                members: None,
                profile: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                Addr::from("random"),
                gov.clone(),
            ),
        )
        .is_err();
}
