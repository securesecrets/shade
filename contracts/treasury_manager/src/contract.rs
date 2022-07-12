use cosmwasm_std::{
    to_binary,
    Api,
    Binary,
    Env,
    Extern,
    HandleResponse,
    InitResponse,
    Querier,
    StdResult,
    Storage,
};

use shade_protocol::contract_interfaces::dao::treasury_manager::{
    Config,
    HandleMsg,
    InitMsg,
    QueryMsg,
    Holding,
    Status,
};

use crate::{
    handle,
    query,
    state::{
        asset_list_w, config_w, self_address_w, viewing_key_w,
        holders_w, holding_w,
    },
};

use shade_protocol::contract_interfaces::dao::manager;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    config_w(&mut deps.storage).save(&Config {
        admin: msg.admin.unwrap_or(env.message.sender.clone()),
        treasury: msg.treasury.clone(),
    })?;

    viewing_key_w(&mut deps.storage).save(&msg.viewing_key)?;
    self_address_w(&mut deps.storage).save(&env.contract.address)?;
    asset_list_w(&mut deps.storage).save(&Vec::new())?;
    holders_w(&mut deps.storage).save(&vec![msg.treasury.clone()])?;
    holding_w(&mut deps.storage).save(
        msg.treasury.as_str().as_bytes(),
        &Holding {
            balances: vec![],
            unbondings: vec![],
            status: Status::Active,
        },
    )?;

    Ok(InitResponse {
        messages: vec![],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => handle::receive(deps, env, sender, from, amount, msg),
        HandleMsg::UpdateConfig { config } => handle::try_update_config(deps, env, config),
        HandleMsg::RegisterAsset { contract } => handle::try_register_asset(deps, &env, &contract),
        HandleMsg::Allocate { asset, allocation } => {
            handle::allocate(deps, &env, asset, allocation)
        },
        HandleMsg::AddHolder { holder } => handle::add_holder(deps, &env, holder),
        HandleMsg::RemoveHolder { holder } => handle::remove_holder(deps, &env, holder),
        HandleMsg::Manager(a) => match a {
            manager::SubHandleMsg::Unbond { asset, amount } => {
                handle::unbond(deps, &env, asset, amount)
            }
            manager::SubHandleMsg::Claim { asset } => handle::claim(deps, &env, asset),
            manager::SubHandleMsg::Update { asset } => handle::update(deps, &env, asset),
        },
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
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
            manager::SubQueryMsg::Reserves { asset, holder } => to_binary(&query::reserves(deps, &asset, holder)?),
        }
    }
}
