use crate::{
    execute::*,
    helper::store_empty_reward_set,
    query::*,
    state::{EPOCH_STORE, EXPIRED_AT_LOGGER, LAST_CLAIMED_EXPIRED_REWARDS_EPOCH_ID, STATE},
};
use shade_protocol::{
    c_std::{
        shd_entry_point, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
        StdResult,
    },
    liquidity_book::lb_staking::{
        Auth, AuthPermit, EpochInfo, ExecuteMsg, InstantiateMsg, QueryMsg, State,
    },
    query_auth::helpers::{authenticate_permit, authenticate_vk, PermitAuthentication},
    snip20::helpers::{register_receive, set_viewing_key_msg},
    utils::pad_handle_result,
    Contract, BLOCK_SIZE,
};

pub const SHADE_STAKING_VIEWING_KEY: &str = "SHADE_STAKING_VIEWING_KEY";
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        lb_token: msg.lb_token.valid(deps.api)?,
        lb_pair: deps.api.addr_validate(&msg.amm_pair)?,
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
        query_auth: msg.query_auth.into_valid(deps.api)?,
        epoch_index: msg.epoch_index,
        epoch_durations: msg.epoch_duration,
        expiry_durations: msg.expiry_duration,
        tx_id: 0,
        recover_funds_receiver: msg.recover_funds_receiver,
    };

    let now = env.block.time.seconds();
    EPOCH_STORE.save(
        deps.storage,
        state.epoch_index,
        &EpochInfo {
            rewards_distribution: None,
            start_time: now,
            end_time: now + state.epoch_durations,
            duration: state.epoch_durations,
            reward_tokens: None,
            expired_at: None,
        },
    )?;

    let messages = vec![
        register_receive(
            env.contract.code_hash.clone(),
            None,
            &state.lb_token.clone().into(),
        )?,
        set_viewing_key_msg(
            SHADE_STAKING_VIEWING_KEY.to_string(),
            None,
            &state.lb_token.clone().into(),
        )?,
    ];

    let mut response: Response = Response::new();
    response.data = Some(env.contract.address.as_bytes().into());

    store_empty_reward_set(deps.storage)?;

    STATE.save(deps.storage, &state)?;
    LAST_CLAIMED_EXPIRED_REWARDS_EPOCH_ID.save(deps.storage, &None)?;
    EXPIRED_AT_LOGGER.save(deps.storage, &vec![])?;

    Ok(response.add_messages(messages))
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    pad_handle_result(
        match msg {
            ExecuteMsg::Snip1155Receive(msg) => {
                let checked_from = deps.api.addr_validate(&msg.from.as_str())?;
                receiver_callback_snip_1155(
                    deps,
                    env,
                    info,
                    checked_from,
                    msg.token_id,
                    msg.amount,
                    msg.msg,
                )
            }
            ExecuteMsg::Receive(msg) => {
                let checked_from = deps.api.addr_validate(&msg.from.as_str())?;
                receiver_callback(deps, info, checked_from, msg.amount, msg.msg)
            }
            ExecuteMsg::ClaimRewards {} => try_claim_rewards(deps, env, info),
            ExecuteMsg::Unstake { token_ids, amounts } => {
                try_unstake(deps, env, info, token_ids, amounts)
            }
            ExecuteMsg::RegisterRewardTokens(tokens) => {
                try_register_reward_tokens(deps, env, info, tokens)
            }
            ExecuteMsg::UpdateConfig {
                admin_auth,
                query_auth,
                epoch_duration,
                expiry_duration,
            } => try_update_config(
                deps,
                info,
                env,
                admin_auth,
                query_auth,
                epoch_duration,
                expiry_duration,
            ),
            ExecuteMsg::RecoverExpiredFunds { .. } => try_recover_expired_funds(deps, env, info),
            ExecuteMsg::EndEpoch {
                rewards_distribution,
                epoch_index,
            } => try_end_epoch(deps, env, info, epoch_index, rewards_distribution),

            ExecuteMsg::RecoverFunds {
                token,
                amount,
                to,
                msg,
            } => try_recover_funds(deps, env, info, token, amount, to, msg),
        },
        BLOCK_SIZE,
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => query_contract_info(deps),
        QueryMsg::EpochInfo { index } => query_epoch_info(deps, index),

        QueryMsg::RegisteredTokens {} => query_registered_tokens(deps),
        QueryMsg::IdTotalBalance { id } => query_token_id_balance(deps, id),
        QueryMsg::Balance { auth, token_id } => query_balance(deps, auth, token_id),
        QueryMsg::StakerInfo { auth } => query_staker_info(deps, auth),
        QueryMsg::AllBalances {
            auth,
            page,
            page_size,
        } => query_all_balances(deps, auth, page, page_size),
        QueryMsg::Liquidity {
            auth,
            round_index,
            token_ids,
        } => query_liquidity(deps, auth, token_ids, round_index),
        QueryMsg::TransactionHistory {
            auth,
            page,
            page_size,
            txn_type,
        } => query_transaction_history(deps, auth, page, page_size, txn_type),
    }
}
