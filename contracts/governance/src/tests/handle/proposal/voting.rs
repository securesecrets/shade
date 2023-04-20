use crate::tests::{get_proposals, handle::proposal::init_funding_token, init_chain};
use shade_multi_test::multi::{governance::Governance, snip20::Snip20};
use shade_protocol::{
    c_std::{to_binary, Addr, ContractInfo, StdResult, Uint128},
    contract_interfaces::{
        governance,
        governance::{
            profile::{Count, Profile, VoteProfile},
            proposal::Status,
            vote::Vote,
            InstantiateMsg,
        },
        snip20,
    },
    governance::AssemblyInit,
    multi_test::{App, AppResponse},
    query_auth,
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable},
    AnyResult,
};

pub fn init_voting_governance_with_proposal() -> StdResult<(App, ContractInfo, String, ContractInfo)>
{
    let (mut chain, auth) = init_chain();

    // Register snip20
    let _snip20 = init_funding_token(
        &mut chain,
        Some(vec![
            snip20::InitialBalance {
                address: "alpha".into(),
                amount: Uint128::new(20_000_000),
            },
            snip20::InitialBalance {
                address: "beta".into(),
                amount: Uint128::new(20_000_000),
            },
            snip20::InitialBalance {
                address: "charlie".into(),
                amount: Uint128::new(20_000_000),
            },
        ]),
        Some(&auth),
    )
    .unwrap();

    // Fake init token so it has a valid codehash
    let stkd_tkn = snip20::InstantiateMsg {
        name: "token".to_string(),
        admin: None,
        symbol: "TKN".to_string(),
        decimals: 6,
        initial_balances: None,
        prng_seed: to_binary("some seed").unwrap(),
        config: None,
        query_auth: None,
    }
    .test_init(
        Snip20::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "staked_token",
        &[],
    )
    .unwrap();
    // Assume they got 20_000_000 total staked

    // Register governance
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
            address: auth.address.clone(),
            code_hash: auth.code_hash.clone(),
        },
        assemblies: Some(AssemblyInit {
            admin_members: vec![
                Addr::unchecked("alpha"),
                Addr::unchecked("beta"),
                Addr::unchecked("charlie"),
            ],
            admin_profile: Profile {
                name: "admin".to_string(),
                enabled: true,
                assembly: None,
                funding: None,
                token: Some(VoteProfile {
                    deadline: 10000,
                    threshold: Count::LiteralCount {
                        count: Uint128::new(10_000_000),
                    },
                    yes_threshold: Count::LiteralCount {
                        count: Uint128::new(15_000_000),
                    },
                    veto_threshold: Count::LiteralCount {
                        count: Uint128::new(15_000_000),
                    },
                }),
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
        funding_token: None,
        vote_token: Some(Contract {
            address: stkd_tkn.address.clone(),
            code_hash: stkd_tkn.code_hash.clone(),
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

    Ok((chain, gov, stkd_tkn.address.to_string(), auth))
}

pub fn vote(
    gov: &ContractInfo,
    chain: &mut App,
    stkd: &str,
    voter: &str,
    vote: governance::vote::ReceiveBalanceMsg,
    balance: Uint128,
) -> AnyResult<AppResponse> {
    governance::ExecuteMsg::ReceiveBalance {
        sender: Addr::unchecked(voter),
        msg: Some(to_binary(&vote).unwrap()),
        balance,
        memo: None,
    }
    .test_exec(gov, chain, Addr::unchecked(stkd), &[])
}

#[test]
fn voting() {
    let (mut chain, gov, _, _auth) = init_voting_governance_with_proposal().unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert_eq!(prop.title, "Title".to_string());
    assert_eq!(prop.metadata, "Text only proposal".to_string());
    assert_eq!(prop.proposer, Addr::unchecked("alpha"));
    assert_eq!(prop.assembly, 1);

    match prop.status {
        Status::Voting { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn update_before_deadline() {
    let (mut chain, _gov, _, auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        governance::ExecuteMsg::Update {
            proposal: 0,
            padding: None
        }
        .test_exec(&auth, &mut chain, Addr::unchecked("alpha"), &[])
        .is_err()
    );
}

// TODO
/*#[test]
fn update_after_deadline() {
    let (mut chain, gov, _, _auth) = init_voting_governance_with_proposal().unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    // TODO: will crash until i get staking back up
    assert!(
        governance::ExecuteMsg::Update {
            proposal: 0,
            padding: None
        }
        .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
        .is_ok()
    );
}*/

#[test]
fn invalid_vote() {
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(25_000_000),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_err()
    );
}

#[test]
fn vote_after_deadline() {
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(10_000_000),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_err()
    );
}

#[test]
fn vote_yes() {
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(1_000_000),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Voting { .. } => assert!(true),
        _ => assert!(false),
    };

    assert_eq!(
        prop.public_vote_tally,
        Some(Vote {
            yes: Uint128::new(1_000_000),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero()
        })
    )
}

#[test]
fn vote_abstain() {
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::new(1_000_000)
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Voting { .. } => assert!(true),
        _ => assert!(false),
    };

    assert_eq!(
        prop.public_vote_tally,
        Some(Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::new(1_000_000)
        })
    )
}

#[test]
fn vote_no() {
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::new(1_000_000),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Voting { .. } => assert!(true),
        _ => assert!(false),
    };

    assert_eq!(
        prop.public_vote_tally,
        Some(Vote {
            yes: Uint128::zero(),
            no: Uint128::new(1_000_000),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero()
        })
    )
}

#[test]
fn vote_veto() {
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::new(1_000_000),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Voting { .. } => assert!(true),
        _ => assert!(false),
    };

    assert_eq!(
        prop.public_vote_tally,
        Some(Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::new(1_000_000),
            abstain: Uint128::zero()
        })
    )
}

// TODO
/*#[test]
fn vote_passed() {
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(10_000_000),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );
    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "beta",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(10_000_000),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    // Check that history works
    match prop.status_history[0] {
        Status::Voting { .. } => assert!(true),
        _ => assert!(false),
    }

    match prop.status {
        Status::Passed { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn vote_abstained() {
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::new(10_000_000)
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );
    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "beta",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::new(10_000_000)
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

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
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::new(10_000_000),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );
    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "beta",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::new(10_000_000),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

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
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::new(10_000_000),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );
    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "beta",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::new(10_000_000),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Vetoed { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn vote_no_quorum() {
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(10),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );
    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "beta",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(10),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    chain.update_block(|block| block.time = block.time.plus_seconds(30000));

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Expired { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn vote_total() {
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(10),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );
    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "beta",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(10),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::new(10_000),
                    abstain: Uint128::zero()
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );
    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "charlie",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::new(23_000),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::new(10_000),
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Voting { .. } => assert!(true),
        _ => assert!(false),
    };

    assert_eq!(
        prop.public_vote_tally,
        Some(Vote {
            yes: Uint128::new(20),
            no: Uint128::new(23_000),
            no_with_veto: Uint128::new(10_000),
            abstain: Uint128::new(10_000)
        })
    )
}*/

#[test]
fn update_vote() {
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::zero(),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::new(22_000),
                    abstain: Uint128::zero(),
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert_eq!(
        prop.public_vote_tally,
        Some(Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::new(22_000),
            abstain: Uint128::zero()
        })
    );

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(10_000),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero(),
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert_eq!(
        prop.public_vote_tally,
        Some(Vote {
            yes: Uint128::new(10_000),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero()
        })
    );
}

// TODO
/*#[test]
fn vote_count() {
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(10_000_000),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero(),
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "beta",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(10_000_000),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero(),
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

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
    let (mut chain, gov, stkd_tkn, _auth) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(10_000_000),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero(),
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "beta",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(10_000_000),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero(),
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

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
}*/
