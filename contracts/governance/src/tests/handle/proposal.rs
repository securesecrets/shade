use cosmwasm_std::HumanAddr;
use shade_protocol::governance;
use fadroma_ensemble::MockEnv;
use fadroma_platform_scrt::ContractLink;
use cosmwasm_math_compat::Uint128;
use shade_protocol::governance::proposal::Status;
use crate::tests::{admin_only_governance, get_proposals};

#[test]
fn trigger_admin_command() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain.execute(
        &governance::HandleMsg::AssemblyProposal {
            assembly: Uint128::new(1),
            metadata: "Proposal metadata".to_string(),
            contract: None,
            assembly_msg: None,
            variables: None,
            coins: None,
            padding: None
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: gov.address,
                code_hash: gov.code_hash,
            }
        )
    ).unwrap();
}

#[test]
fn unauthorized_trigger_admin_command() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(chain.execute(
        &governance::HandleMsg::AssemblyProposal {
            assembly: Uint128::new(1),
            metadata: "Proposal metadata".to_string(),
            contract: None,
            assembly_msg: None,
            variables: None,
            coins: None,
            padding: None
        },
        MockEnv::new(
            "random",
            gov.clone()
        )
    ).is_err());
}

#[test]
fn text_only_proposal() {
    let (mut chain, gov) = admin_only_governance().unwrap();
    
    chain.execute(
        &governance::HandleMsg::AssemblyProposal {
            assembly: Uint128::new(1),
            metadata: "Text only proposal".to_string(),
            contract: None,
            assembly_msg: None,
            variables: None,
            coins: None,
            padding: None
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    ).unwrap();

    let prop = get_proposals(
        &mut chain,
        &gov,
        Uint128::zero(),
        Uint128::new(2)
    ).unwrap()[0].clone();

    assert_eq!(prop.proposer, HumanAddr::from("admin"));
    assert_eq!(prop.metadata, "Text only proposal".to_string());
    assert_eq!(prop.target, None);
    assert_eq!(prop.assembly_msg, None);
    assert_eq!(prop.msg, None);
    assert_eq!(prop.send, None);
    assert_eq!(prop.assembly, Uint128::new(1));
    assert_eq!(prop.assembly_vote_tally, None);
    assert_eq!(prop.public_vote_tally, None);
    match prop.status {
        Status::Passed {..} => assert!(true),
        _ => assert!(false)
    };
    assert_eq!(prop.status_history.len(), 0);
    assert_eq!(prop.funders, None);
    
}

// TODO: Create normal proposal

// TODO: Try assembly voting
// TODO: Try update while in assembly voting
// TODO: Try update on yes
// TODO: Try update on abstain
// TODO: Try update on no
// TODO: Try update on veto

// TODO: try funding
// TODO: Try update while funding
// TODO: Update while fully funded
// TODO: Update after failed funding

// TODO: Try voting
// TODO: Try update while in voting
// TODO: Try update on yes
// TODO: Try update on abstain
// TODO: Try update on no
// TODO: Try update on veto

// TODO: Trigger a failed contract and then cancel
// TODO: Cancel contract