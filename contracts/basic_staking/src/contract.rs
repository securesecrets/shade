use shade_protocol::{
    basic_staking::{ExecuteMsg, InstantiateMsg, QueryMsg},
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
    snip20::helpers::{register_receive, set_viewing_key_msg},
};

use crate::{execute, query, storage::*};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        admin_auth: msg.admin_auth,
    };

    CONFIG.save(deps.storage, &config)?;
    STAKE_TOKEN.save(deps.storage, &msg.stake_token)?;
    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;

    REWARD_TOKENS.save(deps.storage, &vec![&msg.stake_token])?;
    REWARD_POOLS.save(deps.storage, &vec![])?;

    TOTAL_STAKED.save(deps.storage, &Uint128::zero());

    let resp = Response::new().add_messages(vec![
        set_viewing_key_msg(msg.viewing_key, None, &config.stake_token)?,
        register_receive(env.contract.code_hash, None, &config.stake_token)?,
    ]);

    Ok(resp)
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { config } => execute::update_config(deps, env, info, config),
        ExecuteMsg::RegisterRewards { token } => execute::register_reward(deps, env, info, token),
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

#[entry_point]
pub fn query(deps: Deps, env: Env, info: MessageInfo, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps, env, info)?),
        QueryMsg::TotalStaked {} => to_binary(&query::total_staked(deps, env, info)?),
        QueryMsg::RewardTokens {} => to_binary(&query::reward_tokens(deps, env, info)?),
        QueryMsg::RewardPool {} => to_binary(&query::reward_pool(deps, env, info)?),
        QueryMsg::Balance {} => to_binary(&query::user_balance(deps, env, info)?),
        QueryMsg::Share {} => to_binary(&query::user_share(deps, env, info)?),
        QueryMsg::Rewards {} => to_binary(&query::user_rewards(deps, env, info)?),
        QueryMsg::Unbonding {} => to_binary(&query::user_unbonding(deps, env, info)?),
    }
}
