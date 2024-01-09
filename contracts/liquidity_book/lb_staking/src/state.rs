use shade_protocol::{
    c_std::{Addr, ContractInfo, StdResult, Storage, Uint256},
    lb_libraries::types::TreeUint24,
    liquidity_book::lb_staking::{
        EpochInfo,
        Reward,
        RewardTokenInfo,
        StakerInfo,
        StakerLiquidity,
        StakerLiquiditySnapshot,
        State,
        TotalLiquidity,
        TotalLiquiditySnapshot,
        Tx,
        TxAction,
    },
    s_toolkit::storage::AppendStore,
    secret_storage_plus::{Bincode2, Item, Map},
};

pub const STATE: Item<State, Bincode2> = Item::new("state");
pub const REWARD_TOKEN_INFO: Map<&Addr, Vec<RewardTokenInfo>> = Map::new("reward_token_info");
pub const REWARD_TOKENS: Item<Vec<ContractInfo>> = Item::new("reward_tokens");
pub const EPOCH_STORE: Map<u64, EpochInfo, Bincode2> = Map::new("epoch");
pub const STAKERS: Map<&Addr, StakerInfo, Bincode2> = Map::new("stakers");
pub const STAKERS_LIQUIDITY: Map<(&Addr, u32), StakerLiquidity, Bincode2> =
    Map::new("stakers_liquidity");
pub const STAKERS_BIN_TREE: Map<&Addr, TreeUint24, Bincode2> = Map::new("stakers_bin_map");
pub const STAKERS_LIQUIDITY_SNAPSHOT: Map<(&Addr, u64, u32), StakerLiquiditySnapshot, Bincode2> =
    Map::new("stakers_liquidity_snapshot");
pub const TOTAL_LIQUIDITY: Map<u32, TotalLiquidity, Bincode2> = Map::new("total_liquidity");
pub const TOTAL_LIQUIDITY_SNAPSHOT: Map<(u64, u32), TotalLiquiditySnapshot, Bincode2> =
    Map::new("total_liquidity_snapshot");
pub static TX_ID_STORE: AppendStore<u64> = AppendStore::new(b"txids");
pub const TX_STORE: Map<u64, Tx> = Map::new("tx_store");
pub const LAST_CLAIMED_EXPIRED_REWARDS_EPOCH_ID: Item<Option<u64>> =
    Item::new("last_claimed_expired_rewards_epoch_id");

pub const EXPIRED_AT_LOGGER: Item<Vec<u64>> = Item::new("expired_at");
pub const EXPIRED_AT_LOGGER_MAP: Map<u64, Vec<u64>> = Map::new("expired_at_map");

pub fn store_stake(
    storage: &mut dyn Storage,
    addr: Addr,
    state: &mut State,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
    block_time: u64,
    block_height: u64,
) -> StdResult<()> {
    let action = TxAction::Stake { ids, amounts };

    let tx = Tx {
        tx_id: state.tx_id,
        block_height,
        block_time,
        action,
    };

    TX_STORE.save(storage, state.tx_id, &tx)?;
    append_tx_for_addr(storage, state.tx_id, &addr)?;
    append_stake_tx_for_addr(storage, state.tx_id, &addr)?;
    state.tx_id += 1;
    Ok(())
}

pub fn store_unstake(
    storage: &mut dyn Storage,
    addr: Addr,
    state: &mut State,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
    block_time: u64,
    block_height: u64,
) -> StdResult<()> {
    let action = TxAction::UnStake { ids, amounts };
    let tx = Tx {
        tx_id: state.tx_id,
        block_height,
        block_time,
        action,
    };

    TX_STORE.save(storage, state.tx_id, &tx)?;
    append_tx_for_addr(storage, state.tx_id, &addr)?;
    append_unstake_tx_for_addr(storage, state.tx_id, &addr)?;
    state.tx_id += 1;
    Ok(())
}

pub fn store_claim_rewards(
    storage: &mut dyn Storage,
    addr: Addr,
    state: &mut State,
    rewards: Vec<Reward>,
    block_time: u64,
    block_height: u64,
) -> StdResult<()> {
    let action = TxAction::ClaimRewards(rewards);
    let tx = Tx {
        tx_id: state.tx_id,
        block_height,
        block_time,
        action,
    };

    TX_STORE.save(storage, state.tx_id, &tx)?;
    append_tx_for_addr(storage, state.tx_id, &addr)?;
    append_claim_rewards_tx_for_addr(storage, state.tx_id, &addr)?;
    state.tx_id += 1;
    Ok(())
}

fn append_tx_for_addr(storage: &mut dyn Storage, tx_id: u64, address: &Addr) -> StdResult<()> {
    let addr_store = TX_ID_STORE.add_suffix(address.as_bytes());
    addr_store.push(storage, &tx_id)
}

fn append_stake_tx_for_addr(
    storage: &mut dyn Storage,
    tx_id: u64,
    address: &Addr,
) -> StdResult<()> {
    let addr_store = TX_ID_STORE.add_suffix(address.as_bytes());
    let stake_store = addr_store.add_suffix("STAKE".as_bytes());
    stake_store.push(storage, &tx_id)
}

fn append_unstake_tx_for_addr(
    storage: &mut dyn Storage,
    tx_id: u64,
    address: &Addr,
) -> StdResult<()> {
    let addr_store = TX_ID_STORE.add_suffix(address.as_bytes());
    let unstake_store = addr_store.add_suffix("UNSTAKE".as_bytes());
    unstake_store.push(storage, &tx_id)
}

fn append_claim_rewards_tx_for_addr(
    storage: &mut dyn Storage,
    tx_id: u64,
    address: &Addr,
) -> StdResult<()> {
    let addr_store = TX_ID_STORE.add_suffix(address.as_bytes());
    let claim_rewards_store = addr_store.add_suffix("CLAIM_REWARDS".as_bytes());
    claim_rewards_store.push(storage, &tx_id)
}
