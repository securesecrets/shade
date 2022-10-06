use shade_protocol::{
    c_std::{to_binary, Addr, DepsMut, Env, MessageInfo, Response, StdResult, Storage, SubMsg},
    contract_interfaces::governance::{Config, ExecuteAnswer, RuntimeState},
    governance::{
        assembly::{Assembly, AssemblyData},
        errors::Error,
        profile::Profile,
    },
    snip20::helpers::register_receive,
    utils::{asset::Contract, generic_response::ResponseStatus, storage::plus::ItemStorage},
};

pub mod assembly;
pub mod assembly_msg;
pub mod contract;
pub mod migration;
pub mod profile;
pub mod proposal;

/// Checks that state can be updated
pub fn assembly_state_valid(storage: &dyn Storage, assembly: u16) -> StdResult<()> {
    match RuntimeState::load(storage)? {
        RuntimeState::Normal => {}
        RuntimeState::SpecificAssemblies { assemblies } => {
            if !assemblies.contains(&assembly) {
                return Err(Error::assemblies_limited(vec![]));
            }
        }
        RuntimeState::Migrated { .. } => return Err(Error::migrated(vec![])),
    };

    Ok(())
}

/// Authorizes the assembly, returns assembly data to avoid redundant loading
pub fn authorize_assembly(
    storage: &dyn Storage,
    info: &MessageInfo,
    assembly: u16,
) -> StdResult<AssemblyData> {
    assembly_state_valid(storage, assembly)?;

    let data = Assembly::data(storage, assembly)?;

    // Check that the user is in the non-public assembly
    if data.profile != 0 && !data.members.contains(&info.sender) {
        return Err(Error::not_in_assembly(vec![
            info.sender.as_str(),
            &assembly.to_string(),
        ]));
    };

    // Check if enabled
    if !Profile::data(storage, data.profile)?.enabled {
        return Err(Error::profile_disabled(vec![&data.profile.to_string()]));
    }

    Ok(data)
}

/// Checks that the message sender is self and also not migrated
pub fn authorized(storage: &dyn Storage, env: &Env, info: &MessageInfo) -> StdResult<()> {
    if info.sender != env.contract.address {
        return Err(Error::not_self(vec![]));
    } else if let RuntimeState::Migrated { .. } = RuntimeState::load(storage)? {
        return Err(Error::migrated(vec![]));
    }

    Ok(())
}

pub fn try_set_config(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    query_auth: Option<Contract>,
    treasury: Option<Addr>,
    vote_token: Option<Contract>,
    funding_token: Option<Contract>,
) -> StdResult<Response> {
    let mut messages = vec![];
    let mut config = Config::load(deps.storage)?;

    // Vote and funding tokens cannot be set to none after being set
    if let Some(vote_token) = vote_token {
        config.vote_token = Some(vote_token.clone());
        messages.push(SubMsg::new(register_receive(
            env.contract.code_hash.clone(),
            None,
            &vote_token,
        )?));
    }

    if let Some(funding_token) = funding_token {
        config.funding_token = Some(funding_token.clone());
        messages.push(SubMsg::new(register_receive(
            env.contract.code_hash.clone(),
            None,
            &funding_token,
        )?));
    }

    if let Some(treasury) = treasury {
        config.treasury = treasury;
    }

    if let Some(query_auth) = query_auth {
        config.query = query_auth;
    }

    config.save(deps.storage)?;
    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::SetConfig {
            status: ResponseStatus::Success,
        })?)
        .add_submessages(messages))
}

pub fn try_set_runtime_state(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    state: RuntimeState,
) -> StdResult<Response> {
    if let RuntimeState::Migrated { .. } = state {
        return Err(Error::migrated(vec![]));
    }

    state.save(deps.storage)?;
    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::SetRuntimeState {
            status: ResponseStatus::Success,
        })?),
    )
}
