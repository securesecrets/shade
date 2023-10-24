use crate::tests::{get_proposals, init_chain};
use shade_multi_test::multi::governance::Governance;
use shade_protocol::{
    c_std::{Addr, ContractInfo, StdResult, Uint128},
    contract_interfaces::{
        governance,
        governance::{
            profile::{Count, Profile, VoteProfile},
            proposal::Status,
            vote::Vote,
            InstantiateMsg,
        },
        query_auth,
    },
    governance::AssemblyInit,
    multi_test::App,
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable},
};

pub fn init_assembly_governance_with_proposal() -> StdResult<(App, ContractInfo)> {
    let (mut chain, auth) = init_chain();

    query_auth::ExecuteMsg::SetViewingKey {
        key: "password".to_string(),
        padding: None,
    }
    .test_exec(&auth, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    query_auth::ExecuteMsg::SetViewingKey {
        key: "password".to_string(),
        padding: None,
    }
    .test_exec(&auth, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    query_auth::ExecuteMsg::SetViewingKey {
        key: "password".to_string(),
        padding: None,
    }
    .test_exec(&auth, &mut chain, Addr::unchecked("charlie"), &[])
    .unwrap();

    let gov = InstantiateMsg {
        treasury: Addr::unchecked("treasury"),
        query_auth: Contract {
            address: auth.address,
            code_hash: auth.code_hash,
        },
        funding_token: None,
        vote_token: None,
        assemblies: Some(AssemblyInit {
            admin_members: vec![
                Addr::unchecked("alpha"),
                Addr::unchecked("beta"),
                Addr::unchecked("charlie"),
            ],
            admin_profile: Profile {
                name: "admin".to_string(),
                enabled: true,
                assembly: Some(VoteProfile {
                    deadline: 10000,
                    threshold: Count::LiteralCount {
                        count: Uint128::new(2),
                    },
                    yes_threshold: Count::LiteralCount {
                        count: Uint128::new(2),
                    },
                    veto_threshold: Count::LiteralCount {
                        count: Uint128::new(3),
                    },
                }),
                funding: None,
                token: None,
                cancel_deadline: 0,
            },
            public_profile: Profile {
                name: "public".to_string(),
                enabled: false,
                assembly: None,
                funding: None,
                token: None,
                cancel_deadline: 0,
            },
        }),
        migrator: None,
    }
    .test_init(
        Governance::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "governance",
        &[],
    )
    .unwrap();

    governance::ExecuteMsg::AssemblyProposal {
        assembly: 1,
        title: "Title".to_string(),
        metadata: "Text only proposal".to_string(),
        msgs: None,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    Ok((chain, gov))
}

#[test]
fn assembly_voting() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert_eq!(prop.title, "Title".to_string());
    assert_eq!(prop.metadata, "Text only proposal".to_string());
    assert_eq!(prop.proposer, Addr::unchecked("alpha"));
    assert_eq!(prop.assembly, 1);

    match prop.status {
        Status::AssemblyVote { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn update_before_deadline() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    assert!(
        governance::ExecuteMsg::Update {
            proposal: 0,
            padding: None
        }
        .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
        .is_err()
    );
}

#[test]
fn update_after_deadline() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    assert!(
        governance::ExecuteMsg::Update {
            proposal: 0,
            padding: None
        }
        .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
        .is_ok()
    );
}

#[test]
fn invalid_vote() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    assert!(
        governance::ExecuteMsg::AssemblyVote {
            proposal: 0,
            vote: Vote {
                yes: Uint128::new(1),
                no: Uint128::new(1),
                no_with_veto: Default::default(),
                abstain: Default::default()
            },
            padding: None
        }
        .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
        .is_err()
    );
}

#[test]
fn unauthorised_vote() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    assert!(
        governance::ExecuteMsg::AssemblyVote {
            proposal: 0,
            vote: Vote {
                yes: Uint128::zero(),
                no: Uint128::new(1),
                no_with_veto: Uint128::zero(),
                abstain: Uint128::zero()
            },
            padding: None
        }
        .test_exec(&gov, &mut chain, Addr::unchecked("foxtrot"), &[])
        .is_err()
    );
}

#[test]
fn vote_after_deadline() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    assert!(
        governance::ExecuteMsg::AssemblyVote {
            proposal: 0,
            vote: Vote {
                yes: Uint128::zero(),
                no: Uint128::new(1),
                no_with_veto: Uint128::zero(),
                abstain: Uint128::zero()
            },
            padding: None
        }
        .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
        .is_err()
    );
}

#[test]
fn vote_yes() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::AssemblyVote { .. } => assert!(true),
        _ => assert!(false),
    };

    assert_eq!(
        prop.assembly_vote_tally,
        Some(Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero()
        })
    )
}

#[test]
fn vote_abstain() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::new(1),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::AssemblyVote { .. } => assert!(true),
        _ => assert!(false),
    };

    assert_eq!(
        prop.assembly_vote_tally,
        Some(Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::new(1)
        })
    )
}

#[test]
fn vote_no() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::zero(),
            no: Uint128::new(1),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::AssemblyVote { .. } => assert!(true),
        _ => assert!(false),
    };

    assert_eq!(
        prop.assembly_vote_tally,
        Some(Vote {
            yes: Uint128::zero(),
            no: Uint128::new(1),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero()
        })
    )
}

#[test]
fn vote_veto() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::new(1),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::AssemblyVote { .. } => assert!(true),
        _ => assert!(false),
    };

    assert_eq!(
        prop.assembly_vote_tally,
        Some(Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::new(1),
            abstain: Uint128::zero()
        })
    )
}

#[test]
fn vote_passed() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Passed { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn vote_abstained() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::new(1),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::new(1),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Rejected { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn vote_rejected() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::zero(),
            no: Uint128::new(1),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::zero(),
            no: Uint128::new(1),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Rejected { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn vote_vetoed() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::new(1),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::new(1),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        // NOTE: assembly votes cannot be vetoed
        Status::Rejected { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn vote_no_quorum() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::new(1),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert_eq!(prop.status, Status::Expired);
}

#[test]
fn vote_total() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::new(1),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("charlie"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::AssemblyVote { .. } => assert!(true),
        _ => assert!(false),
    };

    assert_eq!(
        prop.assembly_vote_tally,
        Some(Vote {
            yes: Uint128::new(2),
            no: Uint128::zero(),
            no_with_veto: Uint128::new(1),
            abstain: Uint128::zero()
        })
    )
}

#[test]
fn update_vote() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::new(1),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert_eq!(
        prop.assembly_vote_tally,
        Some(Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::new(1),
            abstain: Uint128::zero()
        })
    );

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert_eq!(
        prop.assembly_vote_tally,
        Some(Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero()
        })
    );
}

#[test]
fn vote_count() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Passed { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn vote_count_percentage() {
    let (mut chain, auth) = init_chain();

    let gov = InstantiateMsg {
        treasury: Addr::unchecked("treasury"),
        query_auth: Contract {
            address: auth.address,
            code_hash: auth.code_hash,
        },
        funding_token: None,
        vote_token: None,
        assemblies: Some(AssemblyInit {
            admin_members: vec![
                Addr::unchecked("alpha"),
                Addr::unchecked("beta"),
                Addr::unchecked("charlie"),
            ],
            admin_profile: Profile {
                name: "admin".to_string(),
                enabled: true,
                assembly: Some(VoteProfile {
                    deadline: 10000,
                    threshold: Count::Percentage { percent: 6500 },
                    yes_threshold: Count::Percentage { percent: 6500 },
                    veto_threshold: Count::Percentage { percent: 6500 },
                }),
                funding: None,
                token: None,
                cancel_deadline: 0,
            },
            public_profile: Profile {
                name: "public".to_string(),
                enabled: false,
                assembly: None,
                funding: None,
                token: None,
                cancel_deadline: 0,
            },
        }),
        migrator: None,
    }
    .test_init(
        Governance::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "governance",
        &[],
    )
    .unwrap();

    governance::ExecuteMsg::AssemblyProposal {
        assembly: 1,
        title: "Title".to_string(),
        metadata: "Text only proposal".to_string(),
        msgs: None,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    governance::ExecuteMsg::AssemblyVote {
        proposal: 0,
        vote: Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Passed { .. } => assert!(true),
        _ => assert!(false),
    };
}
