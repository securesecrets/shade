use shade_protocol::{
    c_std::{
        entry_point,
        Deps,
        to_binary,
        Api,
        Binary,
        Env,
        DepsMut,
        Response,
        Querier,
        StdError,
        StdResult,
        Storage,
        Uint128, MessageInfo,
    },
    dao::{
        adapter,
        scrt_staking::{
            Config,
            ExecuteMsg,
            InstantiateMsg,
            QueryMsg,
        },
    },
    snip20::helpers::{register_receive, set_viewing_key_msg},
};

use crate::{
    handle,
    query,
    storage::{CONFIG, SELF_ADDRESS, UNBONDING, VIEWING_KEY},
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
        sscrt: msg.sscrt.into_valid(deps.api)?,
        owner: deps.api.addr_validate(msg.owner.as_str())?,
        validator_bounds: msg.validator_bounds,
    };

    CONFIG.save(deps.storage, &config)?;

    SELF_ADDRESS.save(deps.storage, &env.contract.address)?;
    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;
    UNBONDING.save(deps.storage, &Uint128::zero())?;

    let resp = Response::new()
        .add_messages(vec![
            set_viewing_key_msg(
                msg.viewing_key,
                None,
                &config.sscrt,
            )?,
            register_receive(
                env.contract.code_hash,
                None,
                &config.sscrt
            )?,
        ]);

    Ok(resp)
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
        } => handle::receive(deps, env, info, sender, from, amount, msg),
        ExecuteMsg::UpdateConfig { config } => handle::try_update_config(deps, env, info, config),
        ExecuteMsg::Adapter(adapter) => match adapter {
            adapter::SubExecuteMsg::Unbond { asset, amount } => handle::unbond(deps, env, info, asset, amount),
            adapter::SubExecuteMsg::Claim { asset } => handle::claim(deps, env, info, asset),
            adapter::SubExecuteMsg::Update { asset } => handle::update(deps, env, info, asset),
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
        QueryMsg::Delegations {} => to_binary(&query::delegations(deps)?),
        QueryMsg::Rewards {} => to_binary(&query::rewards(deps)?),
        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => to_binary(&query::balance(deps, asset)?),
            adapter::SubQueryMsg::Claimable { asset } => to_binary(&query::claimable(deps, asset)?),
            adapter::SubQueryMsg::Unbonding { asset } => to_binary(&query::unbonding(deps, asset)?),
            adapter::SubQueryMsg::Unbondable { asset } => to_binary(&query::unbondable(deps, asset)?),
            adapter::SubQueryMsg::Reserves { asset } => to_binary(&query::reserves(deps, asset)?),
        }
    }
}
