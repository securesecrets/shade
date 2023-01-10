use shade_protocol::{
    c_std::{Addr, Storage, Uint128},
    staking,
};

use shade_protocol::secret_storage_plus::{Item, Map};

pub const CONFIG: Item<staking::Config> = Item::new("config");
pub const STAKE_TOKEN: Item<Contract> = Item::new("stake_token");
pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");

pub const TOTAL_STAKED: Item<Uint128> = Item::new("total_stake");

pub const REWARD_TOKENS: Item<Vec<Contract>> = Item::new("reward_tokens");
pub const REWARD_POOLS: Item<Vec<staking::RewardPool>> = Item::new("reward_pools");

pub const REWARD_PER_TOKEN: Map<Uint128, Uint128> = Map::new("reward_per_token");

pub const USER_STAKED: Map<Addr, Uint128> = Map::new("user_stake");
pub const USER_LAST_CLAIM: Map<Addr, Uint128> = Map::new("user_last_claim");
pub const USER_UNBONDING: Map<Addr, Vec<staking::Unbonding>> = Map::new("user_unbonding");
pub const USER_REWARD_PER_TOKEN: Map<Addr, Uint128> = Map::new("user_reward_per_token");
