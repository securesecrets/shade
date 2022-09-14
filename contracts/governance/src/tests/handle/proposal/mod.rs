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
use shade_multi_test::multi::snip20::Snip20;
use shade_protocol::{
    c_std::{to_binary, Addr, ContractInfo, StdResult},
    contract_interfaces::{
        governance,
        governance::proposal::{ProposalMsg, Status},
    },
    multi_test::App,
    query_auth,
    snip20::{self, InitialBalance},
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable},
};

pub fn init_funding_token(
    chain: &mut App,
    initial_balances: Option<Vec<InitialBalance>>,
    query_auth: Option<&ContractInfo>,
) -> StdResult<ContractInfo> {
    let snip20 = snip20::InstantiateMsg {
        name: "funding_token".to_string(),
        admin: None,
        symbol: "FND".to_string(),
        decimals: 6,
        initial_balances: initial_balances.clone(),
        prng_seed: Default::default(),
        config: None,
        query_auth: None,
    }
    .test_init(
        Snip20::default(),
        chain,
        Addr::unchecked("admin"),
        "funding_token",
        &[],
    )
    .unwrap();

    if let Some(balances) = initial_balances {
        if let Some(auth) = query_auth {
            for balance in balances {
                query_auth::ExecuteMsg::SetViewingKey {
                    key: "password".to_string(),
                    padding: None,
                }
                .test_exec(&auth, chain, Addr::unchecked(balance.address), &[])
                .unwrap();
            }
        }
    }

    Ok(snip20)
}

#[test]
fn trigger_admin_command() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    governance::ExecuteMsg::AssemblyProposal {
        assembly: 1,
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
            assembly: 1,
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
        assembly: 1,
        title: "Title".to_string(),
        metadata: "Text only proposal".to_string(),
        msgs: None,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert_eq!(prop.proposer, Addr::unchecked("admin"));
    assert_eq!(prop.title, "Title".to_string());
    assert_eq!(prop.metadata, "Text only proposal".to_string());
    assert_eq!(prop.msgs, None);
    assert_eq!(prop.assembly, 1);
    assert_eq!(prop.assembly_vote_tally, None);
    assert_eq!(prop.public_vote_tally, None);
    match prop.status {
        Status::Passed { .. } => assert!(true),
        _ => assert!(false),
    };
    assert_eq!(prop.status_history.len(), 0);
    assert_eq!(prop.funders, None);

    governance::ExecuteMsg::Trigger {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

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
            id: 1,
            name: Some("Random name".to_string()),
            metadata: None,
            members: None,
            profile: None,
            padding: None,
        },
    )
    .unwrap();

    let old_assembly = get_assemblies(&mut chain, &gov, 1, 2).unwrap()[0].clone();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Passed { .. } => assert!(true),
        _ => assert!(false),
    };

    governance::ExecuteMsg::Trigger {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert!(prop.msgs.is_some());
    assert_eq!(prop.status, Status::Success);

    let new_assembly = get_assemblies(&mut chain, &gov, 1, 2).unwrap()[0].clone();

    assert_ne!(new_assembly.name, old_assembly.name);
}

#[test]
fn multi_msg_proposal() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    gov_msg_proposal(&mut chain, &gov, "admin", vec![
        ProposalMsg {
            target: 0,
            assembly_msg: 0,
            msg: to_binary(&vec![
                serde_json::to_string(&governance::ExecuteMsg::SetAssembly {
                    id: 1,
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
            target: 0,
            assembly_msg: 0,
            msg: to_binary(&vec![
                serde_json::to_string(&governance::ExecuteMsg::SetAssembly {
                    id: 1,
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

    let old_assembly = get_assemblies(&mut chain, &gov, 1, 2).unwrap()[0].clone();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Passed { .. } => assert!(true),
        _ => assert!(false),
    };

    governance::ExecuteMsg::Trigger {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert_eq!(prop.status, Status::Success);

    let new_assembly = get_assemblies(&mut chain, &gov, 1, 2).unwrap()[0].clone();

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
            id: 3,
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
            proposal: 0,
            padding: None
        }
        .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
        .is_err()
    );

    chain.update_block(|block| block.time = block.time.plus_seconds(100000));

    governance::ExecuteMsg::Cancel {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert_eq!(prop.status, Status::Canceled);
}
