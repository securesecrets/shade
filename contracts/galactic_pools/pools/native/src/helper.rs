//Crate Import
use crate::{
    constants::*,
    state::{
        ConfigInfo,
        GlobalSponsorDisplayRequestListState,
        GlobalSponsorState,
        PoolLiqState,
        PoolState,
        RewardsState,
        RoundInfo,
        SponsorInfo,
        UnbondingBatch,
        UserInfo,
        UserLiqState,
        UserRewardsLog,
    },
    viewing_key::ViewingKey,
};

//Cosmwasm import
use shade_protocol::c_std::{Addr, StdResult, Storage, Uint128};

//secret toolkit import
use secret_storage_plus::{AppendStore, DequeStore, Item, Map};
use shade_protocol::secret_storage_plus;

pub const CONFIG_STORE: Item<ConfigInfo> = Item::new(CONFIG_KEY);
pub fn config_helper_read_only(storage: &dyn Storage) -> StdResult<ConfigInfo> {
    return Ok(CONFIG_STORE.load(storage)?);
}

pub fn config_helper_store(storage: &mut dyn Storage, config: &ConfigInfo) -> StdResult<()> {
    CONFIG_STORE.save(storage, &config)?;
    Ok(())
}

pub const POOL_STATE_STORE: Item<PoolState> = Item::new(POOL_STATE_KEY);
pub fn pool_state_helper_read_only(storage: &dyn Storage) -> StdResult<PoolState> {
    return Ok(POOL_STATE_STORE.load(storage)?);
}

pub fn pool_state_helper_store(
    storage: &mut dyn Storage,
    pool_state_obj: &PoolState,
) -> StdResult<()> {
    let _ = POOL_STATE_STORE.save(storage, &pool_state_obj);
    Ok(())
}

pub const ROUND_STORE: Item<RoundInfo> = Item::new(ROUND_KEY);
pub fn round_helper_read_only(storage: &dyn Storage) -> StdResult<RoundInfo> {
    return Ok(ROUND_STORE.load(storage)?);
}
pub fn round_helper_store(storage: &mut dyn Storage, round_obj: &RoundInfo) -> StdResult<()> {
    ROUND_STORE.save(storage, &round_obj)?;
    Ok(())
}

pub const USER_INFO_STORE: Map<&Addr, UserInfo> = Map::new(USER_INFO_KEY);
pub fn user_info_helper_read_only(storage: &dyn Storage, sender: &Addr) -> StdResult<UserInfo> {
    let user_info_obj = USER_INFO_STORE.load(storage, sender).unwrap_or(UserInfo {
        amount_delegated: Uint128::zero(),
        amount_withdrawable: Uint128::zero(),
        starting_round: None,
        total_won: Uint128::zero(),
        last_claim_rewards_round: None,
        amount_unbonding: Uint128::zero(),
        unbonding_batches: Vec::new(),
    });
    return Ok(user_info_obj);
}

pub fn user_info_helper_store(
    storage: &mut dyn Storage,
    sender: &Addr,
    user_info_obj: &UserInfo,
) -> StdResult<()> {
    let _ = USER_INFO_STORE.save(storage, sender, user_info_obj)?;
    Ok(())
}

pub const USER_UNBOND_STORE: Map<(&Addr, u64), Uint128> = Map::new(USER_UNBOND_KEY);
pub fn user_unbond_helper_read_only(
    storage: &dyn Storage,
    unbond_batch_index: u64,
    sender: &Addr,
) -> StdResult<Uint128> {
    let user_unbonding_obj = USER_UNBOND_STORE
        .load(storage, (sender, unbond_batch_index))
        .unwrap_or_default();
    return Ok(user_unbonding_obj);
}

pub fn user_unbond_helper_store(
    storage: &mut dyn Storage,
    unbond_batch_index: u64,

    sender: &Addr,
    unbonding_amount: Uint128,
) -> StdResult<()> {
    let _ = USER_UNBOND_STORE.save(storage, (sender, unbond_batch_index), &unbonding_amount)?;
    Ok(())
}

pub const ADMIN_UNBOND_STORE: Map<u64, Uint128> = Map::new(USER_UNBOND_KEY);
pub fn admin_unbond_helper_read_only(
    storage: &dyn Storage,
    unbond_batch_index: u64,
) -> StdResult<Uint128> {
    let admin_unbonding_obj = ADMIN_UNBOND_STORE
        .load(storage, unbond_batch_index)
        .unwrap_or_default();
    return Ok(admin_unbonding_obj);
}

pub fn admin_unbond_helper_store(
    storage: &mut dyn Storage,
    unbond_batch_index: u64,
    unbonding_amount: Uint128,
) -> StdResult<()> {
    let _ = ADMIN_UNBOND_STORE.save(storage, unbond_batch_index, &unbonding_amount)?;
    Ok(())
}

pub const SPONSOR_INFO_STORE: Map<&Addr, SponsorInfo> = Map::new(SPONSOR_INFO_KEY);
pub fn sponsor_info_helper_read_only(
    storage: &dyn Storage,
    sender: &Addr,
) -> StdResult<SponsorInfo> {
    let sponsor_obj = SPONSOR_INFO_STORE.load(storage, sender).unwrap_or_default();

    return Ok(sponsor_obj);
}

pub fn sponsor_info_helper_store(
    storage: &mut dyn Storage,
    sender: &Addr,
    sponsor_info_obj: &SponsorInfo,
) {
    SPONSOR_INFO_STORE
        .save(storage, sender, sponsor_info_obj)
        .unwrap_or_default();
}

pub const SPONSOR_UNBOND_STORE: Map<(&Addr, u64), Uint128> = Map::new(SPONSOR_UNBONDING_KEY);
pub fn sponsor_unbond_helper_read_only(
    storage: &dyn Storage,
    unbond_batch_index: u64,
    sender: &Addr,
) -> StdResult<Uint128> {
    let sponsor_unbonding_obj = SPONSOR_UNBOND_STORE
        .load(storage, (sender, unbond_batch_index))
        .unwrap_or_default();
    return Ok(sponsor_unbonding_obj);
}

pub fn sponsor_unbond_helper_store(
    storage: &mut dyn Storage,
    unbond_batch_index: u64,

    sender: &Addr,
    unbonding_amount: Uint128,
) -> StdResult<()> {
    let _ = SPONSOR_UNBOND_STORE.save(storage, (sender, unbond_batch_index), &unbonding_amount)?;
    Ok(())
}

pub const SPONSOR_STATS_STORE: Item<GlobalSponsorState> = Item::new(SPONSOR_STATS_KEY);
pub fn sponsor_stats_helper_read_only(storage: &dyn Storage) -> StdResult<GlobalSponsorState> {
    return Ok(SPONSOR_STATS_STORE.load(storage)?);
}

pub fn sponsor_stats_helper_store(
    storage: &mut dyn Storage,
    sponsor_stats_obj: &GlobalSponsorState,
) -> StdResult<()> {
    let _ = SPONSOR_STATS_STORE.save(storage, &sponsor_stats_obj);
    Ok(())
}

pub const SPONSOR_LIST_STORE: Map<u32, Addr> = Map::new(SPONSOR_ADDRESS_LIST_KEY);
pub fn sponsor_addr_list_helper_read_only(storage: &dyn Storage, index: u32) -> StdResult<Addr> {
    return Ok(SPONSOR_LIST_STORE.load(storage, index)?);
}

pub fn sponsor_addr_list_helper_store(
    storage: &mut dyn Storage,
    index: u32,

    addr: &Addr,
) -> StdResult<()> {
    let _ = SPONSOR_LIST_STORE.save(storage, index, addr);
    Ok(())
}

pub fn sponsor_addr_list_remove_helper_store(
    storage: &mut dyn Storage,
    index: u32,
) -> StdResult<()> {
    let _ = SPONSOR_LIST_STORE.remove(storage, index);
    Ok(())
}

//Sponsor Name & Message Display Request
pub const SPONSOR_DISPLAY_REQ_STORE: DequeStore<GlobalSponsorDisplayRequestListState> =
    DequeStore::new(SPONSOR_NAME_AND_MESSAGE_DISPLAY_REQUEST_KEY);
pub fn sponsor_display_request_deque_push_back_helper(
    storage: &mut dyn Storage,
    item: &GlobalSponsorDisplayRequestListState,
) -> StdResult<()> {
    let spn_disp_req_store = SPONSOR_DISPLAY_REQ_STORE;
    spn_disp_req_store.push_back(storage, item)?;
    Ok(())
}

pub fn sponsor_display_request_deque_helper_remove(
    storage: &mut dyn Storage,
    index: u32,
) -> StdResult<()> {
    let spn_disp_req_store = SPONSOR_DISPLAY_REQ_STORE;
    spn_disp_req_store.remove(storage, index)?;

    return Ok(());
}

pub const USER_REWARDS_LOG_STORE: AppendStore<UserRewardsLog> =
    AppendStore::new(USER_REWARDS_LOG_KEY);
pub fn user_records_helper_read_only(
    storage: &dyn Storage,
    sender: &Addr,
    start_page: Option<u32>,
    page_size: Option<u32>,
) -> StdResult<Vec<UserRewardsLog>> {
    let user_store = USER_REWARDS_LOG_STORE.add_suffix(sender.as_str());
    let page_size = page_size.unwrap_or(5);
    let start_page = start_page.unwrap_or(0);
    let values = user_store.paging(storage, start_page, page_size)?;

    return Ok(values);
}

pub fn user_rewards_log_helper_store(
    storage: &mut dyn Storage,
    sender: &Addr,
    user_rewards_log_obj: &UserRewardsLog,
) -> StdResult<()> {
    let user_store = USER_REWARDS_LOG_STORE.add_suffix(sender.as_str());
    user_store.push(storage, user_rewards_log_obj)?;
    Ok(())
}

pub const USER_LIQUIDITY_STATS_STORE: Map<(&Addr, u64), UserLiqState> =
    Map::new(USER_LIQUIDITY_KEY);
pub fn user_liquidity_snapshot_stats_helper_read_only(
    storage: &dyn Storage,
    round_index: u64,
    sender: &Addr,
) -> StdResult<UserLiqState> {
    let user_liquidity_snapshot_obj = USER_LIQUIDITY_STATS_STORE
        .load(storage, (sender, round_index))
        .unwrap_or(UserLiqState {
            amount_delegated: None,
            liquidity: None,
            tickets_used: None,
        });
    Ok(user_liquidity_snapshot_obj)
}

pub fn user_liquidity_snapshot_stats_helper_store(
    storage: &mut dyn Storage,
    round_index: u64,
    sender: &Addr,
    user_liq_obj: UserLiqState,
) -> StdResult<()> {
    USER_LIQUIDITY_STATS_STORE.save(storage, (sender, round_index), &user_liq_obj)?;
    Ok(())
}

pub const POOL_STATE_LIQUIDITY_STATS_STORE: Map<u64, PoolLiqState> =
    Map::new(POOL_STATE_LIQUIDITY_KEY);

pub fn pool_state_liquidity_helper_read_only(
    storage: &dyn Storage,
    current_round_index: u64,
) -> StdResult<PoolLiqState> {
    let pool_state_liquidity_snapshot_obj: PoolLiqState = POOL_STATE_LIQUIDITY_STATS_STORE
        .load(storage, current_round_index)
        .unwrap_or(PoolLiqState {
            total_delegated: None,
            total_liquidity: None,
        });
    Ok(pool_state_liquidity_snapshot_obj)
}

pub fn pool_state_liquidity_helper_store(
    storage: &mut dyn Storage,
    current_round_index: u64,
    pool_state_liquidity_snapshot_obj: PoolLiqState,
) -> StdResult<()> {
    POOL_STATE_LIQUIDITY_STATS_STORE.save(
        storage,
        current_round_index,
        &pool_state_liquidity_snapshot_obj,
    )?;
    Ok(())
}

pub const UNBONDING_BATCH_STORE: Map<u64, UnbondingBatch> = Map::new(UNBONDING_BATCH_KEY);
pub fn unbonding_batch_helper_read_only(
    storage: &dyn Storage,
    unbonding_round_index: u64,
) -> StdResult<UnbondingBatch> {
    let unbonding_batch_snapshot_obj: UnbondingBatch = UNBONDING_BATCH_STORE
        .load(storage, unbonding_round_index)
        .unwrap_or(UnbondingBatch {
            amount: None,
            unbonding_time: None,
        });

    Ok(unbonding_batch_snapshot_obj)
}

pub fn unbonding_batch_helper_store(
    storage: &mut dyn Storage,
    unbonding_batch_index: u64,
    unbonding_batch_obj: &UnbondingBatch,
) -> StdResult<()> {
    UNBONDING_BATCH_STORE.save(storage, unbonding_batch_index, unbonding_batch_obj)?;
    Ok(())
}

pub const REWARDS_STATS_FOR_NTH_ROUND_STORE: Map<u64, RewardsState> =
    Map::new(REWARD_STATS_FOR_NTH_ROUND_KEY);

pub fn reward_stats_for_nth_round_helper_read_only(
    storage: &dyn Storage,
    round_index: u64,
) -> StdResult<RewardsState> {
    let reward_stats_for_nth_round = REWARDS_STATS_FOR_NTH_ROUND_STORE
        .load(storage, round_index)
        .unwrap_or(Default::default());

    Ok(reward_stats_for_nth_round)
}

pub fn reward_stats_for_nth_round_helper_store(
    storage: &mut dyn Storage,
    current_round_index: u64,
    reward_stats_for_nth_round_obj: &RewardsState,
) {
    REWARDS_STATS_FOR_NTH_ROUND_STORE
        .save(storage, current_round_index, reward_stats_for_nth_round_obj)
        .unwrap();
}

pub const ADMIN_AMOUNT_AVAIABLE_FOR_WITHDRAW_STORE: Item<Uint128> = Item::new(ADMIN_WITHDRAW_KEY);

pub fn admin_withdraw_helper_read_only(deps_storage: &dyn Storage) -> StdResult<Uint128> {
    let admin_withdraw_obj = ADMIN_AMOUNT_AVAIABLE_FOR_WITHDRAW_STORE
        .load(deps_storage)
        .unwrap_or(Default::default());
    return Ok(admin_withdraw_obj);
}

pub fn admin_withdraw_helper_store(
    deps_storage: &mut dyn Storage,
    admin_withdraw_obj: &Uint128,
) -> StdResult<()> {
    ADMIN_AMOUNT_AVAIABLE_FOR_WITHDRAW_STORE.save(deps_storage, &admin_withdraw_obj)?;
    Ok(())
}

pub const VIEWING_KEY_STORE: Map<&Addr, [u8; 32]> = Map::new(PREFIX_VIEW_KEY);
pub fn write_viewing_key_helper(
    storage: &mut dyn Storage,
    owner: &Addr,
    v_key: &ViewingKey,
) -> StdResult<()> {
    VIEWING_KEY_STORE.save(storage, owner, &v_key.to_hashed())?;
    Ok(())
}

pub fn read_viewing_key(storage: &dyn Storage, owner: &Addr) -> Option<[u8; 32]> {
    let vk = VIEWING_KEY_STORE.load(storage, owner);
    if vk.is_ok() {
        return Some(vk.unwrap());
    } else {
        return None;
    }
}
