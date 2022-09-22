use crate::tests::{get_proposals, handle::proposal::init_funding_token, init_chain};
use shade_multi_test::multi::{governance::Governance, snip20::Snip20};
use shade_protocol::{
    c_std::{to_binary, Addr, ContractInfo, StdResult, Uint128},
    contract_interfaces::{
        governance,
        governance::{
            profile::{Count, FundProfile, Profile, UpdateProfile, UpdateVoteProfile},
            proposal::Status,
            vote::Vote,
            InstantiateMsg,
        },
        query_auth,
        snip20,
    },
    governance::AssemblyInit,
    multi_test::App,
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

pub fn init_funding_governance_with_proposal()
-> StdResult<(App, ContractInfo, ContractInfo, ContractInfo)> {
    let (mut chain, auth) = init_chain();

    // Register snip20
    let snip20 = snip20::InstantiateMsg {
        name: "funding_token".to_string(),
        admin: None,
        symbol: "FND".to_string(),
        decimals: 6,
        initial_balances: Some(vec![
            snip20::InitialBalance {
                address: "alpha".into(),
                amount: Uint128::new(10000),
            },
            snip20::InitialBalance {
                address: "beta".into(),
                amount: Uint128::new(10000),
            },
            snip20::InitialBalance {
                address: "charlie".into(),
                amount: Uint128::new(10000),
            },
        ]),
        prng_seed: Default::default(),
        config: None,
        query_auth: None,
    }
    .test_init(
        Snip20::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "funding_token",
        &[],
    )
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
                funding: Some(FundProfile {
                    deadline: 1000,
                    required: Uint128::new(2000),
                    privacy: false,
                    veto_deposit_loss: Default::default(),
                }),
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
        funding_token: Some(Contract {
            address: snip20.address.clone(),
            code_hash: snip20.code_hash.clone(),
        }),
        vote_token: None,
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

    snip20::ExecuteMsg::SetViewingKey {
        key: "password".to_string(),
        padding: None,
    }
    .test_exec(&snip20, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    query_auth::ExecuteMsg::SetViewingKey {
        key: "password".to_string(),
        padding: None,
    }
    .test_exec(&auth, &mut chain, Addr::unchecked("alpha"), &[])
    .unwrap();

    snip20::ExecuteMsg::SetViewingKey {
        key: "password".to_string(),
        padding: None,
    }
    .test_exec(&snip20, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    snip20::ExecuteMsg::SetViewingKey {
        key: "password".to_string(),
        padding: None,
    }
    .test_exec(&snip20, &mut chain, Addr::unchecked("charlie"), &[])
    .unwrap();

    Ok((chain, gov, snip20, auth))
}

#[test]
fn assembly_to_funding_transition() {
    let (mut chain, gov, _snip20, _auth) = init_funding_governance_with_proposal().unwrap();

    governance::ExecuteMsg::SetProfile {
        id: 1,
        profile: UpdateProfile {
            name: None,
            enabled: None,
            disable_assembly: false,
            assembly: Some(UpdateVoteProfile {
                deadline: Some(1000),
                threshold: Some(Count::LiteralCount {
                    count: Uint128::new(1),
                }),
                yes_threshold: Some(Count::LiteralCount {
                    count: Uint128::new(1),
                }),
                veto_threshold: Some(Count::LiteralCount {
                    count: Uint128::new(1),
                }),
            }),
            disable_funding: false,
            funding: None,
            disable_token: false,
            token: None,
            cancel_deadline: None,
        },
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
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
        proposal: 1,
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
        proposal: 1,
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
        proposal: 1,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 1, 2).unwrap()[0].clone();

    assert_eq!(prop.title, "Title".to_string());
    assert_eq!(prop.metadata, "Text only proposal".to_string());
    assert_eq!(prop.proposer, Addr::unchecked("alpha"));
    assert_eq!(prop.assembly, 1);

    // Check that history works
    match prop.status_history[0] {
        Status::AssemblyVote { .. } => assert!(true),
        _ => assert!(false),
    }

    match prop.status {
        Status::Funding { .. } => assert!(true),
        _ => assert!(false),
    };
}
#[test]
fn fake_funding_token() {
    let (mut chain, gov, snip20, _) = init_funding_governance_with_proposal().unwrap();

    let other = snip20::InstantiateMsg {
        name: "funding_token".to_string(),
        admin: None,
        symbol: "FND".to_string(),
        decimals: 6,
        initial_balances: Some(vec![
            snip20::InitialBalance {
                address: "alpha".into(),
                amount: Uint128::new(10000),
            },
            snip20::InitialBalance {
                address: "beta".into(),
                amount: Uint128::new(10000),
            },
            snip20::InitialBalance {
                address: "charlie".into(),
                amount: Uint128::new(10000),
            },
        ]),
        prng_seed: Default::default(),
        config: None,
        query_auth: None,
    }
    .test_init(
        Snip20::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "other_snip20",
        &[],
    )
    .unwrap();

    governance::ExecuteMsg::SetConfig {
        query_auth: None,
        treasury: None,
        funding_token: Some(Contract {
            address: other.address.clone(),
            code_hash: other.code_hash,
        }),
        vote_token: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    assert!(
        snip20::ExecuteMsg::Send {
            recipient: gov.address.into(),
            recipient_code_hash: None,
            amount: Uint128::new(100),
            msg: None,
            memo: None,
            padding: None
        }
        .test_exec(
            // Sender is self
            &snip20,
            &mut chain,
            Addr::unchecked("alpha"),
            &[]
        )
        .is_err()
    );
}
#[test]
fn funding_proposal_without_msg() {
    let (mut chain, gov, snip20, _auth) = init_funding_governance_with_proposal().unwrap();

    assert!(
        snip20::ExecuteMsg::Send {
            recipient: gov.address.into(),
            recipient_code_hash: None,
            amount: Uint128::new(100),
            msg: None,
            memo: None,
            padding: None
        }
        .test_exec(
            // Sender is self
            &snip20,
            &mut chain,
            Addr::unchecked("alpha"),
            &[]
        )
        .is_err()
    );
}
#[test]
fn funding_proposal() {
    let (mut chain, gov, snip20, _auth) = init_funding_governance_with_proposal().unwrap();

    snip20::ExecuteMsg::Send {
        recipient: gov.address.clone().into(),
        recipient_code_hash: None,
        amount: Uint128::new(100),
        msg: Some(to_binary(&0).unwrap()),
        memo: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &snip20,
        &mut chain,
        Addr::unchecked("alpha"),
        &[],
    )
    .unwrap();

    snip20::ExecuteMsg::Send {
        recipient: gov.address.clone().into(),
        recipient_code_hash: None,
        amount: Uint128::new(100),
        msg: Some(to_binary(&0).unwrap()),
        memo: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &snip20,
        &mut chain,
        Addr::unchecked("beta"),
        &[],
    )
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Funding { amount, .. } => assert_eq!(amount, Uint128::new(200)),
        _ => assert!(false),
    };
}
#[test]
fn funding_proposal_after_deadline() {
    let (mut chain, gov, snip20, _auth) = init_funding_governance_with_proposal().unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(10000));

    assert!(
        snip20::ExecuteMsg::Send {
            recipient: gov.address.into(),
            recipient_code_hash: None,
            amount: Uint128::new(100),
            msg: Some(to_binary(&0).unwrap()),
            memo: None,
            padding: None
        }
        .test_exec(
            // Sender is self
            &snip20,
            &mut chain,
            Addr::unchecked("alpha"),
            &[]
        )
        .is_err()
    )
}
#[test]
fn update_while_funding() {
    let (mut chain, gov, _snip20, _auth) = init_funding_governance_with_proposal().unwrap();

    assert!(
        governance::ExecuteMsg::Update {
            proposal: 0,
            padding: None
        }
        .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
        .is_err()
    );
}
#[test]
fn update_when_fully_funded() {
    let (mut chain, gov, snip20, _auth) = init_funding_governance_with_proposal().unwrap();

    snip20::ExecuteMsg::Send {
        recipient: gov.address.clone().into(),
        recipient_code_hash: None,
        amount: Uint128::new(1000),
        msg: Some(to_binary(&0).unwrap()),
        memo: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &snip20,
        &mut chain,
        Addr::unchecked("alpha"),
        &[],
    )
    .unwrap();

    snip20::ExecuteMsg::Send {
        recipient: gov.address.clone().into(),
        recipient_code_hash: None,
        amount: Uint128::new(1000),
        msg: Some(to_binary(&0).unwrap()),
        memo: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &snip20,
        &mut chain,
        Addr::unchecked("beta"),
        &[],
    )
    .unwrap();

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
fn update_after_failed_funding() {
    let (mut chain, gov, snip20, _auth) = init_funding_governance_with_proposal().unwrap();

    snip20::ExecuteMsg::Send {
        recipient: gov.address.clone().into(),
        recipient_code_hash: None,
        amount: Uint128::new(1000),
        msg: Some(to_binary(&0).unwrap()),
        memo: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &snip20,
        &mut chain,
        Addr::unchecked("alpha"),
        &[],
    )
    .unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(10000));

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    match prop.status {
        Status::Expired {} => assert!(true),
        _ => assert!(false),
    };
}
#[test]
fn claim_when_not_finished() {
    let (mut chain, gov, snip20, _auth) = init_funding_governance_with_proposal().unwrap();

    snip20::ExecuteMsg::Send {
        recipient: gov.address.into(),
        recipient_code_hash: None,
        amount: Uint128::new(1000),
        msg: Some(to_binary(&0).unwrap()),
        memo: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &snip20,
        &mut chain,
        Addr::unchecked("alpha"),
        &[],
    )
    .unwrap();

    assert!(
        governance::ExecuteMsg::ClaimFunding { id: 0 }
            .test_exec(
                // Sender is self
                &snip20,
                &mut chain,
                Addr::unchecked("alpha"),
                &[]
            )
            .is_err()
    );
}
#[test]
fn claim_after_failing() {
    let (mut chain, gov, snip20, _auth) = init_funding_governance_with_proposal().unwrap();

    snip20::ExecuteMsg::Send {
        recipient: gov.address.clone().into(),
        recipient_code_hash: None,
        amount: Uint128::new(1000),
        msg: Some(to_binary(&0).unwrap()),
        memo: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &snip20,
        &mut chain,
        Addr::unchecked("alpha"),
        &[],
    )
    .unwrap();

    chain.update_block(|block| block.time = block.time.plus_seconds(10000));

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    governance::ExecuteMsg::ClaimFunding { id: 0 }
        .test_exec(
            // Sender is self
            &gov,
            &mut chain,
            Addr::unchecked("alpha"),
            &[],
        )
        .unwrap();

    let query: snip20::QueryAnswer = snip20::QueryMsg::Balance {
        address: "alpha".into(),
        key: "password".to_string(),
    }
    .test_query(&snip20, &chain)
    .unwrap();

    match query {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::new(10000))
        }
        _ => assert!(false),
    };
}
#[test]
fn claim_after_passing() {
    let (mut chain, gov, snip20, _auth) = init_funding_governance_with_proposal().unwrap();

    snip20::ExecuteMsg::Send {
        recipient: gov.address.clone().into(),
        recipient_code_hash: None,
        amount: Uint128::new(2000),
        msg: Some(to_binary(&0).unwrap()),
        memo: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &snip20,
        &mut chain,
        Addr::unchecked("alpha"),
        &[],
    )
    .unwrap();

    governance::ExecuteMsg::Update {
        proposal: 0,
        padding: None,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("beta"), &[])
    .unwrap();

    governance::ExecuteMsg::ClaimFunding { id: 0 }
        .test_exec(
            // Sender is self
            &gov,
            &mut chain,
            Addr::unchecked("alpha"),
            &[],
        )
        .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert_eq!(
        prop.funders.unwrap()[0],
        (Addr::unchecked("alpha"), Uint128::new(2000))
    );

    let query: snip20::QueryAnswer = snip20::QueryMsg::Balance {
        address: "alpha".into(),
        key: "password".to_string(),
    }
    .test_query(&snip20, &chain)
    .unwrap();

    match query {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::new(10000))
        }
        _ => assert!(false),
    };
}

fn init_funding_governance_with_proposal_with_privacy()
-> StdResult<(App, ContractInfo, ContractInfo, ContractInfo)> {
    let (mut chain, auth) = init_chain();

    // Register snip20
    let snip20 = init_funding_token(
        &mut chain,
        Some(vec![
            snip20::InitialBalance {
                address: "alpha".into(),
                amount: Uint128::new(10000),
            },
            snip20::InitialBalance {
                address: "beta".into(),
                amount: Uint128::new(10000),
            },
            snip20::InitialBalance {
                address: "charlie".into(),
                amount: Uint128::new(10000),
            },
        ]),
        Some(&auth),
    )?;

    // Register governance
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
                funding: Some(FundProfile {
                    deadline: 1000,
                    required: Uint128::new(2000),
                    privacy: true,
                    veto_deposit_loss: Default::default(),
                }),
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
        funding_token: Some(Contract {
            address: snip20.address.clone(),
            code_hash: snip20.code_hash.clone(),
        }),
        vote_token: None,
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

    Ok((chain, gov, snip20, auth))
}

#[test]
fn funding_privacy() {
    let (mut chain, gov, snip20, _auth) =
        init_funding_governance_with_proposal_with_privacy().unwrap();

    snip20::ExecuteMsg::Send {
        recipient: gov.address.clone().into(),
        recipient_code_hash: None,
        amount: Uint128::new(2000),
        msg: Some(to_binary(&0).unwrap()),
        memo: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &snip20,
        &mut chain,
        Addr::unchecked("alpha"),
        &[],
    )
    .unwrap();

    let prop = get_proposals(&mut chain, &gov, 0, 2).unwrap()[0].clone();

    assert!(prop.funders.is_none());
}
