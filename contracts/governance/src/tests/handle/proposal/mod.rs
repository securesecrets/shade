pub mod assembly_voting;
pub mod funding;
pub mod voting;

use crate::tests::{
    admin_only_governance,
    get_assemblies,
    get_proposals,
    gov_generic_proposal,
    gov_msg_proposal,
    init_governance,
};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{to_binary, Binary, HumanAddr, StdResult};
use fadroma::{
    ensemble::{ContractEnsemble, MockEnv},
    ContractLink,
};
use shade_protocol::{
    contract_interfaces::{
        governance,
        governance::{
            profile::{Count, FundProfile, Profile, UpdateProfile, UpdateVoteProfile, VoteProfile},
            proposal::{ProposalMsg, Status},
            vote::Vote,
            InitMsg,
        },
    },
    utils::asset::Contract,
};

#[test]
fn trigger_admin_command() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::AssemblyProposal {
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
                &governance::HandleMsg::AssemblyProposal {
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
            &governance::HandleMsg::AssemblyProposal {
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

    assert_eq!(prop.proposer, HumanAddr::from("admin"));
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
            &governance::HandleMsg::Trigger {
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
        governance::HandleMsg::SetAssembly {
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
            &governance::HandleMsg::Trigger {
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
}

#[test]
fn multi_msg_proposal() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    gov_msg_proposal(&mut chain, &gov, "admin", vec![
        ProposalMsg {
            target: Uint128::zero(),
            assembly_msg: Uint128::zero(),
            msg: to_binary(&vec![
                serde_json::to_string(&governance::HandleMsg::SetAssembly {
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
                serde_json::to_string(&governance::HandleMsg::SetAssembly {
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
    ]);

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
            &governance::HandleMsg::Trigger {
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
        governance::HandleMsg::SetAssembly {
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
                &governance::HandleMsg::Trigger {
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

    chain.block().time += 100000;

    chain
        .execute(
            &governance::HandleMsg::Cancel {
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

// TODO: Assembly update if assembly setting removed from profile
// TODO: funding update if funding setting removed from profile
// TODO: voting update if voting setting removed from profile
