use crate::tests::{get_proposals, init_governance};
use contract_harness::harness::{
    governance::Governance,
    snip20::Snip20,
    snip20_staking::Snip20Staking,
};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{to_binary, HumanAddr, StdResult};
use fadroma::ensemble::{ContractEnsemble, MockEnv};
use fadroma::platform_scrt::ContractLink;
use shade_protocol::{
    contract_interfaces::{
        governance,
        governance::{
            profile::{Count, Profile, VoteProfile},
            proposal::Status,
            vote::Vote,
            InitMsg,
        },
        staking::snip20_staking,
    },
    utils::asset::Contract,
};

fn init_voting_governance_with_proposal() -> StdResult<(
    ContractEnsemble,
    ContractLink<HumanAddr>,
    ContractLink<HumanAddr>,
)> {
    let mut chain = ContractEnsemble::new(50);

    // Register snip20
    let snip20 = chain.register(Box::new(Snip20));
    let snip20 = chain.instantiate(
        snip20.id,
        &snip20_reference_impl::msg::InitMsg {
            name: "token".to_string(),
            admin: None,
            symbol: "TKN".to_string(),
            decimals: 6,
            initial_balances: Some(vec![
                snip20_reference_impl::msg::InitialBalance {
                    address: HumanAddr::from("alpha"),
                    amount: cosmwasm_std::Uint128(20_000_000),
                },
                snip20_reference_impl::msg::InitialBalance {
                    address: HumanAddr::from("beta"),
                    amount: cosmwasm_std::Uint128(20_000_000),
                },
                snip20_reference_impl::msg::InitialBalance {
                    address: HumanAddr::from("charlie"),
                    amount: cosmwasm_std::Uint128(20_000_000),
                },
            ]),
            prng_seed: Default::default(),
            config: None,
        },
        MockEnv::new("admin", ContractLink {
            address: "token".into(),
            code_hash: snip20.code_hash,
        }),
    )?;

    let stkd_tkn = chain.register(Box::new(Snip20Staking));
    let stkd_tkn = chain.instantiate(
        stkd_tkn.id,
        &spip_stkd_0::msg::InitMsg {
            name: "Staked TKN".to_string(),
            admin: None,
            symbol: "TKN".to_string(),
            decimals: Some(6),
            share_decimals: 18,
            prng_seed: Default::default(),
            config: None,
            unbond_time: 0,
            staked_token: Contract {
                address: snip20.address.clone(),
                code_hash: snip20.code_hash.clone(),
            },
            treasury: None,
            treasury_code_hash: None,
            limit_transfer: false,
            distributors: None,
        },
        MockEnv::new("admin", ContractLink {
            address: "staked_token".into(),
            code_hash: stkd_tkn.code_hash,
        }),
    )?;

    // Stake tokens
    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::Send {
            recipient: stkd_tkn.address.clone(),
            recipient_code_hash: None,
            amount: cosmwasm_std::Uint128(20_000_000),
            memo: None,
            msg: Some(to_binary(&snip20_staking::ReceiveType::Bond { use_from: None }).unwrap()),
            padding: None,
        },
        MockEnv::new("alpha", ContractLink {
            address: snip20.address.clone(),
            code_hash: snip20.code_hash.clone(),
        }),
    )?;
    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::Send {
            recipient: stkd_tkn.address.clone(),
            recipient_code_hash: None,
            amount: cosmwasm_std::Uint128(20_000_000),
            memo: None,
            msg: Some(to_binary(&snip20_staking::ReceiveType::Bond { use_from: None }).unwrap()),
            padding: None,
        },
        MockEnv::new("beta", ContractLink {
            address: snip20.address.clone(),
            code_hash: snip20.code_hash.clone(),
        }),
    )?;
    chain.execute(
        &snip20_reference_impl::msg::HandleMsg::Send {
            recipient: stkd_tkn.address.clone(),
            recipient_code_hash: None,
            amount: cosmwasm_std::Uint128(20_000_000),
            memo: None,
            msg: Some(to_binary(&snip20_staking::ReceiveType::Bond { use_from: None }).unwrap()),
            padding: None,
        },
        MockEnv::new("charlie", ContractLink {
            address: snip20.address.clone(),
            code_hash: snip20.code_hash.clone(),
        }),
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
                HumanAddr::from("charlie"),
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
            funding_token: None,
            vote_token: Some(Contract {
                address: stkd_tkn.address.clone(),
                code_hash: stkd_tkn.code_hash.clone(),
            }),
        },
        MockEnv::new("admin", ContractLink {
            address: "gov".into(),
            code_hash: gov.code_hash,
        }),
    )?;

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

    Ok((chain, gov, stkd_tkn))
}

#[test]
fn voting() {
    let (mut chain, gov, _) = init_voting_governance_with_proposal().unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    match prop.status {
        Status::Voting { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn update_before_deadline() {
    let (mut chain, gov, _) = init_voting_governance_with_proposal().unwrap();

    assert!(
        chain
            .execute(
                &governance::HandleMsg::Update {
                    proposal: Uint128::new(0),
                    padding: None
                },
                MockEnv::new("alpha", ContractLink {
                    address: gov.address.clone(),
                    code_hash: gov.code_hash.clone(),
                })
            )
            .is_err()
    );
}

#[test]
fn update_after_deadline() {
    let (mut chain, gov, _) = init_voting_governance_with_proposal().unwrap();

    chain.block().time += 30000;

    assert!(
        chain
            .execute(
                &governance::HandleMsg::Update {
                    proposal: Uint128::new(0),
                    padding: None
                },
                MockEnv::new("alpha", ContractLink {
                    address: gov.address.clone(),
                    code_hash: gov.code_hash.clone(),
                })
            )
            .is_ok()
    );
}

#[test]
fn invalid_vote() {
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    assert!(
        chain
            .execute(
                &snip20_staking::HandleMsg::ExposeBalance {
                    recipient: gov.address,
                    code_hash: None,
                    msg: Some(
                        to_binary(&governance::vote::ReceiveBalanceMsg {
                            vote: Vote {
                                yes: Uint128::new(25_000_000),
                                no: Default::default(),
                                no_with_veto: Default::default(),
                                abstain: Default::default()
                            },
                            proposal: Uint128::zero()
                        })
                        .unwrap()
                    ),
                    memo: None,
                    padding: None
                },
                MockEnv::new("alpha", ContractLink {
                    address: stkd_tkn.address.clone(),
                    code_hash: stkd_tkn.code_hash.clone(),
                })
            )
            .is_err()
    );
}

#[test]
fn vote_after_deadline() {
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain.block().time += 30000;

    assert!(
        chain
            .execute(
                &snip20_staking::HandleMsg::ExposeBalance {
                    recipient: gov.address,
                    code_hash: None,
                    msg: Some(
                        to_binary(&governance::vote::ReceiveBalanceMsg {
                            vote: Vote {
                                yes: Uint128::new(25_000_000),
                                no: Default::default(),
                                no_with_veto: Default::default(),
                                abstain: Default::default()
                            },
                            proposal: Uint128::zero()
                        })
                        .unwrap()
                    ),
                    memo: None,
                    padding: None
                },
                MockEnv::new("alpha", ContractLink {
                    address: stkd_tkn.address.clone(),
                    code_hash: stkd_tkn.code_hash.clone(),
                })
            )
            .is_err()
    );
}

#[test]
fn vote_yes() {
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

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

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
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::zero(),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::new(1_000_000),
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

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

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
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::zero(),
                            no: Uint128::new(1_000_000),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
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

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

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
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::zero(),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::new(1_000_000),
                            abstain: Uint128::zero(),
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

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

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

#[test]
fn vote_passed() {
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::new(10_000_000),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
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
    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::new(10_000_000),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
                        },
                        proposal: Uint128::zero(),
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: stkd_tkn.address.clone(),
                code_hash: stkd_tkn.code_hash.clone(),
            }),
        )
        .unwrap();

    chain.block().time += 30000;

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
fn vote_abstained() {
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::zero(),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::new(10_000_000),
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
    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::zero(),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::new(10_000_000),
                        },
                        proposal: Uint128::zero(),
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: stkd_tkn.address.clone(),
                code_hash: stkd_tkn.code_hash.clone(),
            }),
        )
        .unwrap();

    chain.block().time += 30000;

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
        Status::Rejected { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn vote_rejected() {
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::zero(),
                            no: Uint128::new(10_000_000),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
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
    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::zero(),
                            no: Uint128::new(10_000_000),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
                        },
                        proposal: Uint128::zero(),
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: stkd_tkn.address.clone(),
                code_hash: stkd_tkn.code_hash.clone(),
            }),
        )
        .unwrap();

    chain.block().time += 30000;

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
        Status::Rejected { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn vote_vetoed() {
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::zero(),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::new(10_000_000),
                            abstain: Uint128::zero(),
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
    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::zero(),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::new(10_000_000),
                            abstain: Uint128::zero(),
                        },
                        proposal: Uint128::zero(),
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: stkd_tkn.address.clone(),
                code_hash: stkd_tkn.code_hash.clone(),
            }),
        )
        .unwrap();

    chain.block().time += 30000;

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
        Status::Vetoed { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn vote_no_quorum() {
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::new(10),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
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
    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::new(10),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
                        },
                        proposal: Uint128::zero(),
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: stkd_tkn.address.clone(),
                code_hash: stkd_tkn.code_hash.clone(),
            }),
        )
        .unwrap();

    chain.block().time += 30000;

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
        Status::Expired { .. } => assert!(true),
        _ => assert!(false),
    };
}

#[test]
fn vote_total() {
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::new(10),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
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

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::new(10),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::new(10_000),
                            abstain: Uint128::zero(),
                        },
                        proposal: Uint128::zero(),
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: stkd_tkn.address.clone(),
                code_hash: stkd_tkn.code_hash.clone(),
            }),
        )
        .unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::zero(),
                            no: Uint128::new(23_000),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::new(10_000),
                        },
                        proposal: Uint128::zero(),
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new("charlie", ContractLink {
                address: stkd_tkn.address.clone(),
                code_hash: stkd_tkn.code_hash.clone(),
            }),
        )
        .unwrap();

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

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
}

#[test]
fn update_vote() {
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::zero(),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::new(22_000),
                            abstain: Uint128::zero(),
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

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

    assert_eq!(
        prop.public_vote_tally,
        Some(Vote {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::new(22_000),
            abstain: Uint128::zero()
        })
    );

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::new(10_000),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
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

    let prop =
        get_proposals(&mut chain, &gov, Uint128::zero(), Uint128::new(2)).unwrap()[0].clone();

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

#[test]
fn vote_count() {
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::new(10_000_000),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
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
    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::new(10_000_000),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
                        },
                        proposal: Uint128::zero(),
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: stkd_tkn.address.clone(),
                code_hash: stkd_tkn.code_hash.clone(),
            }),
        )
        .unwrap();

    chain.block().time += 30000;

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
fn vote_count_percentage() {
    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    let mut chain = ContractEnsemble::new(50);

    // Register snip20
    let snip20 = chain.register(Box::new(Snip20));
    let snip20 = chain
        .instantiate(
            snip20.id,
            &snip20_reference_impl::msg::InitMsg {
                name: "token".to_string(),
                admin: None,
                symbol: "TKN".to_string(),
                decimals: 6,
                initial_balances: Some(vec![
                    snip20_reference_impl::msg::InitialBalance {
                        address: HumanAddr::from("alpha"),
                        amount: cosmwasm_std::Uint128(20_000_000),
                    },
                    snip20_reference_impl::msg::InitialBalance {
                        address: HumanAddr::from("beta"),
                        amount: cosmwasm_std::Uint128(20_000_000),
                    },
                    snip20_reference_impl::msg::InitialBalance {
                        address: HumanAddr::from("charlie"),
                        amount: cosmwasm_std::Uint128(20_000_000),
                    },
                ]),
                prng_seed: Default::default(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: "token".into(),
                code_hash: snip20.code_hash,
            }),
        )
        .unwrap();

    let stkd_tkn = chain.register(Box::new(Snip20Staking));
    let stkd_tkn = chain
        .instantiate(
            stkd_tkn.id,
            &spip_stkd_0::msg::InitMsg {
                name: "Staked TKN".to_string(),
                admin: None,
                symbol: "TKN".to_string(),
                decimals: Some(6),
                share_decimals: 18,
                prng_seed: Default::default(),
                config: None,
                unbond_time: 0,
                staked_token: Contract {
                    address: snip20.address.clone(),
                    code_hash: snip20.code_hash.clone(),
                },
                treasury: None,
                treasury_code_hash: None,
                limit_transfer: false,
                distributors: None,
            },
            MockEnv::new("admin", ContractLink {
                address: "staked_token".into(),
                code_hash: stkd_tkn.code_hash,
            }),
        )
        .unwrap();

    // Stake tokens
    chain
        .execute(
            &snip20_reference_impl::msg::HandleMsg::Send {
                recipient: stkd_tkn.address.clone(),
                recipient_code_hash: None,
                amount: cosmwasm_std::Uint128(20_000_000),
                memo: None,
                msg: Some(
                    to_binary(&snip20_staking::ReceiveType::Bond { use_from: None }).unwrap(),
                ),
                padding: None,
            },
            MockEnv::new("alpha", ContractLink {
                address: snip20.address.clone(),
                code_hash: snip20.code_hash.clone(),
            }),
        )
        .unwrap();
    chain
        .execute(
            &snip20_reference_impl::msg::HandleMsg::Send {
                recipient: stkd_tkn.address.clone(),
                recipient_code_hash: None,
                amount: cosmwasm_std::Uint128(20_000_000),
                memo: None,
                msg: Some(
                    to_binary(&snip20_staking::ReceiveType::Bond { use_from: None }).unwrap(),
                ),
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: snip20.address.clone(),
                code_hash: snip20.code_hash.clone(),
            }),
        )
        .unwrap();
    chain
        .execute(
            &snip20_reference_impl::msg::HandleMsg::Send {
                recipient: stkd_tkn.address.clone(),
                recipient_code_hash: None,
                amount: cosmwasm_std::Uint128(20_000_000),
                memo: None,
                msg: Some(
                    to_binary(&snip20_staking::ReceiveType::Bond { use_from: None }).unwrap(),
                ),
                padding: None,
            },
            MockEnv::new("charlie", ContractLink {
                address: snip20.address.clone(),
                code_hash: snip20.code_hash.clone(),
            }),
        )
        .unwrap();

    // Register governance
    let gov = chain.register(Box::new(Governance));
    let gov = chain
        .instantiate(
            gov.id,
            &InitMsg {
                treasury: HumanAddr::from("treasury"),
                admin_members: vec![
                    HumanAddr::from("alpha"),
                    HumanAddr::from("beta"),
                    HumanAddr::from("charlie"),
                ],
                admin_profile: Profile {
                    name: "admin".to_string(),
                    enabled: true,
                    assembly: None,
                    funding: None,
                    token: Some(VoteProfile {
                        deadline: 10000,
                        threshold: Count::Percentage { percent: 3300 },
                        yes_threshold: Count::Percentage { percent: 6600 },
                        veto_threshold: Count::Percentage { percent: 3300 },
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
                funding_token: None,
                vote_token: Some(Contract {
                    address: stkd_tkn.address.clone(),
                    code_hash: stkd_tkn.code_hash.clone(),
                }),
            },
            MockEnv::new("admin", ContractLink {
                address: "gov".into(),
                code_hash: gov.code_hash,
            }),
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

    let (mut chain, gov, stkd_tkn) = init_voting_governance_with_proposal().unwrap();

    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::new(10_000_000),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
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
    chain
        .execute(
            &snip20_staking::HandleMsg::ExposeBalance {
                recipient: gov.address.clone(),
                code_hash: None,
                msg: Some(
                    to_binary(&governance::vote::ReceiveBalanceMsg {
                        vote: Vote {
                            yes: Uint128::new(10_000_000),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                            abstain: Uint128::zero(),
                        },
                        proposal: Uint128::zero(),
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new("beta", ContractLink {
                address: stkd_tkn.address.clone(),
                code_hash: stkd_tkn.code_hash.clone(),
            }),
        )
        .unwrap();

    chain.block().time += 30000;

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
