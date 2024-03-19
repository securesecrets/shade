use std::str::FromStr;

use shade_protocol::{
    c_std::{to_binary, Binary, Deps, StdError, StdResult, Uint256},
    liquidity_book::lb_staking::{
        Auth,
        EpochInfo,
        Liquidity,
        OwnerBalance,
        QueryAnswer,
        QueryTxnType,
        State,
    },
};

use crate::{
    contract::authenticate,
    helper::get_txs,
    state::{
        EPOCH_STORE,
        REWARD_TOKENS,
        STAKERS,
        STAKERS_BIN_TREE,
        STAKERS_LIQUIDITY,
        STAKERS_LIQUIDITY_SNAPSHOT,
        STATE,
        TOTAL_LIQUIDITY,
        TOTAL_LIQUIDITY_SNAPSHOT,
    },
};

pub const SHADE_STAKING_VIEWING_KEY: &str = "SHADE_STAKING_VIEWING_KEY";
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";

pub fn query_contract_info(deps: Deps) -> StdResult<Binary> {
    let state: State = STATE.load(deps.storage)?;

    let response = QueryAnswer::ContractInfo {
        lb_token: state.lb_token,
        lb_pair: state.lb_pair,
        admin_auth: state.admin_auth,
        query_auth: state.query_auth,
        epoch_index: state.epoch_index,
        epoch_durations: state.epoch_durations,
        expiry_durations: state.expiry_durations,
    };
    to_binary(&response)
}

pub fn query_epoch_info(deps: Deps, epoch_index: Option<u64>) -> StdResult<Binary> {
    let state: State = STATE.load(deps.storage)?;
    let epoch_info: EpochInfo =
        EPOCH_STORE.load(deps.storage, epoch_index.unwrap_or(state.epoch_index))?;

    let response = QueryAnswer::EpochInfo {
        rewards_distribution: epoch_info.rewards_distribution,
        reward_tokens: epoch_info.reward_tokens,
        start_time: epoch_info.start_time,
        end_time: epoch_info.end_time,
        duration: epoch_info.duration,
        expired_at: epoch_info.expired_at,
    };

    to_binary(&response)
}

pub fn query_registered_tokens(deps: Deps) -> StdResult<Binary> {
    let reg_tokens = REWARD_TOKENS.load(deps.storage)?;

    let response = QueryAnswer::RegisteredTokens(reg_tokens);
    to_binary(&response)
}

pub fn query_token_id_balance(deps: Deps, token_id: String) -> StdResult<Binary> {
    let id = u32::from_str(&token_id)
        .map_err(|_| StdError::generic_err(format!("token_id {} cannot be parsed", token_id)))?;

    let liquidity = TOTAL_LIQUIDITY
        .load(deps.storage, id)
        .map_err(|_| StdError::generic_err(format!("token_id {} does not exist", token_id)))?;

    let response = QueryAnswer::IdTotalBalance {
        amount: liquidity.amount_delegated,
    };
    to_binary(&response)
}

pub fn query_staker_info(deps: Deps, auth: Auth) -> StdResult<Binary> {
    let state: State = STATE.load(deps.storage)?;
    let owner = &authenticate(deps, auth, state.query_auth)?;
    let staker_info = STAKERS.load(deps.storage, owner)?;

    let response = QueryAnswer::StakerInfo {
        starting_round: staker_info.starting_round,
        total_rewards_earned: staker_info.total_rewards_earned,
        last_claim_rewards_round: staker_info.last_claim_rewards_round,
    };
    to_binary(&response)
}

pub fn query_balance(deps: Deps, auth: Auth, token_id: String) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    let owner = &authenticate(deps, auth, state.query_auth)?;
    let id = u32::from_str(&token_id)
        .map_err(|_| StdError::generic_err(format!("token_id {} cannot be parsed", token_id)))?;

    let liquidity = STAKERS_LIQUIDITY
        .load(deps.storage, (owner, id))
        .map_err(|_| StdError::generic_err(format!("token_id {} does not exist", token_id)))?;

    let response = QueryAnswer::Balance {
        amount: liquidity.amount_delegated,
    };
    to_binary(&response)
}

pub fn query_all_balances(
    deps: Deps,
    auth: Auth,
    page: Option<u32>,
    page_size: Option<u32>,
) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    let owner = &authenticate(deps, auth, state.query_auth)?;

    let page = page.unwrap_or(0u32);
    let page_size = page_size.unwrap_or(50u32);
    let tree = STAKERS_BIN_TREE.load(deps.storage, owner)?;

    let mut token_id = 0u32;
    for _ in 0..(page * page_size) {
        token_id = tree.find_first_left(token_id); //skipping there bins_id 
    }

    let mut balances = Vec::new();

    for _ in 0..(page * page_size) {
        let liquidity = STAKERS_LIQUIDITY
            .load(deps.storage, (owner, token_id))
            .map_err(|_| StdError::generic_err(format!("token_id {} does not exist", token_id)))?;
        token_id = tree.find_first_left(token_id);
        balances.push(OwnerBalance {
            token_id: token_id.to_string(),
            amount: liquidity.amount_delegated,
        })
    }

    let response = QueryAnswer::AllBalances(balances);
    to_binary(&response)
}

pub fn query_transaction_history(
    deps: Deps,
    auth: Auth,
    page: Option<u32>,
    page_size: Option<u32>,
    query_type: QueryTxnType,
) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    let owner = &authenticate(deps, auth, state.query_auth)?;
    let page = page.unwrap_or(0u32);
    let page_size = page_size.unwrap_or(50u32);

    let (txns, count) = get_txs(deps.storage, owner, page, page_size, query_type)?;

    let response = QueryAnswer::TransactionHistory { txns, count };
    to_binary(&response)
}

pub fn query_liquidity(
    deps: Deps,
    auth: Auth,
    token_ids: Vec<u32>,
    round_index: Option<u64>,
) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    let owner = &authenticate(deps, auth, state.query_auth)?;
    let staker_result = STAKERS.load(deps.storage, &owner);
    let mut liquidity_response = Vec::new();
    // Check if staker exists
    let staker = match staker_result {
        Ok(staker) => staker,
        Err(_) => {
            return Err(StdError::generic_err(format!(
                "No staker found for address {:?}.",
                owner
            )));
        }
    };

    for token_id in token_ids {
        let staker_liq_snap = STAKERS_LIQUIDITY_SNAPSHOT.load(
            deps.storage,
            (&owner, round_index.unwrap_or(state.epoch_index), token_id),
        )?;
        let total_liq_snap = TOTAL_LIQUIDITY_SNAPSHOT.load(
            deps.storage,
            (round_index.unwrap_or(state.epoch_index), token_id),
        )?;

        //Calculating User Liquidity to generate n tickets for the round
        let liquidity_current_round: Uint256;
        let mut legacy_bal = Uint256::zero();
        let total_liq;

        // Total Liquidity
        if total_liq_snap.liquidity.is_zero() {
            let total_liquidity = TOTAL_LIQUIDITY.load(deps.storage, token_id)?;
            total_liq = total_liquidity.amount_delegated;
        } else {
            total_liq = total_liq_snap.liquidity;
        }

        if !staker_liq_snap.liquidity.is_zero() {
            liquidity_current_round = staker_liq_snap.liquidity;
        } else {
            let mut finding_liq_round: u64 =
                if let Some(r_i) = round_index.unwrap_or(state.epoch_index).checked_sub(1) {
                    r_i
                } else {
                    return Err(StdError::generic_err("Under-flow sub error query_liq 1"));
                };

            let start = if staker.last_claim_rewards_round.is_some() {
                staker.last_claim_rewards_round.unwrap()
            } else {
                if staker.starting_round.is_some() {
                    staker.starting_round.unwrap()
                } else {
                    liquidity_response.push(Liquidity {
                        token_id: token_id.to_string(),
                        user_liquidity: Uint256::zero(),
                        total_liquidity: total_liq,
                    });

                    let response = QueryAnswer::Liquidity(liquidity_response);

                    return to_binary(&response);
                }
            };

            while finding_liq_round >= start {
                let staker_liq_snap_prev = STAKERS_LIQUIDITY_SNAPSHOT
                    .load(deps.storage, (&owner, finding_liq_round, token_id))?;

                if !staker_liq_snap_prev.amount_delegated.is_zero() {
                    legacy_bal = staker_liq_snap_prev.amount_delegated;
                    break;
                } else {
                    finding_liq_round = if let Some(f_liq) = finding_liq_round.checked_sub(1) {
                        f_liq
                    } else {
                        return Err(StdError::generic_err("Under-flow sub error query_liq 2"));
                    }
                }
            }

            liquidity_current_round = legacy_bal;
        }
        liquidity_response.push(Liquidity {
            token_id: token_id.to_string(),
            user_liquidity: liquidity_current_round,
            total_liquidity: total_liq,
        });
    }

    let response = QueryAnswer::Liquidity(liquidity_response);

    to_binary(&response)
}
