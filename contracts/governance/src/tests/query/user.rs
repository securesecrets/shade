use crate::tests::{
    handle::proposal::{
        assembly_voting::init_assembly_governance_with_proposal,
        funding::init_funding_governance_with_proposal,
        voting::init_voting_governance_with_proposal,
    },
    init_query_auth,
};
use contract_harness::harness;
use shade_protocol::{
    c_std::{to_binary, HumanAddr, StdResult},
    contract_interfaces::{
        governance::{self, profile::Profile, vote::Vote, AuthQuery, Pagination, QueryAnswer},
        query_auth,
        snip20,
        staking::snip20_staking,
    },
    fadroma::{
        core::ContractLink,
        ensemble::{ContractEnsemble, MockEnv},
    },
    math_compat::Uint128,
    utils::asset::Contract,
};

#[test]
fn proposals() {
    let mut chain = ContractEnsemble::new(50);
    let auth = init_query_auth(&mut chain).unwrap();

    chain
        .execute(
            &query_auth::HandleMsg::SetViewingKey {
                key: "password".to_string(),
                padding: None,
            },
            MockEnv::new("admin", auth.clone()),
        )
        .unwrap();

    let msg = governance::InitMsg {
        treasury: HumanAddr("treasury".to_string()),
        query_auth: Contract {
            address: auth.address,
            code_hash: auth.code_hash,
        },
        admin_members: vec![HumanAddr("admin".to_string())],
        admin_profile: Profile {
            name: "admin".to_string(),
            enabled: true,
            assembly: None,
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
        funding_token: None,
        vote_token: None,
    };

    let gov = harness::governance::init(&mut chain, &msg).unwrap();

    chain
        .execute(
            &governance::HandleMsg::AssemblyProposal {
                assembly: Uint128::new(1),
                title: "Title".to_string(),
                metadata: "Text".to_string(),
                msgs: None,
                padding: None,
            },
            MockEnv::new("admin", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    let query: governance::QueryAnswer = chain
        .query(gov.address.clone(), &governance::QueryMsg::WithVK {
            user: HumanAddr::from("admin"),
            key: "password".to_string(),
            query: AuthQuery::Proposals {
                pagination: Pagination {
                    page: 0,
                    amount: 10,
                },
            },
        })
        .unwrap();

    match query {
        QueryAnswer::UserProposals { props, total } => {
            assert_eq!(total, Uint128::zero());
            assert_eq!(props.len(), 1);
        }
        _ => assert!(false),
    }

    chain
        .execute(
            &governance::HandleMsg::AssemblyProposal {
                assembly: Uint128::new(1),
                title: "Title".to_string(),
                metadata: "Text".to_string(),
                msgs: None,
                padding: None,
            },
            MockEnv::new("admin", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    let query: governance::QueryAnswer = chain
        .query(gov.address.clone(), &governance::QueryMsg::WithVK {
            user: HumanAddr::from("admin"),
            key: "password".to_string(),
            query: AuthQuery::Proposals {
                pagination: Pagination {
                    page: 0,
                    amount: 10,
                },
            },
        })
        .unwrap();

    match query {
        QueryAnswer::UserProposals { props, total } => {
            assert_eq!(total, Uint128::new(1));
            assert_eq!(props.len(), 2);
        }
        _ => assert!(false),
    }

    let query: StdResult<governance::QueryAnswer> =
        chain.query(gov.address.clone(), &governance::QueryMsg::WithVK {
            user: HumanAddr::from("admin"),
            key: "not_password".to_string(),
            query: AuthQuery::Proposals {
                pagination: Pagination {
                    page: 0,
                    amount: 10,
                },
            },
        });
    assert!(query.is_err())
}

#[test]
fn assembly_votes() {
    let (mut chain, gov) = init_assembly_governance_with_proposal().unwrap();

    chain
        .execute(
            &governance::HandleMsg::AssemblyVote {
                proposal: Uint128::new(0),
                vote: Vote {
                    yes: Uint128::new(1),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero(),
                },
                padding: None,
            },
            MockEnv::new("alpha", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    let query: governance::QueryAnswer = chain
        .query(gov.address.clone(), &governance::QueryMsg::WithVK {
            user: HumanAddr::from("alpha"),
            key: "password".to_string(),
            query: AuthQuery::AssemblyVotes {
                pagination: Pagination {
                    page: 0,
                    amount: 10,
                },
            },
        })
        .unwrap();

    match query {
        QueryAnswer::UserAssemblyVotes { votes, total } => {
            assert_eq!(total, Uint128::zero());
            assert_eq!(votes.len(), 1);
        }
        _ => assert!(false),
    }
}

#[test]
fn funding() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20::HandleMsg::Send {
                recipient: gov.address.clone(),
                recipient_code_hash: None,
                amount: Uint128::new(100),
                msg: Some(to_binary(&Uint128::zero()).unwrap()),
                memo: None,
                padding: None,
            },
            MockEnv::new(HumanAddr::from("alpha"), snip20.clone()),
        )
        .unwrap();

    let query: governance::QueryAnswer = chain
        .query(gov.address.clone(), &governance::QueryMsg::WithVK {
            user: HumanAddr::from("alpha"),
            key: "password".to_string(),
            query: AuthQuery::Funding {
                pagination: Pagination {
                    page: 0,
                    amount: 10,
                },
            },
        })
        .unwrap();

    match query {
        QueryAnswer::UserFunding { funds, total } => {
            assert_eq!(total, Uint128::zero());
            assert_eq!(funds.len(), 1);
        }
        _ => assert!(false),
    }
}

#[test]
fn votes() {
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::new(1_000_000),
                            no: Default::default(),
                            no_with_veto: Default::default(),
                            abstain: Default::default(),
                        },
                        proposal: Uint128::zero(),
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new("alpha", ContractLink {
                address: stkd_tkn.address.clone(),
                code_hash: stkd_tkn.code_hash.clone(),
            }),
        )
        .unwrap();

    let query: governance::QueryAnswer = chain
        .query(gov.address.clone(), &governance::QueryMsg::WithVK {
            user: HumanAddr::from("alpha"),
            key: "password".to_string(),
            query: AuthQuery::Votes {
                pagination: Pagination {
                    page: 0,
                    amount: 10,
                },
            },
        })
        .unwrap();

    match query {
        QueryAnswer::UserVotes { votes, total } => {
            assert_eq!(total, Uint128::zero());
            assert_eq!(votes.len(), 1);
        }
        _ => assert!(false),
    }
}
