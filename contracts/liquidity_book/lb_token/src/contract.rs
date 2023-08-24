use cosmwasm_std::Event;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, Uint256,
};

use crate::msg::*;
use crate::prelude::*;
use crate::state::*;

/////////////// INSTANTIATE ///////////////

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response> {
    let admin = deps.api.addr_canonicalize(info.sender.as_str())?;
    let lb_pair = deps.api.addr_canonicalize(msg.lb_pair.as_str())?;

    let state = Config {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        admin,
        lb_pair,
    };

    deps.api
        .debug(format!("Contract was initialized by {}", info.sender).as_str());
    CONFIG.save(deps.storage, &state)?;

    let mut response = Response::new();
    response.data = Some(env.contract.address.as_bytes().into());
    Ok(response)
}

/////////////// EXECUTE ///////////////

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response> {
    match msg {
        ExecuteMsg::ApproveForAll { spender, approved } => {
            try_approve_for_all(deps, env, info, spender, approved)
        }
        ExecuteMsg::BatchTransferFrom {
            from,
            to,
            ids,
            amounts,
        } => try_batch_transfer_from(deps, env, info, from, to, ids, amounts),
        ExecuteMsg::Mint {
            recipient,
            id,
            amount,
        } => try_mint(deps, env, info, recipient, id, amount),
        ExecuteMsg::Burn { owner, id, amount } => try_burn(deps, env, info, owner, id, amount),
    }
}

fn try_batch_transfer_from(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    from: Addr,
    to: Addr,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
) -> Result<Response> {
    if ids.len() != amounts.len() || ids.len() == 0 || amounts.len() == 0 {
        return Err(Error::InvalidInput(
            "ids and amounts length must be equal.".to_string(),
        ));
    }

    let spender = info.sender;
    let approval_check = check_approval(deps.as_ref(), &from, &spender)?;
    if !approval_check {
        return Err(Error::SpenderNotApproved {});
    }

    let from_canonical = deps.api.addr_canonicalize(from.as_str())?;
    let to_canonical = deps.api.addr_canonicalize(to.as_str())?;

    for (id, amount) in ids.iter().zip(amounts.iter()) {
        let key = (from_canonical.clone(), *id);
        let mut balance = BALANCES.get(deps.storage, &key).unwrap_or(Uint256::zero());

        if balance < *amount {
            return Err(Error::InsufficientFunds {});
        }

        // Update the balances
        balance -= *amount;
        BALANCES.insert(deps.storage, &key, &balance)?;

        let key_to = (to_canonical.clone(), *id);
        let mut balance_to = BALANCES
            .get(deps.storage, &key_to)
            .unwrap_or(Uint256::zero());
        balance_to += *amount;
        BALANCES.insert(deps.storage, &key_to, &balance_to)?;
    }

    // Emit TransferBatch event
    let events = vec![Event::new("transfer_batch")
        .add_attribute("from", from)
        .add_attribute("to", to)];

    Ok(Response::new()
        .add_events(events)
        .add_attribute("action", "batch_transfer_from"))
}

pub fn check_approval(deps: Deps, owner: &Addr, spender: &Addr) -> Result<bool> {
    if owner == spender {
        return Ok(true);
    }
    let owner_raw = deps.api.addr_canonicalize(owner.as_str())?;
    let spender_raw = deps.api.addr_canonicalize(spender.as_str())?;

    let key = (owner_raw, spender_raw);
    let approval = SPENDER_APPROVALS.get(deps.storage, &key).unwrap_or(false);
    Ok(approval)
}

fn try_approve_for_all(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    spender: Addr,
    new_approval: bool,
) -> Result<Response> {
    let owner = info.sender;
    let owner_canonical = deps.api.addr_canonicalize(owner.as_str())?;
    let spender_canonical = deps.api.addr_canonicalize(spender.as_str())?;

    if owner_canonical == spender_canonical {
        return Err(Error::SelfApproval {});
    }

    let approved = SPENDER_APPROVALS
        .get(
            deps.storage,
            &(owner_canonical.clone(), spender_canonical.clone()),
        )
        .unwrap_or(false);

    if approved == new_approval {
        // Check if the current approval status is the same as the new one
        return Err(Error::AlreadyApproved {});
    }

    SPENDER_APPROVALS.insert(
        deps.storage,
        &(owner_canonical, spender_canonical),
        &new_approval,
    )?;

    let response = Response::new()
        .add_attribute("action", "approve_for_all")
        .add_attribute("owner", owner)
        .add_attribute("spender", spender)
        .add_attribute("approved", approved.to_string());

    Ok(response)
}

fn try_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: Addr,
    id: u32,
    amount: Uint256,
) -> Result<Response> {
    let config = CONFIG.load(deps.storage)?;
    let sender = info.sender;
    let lb_pair = deps.api.addr_humanize(&config.lb_pair)?;

    if sender != lb_pair {
        return Err(Error::Unauthorized);
    }

    let recipient_canonical = deps.api.addr_canonicalize(recipient.as_str())?;
    let current_balance = BALANCES
        .get(deps.storage, &(recipient_canonical.clone(), id))
        .unwrap_or_else(Uint256::zero);
    let new_balance = current_balance + amount;
    BALANCES.insert(deps.storage, &(recipient_canonical, id), &new_balance)?;

    let mut total_supply = TOTAL_SUPPLY
        .get(deps.storage, &id)
        .unwrap_or_else(Uint256::zero);
    total_supply += amount;
    TOTAL_SUPPLY.insert(deps.storage, &id, &total_supply)?;

    let response = Response::new()
        .add_attribute("action", "mint")
        .add_attribute("recipient", recipient)
        .add_attribute("id", id.to_string())
        .add_attribute("amount", amount.to_string());

    Ok(response)
}

fn try_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Addr,
    id: u32,
    amount: Uint256,
) -> Result<Response> {
    let config = CONFIG.load(deps.storage)?;
    let sender = info.sender;
    let lb_pair = deps.api.addr_humanize(&config.lb_pair)?;

    if sender != lb_pair {
        return Err(Error::Unauthorized);
    }

    let owner_canonical = deps.api.addr_canonicalize(owner.as_str())?;
    let current_balance = BALANCES
        .get(deps.storage, &(owner_canonical.clone(), id))
        .unwrap_or_else(Uint256::zero);

    if amount > current_balance {
        return Err(Error::InsufficientFunds);
    }

    let new_balance = current_balance - amount;
    BALANCES.insert(deps.storage, &(owner_canonical, id), &new_balance)?;

    let mut total_supply = TOTAL_SUPPLY
        .get(deps.storage, &id)
        .unwrap_or_else(Uint256::zero);

    if amount > total_supply {
        return Err(Error::InsufficientSupply);
    }

    total_supply -= amount;
    TOTAL_SUPPLY.insert(deps.storage, &id, &total_supply)?;

    let response = Response::new()
        .add_attribute("action", "burn")
        .add_attribute("owner", owner)
        .add_attribute("id", id.to_string())
        .add_attribute("amount", amount.to_string());

    Ok(response)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::Name {} => to_binary(&query_name(deps)?).map_err(|err| Error::CwErr(err)),
        QueryMsg::Symbol {} => to_binary(&query_symbol(deps)?).map_err(|err| Error::CwErr(err)),
        QueryMsg::Decimals {} => to_binary(&query_decimals(deps)?).map_err(|err| Error::CwErr(err)),
        QueryMsg::TotalSupply { id } => {
            to_binary(&query_total_supply(deps, id)?).map_err(|err| Error::CwErr(err))
        }
        QueryMsg::BalanceOf { owner, id } => {
            to_binary(&query_balance_of(deps, owner, id)?).map_err(|err| Error::CwErr(err))
        }
        QueryMsg::BalanceOfBatch { owners, ids } => {
            to_binary(&query_balance_of_batch(deps, owners, ids)?).map_err(|err| Error::CwErr(err))
        }
        QueryMsg::IsApprovedForAll { owner, spender } => {
            to_binary(&query_is_approved_for_all(deps, owner, spender)?)
                .map_err(|err| Error::CwErr(err))
        }
    }
}

fn query_name(deps: Deps) -> Result<NameResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(NameResponse { name: config.name })
}

fn query_symbol(deps: Deps) -> Result<SymbolResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(SymbolResponse {
        symbol: config.symbol,
    })
}

fn query_decimals(deps: Deps) -> Result<DecimalsResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(DecimalsResponse {
        decimals: config.decimals,
    })
}

fn query_total_supply(deps: Deps, id: u32) -> Result<TotalSupplyResponse> {
    let total_supply: Uint256 = TOTAL_SUPPLY
        .get(deps.storage, &id)
        .unwrap_or(Uint256::zero());
    Ok(TotalSupplyResponse { total_supply })
}

fn query_balance_of(deps: Deps, owner: Addr, id: u32) -> Result<BalanceOfResponse> {
    let balance = get_balance(deps, owner, id)?;
    Ok(BalanceOfResponse { balance })
}

fn get_balance(deps: Deps, owner: Addr, id: u32) -> Result<Uint256> {
    let balance: Uint256 = BALANCES
        .get(
            deps.storage,
            &(deps.api.addr_canonicalize(owner.as_str())?, id),
        )
        .unwrap_or(Uint256::zero());
    Ok(balance)
}

fn query_balance_of_batch(
    deps: Deps,
    owners: Vec<Addr>,
    ids: Vec<u32>,
) -> Result<BalanceOfBatchResponse> {
    let balances: Result<Vec<Uint256>> = owners
        .iter()
        .zip(ids.iter())
        .map(|(owner, id)| get_balance(deps, owner.clone(), *id))
        .collect();
    Ok(BalanceOfBatchResponse {
        balances: balances?,
    })
}

fn query_is_approved_for_all(
    deps: Deps,
    owner: Addr,
    spender: Addr,
) -> Result<IsApprovedForAllResponse> {
    let is_approved = get_is_approved_for_all(deps, owner, spender)?;
    Ok(IsApprovedForAllResponse { is_approved })
}

fn get_is_approved_for_all(deps: Deps, owner: Addr, spender: Addr) -> Result<bool> {
    let is_approved: bool = SPENDER_APPROVALS
        .get(
            deps.storage,
            &(
                deps.api.addr_canonicalize(owner.as_str())?,
                deps.api.addr_canonicalize(spender.as_str())?,
            ),
        )
        .unwrap_or(false);
    Ok(is_approved)
}
