use shade_protocol::{
    c_std::{
        Deps,
        MessageInfo,
        to_binary,
        Api,
        Binary,
        Env,
        DepsMut,
        Response,
        Querier,
        StdResult,
        Storage,
        entry_point,
    },
    dao::{
        manager,
        treasury_manager::{
            storage::*,
            Config,
            ExecuteMsg,
            InstantiateMsg,
            QueryMsg,
            Holding,
            Status,
        },
    }
};


use crate::{
    execute,
    query,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {

    CONFIG.save(deps.storage, &Config {
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

#[entry_point]
pub fn execute(
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
        } => execute::receive(deps, env, info, sender, from, amount, msg),
        ExecuteMsg::UpdateConfig { config } => execute::try_update_config(deps, env, info, config),
        ExecuteMsg::RegisterAsset { contract } => execute::try_register_asset(deps, &env, info, &contract),
        ExecuteMsg::Allocate { asset, allocation } => {
            execute::allocate(deps, &env, info, asset, allocation)
        },
        ExecuteMsg::AddHolder { holder } => execute::add_holder(deps, &env, info, holder),
        ExecuteMsg::RemoveHolder { holder } => execute::remove_holder(deps, &env, info, holder),
        ExecuteMsg::Manager(a) => match a {
            manager::SubExecuteMsg::Unbond { asset, amount } => {
                execute::unbond(deps, &env, info, asset, amount)
            }
            manager::SubExecuteMsg::Claim { asset } => execute::claim(deps, &env, info, asset),
            manager::SubExecuteMsg::Update { asset } => execute::update(deps, &env, info, asset),
        },
    }
}

#[entry_point]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Assets {} => to_binary(&query::assets(deps)?),
        QueryMsg::Allocations { asset } => to_binary(&query::allocations(deps, asset)?),
        QueryMsg::PendingAllowance { asset } => to_binary(&query::pending_allowance(deps, asset)?),
        QueryMsg::Holders {} => to_binary(&query::holders(deps)?),
        QueryMsg::Holding { holder } => to_binary(&query::holding(deps, holder)?),

        QueryMsg::Manager(a) => match a {
            manager::SubQueryMsg::Balance { asset, holder } => to_binary(&query::balance(deps, asset, holder)?),
            manager::SubQueryMsg::Unbonding { asset, holder } => to_binary(&query::unbonding(deps, asset, holder)?),
            manager::SubQueryMsg::Unbondable { asset, holder } => to_binary(&query::unbondable(deps, asset, holder)?),
            manager::SubQueryMsg::Claimable { asset, holder } => to_binary(&query::claimable(deps, asset, holder)?),
            manager::SubQueryMsg::Reserves { asset, holder } => to_binary(&query::reserves(deps, asset, holder)?),
        }
    }
}
