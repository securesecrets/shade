use shade_protocol::{
    basic_staking::{Auth, AuthPermit, Config, ExecuteMsg, InstantiateMsg, QueryAnswer, QueryMsg},
    c_std::{
        shd_entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response,
        StdError, StdResult, Uint128,
    },
    query_auth::helpers::{authenticate_permit, authenticate_vk, PermitAuthentication},
    snip20::helpers::{register_receive, set_viewing_key_msg},
    utils::{asset::Contract, pad_handle_result},
};

use crate::{execute, query, storage::*};

pub const RESPONSE_BLOCK_SIZE: usize = 256;

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            admin_auth: msg.admin_auth.into_valid(deps.api)?,
            query_auth: msg.query_auth.into_valid(deps.api)?,
            airdrop: match msg.airdrop {
                Some(airdrop) => Some(airdrop.into_valid(deps.api)?),
                None => None,
            },
            unbond_period: msg.unbond_period,
            max_user_pools: msg.max_user_pools,
        },
    )?;

    let stake_token = msg.stake_token.into_valid(deps.api)?;

    STAKE_TOKEN.save(deps.storage, &stake_token)?;
    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;

    REWARD_TOKENS.save(deps.storage, &vec![stake_token.clone()])?;
    REWARD_POOLS.save(deps.storage, &vec![])?;
    MAX_POOL_ID.save(deps.storage, &Uint128::zero())?;

    TRANSFER_WL.save(deps.storage, &vec![])?;

    TOTAL_STAKED.save(deps.storage, &Uint128::zero())?;

    let resp = Response::new().add_messages(vec![
        set_viewing_key_msg(msg.viewing_key, None, &stake_token)?,
        register_receive(env.contract.code_hash, None, &stake_token)?,
    ]);

    Ok(resp)
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateConfig {
                admin_auth,
                query_auth,
                airdrop,
                unbond_period,
                max_user_pools,
                padding,
            } => execute::update_config(
                deps,
                env,
                info,
                admin_auth,
                query_auth,
                airdrop,
                unbond_period,
                max_user_pools,
            ),
            ExecuteMsg::RegisterRewards { token, padding } => {
                let api = deps.api;
                execute::register_reward(deps, env, info, token.into_valid(api)?)
            }
            ExecuteMsg::AddTransferWhitelist { user, padding } => {
                let api = deps.api;
                execute::add_transfer_whitelist(deps, env, info, api.addr_validate(&user)?)
            }
            ExecuteMsg::RemoveTransferWhitelist { user, padding } => {
                let api = deps.api;
                execute::rm_transfer_whitelist(deps, env, info, api.addr_validate(&user)?)
            }
            ExecuteMsg::Receive {
                sender,
                from,
                amount,
                msg,
                ..
            } => execute::receive(deps, env, info, sender, from, amount, msg),
            ExecuteMsg::Claim { padding } => execute::claim(deps, env, info),
            ExecuteMsg::Unbond {
                amount,
                compound,
                padding,
            } => execute::unbond(deps, env, info, amount, compound.unwrap_or(false)),
            ExecuteMsg::Withdraw { ids, padding } => {
                execute::withdraw(deps, env, info.clone(), ids)
            }
            ExecuteMsg::Compound { padding } => execute::compound(deps, env, info),
            ExecuteMsg::EndRewardPool { id, force, padding } => {
                execute::end_reward_pool(deps, env, info, id, force.unwrap_or(false))
            }
            ExecuteMsg::TransferStake {
                amount,
                recipient,
                compound,
                padding,
            } => {
                let api = deps.api;
                execute::transfer_stake(
                    deps,
                    env,
                    info,
                    amount,
                    api.addr_validate(&recipient)?,
                    compound.unwrap_or(false),
                )
            }
        },
        RESPONSE_BLOCK_SIZE,
    )
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
        QueryMsg::StakeToken {} => to_binary(&query::stake_token(deps)?),
        QueryMsg::StakingInfo {} => to_binary(&query::staking_info(deps)?),
        QueryMsg::TotalStaked {} => to_binary(&query::total_staked(deps)?),
        QueryMsg::RewardTokens {} => to_binary(&query::reward_tokens(deps)?),
        QueryMsg::RewardPools {} => to_binary(&query::reward_pools(deps)?),
        QueryMsg::Balance {
            auth,
            unbonding_ids,
        } => {
            let config = CONFIG.load(deps.storage)?;
            let user = authenticate(deps, auth, config.query_auth)?;
            let unbonding_ids = match unbonding_ids {
                Some(ids) => ids,
                None => {
                    if let Some(ids) = USER_UNBONDING_IDS.may_load(deps.storage, user.clone())? {
                        ids
                    } else {
                        vec![]
                    }
                }
            };
            to_binary(&query::user_balance(
                deps,
                env,
                user.clone(),
                unbonding_ids,
            )?)
        }
        QueryMsg::Staked { auth } => {
            let config = CONFIG.load(deps.storage)?;
            to_binary(&query::user_staked(
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
        QueryMsg::Unbonding { auth, ids } => {
            let config = CONFIG.load(deps.storage)?;
            let user = authenticate(deps, auth, config.query_auth)?;
            to_binary(&query::user_unbondings(
                deps,
                user.clone(),
                ids.unwrap_or(USER_UNBONDING_IDS.load(deps.storage, user)?),
            )?)
        }
        QueryMsg::TransferWhitelist {} => to_binary(&QueryAnswer::TransferWhitelist {
            whitelist: TRANSFER_WL.load(deps.storage)?,
        }),
    }
}
