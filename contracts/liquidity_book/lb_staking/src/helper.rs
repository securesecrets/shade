use shade_protocol::{
    c_std::{
        Addr,
        ContractInfo,
        CosmosMsg,
        MessageInfo,
        StdError,
        StdResult,
        Storage,
        Uint128,
        Uint256,
    },
    liquidity_book::lb_staking::{
        QueryTxnType,
        StakerInfo,
        StakerLiquiditySnapshot,
        State,
        TotalLiquiditySnapshot,
        Tx,
    },
    snip20::helpers::{register_receive, set_viewing_key_msg},
    Contract,
};
use std::hash::{Hash, Hasher};

use crate::{
    contract::SHADE_STAKING_VIEWING_KEY,
    state::{
        REWARD_TOKENS,
        STAKERS,
        STAKERS_LIQUIDITY_SNAPSHOT,
        TOTAL_LIQUIDITY,
        TOTAL_LIQUIDITY_SNAPSHOT,
        TX_ID_STORE,
        TX_STORE,
    },
};

pub fn register_reward_tokens(
    storage: &mut dyn Storage,
    tokens: Vec<ContractInfo>,
    contract_code_hash: String,
) -> StdResult<Vec<CosmosMsg>> {
    let mut reg_tokens = REWARD_TOKENS.load(storage)?;
    let mut messages = Vec::new();
    for token in tokens.iter() {
        if !reg_tokens.contains(token) {
            reg_tokens.push(token.clone());

            let contract = &Contract {
                address: token.address.to_owned(),
                code_hash: token.code_hash.to_owned(),
            };

            //register receive
            messages.push(register_receive(
                contract_code_hash.to_owned(),
                None,
                contract,
            )?);
            messages.push(set_viewing_key_msg(
                SHADE_STAKING_VIEWING_KEY.to_string(),
                None,
                contract,
            )?);
            //set viewing_key
        } else {
            return Err(StdError::generic_err("Reward token already exists"));
        }
    }
    REWARD_TOKENS.save(storage, &reg_tokens)?;
    Ok(messages)
}

pub fn store_empty_reward_set(storage: &mut dyn Storage) -> StdResult<()> {
    match REWARD_TOKENS.may_load(storage)? {
        Some(_) => Err(StdError::generic_err("Reward token storage already exists")),
        None => REWARD_TOKENS.save(storage, &(vec![])),
    }
}

pub fn require_lb_token(state: &State, addr: &Addr) -> StdResult<()> {
    if state.lb_token.address.eq(addr) {
        return Ok(());
    }
    Err(StdError::generic_err(format!(
        "Must stake the LP token {}. Attempted to stake {addr}.",
        state.lb_token.address
    )))
}

pub fn staker_init_checker(
    storage: &mut dyn Storage,
    state: &State,
    staker_addr: &Addr,
) -> StdResult<()> {
    let staker = STAKERS.load(storage, &staker_addr);

    if staker.is_err() {
        STAKERS.save(storage, &staker_addr, &StakerInfo {
            starting_round: Some(state.epoch_index),
            total_rewards_earned: Uint128::default(),
            last_claim_rewards_round: None,
        })?;
    }
    Ok(())
}

pub fn assert_lb_pair(state: &State, info: MessageInfo) -> StdResult<()> {
    if info.sender != state.lb_pair {
        Err(StdError::generic_err(format!(
            "Onlt accessible by  lb-pair {}",
            state.lb_pair
        )))
    } else {
        Ok(())
    }
}

pub fn check_if_claimable(starting_round: Option<u64>, current_round_index: u64) -> StdResult<()> {
    if starting_round.is_none() {
        return Err(StdError::generic_err(format!(
            "You have not deposited to the pook contract. No tickets found"
        )));
    } else {
        if starting_round.unwrap() == current_round_index {
            return Err(StdError::generic_err(format!(
                "You are not yet able to claim rewards. Wait for this round to end"
            )));
        }
    }
    Ok(())
}

pub fn finding_user_liquidity(
    storage: &dyn Storage,
    info: &MessageInfo,
    staker_info: &StakerInfo,
    epoch_index: u64,
    bin_id: u32,
) -> StdResult<(StakerLiquiditySnapshot, bool)> {
    let mut legacy_bal: Uint256 = Uint256::zero();
    let mut staker_liq_snap = STAKERS_LIQUIDITY_SNAPSHOT
        .load(storage, (&info.sender, epoch_index, bin_id))
        .unwrap_or_default();

    if !staker_liq_snap.liquidity.is_zero() {
        Ok((staker_liq_snap, false))
    } else {
        let mut finding_liq_round: u64 = if let Some(rn) = epoch_index.checked_sub(1) {
            rn
        } else {
            return Err(StdError::generic_err("Under-flow sub error 3"));
        };
        let start = if staker_info.last_claim_rewards_round.is_some() {
            staker_info.last_claim_rewards_round.unwrap()
        } else {
            staker_info.starting_round.unwrap()
        };

        while finding_liq_round >= start {
            let staker_liq_snap_prev_round = STAKERS_LIQUIDITY_SNAPSHOT
                .load(storage, (&info.sender, finding_liq_round, bin_id))
                .unwrap_or_default();
            if !staker_liq_snap_prev_round.amount_delegated.is_zero() {
                legacy_bal = staker_liq_snap_prev_round.amount_delegated;
                break;
            } else {
                finding_liq_round = if let Some(f_liq_round) = finding_liq_round.checked_sub(1) {
                    f_liq_round
                } else {
                    return Err(StdError::generic_err("Under-flow sub error 4"));
                };
            }
        }

        staker_liq_snap.liquidity = legacy_bal;
        staker_liq_snap.amount_delegated = legacy_bal;

        Ok((staker_liq_snap, true))
    }
}

pub fn finding_total_liquidity(
    storage: &dyn Storage,
    epoch_index: u64,
    bin_id: u32,
) -> StdResult<(TotalLiquiditySnapshot, bool)> {
    let mut legacy_bal: Uint256 = Uint256::zero();
    let mut total_liq_snap = TOTAL_LIQUIDITY_SNAPSHOT
        .load(storage, (epoch_index, bin_id))
        .unwrap_or_default();
    let total_liq = TOTAL_LIQUIDITY.load(storage, bin_id).unwrap_or_default();

    if !total_liq_snap.liquidity.is_zero() {
        Ok((total_liq_snap, false))
    } else {
        let mut finding_liq_round: u64 = if let Some(rn) = epoch_index.checked_sub(1) {
            rn
        } else {
            return Err(StdError::generic_err("Under-flow sub error 3"));
        };
        let start = if total_liq.last_deposited.is_some() {
            total_liq.last_deposited.unwrap()
        } else {
            return Ok((total_liq_snap, false));
        };
        while finding_liq_round >= start {
            let staker_liq_snap_prev_round = TOTAL_LIQUIDITY_SNAPSHOT
                .load(storage, (finding_liq_round, bin_id))
                .unwrap_or_default();
            if !staker_liq_snap_prev_round.amount_delegated.is_zero() {
                legacy_bal = staker_liq_snap_prev_round.amount_delegated;
                break;
            } else {
                finding_liq_round = if let Some(f_liq_round) = finding_liq_round.checked_sub(1) {
                    f_liq_round
                } else {
                    return Err(StdError::generic_err("Under-flow sub error 4"));
                };
            }
        }

        total_liq_snap.liquidity = legacy_bal;
        total_liq_snap.amount_delegated = legacy_bal;

        Ok((total_liq_snap, true))
    }
}

#[derive(Clone, Debug)]
pub struct TokenKey {
    pub address: Addr,
    pub code_hash: String,
}

impl PartialEq for TokenKey {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.code_hash == other.code_hash
    }
}

impl Eq for TokenKey {}

impl Hash for TokenKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        self.code_hash.hash(state);
    }
}

pub fn get_txs(
    storage: &dyn Storage,
    address: &Addr,
    page: u32,
    page_size: u32,
    query_tx_type: QueryTxnType,
) -> StdResult<(Vec<Tx>, u64)> {
    let mut addr_store = TX_ID_STORE.add_suffix(address.as_bytes());

    addr_store = match query_tx_type {
        QueryTxnType::All => addr_store,
        QueryTxnType::Stake => addr_store.add_suffix("STAKE".as_bytes()),
        QueryTxnType::UnStake => addr_store.add_suffix("UNSTAKE".as_bytes()),
        QueryTxnType::ClaimRewards => addr_store.add_suffix("CLAIM_REWARDS".as_bytes()),
    };

    let count = addr_store.get_len(storage)? as u64;
    // access tx storage
    // Take `page_size` txs starting from the latest tx, potentially skipping `page * page_size`
    // txs from the start.
    let txs: StdResult<Vec<Tx>> = addr_store
        .iter(storage)?
        .rev()
        .skip((page * page_size) as usize)
        .take(page_size as usize)
        .map(|id| id.map(|id| TX_STORE.load(storage, id)).and_then(|x| x))
        .collect();

    txs.map(|t| (t, count))
}
