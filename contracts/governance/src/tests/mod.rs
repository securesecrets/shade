pub mod handle;
pub mod query;

use shade_multi_test::multi::{
    admin::init_admin_auth,
    governance::Governance,
    query_auth::QueryAuth,
};
use shade_protocol::{
    c_std::{to_binary, Addr, Binary, ContractInfo, StdError, StdResult},
    contract_interfaces::{
        governance,
        governance::{
            assembly::{Assembly, AssemblyMsg},
            contract::AllowedContract,
            profile::Profile,
            proposal::{Proposal, ProposalMsg},
            Config,
        },
    },
    governance::AssemblyInit,
    multi_test::App,
    query_auth,
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

pub fn init_chain() -> (App, ContractInfo) {
    let mut chain = App::default();

    let admin = init_admin_auth(&mut chain, &Addr::unchecked("admin"));

    let auth = query_auth::InstantiateMsg {
        admin_auth: Contract {
            address: admin.address.clone(),
            code_hash: admin.code_hash.clone(),
        },
        prng_seed: Binary::from("random".as_bytes()),
    }
    .test_init(
        QueryAuth::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "query_auth",
        &[],
    )
    .unwrap();

    (chain, auth)
}

pub fn admin_only_governance() -> StdResult<(App, ContractInfo)> {
    let (mut chain, auth) = init_chain();

    let gov = governance::InstantiateMsg {
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
    }
    .test_init(
        Governance::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "gov",
        &[],
    )
    .unwrap();

    Ok((chain, gov))
}

pub fn gov_generic_proposal(
    chain: &mut App,
    gov: &ContractInfo,
    sender: &str,
    msg: governance::ExecuteMsg,
) -> StdResult<()> {
    gov_msg_proposal(chain, gov, sender, vec![ProposalMsg {
        target: 0,
        assembly_msg: 0,
        msg: to_binary(&vec![serde_json::to_string(&msg).unwrap()]).unwrap(),
        send: vec![],
    }])
}

pub fn gov_msg_proposal(
    chain: &mut App,
    gov: &ContractInfo,
    sender: &str,
    msgs: Vec<ProposalMsg>,
) -> StdResult<()> {
    governance::ExecuteMsg::AssemblyProposal {
        assembly: 1,
        title: "Title".to_string(),
        metadata: "Proposal metadata".to_string(),
        msgs: Some(msgs),
        padding: None,
    }
    .test_exec(gov, chain, Addr::unchecked(sender), &[])
    .unwrap();

    Ok(())
}

pub fn get_assembly_msgs(
    chain: &mut App,
    gov: &ContractInfo,
    start: u16,
    end: u16,
) -> StdResult<Vec<AssemblyMsg>> {
    let query: governance::QueryAnswer =
        governance::QueryMsg::AssemblyMsgs { start, end }.test_query(&gov, &chain)?;

    let msgs = match query {
        governance::QueryAnswer::AssemblyMsgs { msgs } => msgs,
        _ => return Err(StdError::generic_err("Returned wrong enum")),
    };

    Ok(msgs)
}

pub fn get_contract(
    chain: &mut App,
    gov: &ContractInfo,
    start: u16,
    end: u16,
) -> StdResult<Vec<AllowedContract>> {
    let query: governance::QueryAnswer =
        governance::QueryMsg::Contracts { start, end }.test_query(&gov, &chain)?;

    match query {
        governance::QueryAnswer::Contracts { contracts } => Ok(contracts),
        _ => return Err(StdError::generic_err("Returned wrong enum")),
    }
}

pub fn get_profiles(
    chain: &mut App,
    gov: &ContractInfo,
    start: u16,
    end: u16,
) -> StdResult<Vec<Profile>> {
    let query: governance::QueryAnswer =
        governance::QueryMsg::Profiles { start, end }.test_query(&gov, &chain)?;

    match query {
        governance::QueryAnswer::Profiles { profiles } => Ok(profiles),
        _ => return Err(StdError::generic_err("Returned wrong enum")),
    }
}

pub fn get_assemblies(
    chain: &mut App,
    gov: &ContractInfo,
    start: u16,
    end: u16,
) -> StdResult<Vec<Assembly>> {
    let query: governance::QueryAnswer =
        governance::QueryMsg::Assemblies { start, end }.test_query(&gov, &chain)?;

    match query {
        governance::QueryAnswer::Assemblies { assemblies } => Ok(assemblies),
        _ => return Err(StdError::generic_err("Returned wrong enum")),
    }
}

pub fn get_proposals(
    chain: &mut App,
    gov: &ContractInfo,
    start: u32,
    end: u32,
) -> StdResult<Vec<Proposal>> {
    let query: governance::QueryAnswer =
        governance::QueryMsg::Proposals { start, end }.test_query(&gov, &chain)?;

    match query {
        governance::QueryAnswer::Proposals { props } => Ok(props),
        _ => return Err(StdError::generic_err("Returned wrong enum")),
    }
}

pub fn get_config(chain: &mut App, gov: &ContractInfo) -> StdResult<Config> {
    let query: governance::QueryAnswer = governance::QueryMsg::Config {}
        .test_query(&gov, &chain)
        .unwrap();

    match query {
        governance::QueryAnswer::Config { config } => Ok(config),
        _ => return Err(StdError::generic_err("Returned wrong enum")),
    }
}
