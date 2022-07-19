pub mod assembly_voting;
pub mod funding;
pub mod voting;

use crate::tests::{
    admin_only_governance,
    get_assemblies,
    get_proposals,
    gov_generic_proposal,
    gov_msg_proposal,
};
use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{to_binary, Binary, Addr, StdResult};
use shade_protocol::fadroma::ensemble::{ContractEnsemble, MockEnv};
use shade_protocol::fadroma::core::ContractLink;
use shade_protocol::{
    contract_interfaces::{
        governance,
        governance::proposal::{ProposalMsg, Status},
    },
    utils::asset::Contract,
};

#[test]
fn trigger_admin_command() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::ExecuteMsg::AssemblyProposal {
                assembly: Uint128::new(1),
                title: "Title".to_string(),
                metadata: "Proposal metadata".to_string(),
                msgs: None,
                padding: None,
            },
            MockEnv::new("admin", ContractLink {
                address: gov.address,
                code_hash: gov.code_hash,
            }),
        )
        .unwrap();
}

#[test]
fn unauthorized_trigger_admin_command() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(
        chain
            .execute(
                &governance::ExecuteMsg::AssemblyProposal {
                    assembly: Uint128::new(1),
                    title: "Title".to_string(),
                    metadata: "Proposal metadata".to_string(),
                    msgs: None,
                    padding: None
                },
                MockEnv::new("random", gov.clone())
            )
            .is_err()
    );
}

#[test]
fn text_only_proposal() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::ExecuteMsg::AssemblyProposal {
                assembly: Uint128::new(1),
                title: "Title".to_string(),
                metadata: "Text only proposal".to_string(),
                msgs: None,
                padding: None,
            },
            MockEnv::new("admin", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    assert_eq!(prop.proposer, Addr::from("admin"));
    assert_eq!(prop.title, "Title".to_string());
    assert_eq!(prop.metadata, "Text only proposal".to_string());
    assert_eq!(prop.msgs, None);
    assert_eq!(prop.assembly, Uint128::new(1));
    assert_eq!(prop.assembly_vote_tally, None);
    assert_eq!(prop.public_vote_tally, None);
    match prop.status {
        Status::Passed { .. } => assert!(true),
        _ => assert!(false),
    };
    assert_eq!(prop.status_history.len(), 0);
    assert_eq!(prop.funders, None);

    chain
        .execute(
            &governance::ExecuteMsg::Trigger {
                proposal: Uint128::new(0),
                padding: None,
            },
            MockEnv::new("admin", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    assert_eq!(prop.status, Status::Success);
    assert_eq!(prop.status_history.len(), 1);
}

#[test]
fn msg_proposal() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    gov_generic_proposal(
        &mut chain,
        &gov,
        "admin",
        governance::ExecuteMsg::SetAssembly {
            id: Uint128::new(1),
            name: Some("Random name".to_string()),
            metadata: None,
            members: None,
            profile: None,
            padding: None,
        },
    )
    .unwrap();

    let old_assembly =
        get_assemblies(&mut chain, &gov, Uint128::new(1), Uint128::new(2)).unwrap()[0].clone();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    match prop.status {
        Status::Passed { .. } => assert!(true),
        _ => assert!(false),
    };

    chain
        .execute(
            &governance::ExecuteMsg::Trigger {
                proposal: Uint128::new(0),
                padding: None,
            },
            MockEnv::new("admin", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    assert!(prop.msgs.is_some());
    assert_eq!(prop.status, Status::Success);

    let new_assembly =
        get_assemblies(&mut chain, &gov, Uint128::new(1), Uint128::new(2)).unwrap()[0].clone();

    assert_ne!(new_assembly.name, old_assembly.name);
}

#[test]
fn multi_msg_proposal() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    gov_msg_proposal(&mut chain, &gov, "admin", vec![
        ProposalMsg {
            target: Uint128::zero(),
            assembly_msg: Uint128::zero(),
            msg: to_binary(&vec![
                serde_json::to_string(&governance::ExecuteMsg::SetAssembly {
                    id: Uint128::new(1),
                    name: Some("Random name".to_string()),
                    metadata: None,
                    members: None,
                    profile: None,
                    padding: None,
                })
                .unwrap(),
            ])
            .unwrap(),
            send: vec![],
        },
        ProposalMsg {
            target: Uint128::zero(),
            assembly_msg: Uint128::zero(),
            msg: to_binary(&vec![
                serde_json::to_string(&governance::ExecuteMsg::SetAssembly {
                    id: Uint128::new(1),
                    name: None,
                    metadata: Some("Random name".to_string()),
                    members: None,
                    profile: None,
                    padding: None,
                })
                .unwrap(),
            ])
            .unwrap(),
            send: vec![],
        },
    ])
    .unwrap();

    let old_assembly =
        get_assemblies(&mut chain, &gov, Uint128::new(1), Uint128::new(2)).unwrap()[0].clone();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    match prop.status {
        Status::Passed { .. } => assert!(true),
        _ => assert!(false),
    };

    chain
        .execute(
            &governance::ExecuteMsg::Trigger {
                proposal: Uint128::new(0),
                padding: None,
            },
            MockEnv::new("admin", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    assert_eq!(prop.status, Status::Success);

    let new_assembly =
        get_assemblies(&mut chain, &gov, Uint128::new(1), Uint128::new(2)).unwrap()[0].clone();

    assert_ne!(new_assembly.name, old_assembly.name);
    assert_ne!(new_assembly.metadata, old_assembly.metadata);
}

#[test]
fn msg_proposal_invalid_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    gov_generic_proposal(
        &mut chain,
        &gov,
        "admin",
        governance::ExecuteMsg::SetAssembly {
            id: Uint128::new(3),
            name: Some("Random name".to_string()),
            metadata: None,
            members: None,
            profile: None,
            padding: None,
        },
    )
    .unwrap();

    assert!(
        chain
            .execute(
                &governance::ExecuteMsg::Trigger {
                    proposal: Uint128::new(0),
                    padding: None
                },
                MockEnv::new("admin", ContractLink {
                    address: gov.address.clone(),
                    code_hash: gov.code_hash.clone(),
                })
            )
            .is_err()
    );

    chain.block_mut().time += 100000;

    chain
        .execute(
            &governance::ExecuteMsg::Cancel {
                proposal: Uint128::new(0),
                padding: None,
            },
            MockEnv::new("admin", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    assert_eq!(prop.status, Status::Canceled);
}
