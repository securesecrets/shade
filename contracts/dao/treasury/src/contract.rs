use crate::{execute, query, storage::*};
use shade_protocol::{
    c_std::{
        shd_entry_point,
        to_binary,
        Addr,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdResult,
    },
    dao::treasury::{Config, ExecuteMsg, InstantiateMsg, QueryAnswer, QueryMsg, RunLevel},
    utils::asset::Contract,
};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(deps.storage, &Config {
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
        multisig: deps.api.addr_validate(&msg.multisig)?,
    })?;

    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;
    RUN_LEVEL.save(deps.storage, &RunLevel::Normal)?;

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
        ExecuteMsg::UpdateConfig {
            admin_auth,
            multisig,
        } => execute::try_update_config(deps, env, info, admin_auth, multisig),
        ExecuteMsg::RegisterAsset { contract } => {
            let contract = contract.into_valid(deps.api)?;
            execute::try_register_asset(deps, &env, info, &contract)
        }
        ExecuteMsg::RegisterManager { contract } => {
            let mut contract = contract.into_valid(deps.api)?;
            execute::register_manager(deps, &env, info, &mut contract)
        }
        ExecuteMsg::RegisterWrap { denom, contract } => {
            let contract = contract.into_valid(deps.api)?;
            execute::register_wrap(deps, &env, info, denom, &contract)
        }
        ExecuteMsg::Allowance {
            asset,
            allowance,
            refresh_now,
        } => {
            let asset = deps.api.addr_validate(&asset)?;
            let allowance = allowance.valid(deps.api)?;
            execute::allowance(deps, &env, info, asset, allowance, refresh_now)
        }
        ExecuteMsg::Update { asset } => {
            let asset = deps.api.addr_validate(&asset)?;
            execute::update(deps, &env, info, asset)
        }
        ExecuteMsg::SetRunLevel { run_level } => {
            execute::set_run_level(deps, &env, info, run_level)
        }
        ExecuteMsg::WrapCoins {} => execute::wrap_coins(deps, &env, info),
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Assets {} => to_binary(&query::assets(deps)?),
        QueryMsg::Allowances { asset } => {
            let asset = deps.api.addr_validate(&asset)?;
            to_binary(&query::allowances(deps, asset)?)
        }
        QueryMsg::Allowance { asset, spender } => {
            let asset = deps.api.addr_validate(&asset)?;
            let spender = deps.api.addr_validate(&spender)?;
            to_binary(&query::allowance(deps, env, asset, spender)?)
        }
        QueryMsg::RunLevel => to_binary(&QueryAnswer::RunLevel {
            run_level: RUN_LEVEL.load(deps.storage)?,
        }),
        //TODO: parse string & format manually to accept all valid date formats
        QueryMsg::Metrics {
            date,
            epoch,
            period,
        } => to_binary(&query::metrics(deps, env, date, epoch, period)?),
        QueryMsg::Balance { asset } => {
            let asset = deps.api.addr_validate(&asset)?;
            to_binary(&query::balance(deps, env, asset)?)
        }
        QueryMsg::BatchBalance { assets } => {
            let mut val_assets = vec![];

            for a in assets {
                val_assets.push(deps.api.addr_validate(&a)?);
            }

            to_binary(&query::batch_balance(deps, env, val_assets)?)
        }
        QueryMsg::Reserves { asset } => {
            let asset = deps.api.addr_validate(&asset)?;
            to_binary(&query::reserves(deps, env, asset)?)
        }
    }
}
