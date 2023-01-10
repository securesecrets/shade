use crate::{
    c_std::{Addr, Binary, Decimal, Delegation, Uint128, Validator},
    utils::{asset::Contract, generic_response::ResponseStatus},
};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Config {
    pub admin_auth: Addr,
    pub unbond_period: Uint128,
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
    amount: Uint128,
    complete: String,
}

#[cw_serde]
pub struct RewardPool {
    uuid: Uint128,
    amount: Uint128,
    start: Uint128,
    end: Uint128,
    token: Contract,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: Addr,
    pub stake_token: RawContract,
    pub unbond_period: Uint128,
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
        delegations: Vec<Addr>,
    },
    Withdraw {
        status: ResponseStatus,
    },
    RegisterRewards {
        status: ResponseStatus,
    },
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
    Balance {},
    Share {},
    Rewards {},
    Unbonding {},
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
