use crate::tests::{handle::init_funding_token, init_chain};
use rstest::*;
use shade_multi_test::multi::governance::Governance;
use shade_protocol::{
    c_std::{to_binary, Addr, ContractInfo, StdResult, Uint128},
    governance,
    governance::{
        profile::{Count, FundProfile, Profile, VoteProfile},
        vote::Vote,
        AssemblyInit,
        InstantiateMsg,
        RuntimeState,
    },
    multi_test::{App, AppResponse, Executor},
    snip20,
    utils::{asset::Contract, ExecuteCallback, MultiTestable},
    AnyResult,
};

pub fn init_gov() -> StdResult<(App, ContractInfo, ContractInfo, u64)> {
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
    let stored_code = chain.store_code(Governance::default().contract());
    let gov = chain
        .instantiate_contract(
            stored_code.clone(),
            Addr::unchecked("admin"),
            &InstantiateMsg {
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
                        assembly: Some(VoteProfile {
                            deadline: 1000,
                            threshold: Count::LiteralCount {
                                count: Uint128::new(1),
                            },
                            yes_threshold: Count::LiteralCount {
                                count: Uint128::new(1),
                            },
                            veto_threshold: Count::LiteralCount {
                                count: Uint128::new(1),
                            },
                        }),
                        funding: Some(FundProfile {
                            deadline: 1000,
                            required: Uint128::new(1000),
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
            },
            &vec![],
            "governance",
            None,
        )
        .unwrap();

    Ok((chain, gov, snip20, stored_code.code_id))
}

fn update_proposal(chain: &mut App, gov: &ContractInfo, proposal: u32) -> AnyResult<AppResponse> {
    governance::ExecuteMsg::Update {
        proposal,
        padding: None,
    }
    .test_exec(&gov, chain, Addr::unchecked("beta"), &[])
}

fn create_proposal(chain: &mut App, gov: &ContractInfo, assembly: u16) -> AnyResult<AppResponse> {
    governance::ExecuteMsg::AssemblyProposal {
        assembly,
        title: "Title".to_string(),
        metadata: "Text only proposal".to_string(),
        msgs: None,
        padding: None,
    }
    .test_exec(&gov, chain, Addr::unchecked("alpha"), &[])
}

fn assembly_vote(chain: &mut App, gov: &ContractInfo, proposal: u32) -> AnyResult<AppResponse> {
    governance::ExecuteMsg::AssemblyVote {
        proposal,
        vote: Vote {
            yes: Uint128::new(1),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        },
        padding: None,
    }
    .test_exec(&gov, chain, Addr::unchecked("alpha"), &[])
}

fn fund_proposal(
    chain: &mut App,
    gov: &ContractInfo,
    snip20: &ContractInfo,
    proposal: u32,
) -> AnyResult<AppResponse> {
    snip20::ExecuteMsg::Send {
        recipient: gov.address.clone().into(),
        recipient_code_hash: None,
        amount: Uint128::new(100),
        msg: Some(to_binary(&proposal).unwrap()),
        memo: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &snip20,
        chain,
        Addr::unchecked("alpha"),
        &[],
    )
}

// Use RS test to run all the expected functions under all of the states
#[rstest]
#[case(RuntimeState::Normal, 1, true)]
#[case(RuntimeState::SpecificAssemblies { assemblies: vec![1] }, 2, false)]
#[case(RuntimeState::SpecificAssemblies { assemblies: vec![1] }, 1, true)]
#[case(RuntimeState::Migrated, 1, false)]
fn runstate_states(#[case] state: RuntimeState, #[case] assembly: u16, #[case] expect: bool) {
    let (mut chain, gov, snip20, gov_id) = init_gov().unwrap();

    governance::ExecuteMsg::AddAssembly {
        name: "Other assembly".to_string(),
        metadata: "some data".to_string(),
        members: vec![
            Addr::unchecked("alpha"),
            Addr::unchecked("beta"),
            Addr::unchecked("charlie"),
        ],
        profile: 1,
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

    // Proposal for claiming funding
    create_proposal(&mut chain, &gov, assembly).unwrap();
    assembly_vote(&mut chain, &gov, 0).unwrap();
    chain.update_block(|block| block.time = block.time.plus_seconds(10000));
    update_proposal(&mut chain, &gov, 0).unwrap();
    fund_proposal(&mut chain, &gov, &snip20, 0).unwrap();
    chain.update_block(|block| block.time = block.time.plus_seconds(10000));
    update_proposal(&mut chain, &gov, 0).unwrap();

    // Proposal for updating state
    create_proposal(&mut chain, &gov, assembly).unwrap();
    assembly_vote(&mut chain, &gov, 1).unwrap();
    chain.update_block(|block| block.time = block.time.plus_seconds(10000));

    // Proposal for voting
    create_proposal(&mut chain, &gov, assembly).unwrap();

    match state {
        RuntimeState::Normal => {}
        RuntimeState::SpecificAssemblies { assemblies } => {
            governance::ExecuteMsg::SetRuntimeState {
                state: RuntimeState::SpecificAssemblies { assemblies },
                padding: None,
            }
            .test_exec(&gov, &mut chain, gov.address.clone(), &[])
            .unwrap();
        }
        RuntimeState::Migrated => {
            governance::ExecuteMsg::Migrate {
                id: gov_id,
                label: "migrated".to_string(),
                code_hash: gov.code_hash.clone(),
            }
            .test_exec(
                // Sender is self
                &gov,
                &mut chain,
                gov.address.clone(),
                &[],
            )
            .unwrap();
        }
    }

    // try to create proposal
    assert_eq!(expect, create_proposal(&mut chain, &gov, assembly).is_ok());

    // Claim funding TODO: should this get halted?
    assert_eq!(
        true,
        governance::ExecuteMsg::ClaimFunding { id: 0 }
            .test_exec(&gov, &mut chain, Addr::unchecked("alpha"), &[])
            .is_ok()
    );

    assert_eq!(expect, update_proposal(&mut chain, &gov, 1).is_ok());

    assert_eq!(expect, assembly_vote(&mut chain, &gov, 2).is_ok());

    // TODO: not working but progress to user voting
}
