use shade_protocol::{
    c_std::{
        entry_point,
        from_binary,
        to_binary,
        Addr,
        Attribute,
        BankMsg,
        Binary,
        Coin,
        CosmosMsg,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Storage,
        Uint128,
        Uint256,
    },
    liquidity_book::staking::{
        EpochInfo,
        StakerInfo,
        StakerLiquidity,
        StakerLiquiditySnapshot,
        TotalLiquidity,
        TotalLiquiditySnapshot,
    },
    query_auth::QueryPermit,
    secret_storage_plus::{Bincode2, Item, ItemStorage, Map},
    snip20::helpers::{register_receive, set_viewing_key_msg, token_info},
    swap::staking::{InstantiateMsg, RewardTokenInfo, RewardTokenSet, State},
    Contract,
    BLOCK_SIZE,
};

pub const STATE: Item<State, Bincode2> = Item::new("state");
pub const REWARD_TOKEN_INFO: Map<&Addr, Vec<RewardTokenInfo>> = Map::new("reward_token_info");
pub const REWARD_TOKENS: Item<RewardTokenSet> = Item::new("reward_tokens");
pub const EPOCH_STORE: Map<u64, EpochInfo, Bincode2> = Map::new("epoch");
pub const STAKERS: Map<&Addr, StakerInfo, Bincode2> = Map::new("stakers");
pub const STAKERS_LIQUIDITY: Map<(&Addr, u32), StakerLiquidity, Bincode2> =
    Map::new("stakers_liquidity");
pub const STAKERS_LIQUIDITY_SNAPSHOT: Map<(&Addr, u64, u32), StakerLiquiditySnapshot, Bincode2> =
    Map::new("stakers_liquidity_snapshot");
pub const TOTAL_LIQUIDITY: Map<u32, TotalLiquidity, Bincode2> = Map::new("total_liquidity");
pub const TOTAL_LIQUIDITY_SNAPSHOT: Map<(u64, u32), TotalLiquiditySnapshot, Bincode2> =
    Map::new("total_liquidity_snapshot");
