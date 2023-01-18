use crate::{
    c_std::{Addr, Binary, Decimal, Uint128},
    query_auth::{
        helpers::{authenticate_permit, authenticate_vk, PermitAuthentication},
        QueryPermit,
    },
    utils::{
        asset::{Contract, RawContract},
        generic_response::ResponseStatus,
    },
};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Config {
    pub admin_auth: Contract,
    pub query_auth: Contract,
    pub unbond_period: Uint128,
    pub max_user_pools: Uint128,
}

// For the Snip20 msg field
#[cw_serde]
pub enum Action {
    // Deposit rewards to be distributed
    Stake {},
    Rewards { start: Uint128, end: Uint128 },
}

#[cw_serde]
pub struct Unbonding {
    pub amount: Uint128,
    pub complete: Uint128,
}

#[cw_serde]
pub struct RewardPool {
    pub uuid: Uint128,
    pub amount: Uint128,
    pub start: Uint128,
    pub end: Uint128,
    pub token: Contract,
    pub rate: Uint128,
    pub reward_per_token: Uint128,
    pub last_update: Uint128,
    pub creator: Addr,
    pub admin_created: bool,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: RawContract,
    pub query_auth: RawContract,
    pub stake_token: RawContract,
    pub unbond_period: Uint128,
    pub max_user_pools: Uint128,
    pub viewing_key: String,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        config: Config,
    },
    RegisterRewards {
        token: RawContract,
    },
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    Claim {},
    Unbond {
        amount: Uint128,
    },
    Withdraw {},
    Compound {},
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
    Init {
        status: ResponseStatus,
        address: Addr,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    // Receive Response
    Stake {
        status: ResponseStatus,
    },
    // Receive Response
    Rewards {
        status: ResponseStatus,
    },
    Claim {
        status: ResponseStatus,
    },
    Unbond {
        status: ResponseStatus,
    },
    Withdraw {
        status: ResponseStatus,
    },
    Compound {
        status: ResponseStatus,
    },
    RegisterRewards {
        status: ResponseStatus,
    },
}

#[cw_serde]
pub struct AuthPermit {}

#[cw_serde]
pub enum Auth {
    ViewingKey { key: String, address: String },
    Permit(QueryPermit),
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    // TotalShares {},
    TotalStaked {},
    RewardTokens {},
    // All reward pools in progress
    RewardPool {},

    // User permissioned (vk/permit)
    // Single query for all data?
    Balance { auth: Auth },
    Share { auth: Auth },
    Rewards { auth: Auth },
    Unbonding { auth: Auth },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
    TotalStaked { amount: Uint128 },
    RewardTokens { tokens: Vec<Addr> },
    RewardPool { rewards: Vec<RewardPool> },
    Balance { amount: Uint128 },
    Share { share: Uint128 },
    Rewards { amount: Uint128 },
    Unbonding { unbondings: Vec<Unbonding> },
}
