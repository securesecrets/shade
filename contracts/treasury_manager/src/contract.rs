use shade_protocol::c_std::{

    to_binary,
    Api,
    Binary,
    Env,
    DepsMut,
    Response,
    Querier,
    StdResult,
    Storage,
};

use shade_protocol::contract_interfaces::dao::treasury_manager::{
    storage::*,
    Config,
    ExecuteMsg,
    InstantiateMsg,
    QueryMsg,
    Holding,
    Status,
};

use crate::{
    handle,
    query,
    /*
    state::{
        asset_list_w, config_w, self_address_w, viewing_key_w,
        holders_w, holding_w,
    },
    */
};

use shade_protocol::contract_interfaces::dao::manager;

pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {

    CONFIG.save(&mut deps.storage, &Config {
        admin: msg.admin.unwrap_or(info.sender.clone()),
        treasury: msg.treasury.clone(),
    })?;

    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;
    SELF_ADDRESS.save(deps.storage, &env.contract.address)?;
    ASSET_LIST.save(deps.storage, &Vec::new())?;
    HOLDERS.save(deps.storage, &vec![msg.treasury.clone()])?;
    HOLDING.save(deps.storage,
        msg.treasury,
        &Holding {
            balances: vec![],
            unbondings: vec![],
            status: Status::Active,
        },
    )?;

    Ok(Response::new())
}

pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => handle::receive(deps, env, info, sender, from, amount, msg),
        ExecuteMsg::UpdateConfig { config } => handle::try_update_config(deps, env, info, config),
        ExecuteMsg::RegisterAsset { contract } => handle::try_register_asset(deps, &env, &contract),
        ExecuteMsg::Allocate { asset, allocation } => {
            handle::allocate(deps, &env, asset, allocation)
        },
        ExecuteMsg::AddHolder { holder } => handle::add_holder(deps, &env, holder),
        ExecuteMsg::RemoveHolder { holder } => handle::remove_holder(deps, &env, holder),
        ExecuteMsg::Manager(a) => match a {
            manager::SubHandleMsg::Unbond { asset, amount } => {
                handle::unbond(deps, &env, asset, amount)
            }
            manager::SubHandleMsg::Claim { asset } => handle::claim(deps, &env, asset),
            manager::SubHandleMsg::Update { asset } => handle::update(deps, &env, asset),
        },
    }
}

pub fn query(
    deps: Deps,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Assets {} => to_binary(&query::assets(deps)?),
        QueryMsg::Allocations { asset } => to_binary(&query::allocations(deps, asset)?),
        QueryMsg::PendingAllowance { asset } => to_binary(&query::pending_allowance(deps, asset)?),
        QueryMsg::Holders {} => to_binary(&query::holders(deps)),
        QueryMsg::Holding { holder } => to_binary(&query::holding(deps, holder)),

        /*
        // For holder specific queries
        QueryMsg::Balance { asset, holder } => to_binary(&query::balance(deps, asset, Some(holder))?),
        QueryMsg::Unbonding { asset, holder } => to_binary(&query::unbonding(deps, asset, Some(holder))?),
        QueryMsg::Unbondable { asset, holder } => to_binary(&query::unbondable(deps, asset, Some(holder))?),
        QueryMsg::Claimable { asset, holder } => to_binary(&query::claimable(deps, asset, Some(holder))?),
        */

        QueryMsg::Manager(a) => match a {
            manager::SubQueryMsg::Balance { asset, holder } => to_binary(&query::balance(deps, asset, holder)?),
            manager::SubQueryMsg::Unbonding { asset, holder } => to_binary(&query::unbonding(deps, asset, holder)?),
            manager::SubQueryMsg::Unbondable { asset, holder } => to_binary(&query::unbondable(deps, asset, holder)?),
            manager::SubQueryMsg::Claimable { asset, holder } => to_binary(&query::claimable(deps, asset, holder)?),
            manager::SubQueryMsg::Reserves { asset, holder } => to_binary(&query::reserves(deps, asset, holder)?),
        }
    }
}
