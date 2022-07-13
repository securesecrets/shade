use crate::tests::{get_proposals, init_query_auth};
use contract_harness::harness::{self, snip20::Snip20};
use shade_protocol::{
    c_std::{to_binary, HumanAddr, StdResult},
    contract_interfaces::{
        governance,
        governance::{
            profile::{Count, FundProfile, Profile, UpdateProfile, UpdateVoteProfile},
            proposal::Status,
            vote::Vote,
            InitMsg,
        },
        query_auth,
        snip20,
    },
    fadroma::{
        core::ContractLink,
        ensemble::{ContractEnsemble, MockEnv},
    },
    math_compat::Uint128,
    utils::asset::Contract,
};

pub fn init_funding_governance_with_proposal() -> StdResult<(
    ContractEnsemble,
    ContractLink<HumanAddr>,
    ContractLink<HumanAddr>,
)> {
    let mut chain = ContractEnsemble::new(50);

    // Register snip20
    let snip20 = chain.register(Box::new(Snip20));
    let snip20 = chain
        .instantiate(
            snip20.id,
            &snip20::InitMsg {
                name: "funding_token".to_string(),
                admin: None,
                symbol: "FND".to_string(),
                decimals: 6,
                initial_balances: Some(vec![
                    snip20::InitialBalance {
                        address: HumanAddr::from("alpha"),
                        amount: Uint128::new(10000),
                    },
                    snip20::InitialBalance {
                        address: HumanAddr::from("beta"),
                        amount: Uint128::new(10000),
                    },
                    snip20::InitialBalance {
                        address: HumanAddr::from("charlie"),
                        amount: Uint128::new(10000),
                    },
                ]),
                prng_seed: Default::default(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: "funding_token".into(),
                code_hash: snip20.code_hash,
            }),
        )?
        .instance;

    // Register governance
    let auth = init_query_auth(&mut chain)?;

    chain
        .execute(
            &query_auth::HandleMsg::SetViewingKey {
                key: "password".to_string(),
                padding: None,
            },
            MockEnv::new("alpha", auth.clone()),
        )
        .unwrap();

    chain
        .execute(
            &query_auth::HandleMsg::SetViewingKey {
                key: "password".to_string(),
                padding: None,
            },
            MockEnv::new("beta", auth.clone()),
        )
        .unwrap();

    chain
        .execute(
            &query_auth::HandleMsg::SetViewingKey {
                key: "password".to_string(),
                padding: None,
            },
            MockEnv::new("charlie", auth.clone()),
        )
        .unwrap();

    let gov = harness::governance::init(&mut chain, &InitMsg {
        treasury: HumanAddr::from("treasury"),
        query_auth: Contract {
            address: auth.address,
            code_hash: auth.code_hash,
        },
        admin_members: vec![
            HumanAddr::from("alpha"),
            HumanAddr::from("beta"),
            HumanAddr::from("charlie"),
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
        funding_token: Some(Contract {
            address: snip20.address.clone(),
            code_hash: snip20.code_hash.clone(),
        }),
        vote_token: None,
    })?;

    chain.execute(
        &governance::HandleMsg::AssemblyProposal {
            assembly: Uint128::new(1),
            title: "Title".to_string(),
            metadata: "Text only proposal".to_string(),
            msgs: None,
            padding: None,
        },
        MockEnv::new("alpha", ContractLink {
            address: gov.address.clone(),
            code_hash: gov.code_hash.clone(),
        }),
    )?;

    chain.execute(
        &snip20::HandleMsg::SetViewingKey {
            key: "password".to_string(),
            padding: None,
        },
        MockEnv::new("alpha", ContractLink {
            address: snip20.address.clone(),
            code_hash: snip20.code_hash.clone(),
        }),
    )?;

    chain.execute(
        &snip20::HandleMsg::SetViewingKey {
            key: "password".to_string(),
            padding: None,
        },
        MockEnv::new("beta", ContractLink {
            address: snip20.address.clone(),
            code_hash: snip20.code_hash.clone(),
        }),
    )?;

    chain.execute(
        &snip20::HandleMsg::SetViewingKey {
            key: "password".to_string(),
            padding: None,
        },
        MockEnv::new("charlie", ContractLink {
            address: snip20.address.clone(),
            code_hash: snip20.code_hash.clone(),
        }),
    )?;

    Ok((chain, gov, snip20))
}

#[test]
fn assembly_to_funding_transition() {
    let (mut chain, gov, _snip20) = init_funding_governance_with_proposal().unwrap();
    chain
        .execute(
            &governance::HandleMsg::SetProfile {
                id: Uint128::new(1),
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
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    chain
        .execute(
            &governance::HandleMsg::AssemblyProposal {
                assembly: Uint128::new(1),
                title: "Title".to_string(),
                metadata: "Text only proposal".to_string(),
                msgs: None,
                padding: None,
            },
            MockEnv::new("alpha", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    chain
        .execute(
            &governance::HandleMsg::AssemblyVote {
                proposal: Uint128::new(1),
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

    chain
        .execute(
            &governance::HandleMsg::AssemblyVote {
                proposal: Uint128::new(1),
                vote: Vote {
                    yes: Uint128::new(1),
                    no: Uint128::zero(),
                    no_with_veto: Uint128::zero(),
                    abstain: Uint128::zero(),
                },
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    chain.block_mut().time += 30000;

    chain
        .execute(
            &governance::HandleMsg::Update {
                proposal: Uint128::new(1),
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::new(1), Uint128::new(2)).unwrap()[0].clone();

    assert_eq!(prop.title, "Title".to_string());
    assert_eq!(prop.metadata, "Text only proposal".to_string());
    assert_eq!(prop.proposer, HumanAddr::from("alpha"));
    assert_eq!(prop.assembly, Uint128::new(1));

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
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    let other = chain.register(Box::new(Snip20));
    let other = chain
        .instantiate(
            other.id,
            &snip20::InitMsg {
                name: "funding_token".to_string(),
                admin: None,
                symbol: "FND".to_string(),
                decimals: 6,
                initial_balances: Some(vec![
                    snip20::InitialBalance {
                        address: HumanAddr::from("alpha"),
                        amount: Uint128::new(10000),
                    },
                    snip20::InitialBalance {
                        address: HumanAddr::from("beta"),
                        amount: Uint128::new(10000),
                    },
                    snip20::InitialBalance {
                        address: HumanAddr::from("charlie"),
                        amount: Uint128::new(10000),
                    },
                ]),
                prng_seed: Default::default(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: "other".into(),
                code_hash: snip20.code_hash.clone(),
            }),
        )
        .unwrap()
        .instance;

    chain
        .execute(
            &governance::HandleMsg::SetConfig {
                query_auth: None,
                treasury: None,
                funding_token: Some(Contract {
                    address: other.address.clone(),
                    code_hash: other.code_hash,
                }),
                vote_token: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    assert!(
        chain
            .execute(
                &snip20::HandleMsg::Send {
                    recipient: gov.address,
                    recipient_code_hash: None,
                    amount: Uint128::new(100),
                    msg: None,
                    memo: None,
                    padding: None
                },
                MockEnv::new(
                    // Sender is self
                    HumanAddr::from("alpha"),
                    snip20.clone()
                )
            )
            .is_err()
    );
}
#[test]
fn funding_proposal_without_msg() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    assert!(
        chain
            .execute(
                &snip20::HandleMsg::Send {
                    recipient: gov.address,
                    recipient_code_hash: None,
                    amount: Uint128::new(100),
                    msg: None,
                    memo: None,
                    padding: None
                },
                MockEnv::new(
                    // Sender is self
                    HumanAddr::from("alpha"),
                    snip20.clone()
                )
            )
            .is_err()
    );
}
#[test]
fn funding_proposal() {
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
            MockEnv::new(
                // Sender is self
                HumanAddr::from("alpha"),
                snip20.clone(),
            ),
        )
        .unwrap();

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
            MockEnv::new(
                // Sender is self
                HumanAddr::from("beta"),
                snip20.clone(),
            ),
        )
        .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    match prop.status {
        Status::Funding { amount, .. } => assert_eq!(amount, Uint128::new(200)),
        _ => assert!(false),
    };
}
#[test]
fn funding_proposal_after_deadline() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain.block_mut().time += 10000;

    assert!(
        chain
            .execute(
                &snip20::HandleMsg::Send {
                    recipient: gov.address.clone(),
                    recipient_code_hash: None,
                    amount: Uint128::new(100),
                    msg: Some(to_binary(&Uint128::zero()).unwrap()),
                    memo: None,
                    padding: None
                },
                MockEnv::new(
                    // Sender is self
                    HumanAddr::from("alpha"),
                    snip20.clone()
                )
            )
            .is_err()
    )
}
#[test]
fn update_while_funding() {
    let (mut chain, gov, _snip20) = init_funding_governance_with_proposal().unwrap();

    assert!(
        chain
            .execute(
                &governance::HandleMsg::Update {
                    proposal: Uint128::zero(),
                    padding: None
                },
                MockEnv::new("beta", ContractLink {
                    address: gov.address.clone(),
                    code_hash: gov.code_hash.clone(),
                })
            )
            .is_err()
    );
}
#[test]
fn update_when_fully_funded() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20::HandleMsg::Send {
                recipient: gov.address.clone(),
                recipient_code_hash: None,
                amount: Uint128::new(1000),
                msg: Some(to_binary(&Uint128::zero()).unwrap()),
                memo: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("alpha"),
                snip20.clone(),
            ),
        )
        .unwrap();

    chain
        .execute(
            &snip20::HandleMsg::Send {
                recipient: gov.address.clone(),
                recipient_code_hash: None,
                amount: Uint128::new(1000),
                msg: Some(to_binary(&Uint128::zero()).unwrap()),
                memo: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("beta"),
                snip20.clone(),
            ),
        )
        .unwrap();

    chain
        .execute(
            &governance::HandleMsg::Update {
                proposal: Uint128::zero(),
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    match prop.status {
        Status::Passed { .. } => assert!(true),
        _ => assert!(false),
    };
}
#[test]
fn update_after_failed_funding() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20::HandleMsg::Send {
                recipient: gov.address.clone(),
                recipient_code_hash: None,
                amount: Uint128::new(1000),
                msg: Some(to_binary(&Uint128::zero()).unwrap()),
                memo: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("alpha"),
                snip20.clone(),
            ),
        )
        .unwrap();

    chain.block_mut().time += 10000;

    chain
        .execute(
            &governance::HandleMsg::Update {
                proposal: Uint128::zero(),
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    match prop.status {
        Status::Expired {} => assert!(true),
        _ => assert!(false),
    };
}
#[test]
fn claim_when_not_finished() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20::HandleMsg::Send {
                recipient: gov.address.clone(),
                recipient_code_hash: None,
                amount: Uint128::new(1000),
                msg: Some(to_binary(&Uint128::zero()).unwrap()),
                memo: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("alpha"),
                snip20.clone(),
            ),
        )
        .unwrap();

    assert!(
        chain
            .execute(
                &governance::HandleMsg::ClaimFunding {
                    id: Uint128::new(0)
                },
                MockEnv::new(
                    // Sender is self
                    HumanAddr::from("alpha"),
                    snip20.clone()
                )
            )
            .is_err()
    );
}
#[test]
fn claim_after_failing() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20::HandleMsg::Send {
                recipient: gov.address.clone(),
                recipient_code_hash: None,
                amount: Uint128::new(1000),
                msg: Some(to_binary(&Uint128::zero()).unwrap()),
                memo: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("alpha"),
                snip20.clone(),
            ),
        )
        .unwrap();

    chain.block_mut().time += 10000;

    chain
        .execute(
            &governance::HandleMsg::Update {
                proposal: Uint128::zero(),
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    chain
        .execute(
            &governance::HandleMsg::ClaimFunding {
                id: Uint128::new(0),
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("alpha"),
                gov.clone(),
            ),
        )
        .unwrap();

    let query: snip20::QueryAnswer = chain
        .query(snip20.address.clone(), &snip20::QueryMsg::Balance {
            address: HumanAddr::from("alpha"),
            key: "password".to_string(),
        })
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
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20::HandleMsg::Send {
                recipient: gov.address.clone(),
                recipient_code_hash: None,
                amount: Uint128::new(2000),
                msg: Some(to_binary(&Uint128::zero()).unwrap()),
                memo: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("alpha"),
                snip20.clone(),
            ),
        )
        .unwrap();

    chain
        .execute(
            &governance::HandleMsg::Update {
                proposal: Uint128::zero(),
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: gov.address.clone(),
                code_hash: gov.code_hash.clone(),
            }),
        )
        .unwrap();

    chain
        .execute(
            &governance::HandleMsg::ClaimFunding {
                id: Uint128::new(0),
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("alpha"),
                gov.clone(),
            ),
        )
        .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::new(0), Uint128::new(2)).unwrap()[0].clone();

    assert_eq!(
        prop.funders.unwrap()[0],
        (HumanAddr::from("alpha"), Uint128::new(2000))
    );

    let query: snip20::QueryAnswer = chain
        .query(snip20.address.clone(), &snip20::QueryMsg::Balance {
            address: HumanAddr::from("alpha"),
            key: "password".to_string(),
        })
        .unwrap();

    match query {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::new(10000))
        }
        _ => assert!(false),
    };
}

fn init_funding_governance_with_proposal_with_privacy() -> StdResult<(
    ContractEnsemble,
    ContractLink<HumanAddr>,
    ContractLink<HumanAddr>,
)> {
    let mut chain = ContractEnsemble::new(50);

    // Register snip20
    let snip20 = chain.register(Box::new(Snip20));
    let snip20 = chain
        .instantiate(
            snip20.id,
            &snip20::InitMsg {
                name: "funding_token".to_string(),
                admin: None,
                symbol: "FND".to_string(),
                decimals: 6,
                initial_balances: Some(vec![
                    snip20::InitialBalance {
                        address: HumanAddr::from("alpha"),
                        amount: Uint128::new(10000),
                    },
                    snip20::InitialBalance {
                        address: HumanAddr::from("beta"),
                        amount: Uint128::new(10000),
                    },
                    snip20::InitialBalance {
                        address: HumanAddr::from("charlie"),
                        amount: Uint128::new(10000),
                    },
                ]),
                prng_seed: Default::default(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: "funding_token".into(),
                code_hash: snip20.code_hash,
            }),
        )?
        .instance;

    // Register governance
    let auth = init_query_auth(&mut chain)?;
    let gov = harness::governance::init(&mut chain, &InitMsg {
        treasury: HumanAddr::from("treasury"),
        query_auth: Contract {
            address: auth.address,
            code_hash: auth.code_hash,
        },
        admin_members: vec![
            HumanAddr::from("alpha"),
            HumanAddr::from("beta"),
            HumanAddr::from("charlie"),
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
        funding_token: Some(Contract {
            address: snip20.address.clone(),
            code_hash: snip20.code_hash.clone(),
        }),
        vote_token: None,
    })?;

    chain.execute(
        &governance::HandleMsg::AssemblyProposal {
            assembly: Uint128::new(1),
            title: "Title".to_string(),
            metadata: "Text only proposal".to_string(),
            msgs: None,
            padding: None,
        },
        MockEnv::new("alpha", ContractLink {
            address: gov.address.clone(),
            code_hash: gov.code_hash.clone(),
        }),
    )?;

    Ok((chain, gov, snip20))
}

#[test]
fn funding_privacy() {
    let (mut chain, gov, snip20) = init_funding_governance_with_proposal_with_privacy().unwrap();

    chain
        .execute(
            &snip20::HandleMsg::Send {
                recipient: gov.address.clone(),
                recipient_code_hash: None,
                amount: Uint128::new(2000),
                msg: Some(to_binary(&Uint128::zero()).unwrap()),
                memo: None,
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("alpha"),
                snip20.clone(),
            ),
        )
        .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::new(0), Uint128::new(2)).unwrap()[0].clone();

    assert!(prop.funders.is_none());
}
