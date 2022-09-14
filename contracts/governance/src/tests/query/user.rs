use crate::tests::{
    handle::proposal::{
        assembly_voting::init_assembly_governance_with_proposal,
        funding::init_funding_governance_with_proposal,
        voting::{init_voting_governance_with_proposal, vote},
    },
    init_chain,
};
use shade_multi_test::multi::governance::Governance;
use shade_protocol::{
    c_std::{to_binary, Addr, StdResult, Uint128},
    contract_interfaces::{
        governance::{self, profile::Profile, vote::Vote, AuthQuery, Pagination, QueryAnswer},
        query_auth,
        snip20,
    },
    governance::AssemblyInit,
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

#[test]
fn proposals() {
    let (mut chain, auth) = init_chain();

    query_auth::ExecuteMsg::SetViewingKey {
        key: "password".to_string(),
        padding: None,
    }
    .test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
    .unwrap();

    let msg = governance::InstantiateMsg {
        treasury: Addr::unchecked("treasury".to_string()),
        query_auth: Contract {
            address: auth.address,
            code_hash: auth.code_hash,
        },
        assemblies: Some(AssemblyInit {
            admin_members: vec![Addr::unchecked("admin".to_string())],
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
        }),
        funding_token: None,
        vote_token: None,
        migrator: None,
    };

    let gov = msg
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
        metadata: "Text".to_string(),
        msgs: None,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
    .unwrap();

    let query: governance::QueryAnswer = governance::QueryMsg::WithVK {
        user: Addr::unchecked("admin"),
        key: "password".to_string(),
        query: AuthQuery::Proposals {
            pagination: Pagination {
                page: 0,
                amount: 10,
            },
        },
    }
    .test_query(&gov, &chain)
    .unwrap();

    match query {
        QueryAnswer::UserProposals { props, total } => {
            assert_eq!(total, 0);
            assert_eq!(props.len(), 1);
        }
        _ => assert!(false),
    }

    governance::ExecuteMsg::AssemblyProposal {
        assembly: 1,
        title: "Title".to_string(),
        metadata: "Text".to_string(),
        msgs: None,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("admin"), &[])
    .unwrap();

    let query: governance::QueryAnswer = governance::QueryMsg::WithVK {
        user: Addr::unchecked("admin"),
        key: "password".to_string(),
        query: AuthQuery::Proposals {
            pagination: Pagination {
                page: 0,
                amount: 10,
            },
        },
    }
    .test_query(&gov, &chain)
    .unwrap();

    match query {
        QueryAnswer::UserProposals { props, total } => {
            assert_eq!(total, 1);
            assert_eq!(props.len(), 2);
        }
        _ => assert!(false),
    }

    let query: StdResult<governance::QueryAnswer> = governance::QueryMsg::WithVK {
        user: Addr::unchecked("admin"),
        key: "not_password".to_string(),
        query: AuthQuery::Proposals {
            pagination: Pagination {
                page: 0,
                amount: 10,
            },
        },
    }
    .test_query(&gov, &chain);
    assert!(query.is_err())
}

#[test]
fn assembly_votes() {
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

    let query: governance::QueryAnswer = governance::QueryMsg::WithVK {
        user: Addr::unchecked("alpha"),
        key: "password".to_string(),
        query: AuthQuery::AssemblyVotes {
            pagination: Pagination {
                page: 0,
                amount: 10,
            },
        },
    }
    .test_query(&gov, &chain)
    .unwrap();

    match query {
        QueryAnswer::UserAssemblyVotes { votes, total } => {
            assert_eq!(total, 0);
            assert_eq!(votes.len(), 1);
        }
        _ => assert!(false),
    }
}

#[test]
fn funding() {
    let (mut chain, gov, snip20, _) = init_funding_governance_with_proposal().unwrap();

    snip20::ExecuteMsg::Send {
        recipient: gov.address.clone().into(),
        recipient_code_hash: None,
        amount: Uint128::new(100),
        msg: Some(to_binary(&0).unwrap()),
        memo: None,
        padding: None,
    }
    .test_exec(&snip20, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    let query: governance::QueryAnswer = governance::QueryMsg::WithVK {
        user: Addr::unchecked("alpha"),
        key: "password".to_string(),
        query: AuthQuery::Funding {
            pagination: Pagination {
                page: 0,
                amount: 10,
            },
        },
    }
    .test_query(&gov, &chain)
    .unwrap();

    match query {
        QueryAnswer::UserFunding { funds, total } => {
            assert_eq!(total, 0);
            assert_eq!(funds.len(), 1);
        }
        _ => assert!(false),
    }
}

#[test]
fn votes() {
    let (mut chain, gov, stkd_tkn, _) = init_voting_governance_with_proposal().unwrap();

    assert!(
        vote(
            &gov,
            &mut chain,
            stkd_tkn.as_str(),
            "alpha",
            governance::vote::ReceiveBalanceMsg {
                vote: Vote {
                    yes: Uint128::new(1_000_000),
                    no: Default::default(),
                    no_with_veto: Default::default(),
                    abstain: Default::default(),
                },
                proposal: 0
            },
            Uint128::new(20_000_000)
        )
        .is_ok()
    );

    let query: governance::QueryAnswer = governance::QueryMsg::WithVK {
        user: Addr::unchecked("alpha"),
        key: "password".to_string(),
        query: AuthQuery::Votes {
            pagination: Pagination {
                page: 0,
                amount: 10,
            },
        },
    }
    .test_query(&gov, &chain)
    .unwrap();

    match query {
        QueryAnswer::UserVotes { votes, total } => {
            assert_eq!(total, 0);
            assert_eq!(votes.len(), 1);
        }
        _ => assert!(false),
    }
}
