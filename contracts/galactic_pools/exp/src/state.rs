use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shade_protocol::c_std::{Addr, Uint128};

use shade_protocol::secret_storage_plus::{AppendStore, Item, Map};

/// Basic configuration struct
pub const CONFIG_KEY: &str = "config";

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
/// Basic configuration struct
pub const SUPPLY_POOL: Map<u64, SupplyPool> = Map::new("supply_pool");
/// Revoked permits prefix key
pub const PREFIX_REVOKED_PERMITS: Item<String> = Item::new("revoked");
/// Map of exp amounts per address
pub const EXP_ACCOUNTS: Map<(&Addr, u64), Uint128> = Map::new("exp_accounts");
/// List of verified contracts allowed to interact with manager
pub const VERIFIED_CONTRACTS: Map<&Addr, VerifiedContract> = Map::new("contracts");
/// User XP Append Store
pub const XP_APPEND_STORE: AppendStore<XpSlot> = AppendStore::new("xp_append_store");
/// User XP Nonce
pub const XP_NONCE: Map<u64, Uint128> = Map::new("xp_nonce");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub admins: Vec<Addr>,
    pub contract_address: Addr,
    pub grand_prize_contract: Option<Addr>,
    pub minting_schedule: Schedule,
    pub season_counter: u64,
    pub season_duration: u64,
    pub season_ending_block: u64,
    pub season_starting_block: u64,
    pub total_weight: u64,
    pub verified_contracts: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractStored {
    pub address: Addr,
    pub hash: String,
}

pub fn sort_schedule(s: &mut Schedule) {
    s.sort_by(|s1, s2| s1.end_block.cmp(&s2.end_block))
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ScheduleUnit {
    pub duration: u64,
    pub end_block: u64,
    pub mint_per_block: Uint128,
    pub start_after: Option<u64>,
    pub start_block: u64,
}

pub type Schedule = Vec<ScheduleUnit>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SupplyPool {
    pub season_total_xp_cap: Uint128,
    pub xp_claimed_by_contracts: Uint128,
    pub xp_claimed_by_users: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct VerifiedContract {
    pub available_exp: Uint128,
    pub code_hash: String,
    pub last_claimed: u64,
    pub total_xp: Uint128,
    pub weight: u64,
    pub xp_claimed: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct XpSlot {
    pub ending_slot: Uint128,
    pub starting_slot: Uint128,
    pub user_address: Addr,
}
