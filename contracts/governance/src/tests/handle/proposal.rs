use cosmwasm_std::{HumanAddr, StdResult};
use shade_protocol::governance;
use fadroma_ensemble::{ContractEnsemble, MockEnv};
use fadroma_platform_scrt::ContractLink;
use cosmwasm_math_compat::Uint128;
use shade_protocol::governance::InitMsg;
use shade_protocol::governance::profile::{Count, Profile, VoteProfile};
use shade_protocol::governance::proposal::Status;
use shade_protocol::governance::vote::Vote;
use crate::tests::{admin_only_governance, get_proposals, init_governance};

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

    chain.execute(
        &governance::HandleMsg::Trigger {
            proposal: Uint128::new(0),
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

    assert_eq!(prop.status, Status::Success);
    assert_eq!(prop.status_history.len(), 1);
}

fn init_governance_with_proposal(
) -> StdResult<(ContractEnsemble, ContractLink<HumanAddr>)>
{
    let (mut chain, gov) = init_governance(InitMsg {
        treasury: HumanAddr::from("treasury"),
        admin_members: vec![
            HumanAddr::from("alpha"),
            HumanAddr::from("beta"),
            HumanAddr::from("charlie")
        ],
        admin_profile: Profile {
            name: "admin".to_string(),
            enabled: true,
            assembly: Some(VoteProfile {
                deadline: 10000,
                threshold: Count::LiteralCount { count: Uint128::new(2) },
                yes_threshold: Count::LiteralCount { count: Uint128::new(2) },
                veto_threshold: Count::LiteralCount { count: Uint128::new(3) }
            }),
            funding: None,
            token: None,
            cancel_deadline: 0
        },
        public_profile: Profile {
            name: "public".to_string(),
            enabled: false,
            assembly: None,
            funding: None,
            token: None,
            cancel_deadline: 0
        },
        funding_token: None,
        vote_token: None
    })?;

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
            "alpha",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    )?;

    Ok((chain, gov))
}

#[test]
fn assembly_voting() {
    let (mut chain, gov) = init_governance_with_proposal().unwrap();

    let prop = get_proposals(
        &mut chain,
        &gov,
        Uint128::zero(),
        Uint128::new(2)
    ).unwrap()[0].clone();

    match prop.status {
        Status::AssemblyVote {..} => assert!(true),
        _ => assert!(false)
    };
}

#[test]
fn assembly_vote_update_before_deadline() {
    let (mut chain, gov) = init_governance_with_proposal().unwrap();

    assert!(
        chain.execute(
            &governance::HandleMsg::Update {
                proposal: Uint128::new(0),
                padding: None
            },
            MockEnv::new(
                "alpha",
                ContractLink {
                    address: gov.address.clone(),
                    code_hash: gov.code_hash.clone(),
                }
            )
        ).is_err()
    );
}

#[test]
fn assembly_vote_update_after_deadline() {
    let (mut chain, gov) = init_governance_with_proposal().unwrap();

    chain.block().time += 30000;

    assert!(
        chain.execute(
            &governance::HandleMsg::Update {
                proposal: Uint128::new(0),
                padding: None
            },
            MockEnv::new(
                "alpha",
                ContractLink {
                    address: gov.address.clone(),
                    code_hash: gov.code_hash.clone(),
                }
            )
        ).is_ok()
    );
}

#[test]
fn assembly_voting_invalid_vote() {
    let (mut chain, gov) = init_governance_with_proposal().unwrap();

    assert!(
        chain.execute(
            &governance::HandleMsg::AssemblyVote {
                proposal: Uint128::new(0),
                vote: Vote {
                    yes: Uint128::new(1),
                    no: Uint128::new(1),
                    no_with_veto: Default::default(),
                    abstain: Default::default()
                },
                padding: None
            },
            MockEnv::new(
                "alpha",
                ContractLink {
                    address: gov.address.clone(),
                    code_hash: gov.code_hash.clone(),
                }
            )
        ).is_err()
    );
}

#[test]
fn assembly_voting_unauthorised() {
    let (mut chain, gov) = init_governance_with_proposal().unwrap();

    assert!(
        chain.execute(
            &governance::HandleMsg::AssemblyVote {
                proposal: Uint128::new(0),
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::new(1),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                padding: None
            },
            MockEnv::new(
                "foxtrot",
                ContractLink {
                    address: gov.address.clone(),
                    code_hash: gov.code_hash.clone(),
                }
            )
        ).is_err()
    );
}

#[test]
fn assembly_voting_after_deadline() {
    let (mut chain, gov) = init_governance_with_proposal().unwrap();

    chain.block().time += 30000;

    assert!(
        chain.execute(
            &governance::HandleMsg::AssemblyVote {
                proposal: Uint128::new(0),
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::new(1),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                padding: None
            },
            MockEnv::new(
                "alpha",
                ContractLink {
                    address: gov.address.clone(),
                    code_hash: gov.code_hash.clone(),
                }
            )
        ).is_err()
    );
}

#[test]
fn assembly_voting_vote_yes() {
    let (mut chain, gov) = init_governance_with_proposal().unwrap();

    chain.execute(
        &governance::HandleMsg::AssemblyVote {
            proposal: Uint128::new(0),
            vote: Vote {
                yes: Uint128::new(1),
                no: Uint128::zero(),
                no_with_veto: Uint128::zero(),
                abstain: Uint128::zero()
            },
            padding: None
        },
        MockEnv::new(
            "alpha",
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

    match prop.status {
        Status::AssemblyVote {votes, ..} => {
            assert_eq!(
                votes,
                Vote {
                    yes: Uint128::new(1),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                }
            );
        },
        _ => assert!(false)
    };
}

#[test]
fn assembly_voting_abstain() {
    todo!();
}

#[test]
fn assembly_voting_no() {
    todo!();
}

#[test]
fn assembly_voting_veto() {
    todo!();
}

#[test]
fn assembly_vote_no_quorum() {
    todo!();
}

#[test]
fn assembly_voting_vote_total() {
    todo!();
}

#[test]
fn assembly_voting_update_vote() {
    todo!();
}

#[test]
fn assembly_vote_count_amount() {
    todo!();
}

#[test]
fn assembly_vote_count_percentage() {
    todo!();
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