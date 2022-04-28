pub mod query;
pub mod handle;

use cosmwasm_std::{Binary, Env, from_binary, HandleResponse, HumanAddr, InitResponse, StdError, StdResult, to_binary};
use fadroma_ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv};
use fadroma_platform_scrt::ContractLink;
use cosmwasm_math_compat::Uint128;
use shade_protocol::governance;
use shade_protocol::governance::assembly::{Assembly, AssemblyMsg};
use shade_protocol::governance::profile::Profile;
use shade_protocol::governance::Config;
use shade_protocol::governance::contract::AllowedContract;
use shade_protocol::governance::proposal::Proposal;
use crate::contract::{handle, init, query};

pub struct Governance;
impl ContractHarness for Governance {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        init(
            deps,
            env,
            from_binary(&msg)?,
        )
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        handle(
            deps,
            env,
            from_binary(&msg)?
        )
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        query(
            deps,
            from_binary(&msg)?
        )
    }
}

pub struct Snip20;
impl ContractHarness for Snip20 {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        snip20_reference_impl::contract::init(
            deps,
            env,
            from_binary(&msg)?,
        )
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        snip20_reference_impl::contract::handle(
            deps,
            env,
            from_binary(&msg)?
        )
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        snip20_reference_impl::contract::query(
            deps,
            from_binary(&msg)?
        )
    }
}

pub fn init_governance(msg: governance::InitMsg) -> StdResult<(ContractEnsemble, ContractLink<HumanAddr>)>{
    let mut chain = ContractEnsemble::new(50);

    // Register governance
    let gov = chain.register(Box::new(Governance));
    let gov = chain.instantiate(
        gov.id,
        &msg,
        MockEnv::new(
            "admin",
            ContractLink {
                address: "gov".into(),
                code_hash: gov.code_hash,
            }
        )
    )?;

    Ok((chain, gov))
}

pub fn admin_only_governance() -> StdResult<(ContractEnsemble, ContractLink<HumanAddr>)> {
    init_governance(
        governance::InitMsg {
            treasury: HumanAddr("treasury".to_string()),
            admin_members: vec![HumanAddr("admin".to_string())],
            admin_profile: Profile {
                name: "admin".to_string(),
                enabled: true,
                assembly: None,
                funding: None,
                token: None,
                cancel_deadline: 0
            },
            public_profile: Profile {
                name: "public".to_string(),
                enabled: false,
                assembly: None,
                funding: None,
                token: None,
                cancel_deadline: 0
            },
            funding_token: None,
            vote_token: None
        }
    )
}

pub fn gov_generic_proposal(
    chain: &mut ContractEnsemble,
    gov: &ContractLink<HumanAddr>,
    sender: &str,
    msg: governance::HandleMsg
) -> StdResult<()> {
    chain.execute(
        &governance::HandleMsg::AssemblyProposal {
            assembly: Uint128::new(1),
            metadata: "Proposal metadata".to_string(),
            contract: Some(Uint128::zero()),
            assembly_msg: Some(Uint128::zero()),
            variables: Some(vec![to_binary(&msg)?.to_base64()]),
            coins: None,
            padding: None
        },
        MockEnv::new(
            sender,
            gov.clone()
        )
    )
}

pub fn get_assembly_msgs(
    chain: &mut ContractEnsemble,
    gov: &ContractLink<HumanAddr>,
    start: Uint128,
    end: Uint128
) -> StdResult<Vec<AssemblyMsg>> {

    let query: governance::QueryAnswer = chain.query(
        gov.address.clone(),
        &governance::QueryMsg::AssemblyMsgs { start, end }
    )?;

    let msgs = match query {
        governance::QueryAnswer::AssemblyMsgs { msgs } => msgs,
        _ => return Err(StdError::generic_err("Returned wrong enum"))
    };

    Ok(msgs)
}

pub fn get_contract(
    chain: &mut ContractEnsemble,
    gov: &ContractLink<HumanAddr>,
    start: Uint128,
    end: Uint128
) -> StdResult<Vec<AllowedContract>> {

    let query: governance::QueryAnswer = chain.query(
        gov.address.clone(),
        &governance::QueryMsg::Contracts { start, end }
    )?;

    match query {
        governance::QueryAnswer::Contracts { contracts } => Ok(contracts),
        _ => return Err(StdError::generic_err("Returned wrong enum"))
    }
}

pub fn get_profiles(
    chain: &mut ContractEnsemble,
    gov: &ContractLink<HumanAddr>,
    start: Uint128,
    end: Uint128
) -> StdResult<Vec<Profile>> {

    let query: governance::QueryAnswer = chain.query(
        gov.address.clone(),
        &governance::QueryMsg::Profiles { start, end }
    )?;

    match query {
        governance::QueryAnswer::Profiles { profiles } => Ok(profiles),
        _ => return Err(StdError::generic_err("Returned wrong enum"))
    }
}

pub fn get_assemblies(
    chain: &mut ContractEnsemble,
    gov: &ContractLink<HumanAddr>,
    start: Uint128,
    end: Uint128
) -> StdResult<Vec<Assembly>> {

    let query: governance::QueryAnswer = chain.query(
        gov.address.clone(),
        &governance::QueryMsg::Assemblies { start, end }
    )?;

    match query {
        governance::QueryAnswer::Assemblies { assemblies } => Ok(assemblies),
        _ => return Err(StdError::generic_err("Returned wrong enum"))
    }
}

pub fn get_proposals(
    chain: &mut ContractEnsemble,
    gov: &ContractLink<HumanAddr>,
    start: Uint128,
    end: Uint128
) -> StdResult<Vec<Proposal>> {

    let query: governance::QueryAnswer = chain.query(
        gov.address.clone(),
        &governance::QueryMsg::Proposals{ start, end }
    )?;

    match query {
        governance::QueryAnswer::Proposals { props } => Ok(props),
        _ => return Err(StdError::generic_err("Returned wrong enum"))
    }
}

pub fn get_config(
    chain: &mut ContractEnsemble,
    gov: &ContractLink<HumanAddr>
) -> StdResult<Config> {
    let query: governance::QueryAnswer = chain.query(
        gov.address.clone(),
        &governance::QueryMsg::Config { }
    )?;

    match query {
        governance::QueryAnswer::Config { config } => Ok(config),
        _ => return Err(StdError::generic_err("Returned wrong enum"))
    }
}