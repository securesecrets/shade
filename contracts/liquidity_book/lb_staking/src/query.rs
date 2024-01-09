use std::str::FromStr;

use shade_protocol::{
    c_std::{to_binary, Addr, Binary, Deps, Env, StdError, StdResult, Uint256},
    liquidity_book::lb_staking::{
        Liquidity,
        OwnerBalance,
        QueryAnswer,
        QueryMsg,
        QueryTxnType,
        QueryWithPermit,
        State,
    },
    s_toolkit::{
        permit::{validate, Permit, TokenPermissions},
        viewing_key::{ViewingKey, ViewingKeyStore},
    },
};

use crate::{
    helper::get_txs,
    state::{
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

pub fn viewing_keys_queries(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    let (addresses, key) = msg.get_validation_params()?;

    for address in addresses {
        let result = ViewingKey::check(deps.storage, address.as_str(), key.as_str());
        if result.is_ok() {
            return match msg {
                QueryMsg::Balance {
                    owner, token_id, ..
                } => query_balance(deps, &owner, token_id),

                QueryMsg::AllBalances {
                    owner,
                    page,
                    page_size,
                    ..
                } => query_all_balances(deps, &owner, page, page_size),

                QueryMsg::Liquidity {
                    owner,
                    round_index,
                    token_ids,
                    ..
                } => query_liquidity(deps, &owner, token_ids, round_index),

                QueryMsg::TransactionHistory {
                    owner,
                    page,
                    page_size,
                    txn_type,
                    ..
                } => query_transaction_history(deps, &owner, page, page_size, txn_type),

                QueryMsg::WithPermit { .. } => {
                    unreachable!("This query type does not require viewing key authentication")
                }
                _ => unreachable!("This query type does not require viewing key authentication"),
            };
        }
    }

    to_binary(&QueryAnswer::ViewingKeyError {
        msg: "Wrong viewing key for this address or viewing key not set".to_string(),
    })
}

pub fn query_balance(deps: Deps, owner: &Addr, token_id: String) -> StdResult<Binary> {
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
    owner: &Addr,
    page: Option<u32>,
    page_size: Option<u32>,
) -> StdResult<Binary> {
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
    owner: &Addr,
    page: Option<u32>,
    page_size: Option<u32>,
    query_type: QueryTxnType,
) -> StdResult<Binary> {
    let page = page.unwrap_or(0u32);
    let page_size = page_size.unwrap_or(50u32);

    let (txns, count) = get_txs(deps.storage, owner, page, page_size, query_type)?;

    let response = QueryAnswer::TransactionHistory { txns, count };
    to_binary(&response)
}

pub fn query_liquidity(
    deps: Deps,
    owner: &Addr,
    token_ids: Vec<u32>,
    round_index: Option<u64>,
) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
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
                    return Err(StdError::generic_err("Under-flow sub error"));
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
                        return Err(StdError::generic_err("Under-flow sub error"));
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

pub fn permit_queries(
    deps: Deps,
    env: Env,
    permit: Permit,
    query: QueryWithPermit,
) -> Result<Binary, StdError> {
    // Validate permit content
    let contract_address = env.contract.address;
    let account_str = validate(
        deps,
        PREFIX_REVOKED_PERMITS,
        &permit,
        contract_address.to_string(),
        None,
    )?;
    let account = deps.api.addr_validate(&account_str)?;

    let is_owner = permit.check_permission(&TokenPermissions::Owner);

    // Permit validated! We can now execute the query.
    match query {
        QueryWithPermit::Balance { owner, token_id } => {
            if !is_owner {
                if !permit.check_permission(&TokenPermissions::Balance) {
                    return Err(StdError::generic_err(format!(
                        "`Owner` or `Balance` permit required for permit queries, got permissions {:?}",
                        permit.params.permissions
                    )));
                }
            }
            query_balance(deps, &owner, token_id)
        }
        QueryWithPermit::AllBalances { page, page_size } => {
            if !is_owner {
                if !permit.check_permission(&TokenPermissions::Balance) {
                    return Err(StdError::generic_err(format!(
                        "`Owner` or `Balance` permit required for permit queries, got permissions {:?}",
                        permit.params.permissions
                    )));
                }
            }
            query_all_balances(deps, &account, page, page_size)
        }
        _ => unreachable!("This query type does not require viewing key authentication"),
    }
}
