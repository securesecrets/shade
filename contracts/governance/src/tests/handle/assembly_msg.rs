use crate::tests::{admin_only_governance, get_assembly_msgs};
use shade_protocol::{
    contract_interfaces::{governance, governance::assembly::AssemblyMsg},
    fadroma::ensemble::MockEnv,
    math_compat::Uint128,
};

#[test]
fn add_assembly_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::AddAssemblyMsg {
                name: "Some Assembly name".to_string(),
                msg: "{}".to_string(),
                assemblies: vec![Uint128::zero()],
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let assemblies = get_assembly_msgs(&mut chain, &gov, Uint128::zero(), Uint128::new(1)).unwrap();

    assert_eq!(assemblies.len(), 2);
}

#[test]
fn unauthorised_add_assembly_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::AddAssemblyMsg {
                name: "Some Assembly name".to_string(),
                msg: "{}".to_string(),
                assemblies: vec![Uint128::zero()],
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                "random",
                gov.clone(),
            ),
        )
        .is_err();
}

#[test]
fn set_assembly_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let original_msg =
        get_assembly_msgs(&mut chain, &gov, Uint128::zero(), Uint128::new(1)).unwrap()[0].clone();

    chain
        .execute(
            &governance::HandleMsg::SetAssemblyMsg {
                id: Uint128::zero(),
                name: Some("New name".to_string()),
                msg: None,
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

    let assemblies = get_assembly_msgs(&mut chain, &gov, Uint128::zero(), Uint128::new(1)).unwrap();

    assert_eq!(assemblies.len(), 1);

    assert_ne!(original_msg.name, assemblies[0].name);
    assert_eq!(original_msg.assemblies, assemblies[0].assemblies);
    assert_eq!(original_msg.msg, assemblies[0].msg);
}

#[test]
fn unauthorised_set_assembly_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::SetAssemblyMsg {
                id: Uint128::zero(),
                name: Some("New name".to_string()),
                msg: None,
                assemblies: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                "random",
                gov.clone(),
            ),
        )
        .is_err();
}
