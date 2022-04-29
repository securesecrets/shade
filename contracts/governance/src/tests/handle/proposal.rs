use cosmwasm_std::{Binary, HumanAddr, StdResult, to_binary};
use shade_protocol::governance;
use fadroma_ensemble::{ContractEnsemble, MockEnv};
use fadroma_platform_scrt::ContractLink;
use cosmwasm_math_compat::Uint128;
use shade_protocol::governance::InitMsg;
use shade_protocol::governance::profile::{Count, FundProfile, Profile, UpdateProfile, UpdateVoteProfile, VoteProfile};
use shade_protocol::governance::proposal::Status;
use shade_protocol::governance::vote::Vote;
use shade_protocol::utils::asset::Contract;
use crate::tests::{admin_only_governance, get_proposals, Governance, init_governance, Snip20};

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

fn init_assembly_governance_with_proposal(
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
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

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
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

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
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

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
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

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
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

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
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

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
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

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
        Status::AssemblyVote {..} => assert!(true),
        _ => assert!(false)
    };

    assert_eq!(prop.assembly_vote_tally, Some(Vote {
        yes: Uint128::new(1),
        no: Uint128::zero(),
        no_with_veto: Uint128::zero(),
        abstain: Uint128::zero()
    }))
}

#[test]
fn assembly_voting_abstain() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    chain.execute(
        &governance::HandleMsg::AssemblyVote {
            proposal: Uint128::new(0),
            vote: Vote {
                yes: Uint128::zero(),
                no: Uint128::zero(),
                no_with_veto: Uint128::zero(),
                abstain: Uint128::new(1)
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
        Status::AssemblyVote {..} => assert!(true),
        _ => assert!(false)
    };

    assert_eq!(prop.assembly_vote_tally, Some(Vote {
        yes: Uint128::zero(),
        no: Uint128::zero(),
        no_with_veto: Uint128::zero(),
        abstain: Uint128::new(1)
    }))
}

#[test]
fn assembly_voting_no() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

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
    ).unwrap();

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

    assert_eq!(prop.assembly_vote_tally, Some(Vote {
        yes: Uint128::zero(),
        no: Uint128::new(1),
        no_with_veto: Uint128::zero(),
        abstain: Uint128::zero()
    }))
}

#[test]
fn assembly_voting_veto() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    chain.execute(
        &governance::HandleMsg::AssemblyVote {
            proposal: Uint128::new(0),
            vote: Vote {
                yes: Uint128::zero(),
                no: Uint128::zero(),
                no_with_veto: Uint128::new(1),
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
        Status::AssemblyVote {..} => assert!(true),
        _ => assert!(false)
    };

    assert_eq!(prop.assembly_vote_tally, Some(Vote {
        yes: Uint128::zero(),
        no: Uint128::zero(),
        no_with_veto: Uint128::new(1),
        abstain: Uint128::zero()
    }))
}

#[test]
fn assembly_voting_passed() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

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
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    ).unwrap();

    chain.block().time += 30000;

    chain.execute(
        &governance::HandleMsg::Update {
            proposal: Uint128::zero(),
            padding: None
        },
        MockEnv::new(
            "beta",
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
        Status::Passed {..} => assert!(true),
        _ => assert!(false)
    };
    
}

#[test]
fn assembly_voting_abstained() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    chain.execute(
        &governance::HandleMsg::AssemblyVote {
            proposal: Uint128::new(0),
            vote: Vote {
                yes: Uint128::zero(),
                no: Uint128::zero(),
                no_with_veto: Uint128::zero(),
                abstain: Uint128::new(1)
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

    chain.execute(
        &governance::HandleMsg::AssemblyVote {
            proposal: Uint128::new(0),
            vote: Vote {
                yes: Uint128::zero(),
                no: Uint128::zero(),
                no_with_veto: Uint128::zero(),
                abstain: Uint128::new(1)
            },
            padding: None
        },
        MockEnv::new(
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    ).unwrap();

    chain.block().time += 30000;

    chain.execute(
        &governance::HandleMsg::Update {
            proposal: Uint128::zero(),
            padding: None
        },
        MockEnv::new(
            "beta",
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
        Status::Rejected {..} => assert!(true),
        _ => assert!(false)
    };
}

#[test]
fn assembly_voting_rejected() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

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
    ).unwrap();

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
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    ).unwrap();

    chain.block().time += 30000;

    chain.execute(
        &governance::HandleMsg::Update {
            proposal: Uint128::zero(),
            padding: None
        },
        MockEnv::new(
            "beta",
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
        Status::Rejected {..} => assert!(true),
        _ => assert!(false)
    };
}

#[test]
fn assembly_voting_vetoed() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    chain.execute(
        &governance::HandleMsg::AssemblyVote {
            proposal: Uint128::new(0),
            vote: Vote {
                yes: Uint128::zero(),
                no: Uint128::zero(),
                no_with_veto: Uint128::new(1),
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

    chain.execute(
        &governance::HandleMsg::AssemblyVote {
            proposal: Uint128::new(0),
            vote: Vote {
                yes: Uint128::zero(),
                no: Uint128::zero(),
                no_with_veto: Uint128::new(1),
                abstain: Uint128::zero()
            },
            padding: None
        },
        MockEnv::new(
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    ).unwrap();

    chain.block().time += 30000;

    chain.execute(
        &governance::HandleMsg::Update {
            proposal: Uint128::zero(),
            padding: None
        },
        MockEnv::new(
            "beta",
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
        // NOTE: assembly votes cannot be vetoed
        Status::Rejected {..} => assert!(true),
        _ => assert!(false)
    };
}

#[test]
fn assembly_vote_no_quorum() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    chain.execute(
        &governance::HandleMsg::AssemblyVote {
            proposal: Uint128::new(0),
            vote: Vote {
                yes: Uint128::zero(),
                no: Uint128::zero(),
                no_with_veto: Uint128::new(1),
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

    chain.block().time += 30000;

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
    ).unwrap();

    let prop = get_proposals(
        &mut chain,
        &gov,
        Uint128::zero(),
        Uint128::new(2)
    ).unwrap()[0].clone();

    assert_eq!(prop.status, Status::Expired);
}

#[test]
fn assembly_voting_vote_total() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    chain.execute(
        &governance::HandleMsg::AssemblyVote {
            proposal: Uint128::new(0),
            vote: Vote {
                yes: Uint128::zero(),
                no: Uint128::zero(),
                no_with_veto: Uint128::new(1),
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
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    ).unwrap();

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
            "charlie",
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
        Status::AssemblyVote {..} => assert!(true),
        _ => assert!(false)
    };

    assert_eq!(prop.assembly_vote_tally, Some(Vote {
        yes: Uint128::new(2),
        no: Uint128::zero(),
        no_with_veto: Uint128::new(1),
        abstain: Uint128::zero()
    }))
}

#[test]
fn assembly_voting_update_vote() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    chain.execute(
        &governance::HandleMsg::AssemblyVote {
            proposal: Uint128::new(0),
            vote: Vote {
                yes: Uint128::zero(),
                no: Uint128::zero(),
                no_with_veto: Uint128::new(1),
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

    assert_eq!(prop.assembly_vote_tally, Some(Vote {
        yes: Uint128::zero(),
        no: Uint128::zero(),
        no_with_veto: Uint128::new(1),
        abstain: Uint128::zero()
    }));

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

    assert_eq!(prop.assembly_vote_tally, Some(Vote {
        yes: Uint128::new(1),
        no: Uint128::zero(),
        no_with_veto: Uint128::zero(),
        abstain: Uint128::zero()
    }));
}

#[test]
fn assembly_vote_count_amount() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

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
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    ).unwrap();

    chain.block().time += 30000;

    chain.execute(
        &governance::HandleMsg::Update {
            proposal: Uint128::zero(),
            padding: None
        },
        MockEnv::new(
            "beta",
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
        Status::Passed {..} => assert!(true),
        _ => assert!(false)
    };
}

#[test]
fn assembly_vote_count_percentage() {
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
                threshold: Count::Percentage { percent: 6500 },
                yes_threshold: Count::Percentage { percent: 6500 },
                veto_threshold: Count::Percentage { percent: 6500 }
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
    }).unwrap();

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
    ).unwrap();

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
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    ).unwrap();

    chain.block().time += 30000;

    chain.execute(
        &governance::HandleMsg::Update {
            proposal: Uint128::zero(),
            padding: None
        },
        MockEnv::new(
            "beta",
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
        Status::Passed {..} => assert!(true),
        _ => assert!(false)
    };
}


fn init_funding_governance_with_proposal(
) -> StdResult<(ContractEnsemble, ContractLink<HumanAddr>, ContractLink<HumanAddr>)>
{
    let mut chain = ContractEnsemble::new(50);

    // Register snip20
    let snip20 = chain.register(Box::new(Snip20));
    let snip20 = chain.instantiate(
        snip20.id,
        &snip20_reference_impl::msg::InitMsg {
            name: "funding_token".to_string(),
            admin: None,
            symbol: "FND".to_string(),
            decimals: 6,
            initial_balances: Some(vec![
                snip20_reference_impl::msg::InitialBalance {
                    address: HumanAddr::from("alpha"),
                    amount: cosmwasm_std::Uint128(10000),
                },
                snip20_reference_impl::msg::InitialBalance {
                    address: HumanAddr::from("beta"),
                    amount: cosmwasm_std::Uint128(10000),
                },
                snip20_reference_impl::msg::InitialBalance {
                    address: HumanAddr::from("charlie"),
                    amount: cosmwasm_std::Uint128(10000),
                },
            ]),
            prng_seed: Default::default(),
            config: None
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: "funding_token".into(),
                code_hash: snip20.code_hash,
            }
        )
    )?;

    // Register governance
    let gov = chain.register(Box::new(Governance));
    let gov = chain.instantiate(
        gov.id,
        &InitMsg {
            treasury: HumanAddr::from("treasury"),
            admin_members: vec![
                HumanAddr::from("alpha"),
                HumanAddr::from("beta"),
                HumanAddr::from("charlie")
            ],
            admin_profile: Profile {
                name: "admin".to_string(),
                enabled: true,
                assembly: None,
                funding: Some(FundProfile {
                    deadline: 1000,
                    required: Uint128::new(2000),
                    privacy: false,
                    veto_deposit_loss: Default::default()
                }),
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
            funding_token: Some(Contract {
                address: snip20.address.clone(),
                code_hash: snip20.code_hash.clone()
            }),
            vote_token: None
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: "gov".into(),
                code_hash: gov.code_hash,
            }
        )
    )?;

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

    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::SetViewingKey {
            key: "password".to_string(),
            padding: None
        },
        MockEnv::new(
            "alpha",
            ContractLink {
                address: snip20.address.clone(),
                code_hash: snip20.code_hash.clone(),
            }
        )
    )?;

    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::SetViewingKey {
            key: "password".to_string(),
            padding: None
        },
        MockEnv::new(
            "beta",
            ContractLink {
                address: snip20.address.clone(),
                code_hash: snip20.code_hash.clone(),
            }
        )
    )?;

    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::SetViewingKey {
            key: "password".to_string(),
            padding: None
        },
        MockEnv::new(
            "charlie",
            ContractLink {
                address: snip20.address.clone(),
                code_hash: snip20.code_hash.clone(),
            }
        )
    )?;

    Ok((chain, gov, snip20))
}
// TODO: Assembly update if assembly setting removed from profile
// TODO: funding update if funding setting removed from profile
// TODO: voting update if voting setting removed from profile
#[test]
fn assembly_to_funding_transition() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();
    chain.execute(
        &governance::HandleMsg::SetProfile {
            id: Uint128::new(1),
            profile: UpdateProfile {
                name: None,
                enabled: None,
                disable_assembly: false,
                assembly: Some(UpdateVoteProfile {
                    deadline: Some(1000),
                    threshold: Some(Count::LiteralCount {count:Uint128::new(1)}),
                    yes_threshold: Some(Count::LiteralCount {count:Uint128::new(1)}),
                    veto_threshold: Some(Count::LiteralCount {count:Uint128::new(1)})
                }),
                disable_funding: false,
                funding: None,
                disable_token: false,
                token: None,
                cancel_deadline: None
            },
            padding: None
        },
        MockEnv::new(
            // Sender is self
            gov.address.clone(),
            gov.clone()
        )
    ).unwrap();

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
    ).unwrap();

    chain.execute(
        &governance::HandleMsg::AssemblyVote {
            proposal: Uint128::new(1),
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

    chain.execute(
        &governance::HandleMsg::AssemblyVote {
            proposal: Uint128::new(1),
            vote: Vote {
                yes: Uint128::new(1),
                no: Uint128::zero(),
                no_with_veto: Uint128::zero(),
                abstain: Uint128::zero()
            },
            padding: None
        },
        MockEnv::new(
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    ).unwrap();

    chain.block().time += 30000;

    chain.execute(
        &governance::HandleMsg::Update {
            proposal: Uint128::new(1),
            padding: None
        },
        MockEnv::new(
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    ).unwrap();

    let prop = get_proposals(
        &mut chain,
        &gov,
        Uint128::new(1),
        Uint128::new(2)
    ).unwrap()[0].clone();

    match prop.status {
        Status::Funding {..} => assert!(true),
        _ => assert!(false)
    };
}
#[test]
fn fake_funding_token() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    let other = chain.register(Box::new(Snip20));
    let other = chain.instantiate(
        other.id,
        &snip20_reference_impl::msg::InitMsg {
            name: "funding_token".to_string(),
            admin: None,
            symbol: "FND".to_string(),
            decimals: 6,
            initial_balances: Some(vec![
                snip20_reference_impl::msg::InitialBalance {
                    address: HumanAddr::from("alpha"),
                    amount: cosmwasm_std::Uint128(10000),
                },
                snip20_reference_impl::msg::InitialBalance {
                    address: HumanAddr::from("beta"),
                    amount: cosmwasm_std::Uint128(10000),
                },
                snip20_reference_impl::msg::InitialBalance {
                    address: HumanAddr::from("charlie"),
                    amount: cosmwasm_std::Uint128(10000),
                },
            ]),
            prng_seed: Default::default(),
            config: None
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: "other".into(),
                code_hash: snip20.code_hash.clone(),
            }
        )
    ).unwrap();

    chain.execute(
        &governance::HandleMsg::SetConfig {
            treasury: None,
            funding_token: Some(Contract {
                address: other.address.clone(),
                code_hash: other.code_hash,
            }),
            vote_token: None,
            padding: None
        },
        MockEnv::new(
            // Sender is self
            gov.address.clone(),
            gov.clone()
        )
    ).unwrap();

    assert!(
        chain.execute(
            &snip20_reference_impl::msg::HandleMsg::Send {
                recipient: gov.address,
                recipient_code_hash: None,
                amount: cosmwasm_std::Uint128(100),
                msg: None,
                memo: None,
                padding: None
            },
            MockEnv::new(
            // Sender is self
            HumanAddr::from("alpha"),
            snip20.clone()
        )
    ).is_err()
    );
}
#[test]
fn funding_proposal_without_msg() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    assert!(
        chain.execute(
            &snip20_reference_impl::msg::HandleMsg::Send {
                recipient: gov.address,
                recipient_code_hash: None,
                amount: cosmwasm_std::Uint128(100),
                msg: None,
                memo: None,
                padding: None
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("alpha"),
                snip20.clone()
            )
        ).is_err()
    );
}
#[test]
fn funding_proposal() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::Send {
            recipient: gov.address.clone(),
            recipient_code_hash: None,
            amount: cosmwasm_std::Uint128(100),
            msg: Some(to_binary(&Uint128::zero()).unwrap()),
            memo: None,
            padding: None
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("alpha"),
            snip20.clone()
        )
    ).unwrap();

    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::Send {
            recipient: gov.address.clone(),
            recipient_code_hash: None,
            amount: cosmwasm_std::Uint128(100),
            msg: Some(to_binary(&Uint128::zero()).unwrap()),
            memo: None,
            padding: None
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("beta"),
            snip20.clone()
        )
    ).unwrap();

    let prop = get_proposals(
        &mut chain,
        &gov,
        Uint128::zero(),
        Uint128::new(2)
    ).unwrap()[0].clone();

    match prop.status {
        Status::Funding {amount, ..} => assert_eq!(amount, Uint128::new(200)),
        _ => assert!(false)
    };
}
#[test]
fn funding_proposal_after_deadline() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain.block().time += 10000;

    assert!(chain.execute(
        &snip20_reference_impl::msg::HandleMsg::Send {
            recipient: gov.address.clone(),
            recipient_code_hash: None,
            amount: cosmwasm_std::Uint128(100),
            msg: Some(to_binary(&Uint128::zero()).unwrap()),
            memo: None,
            padding: None
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("alpha"),
            snip20.clone()
        )
    ).is_err())
}
#[test]
fn update_while_funding() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    assert!(chain.execute(
        &governance::HandleMsg::Update {
            proposal: Uint128::zero(),
            padding: None
        },
        MockEnv::new(
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    ).is_err());
}
#[test]
fn update_when_fully_funded() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::Send {
            recipient: gov.address.clone(),
            recipient_code_hash: None,
            amount: cosmwasm_std::Uint128(1000),
            msg: Some(to_binary(&Uint128::zero()).unwrap()),
            memo: None,
            padding: None
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("alpha"),
            snip20.clone()
        )
    ).unwrap();

    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::Send {
            recipient: gov.address.clone(),
            recipient_code_hash: None,
            amount: cosmwasm_std::Uint128(1000),
            msg: Some(to_binary(&Uint128::zero()).unwrap()),
            memo: None,
            padding: None
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("beta"),
            snip20.clone()
        )
    ).unwrap();

    chain.execute(
        &governance::HandleMsg::Update {
            proposal: Uint128::zero(),
            padding: None
        },
        MockEnv::new(
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    );

    let prop = get_proposals(
        &mut chain,
        &gov,
        Uint128::zero(),
        Uint128::new(2)
    ).unwrap()[0].clone();

    match prop.status {
        Status::Passed { .. } => assert!(true),
        _ => assert!(false)
    };
}
#[test]
fn update_after_failed_funding() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::Send {
            recipient: gov.address.clone(),
            recipient_code_hash: None,
            amount: cosmwasm_std::Uint128(1000),
            msg: Some(to_binary(&Uint128::zero()).unwrap()),
            memo: None,
            padding: None
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("alpha"),
            snip20.clone()
        )
    ).unwrap();

    chain.block().time += 10000;

    chain.execute(
        &governance::HandleMsg::Update {
            proposal: Uint128::zero(),
            padding: None
        },
        MockEnv::new(
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    );

    let prop = get_proposals(
        &mut chain,
        &gov,
        Uint128::zero(),
        Uint128::new(2)
    ).unwrap()[0].clone();

    match prop.status {
        Status::Expired { } => assert!(true),
        _ => assert!(false)
    };
}
#[test]
fn claim_when_not_finished() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::Send {
            recipient: gov.address.clone(),
            recipient_code_hash: None,
            amount: cosmwasm_std::Uint128(1000),
            msg: Some(to_binary(&Uint128::zero()).unwrap()),
            memo: None,
            padding: None
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("alpha"),
            snip20.clone()
        )
    ).unwrap();

    assert!(chain.execute(
        &governance::HandleMsg::ClaimFunding {
            id: Uint128::new(0)
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("alpha"),
            snip20.clone()
        )
    ).is_err());
}
#[test]
fn claim_after_failing() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::Send {
            recipient: gov.address.clone(),
            recipient_code_hash: None,
            amount: cosmwasm_std::Uint128(1000),
            msg: Some(to_binary(&Uint128::zero()).unwrap()),
            memo: None,
            padding: None
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("alpha"),
            snip20.clone()
        )
    ).unwrap();

    chain.block().time += 10000;

    chain.execute(
        &governance::HandleMsg::Update {
            proposal: Uint128::zero(),
            padding: None
        },
        MockEnv::new(
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    );

    chain.execute(
        &governance::HandleMsg::ClaimFunding {
            id: Uint128::new(0)
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("alpha"),
            gov.clone()
        )
    ).unwrap();

    let query: snip20_reference_impl::msg::QueryAnswer = chain.query(
        snip20.address.clone(),
        &snip20_reference_impl::msg::QueryMsg::Balance { address: HumanAddr::from("alpha"), key: "password".to_string() }
    ).unwrap();

    match query {
        snip20_reference_impl::msg::QueryAnswer::Balance {amount} => assert_eq!(amount, cosmwasm_std::Uint128(10000)),
        _ => assert!(false)
    };
}
#[test]
fn claim_after_passing() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::Send {
            recipient: gov.address.clone(),
            recipient_code_hash: None,
            amount: cosmwasm_std::Uint128(2000),
            msg: Some(to_binary(&Uint128::zero()).unwrap()),
            memo: None,
            padding: None
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("alpha"),
            snip20.clone()
        )
    ).unwrap();

    chain.execute(
        &governance::HandleMsg::Update {
            proposal: Uint128::zero(),
            padding: None
        },
        MockEnv::new(
            "beta",
            ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }
        )
    );

    chain.execute(
        &governance::HandleMsg::ClaimFunding {
            id: Uint128::new(0)
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("alpha"),
            gov.clone()
        )
    ).unwrap();

    let query: snip20_reference_impl::msg::QueryAnswer = chain.query(
        snip20.address.clone(),
        &snip20_reference_impl::msg::QueryMsg::Balance { address: HumanAddr::from("alpha"), key: "password".to_string() }
    ).unwrap();

    match query {
        snip20_reference_impl::msg::QueryAnswer::Balance {amount} => assert_eq!(amount, cosmwasm_std::Uint128(10000)),
        _ => assert!(false)
    };
}

// TODO: Claim after passing
// TODO: claim after failing
// TODO: claim after veto

// TODO: Try voting
// TODO: Try update while in voting
// TODO: Try update on yes
// TODO: Try update on abstain
// TODO: Try update on no
// TODO: Try update on veto

// TODO: Create normal proposal

// TODO: Trigger a failed contract and then cancel
// TODO: Cancel contract