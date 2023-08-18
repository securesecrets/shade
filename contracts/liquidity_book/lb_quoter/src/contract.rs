#![allow(unused)] // For beginning only.

use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Timestamp, Uint256,
};
use ethnum::U256;

use libraries::types::Bytes32;

use crate::msg::*;
use crate::prelude::*;
use crate::state::*;

/////////////// INSTANTIATE ///////////////

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // TODO: Only the factory should be allowed to instantiate this contract
    // I think you can restrict that on code upload

    let state = State {
        creator: info.sender.clone(),
    };
    deps.api
        .debug(format!("Contract was initialized by {}", info.sender).as_str());
    CONFIG.save(deps.storage, &state)?;

    Ok(Response::default())
}

/////////////// EXECUTE ///////////////

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response> {
    match msg {
        ExecuteMsg::Swap { swap_for_y, to } => try_swap(deps, env, info, swap_for_y, to),
    }
}

fn try_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    swap_for_y: bool,
    to: Addr,
) -> Result<Response> {
    todo!()
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::GetFactory {} => {
            to_binary(&query_creator(deps)?).map_err(|err| Error::CwErr(err))
        }
    }
}

fn query_creator(deps: Deps) -> Result<FactoryResponse> {
    let state = CONFIG.load(deps.storage)?;
    let factory = state.creator;
    Ok(FactoryResponse { factory })
}
