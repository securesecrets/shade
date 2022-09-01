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
use shade_protocol::{
    c_std::{to_binary, Addr, Binary, StdResult, Uint128},
    contract_interfaces::{
        governance,
        governance::proposal::{ProposalMsg, Status},
    },
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, Query},
};

// TODO: update state and retest the relevant functions

#[test]
fn trigger_admin_command() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    governance::ExecuteMsg::AssemblyProposal {
        assembly: Uint128::new(1),
        title: "Title".to_string(),
        metadata: "Proposal metadata".to_string(),
        msgs: None,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
    .unwrap();
}

#[test]
fn unauthorized_trigger_admin_command() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(
        governance::ExecuteMsg::AssemblyProposal {
            assembly: Uint128::new(1),
            title: "Title".to_string(),
            metadata: "Proposal metadata".to_string(),
            msgs: None,
            padding: None
        }
        .test_exec(&gov, &mut chain, Addr::unchecked("random"), &[])
        .is_err()
    );
}

#[test]
fn text_only_proposal() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    governance::ExecuteMsg::AssemblyProposal {
        assembly: Uint128::new(1),
        title: "Title".to_string(),
        metadata: "Text only proposal".to_string(),
        msgs: None,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
    .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    assert_eq!(prop.proposer, Addr::unchecked("admin"));
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

    governance::ExecuteMsg::Trigger {
        proposal: Uint128::new(0),
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
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

    governance::ExecuteMsg::Trigger {
        proposal: Uint128::new(0),
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
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

    governance::ExecuteMsg::Trigger {
        proposal: Uint128::new(0),
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
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
        governance::ExecuteMsg::Trigger {
            proposal: Uint128::new(0),
            padding: None
        }
        .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
        .is_err()
    );

    chain.update_block(|block| block.time = block.time.plus_seconds(100000));

    governance::ExecuteMsg::Cancel {
        proposal: Uint128::new(0),
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
    .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    assert_eq!(prop.status, Status::Canceled);
}
