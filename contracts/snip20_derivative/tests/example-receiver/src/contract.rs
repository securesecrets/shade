use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};

use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg, Snip20Msg};
use crate::state::{config, config_read, State};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        count: msg.count,
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        known_snip_20: vec![],
    };

    config(deps.storage).save(&state)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Increment {} => try_increment(deps, env),
        ExecuteMsg::Reset { count } => try_reset(deps, info, count),
        ExecuteMsg::Register { reg_addr, reg_hash } => try_register(deps, env, reg_addr, reg_hash),
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            msg,
            memo: _,
        } => try_receive(deps, env, info, sender, from, amount, msg),
        ExecuteMsg::Redeem {
            addr,
            hash,
            to,
            amount,
            denom,
        } => try_redeem(deps, addr, hash, to, amount, denom),
        ExecuteMsg::Fail {} => try_fail(),
    }
}

pub fn try_increment(deps: DepsMut, _env: Env) -> StdResult<Response> {
    let mut count = 0;
    config(deps.storage).update(|mut state| -> StdResult<_> {
        state.count += 1;
        count = state.count;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("count", count.to_string()))
}

pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> StdResult<Response> {
    let sender_address_raw = deps.api.addr_canonicalize(info.sender.as_str())?;
    config(deps.storage).update(|mut state| {
        if sender_address_raw != state.owner {
            return Err(StdError::generic_err("Only the owner can reset count"));
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::default())
}

pub fn try_register(
    deps: DepsMut,
    env: Env,
    reg_addr: Addr,
    reg_hash: String,
) -> StdResult<Response> {
    let mut conf = config(deps.storage);
    let mut state = conf.load()?;
    if !state.known_snip_20.contains(&reg_addr) {
        state.known_snip_20.push(reg_addr.clone());
    }
    conf.save(&state)?;

    let msg = to_binary(&Snip20Msg::register_receive(env.contract.code_hash))?;
    let message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: reg_addr.into_string(),
        code_hash: reg_hash,
        msg,
        funds: vec![],
    });

    Ok(Response::new().add_message(message))
}

pub fn try_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    _from: Addr,
    _amount: Uint128,
    msg: Binary,
) -> StdResult<Response> {
    let msg: ExecuteMsg = from_binary(&msg)?;

    if matches!(msg, ExecuteMsg::Receive { .. }) {
        return Err(StdError::generic_err(
            "Recursive call to receive() is not allowed",
        ));
    }

    // let state = config_read(&deps.storage).load()?;
    // if !state.known_snip_20.contains(&env.message.sender) {
    //     return Err(StdError::generic_err(format!(
    //         "{} is not a known SNIP-20 coin that this contract registered to",
    //         env.message.sender
    //     )));
    // }

    /* use sender & amount */
    execute(deps, env, info, msg)
}

fn try_redeem(
    _deps: DepsMut,
    addr: Addr,
    hash: String,
    to: Addr,
    amount: Uint128,
    denom: Option<String>,
) -> StdResult<Response> {
    // let state = config_read(&deps.storage).load()?;
    // if !state.known_snip_20.contains(&addr) {
    //     return Err(StdError::generic_err(format!(
    //         "{} is not a known SNIP-20 coin that this contract registered to",
    //         addr
    //     )));
    // }
    let unwrapped_denom = denom.unwrap_or("uscrt".to_string());

    let msg = to_binary(&Snip20Msg::redeem(amount, unwrapped_denom.clone()))?;
    let secret_redeem = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: addr.into_string(),
        code_hash: hash,
        msg,
        funds: vec![],
    });
    let redeem = CosmosMsg::Bank(BankMsg::Send {
        // unsafe, don't use in production obviously
        amount: vec![Coin::new(amount.u128(), unwrapped_denom)],
        to_address: to.into_string(),
    });

    Ok(Response::new()
        .add_message(secret_redeem)
        .add_message(redeem))
}

fn try_fail() -> StdResult<Response> {
    Err(StdError::generic_err("intentional failure"))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = config_read(deps.storage).load()?;
    Ok(CountResponse { count: state.count })
}
