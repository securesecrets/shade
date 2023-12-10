use shade_protocol::{
    c_std::{Addr, ContractInfo},
    lb_libraries::types::TreeUint24,
    liquidity_book::staking::{
        EpochInfo,
        RewardTokenInfo,
        StakerInfo,
        StakerLiquidity,
        StakerLiquiditySnapshot,
        State,
        TotalLiquidity,
        TotalLiquiditySnapshot,
    },
    secret_storage_plus::{Bincode2, Item, Map},
    Contract,
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
