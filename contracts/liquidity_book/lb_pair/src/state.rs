use lb_libraries::{
    math::tree_math::TreeUint24,
    oracle_helper::Oracle,
    pair_parameter_helper::PairParameters,
    types::{Bytes32, ContractImplementation},
};
// TODO: sort out viewing key strategy
use shade_protocol::swap::core::ViewingKey;
use shade_protocol::{
    c_std::{Addr, ContractInfo, Timestamp, Uint128, Uint256},
    cosmwasm_schema::cw_serde,
    liquidity_book::lb_pair::{ContractStatus, RewardsDistribution, RewardsDistributionAlgorithm},
    secret_storage_plus::{AppendStore, Bincode2, Item, Map},
    swap::core::TokenType,
    utils::asset::RawContract,
    Contract,
};

pub const STATE: Item<State> = Item::new("state");
pub const CONTRACT_STATUS: Item<ContractStatus> = Item::new("contract_status");
pub const BIN_MAP: Map<u32, Bytes32> = Map::new("bins_map");
pub const BIN_TREE: Item<TreeUint24, Bincode2> = Item::new("bin_tree");
pub const ORACLE: Map<u16, Oracle> = Map::new("oracle");
pub const EPHEMERAL_STORAGE: Item<EphemeralStruct> = Item::new("ephemeral_storage");

pub const FEE_APPEND_STORE: AppendStore<FeeLog> = AppendStore::new("fee_logs");
pub const REWARDS_STATS_STORE: Map<u64, RewardDistributionConfig> = Map::new("rewards_stats");
pub const REWARDS_DISTRIBUTION: Map<u64, RewardsDistribution> = Map::new("rewards_distribution");
pub const FEE_MAP_TREE: Map<u64, TreeUint24, Bincode2> = Map::new("fee_tree");
pub const FEE_MAP: Map<u32, Uint256> = Map::new("fee_map");
pub const STAKING_CONTRACT_IMPL: Item<ContractImplementation> = Item::new("staking_contract_impl");
pub const BIN_RESERVES_UPDATED: Map<u64, Vec<u32>> = Map::new("bins_reserves_updated");
pub const BIN_RESERVES_UPDATED_LOG: AppendStore<u64> =
    AppendStore::new("bins_reserves_updated_log");

// pub const VOLUME_ANALYTICS: Map<u64, Bytes32> = Map::new("volume_analytics");
// pub const FEE_ANALYTICS: Map<u64, Bytes32> = Map::new("fee_analytics");

#[cw_serde]
pub struct RewardDistributionConfig {
    pub cumulative_value: Uint256,
    pub cumulative_value_mul_bin_id: Uint256,
    pub rewards_distribution_algorithm: RewardsDistributionAlgorithm,
}

#[cw_serde]
pub struct FeeLog {
    pub bin_id: u32,
    pub is_token_x: bool,
    pub fee: Uint128,
    pub timestamp: Timestamp,
    pub last_rewards_epoch_id: u64,
}

#[cw_serde]
pub struct State {
    // Contract and creator information
    pub creator: Addr,
    pub factory: ContractInfo,
    pub lb_token: ContractInfo,
    pub lb_staking: ContractInfo,

    // Token and trading pair information
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub pair_parameters: PairParameters,
    pub viewing_key: ViewingKey,

    // Administrative and operational fields
    pub protocol_fees_recipient: Addr,
    pub admin_auth: Contract,
    pub last_swap_timestamp: Timestamp,
    pub rewards_epoch_index: u64,
    pub base_rewards_bins: Option<u32>,
    pub toggle_distributions_algorithm: bool,
    pub max_bins_per_swap: u32,

    // Financial fields
    pub reserves: Bytes32,
    pub protocol_fees: Bytes32,
}

#[cw_serde]
pub struct EphemeralStruct {
    // Contract information
    pub lb_token_code_hash: String,
    pub staking_contract: ContractImplementation,
    pub query_auth: RawContract,

    // Token symbols
    pub token_x_symbol: String,
    pub token_y_symbol: String,

    // Epoch and administrative settings
    pub epoch_index: u64,
    pub epoch_duration: u64,
    pub expiry_duration: Option<u64>,
    pub recover_funds_receiver: Addr,
}
