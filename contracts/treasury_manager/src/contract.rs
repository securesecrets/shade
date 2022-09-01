use shade_protocol::{
    c_std::{
        shd_entry_point,
        to_binary,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdResult,
        Uint128,
    },
    dao::{
        manager,
        treasury_manager::{
            Config,
            ExecuteMsg,
            Holding,
            InstantiateMsg,
            QueryAnswer,
            QueryMsg,
            Status,
        },
    },
    utils::cycle::parse_utc_datetime,
};

use crate::{execute, query, storage::*};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let treasury = deps.api.addr_validate(msg.treasury.as_str())?;

    CONFIG.save(deps.storage, &Config {
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
        treasury: treasury.clone(),
    })?;

    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;
    SELF_ADDRESS.save(deps.storage, &env.contract.address)?;
    ASSET_LIST.save(deps.storage, &Vec::new())?;
    HOLDERS.save(deps.storage, &vec![treasury.clone()])?;
    HOLDING.save(deps.storage, treasury, &Holding {
        balances: vec![],
        unbondings: vec![],
        status: Status::Active,
    })?;
    UNBONDINGS.save(deps.storage, &Uint128::zero())?;

    Ok(Response::new())
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => {
            let sender = deps.api.addr_validate(&sender)?;
            let from = deps.api.addr_validate(&from)?;
            execute::receive(deps, env, info, sender, from, amount, msg)
        }
        ExecuteMsg::UpdateConfig { config } => execute::try_update_config(deps, env, info, config),
        ExecuteMsg::RegisterAsset { contract } => {
            let contract = contract.into_valid(deps.api)?;
            execute::try_register_asset(deps, &env, info, &contract)
        }
        ExecuteMsg::Allocate { asset, allocation } => {
            let asset = deps.api.addr_validate(&asset)?;
            execute::allocate(deps, &env, info, asset, allocation)
        }
        ExecuteMsg::AddHolder { holder } => {
            let holder = deps.api.addr_validate(&holder)?;
            execute::add_holder(deps, &env, info, holder)
        }
        ExecuteMsg::RemoveHolder { holder } => {
            let holder = deps.api.addr_validate(&holder)?;
            execute::remove_holder(deps, &env, info, holder)
        }
        ExecuteMsg::Manager(a) => match a {
            manager::SubExecuteMsg::Unbond { asset, amount } => {
                let asset = deps.api.addr_validate(&asset)?;
                execute::unbond(deps, &env, info, asset, amount)
            }
            manager::SubExecuteMsg::Claim { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                execute::claim(deps, &env, info, asset)
            }
            manager::SubExecuteMsg::Update { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                execute::update(deps, &env, info, asset)
            }
        },
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Assets {} => to_binary(&query::assets(deps)?),
        QueryMsg::Allocations { asset } => {
            let asset = deps.api.addr_validate(&asset)?;
            to_binary(&query::allocations(deps, asset)?)
        }
        QueryMsg::PendingAllowance { asset } => {
            let asset = deps.api.addr_validate(&asset)?;
            to_binary(&query::pending_allowance(deps, asset)?)
        }
        QueryMsg::Holders {} => to_binary(&query::holders(deps)?),
        QueryMsg::Holding { holder } => {
            let holder = deps.api.addr_validate(&holder)?;
            to_binary(&query::holding(deps, holder)?)
        }
        QueryMsg::Metrics { date, period } => {
            let key = match date {
                Some(d) => parse_utc_datetime(&d)?.timestamp() as u64,
                None => env.block.time.seconds(),
            };
            to_binary(&QueryAnswer::Metrics {
                metrics: METRICS.load_period(deps.storage, key, period)?,
            })
        }

        QueryMsg::Manager(a) => match a {
            manager::SubQueryMsg::Balance { asset, holder } => {
                let asset = deps.api.addr_validate(&asset)?;
                let holder = deps.api.addr_validate(&holder)?;
                to_binary(&query::balance(deps, asset, holder)?)
            }
            manager::SubQueryMsg::BatchBalance { assets, holder } => {
                let mut val_assets = vec![];

                for a in assets {
                    val_assets.push(deps.api.addr_validate(&a)?);
                }
                let holder = deps.api.addr_validate(&holder)?;

                to_binary(&query::batch_balance(deps, val_assets, holder)?)
            }
            manager::SubQueryMsg::Unbonding { asset, holder } => {
                let asset = deps.api.addr_validate(&asset)?;
                let holder = deps.api.addr_validate(&holder)?;
                to_binary(&query::unbonding(deps, asset, holder)?)
            }
            manager::SubQueryMsg::Unbondable { asset, holder } => {
                let asset = deps.api.addr_validate(&asset)?;
                let holder = deps.api.addr_validate(&holder)?;
                to_binary(&query::unbondable(deps, asset, holder)?)
            }
            manager::SubQueryMsg::Claimable { asset, holder } => {
                let asset = deps.api.addr_validate(&asset)?;
                let holder = deps.api.addr_validate(&holder)?;
                to_binary(&query::claimable(deps, asset, holder)?)
            }
            manager::SubQueryMsg::Reserves { asset, holder } => {
                let asset = deps.api.addr_validate(&asset)?;
                let holder = deps.api.addr_validate(&holder)?;
                to_binary(&query::reserves(deps, asset, holder)?)
            }
        },
    }
}
