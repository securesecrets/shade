pub mod handle;
pub mod query;

use crate::contract::{execute, instantiate, query};
use shade_multi_test;
use shade_protocol::c_std::{ContractInfo, Uint128};
use shade_protocol::c_std::{
    from_binary,
    to_binary,
    Binary,
    Env,
    Addr,
    Response,
    StdError,
    StdResult,
};
use shade_protocol::serde::Serialize;
use shade_protocol::contract_interfaces::{
    governance,
    governance::{
        assembly::{Assembly, AssemblyMsg},
        contract::AllowedContract,
        profile::Profile,
        proposal::{Proposal, ProposalMsg},
        Config,
    },
};
use shade_protocol::multi_test::{App, BasicApp, ContractInfo};
use shade_protocol::query_auth;
use shade_protocol::utils::{ExecuteCallback, InstantiateCallback};
use shade_protocol::utils::wrap::unwrap;

pub fn init_chain(
) -> (BasicApp, ContractInfo) {
    let mut chain = App::default();

    let auth = query_auth::InstantiateMsg {
        admin_auth: Contract {
            address: admin.address.clone(),
            code_hash: admin.code_hash.clone(),
        },
        prng_seed: Binary::from("random".as_bytes()),
    }.test_init(
        QueryAuth::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "query_auth",
        &[]
    ).unwrap();

    (chain, auth)
}

pub fn admin_only_governance() -> StdResult<(BasicApp, ContractInfo)> {

    let (mut chain, auth) = init_chain();

    let gov = governance::InitMsg {
        treasury: Addr("treasury".to_string()),
        query_auth: Contract {
            address: auth.address,
            code_hash: auth.code_hash,
        },
        admin_members: vec![Addr("admin".to_string())],
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
    }.test_init(
        Governance::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "gov",
        &[]
    ).unwrap();

    Ok((chain, gov))
}

pub fn gov_generic_proposal(
    chain: &mut BasicApp,
    gov: &ContractInfo,
    sender: &str,
    msg: governance::ExecuteMsg,
) -> StdResult<()> {
    gov_msg_proposal(chain, gov, sender, vec![ProposalMsg {
        target: Uint128::zero(),
        assembly_msg: Uint128::zero(),
        msg: to_binary(&vec![serde_json::to_string(&msg).unwrap()])?,
        send: vec![],
    }])
}

pub fn gov_msg_proposal(
    chain: &mut BasicApp,
    gov: &ContractInfo,
    sender: &str,
    msgs: Vec<ProposalMsg>,
) -> StdResult<()> {
    governance::ExecuteMsg::AssemblyProposal {
        assembly: Uint128::new(1),
        title: "Title".to_string(),
        metadata: "Proposal metadata".to_string(),
        msgs: Some(msgs),
        padding: None,
    }.test_exec(gov, chain, Addr::unchecked(sender), &[]).unwrap();

    Ok(())
}

pub fn get_assembly_msgs(
    chain: &mut BasicApp,
    gov: &ContractInfo,
    start: Uint128,
    end: Uint128,
) -> StdResult<Vec<AssemblyMsg>> {
    let query: governance::QueryAnswer =
        chain.query(gov.address.clone(), &governance::QueryMsg::AssemblyMsgs {
            start,
            end,
        })?;

    let msgs = match query {
        governance::QueryAnswer::AssemblyMsgs { msgs } => msgs,
        _ => return Err(StdError::generic_err("Returned wrong enum")),
    };

    Ok(msgs)
}

pub fn get_contract(
    chain: &mut BasicApp,
    gov: &ContractInfo,
    start: Uint128,
    end: Uint128,
) -> StdResult<Vec<AllowedContract>> {
    let query: governance::QueryAnswer =
        chain.query(gov.address.clone(), &governance::QueryMsg::Contracts {
            start,
            end,
        })?;

    match query {
        governance::QueryAnswer::Contracts { contracts } => Ok(contracts),
        _ => return Err(StdError::generic_err("Returned wrong enum")),
    }
}

pub fn get_profiles(
    chain: &mut BasicApp,
    gov: &ContractInfo,
    start: Uint128,
    end: Uint128,
) -> StdResult<Vec<Profile>> {
    let query: governance::QueryAnswer =
        chain.query(gov.address.clone(), &governance::QueryMsg::Profiles {
            start,
            end,
        })?;

    match query {
        governance::QueryAnswer::Profiles { profiles } => Ok(profiles),
        _ => return Err(StdError::generic_err("Returned wrong enum")),
    }
}

pub fn get_assemblies(
    chain: &mut BasicApp,
    gov: &ContractInfo,
    start: Uint128,
    end: Uint128,
) -> StdResult<Vec<Assembly>> {
    let query: governance::QueryAnswer =
        chain.query(gov.address.clone(), &governance::QueryMsg::Assemblies {
            start,
            end,
        })?;

    match query {
        governance::QueryAnswer::Assemblies { assemblies } => Ok(assemblies),
        _ => return Err(StdError::generic_err("Returned wrong enum")),
    }
}

pub fn get_proposals(
    chain: &mut BasicApp,
    gov: &ContractInfo,
    start: Uint128,
    end: Uint128,
) -> StdResult<Vec<Proposal>> {
    let query: governance::QueryAnswer =
        chain.query(gov.address.clone(), &governance::QueryMsg::Proposals {
            start,
            end,
        })?;

    match query {
        governance::QueryAnswer::Proposals { props } => Ok(props),
        _ => return Err(StdError::generic_err("Returned wrong enum")),
    }
}

pub fn get_config(
    chain: &mut BasicApp,
    gov: &ContractInfo,
) -> StdResult<Config> {
    let query: governance::QueryAnswer =
        chain.query(gov.address.clone(), &governance::QueryMsg::Config {})?;

    match query {
        governance::QueryAnswer::Config { config } => Ok(config),
        _ => return Err(StdError::generic_err("Returned wrong enum")),
    }
}
