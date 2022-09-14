use crate::tests::{admin_only_governance, get_assembly_msgs};
use shade_protocol::{c_std::Addr, contract_interfaces::governance, utils::ExecuteCallback};

#[test]
fn add_assembly_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    governance::ExecuteMsg::AddAssemblyMsg {
        name: "Some Assembly name".to_string(),
        msg: "{}".to_string(),
        assemblies: vec![0],
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

    let assemblies = get_assembly_msgs(&mut chain, &gov, 0, 1).unwrap();

    assert_eq!(assemblies.len(), 2);
}

#[test]
fn unauthorised_add_assembly_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(
        governance::ExecuteMsg::AddAssemblyMsg {
            name: "Some Assembly name".to_string(),
            msg: "{}".to_string(),
            assemblies: vec![0],
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
fn set_assembly_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let original_msg = get_assembly_msgs(&mut chain, &gov, 0, 1).unwrap()[0].clone();

    governance::ExecuteMsg::SetAssemblyMsg {
        id: 0,
        name: Some("New name".to_string()),
        msg: None,
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

    let assemblies = get_assembly_msgs(&mut chain, &gov, 0, 1).unwrap();

    assert_eq!(assemblies.len(), 1);

    assert_ne!(original_msg.name, assemblies[0].name);
    assert_eq!(original_msg.assemblies, assemblies[0].assemblies);
    assert_eq!(original_msg.msg, assemblies[0].msg);
}

#[test]
fn unauthorised_set_assembly_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(
        governance::ExecuteMsg::SetAssemblyMsg {
            id: 0,
            name: Some("New name".to_string()),
            msg: None,
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
