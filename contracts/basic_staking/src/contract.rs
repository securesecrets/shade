use shade_protocol::{
    basic_staking::{Auth, AuthPermit, Config, ExecuteMsg, InstantiateMsg, QueryMsg},
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
        StdError,
        StdResult,
        Uint128,
    },
    query_auth::helpers::{authenticate_permit, authenticate_vk, PermitAuthentication},
    snip20::helpers::{register_receive, set_viewing_key_msg},
    utils::asset::Contract,
};

use crate::{execute, query, storage::*};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
        query_auth: msg.query_auth.into_valid(deps.api)?,
        unbond_period: msg.unbond_period,
        max_user_pools: msg.max_user_pools,
    };

    let stake_token = msg.stake_token.into_valid(deps.api)?;

    CONFIG.save(deps.storage, &config)?;
    STAKE_TOKEN.save(deps.storage, &stake_token)?;
    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;

    REWARD_TOKENS.save(deps.storage, &vec![stake_token.clone()])?;
    REWARD_POOLS.save(deps.storage, &vec![])?;

    TOTAL_STAKED.save(deps.storage, &Uint128::zero())?;

    let resp = Response::new().add_messages(vec![
        set_viewing_key_msg(msg.viewing_key, None, &stake_token)?,
        register_receive(env.contract.code_hash, None, &stake_token)?,
    ]);

    Ok(resp)
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { config } => execute::update_config(deps, env, info, config),
        ExecuteMsg::RegisterRewards { token } => {
            let api = deps.api;
            execute::register_reward(deps, env, info, token.into_valid(api)?)
        }
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => execute::receive(deps, env, info, sender, from, amount, msg),
        ExecuteMsg::Claim {} => execute::claim(deps, env, info),
        ExecuteMsg::Unbond { amount } => execute::unbond(deps, env, info, amount),
        ExecuteMsg::Withdraw {} => execute::withdraw(deps, env, info),
        ExecuteMsg::Compound {} => execute::compound(deps, env, info),
    }
}

pub fn authenticate(deps: Deps, auth: Auth, query_auth: Contract) -> StdResult<Addr> {
    match auth {
        Auth::ViewingKey { key, address } => {
            let address = deps.api.addr_validate(&address)?;
            if !authenticate_vk(address.clone(), key, &deps.querier, &query_auth)? {
                return Err(StdError::generic_err("Invalid Viewing Key"));
            }
            Ok(address)
        }
        Auth::Permit(permit) => {
            let res: PermitAuthentication<AuthPermit> =
                authenticate_permit(permit, &deps.querier, query_auth)?;
            if res.revoked {
                return Err(StdError::generic_err("Permit Revoked"));
            }
            Ok(res.sender)
        }
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::TotalStaked {} => to_binary(&query::total_staked(deps)?),
        QueryMsg::RewardTokens {} => to_binary(&query::reward_tokens(deps)?),
        QueryMsg::RewardPool {} => to_binary(&query::reward_pool(deps)?),
        QueryMsg::Balance { auth } => {
            let config = CONFIG.load(deps.storage)?;
            to_binary(&query::user_balance(
                deps,
                authenticate(deps, auth, config.query_auth)?,
            )?)
        }
        QueryMsg::Share { auth } => {
            let config = CONFIG.load(deps.storage)?;
            to_binary(&query::user_share(
                deps,
                authenticate(deps, auth, config.query_auth)?,
            )?)
        }
        QueryMsg::Rewards { auth } => {
            let config = CONFIG.load(deps.storage)?;
            to_binary(&query::user_rewards(
                deps,
                env,
                authenticate(deps, auth, config.query_auth)?,
            )?)
        }
        QueryMsg::Unbonding { auth } => {
            let config = CONFIG.load(deps.storage)?;
            to_binary(&query::user_unbonding(
                deps,
                authenticate(deps, auth, config.query_auth)?,
            )?)
        }
    }
}
