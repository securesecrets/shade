use shade_protocol::{
    basic_staking,
    c_std::{Addr, Uint128},
    utils::asset::Contract,
};

use shade_protocol::secret_storage_plus::{Item, Map};

pub const CONFIG: Item<basic_staking::Config> = Item::new("config");
pub const STAKE_TOKEN: Item<Contract> = Item::new("stake_token");
pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");

pub const TOTAL_STAKED: Item<Uint128> = Item::new("total_stake");

pub const REWARD_TOKENS: Item<Vec<Contract>> = Item::new("reward_tokens");
pub const REWARD_POOLS: Item<Vec<basic_staking::RewardPool>> = Item::new("reward_pools");

// Amount that has been claimed from a pool
pub const REWARD_POOL_CLAIMED: Map<u128, Uint128> = Map::new("reward_pool_claimed");

pub const USER_STAKED: Map<Addr, Uint128> = Map::new("user_stake");
pub const USER_LAST_CLAIM: Map<Addr, Uint128> = Map::new("user_last_claim");

// TODO upgrade ID's to 256
pub fn user_unbonding_key(user: Addr, unbond_id: Uint128) -> String {
    format!("{}-{}", user, unbond_id)
}
pub const USER_UNBONDING_IDS: Map<Addr, Vec<Uint128>> = Map::new("user_unbonding_ids");
pub const USER_UNBONDING: Map<String, basic_staking::Unbonding> = Map::new("user_unbonding");

pub fn user_pool_key(user: Addr, pool_id: Uint128) -> String {
    format!("{}-{}", user, pool_id)
}

pub const USER_REWARD_PER_TOKEN_PAID: Map<String, Uint128> = Map::new("user_reward_per_token_paid");
