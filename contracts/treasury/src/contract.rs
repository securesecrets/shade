use shade_protocol::{
    c_std::{
        entry_point,
        to_binary,
        Api,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Querier,
        Response,
        StdError,
        StdResult,
        Storage,
        Uint128,
    },
    dao::{
        adapter,
        treasury::{Config, ExecuteMsg, InstantiateMsg, QueryAnswer, QueryMsg, RunLevel},
    },
    utils::cycle::parse_utc_datetime,
};

use crate::{execute, query, storage::*};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(deps.storage, &Config {
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
        multisig: deps.api.addr_validate(&msg.multisig)?,
    })?;

    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;
    SELF_ADDRESS.save(deps.storage, &env.contract.address)?;
    RUN_LEVEL.save(deps.storage, &RunLevel::Normal)?;

    Ok(Response::new())
}

#[entry_point]
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
        ExecuteMsg::RegisterManager { contract } => {
            let mut contract = contract.into_valid(deps.api)?;
            execute::register_manager(deps, &env, info, &mut contract)
        }
        ExecuteMsg::RegisterWrap { denom, contract } => {
            let contract = contract.into_valid(deps.api)?;
            execute::register_wrap(deps, &env, info, denom, &contract)
        }
        ExecuteMsg::Allowance { asset, allowance } => {
            let asset = deps.api.addr_validate(&asset)?;
            execute::allowance(deps, &env, info, asset, allowance)
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

#[entry_point]
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
            to_binary(&query::allowance(deps, asset, spender)?)
        }
        QueryMsg::RunLevel => to_binary(&QueryAnswer::RunLevel {
            run_level: RUN_LEVEL.load(deps.storage)?,
        }),
        //TODO: parse string & format manually to accept all valid date formats
        QueryMsg::Metrics { date, period } => {
            let key = match date {
                Some(d) => parse_utc_datetime(&d)?.timestamp() as u64,
                None => env.block.time.seconds(),
            };
            to_binary(&QueryAnswer::Metrics {
                metrics: METRICS.load_period(deps.storage, key, period)?,
            })
        }
        QueryMsg::Balance { asset } => {
            let asset = deps.api.addr_validate(&asset)?;
            to_binary(&query::balance(deps, asset)?)
        }
        QueryMsg::BatchBalance { assets } => {
            let mut val_assets = vec![];

            for a in assets {
                val_assets.push(deps.api.addr_validate(&a)?);
            }

            to_binary(&query::batch_balance(deps, val_assets)?)
        }
        QueryMsg::Reserves { asset } => {
            let asset = deps.api.addr_validate(&asset)?;
            to_binary(&query::reserves(deps, asset)?)
        }
    }
}
