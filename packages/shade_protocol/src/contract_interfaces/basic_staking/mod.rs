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
    pub airdrop: Option<Contract>,
    pub unbond_period: Uint128,
    // Number of non-admin pools allowed
    pub max_user_pools: Uint128,
}

#[cw_serde]
pub struct StakingInfo {
    pub stake_token: Addr,
    pub total_staked: Uint128,
    pub unbond_period: Uint128,
    pub reward_pools: Vec<RewardPool>,
}

// For the Snip20 msg field
#[cw_serde]
pub enum Action {
    // Deposit rewards to be distributed
    Stake {
        compound: Option<bool>,
        airdrop_task: Option<bool>,
    },
    Rewards {
        start: Uint128,
        end: Uint128,
    },
}

#[cw_serde]
pub struct Unbonding {
    pub id: Uint128,
    pub amount: Uint128,
    pub complete: Uint128,
}

#[cw_serde]
pub struct Reward {
    pub token: Contract,
    pub amount: Uint128,
}

// Internal storage
#[cw_serde]
pub struct RewardPoolInternal {
    pub id: Uint128,
    pub amount: Uint128,
    pub start: Uint128,
    pub end: Uint128,
    pub token: Contract,
    pub rate: Uint128,
    pub reward_per_token: Uint128,
    pub claimed: Uint128,
    pub last_update: Uint128,
    pub creator: Addr,
    pub official: bool,
}

// Query returned data
#[cw_serde]
pub struct RewardPool {
    pub id: Uint128,
    pub amount: Uint128,
    pub start: Uint128,
    pub end: Uint128,
    pub token: Contract,
    pub rate: Uint128,
    pub official: bool,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: RawContract,
    pub query_auth: RawContract,
    pub airdrop: Option<RawContract>,
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
        admin_auth: Option<RawContract>,
        query_auth: Option<RawContract>,
        airdrop: Option<RawContract>,
        unbond_period: Option<Uint128>,
        max_user_pools: Option<Uint128>,
        padding: Option<String>,
    },
    RegisterRewards {
        token: RawContract,
        padding: Option<String>,
    },
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    Unbond {
        amount: Uint128,
        compound: Option<bool>,
        padding: Option<String>,
    },
    Withdraw {
        ids: Option<Vec<Uint128>>,
        padding: Option<String>,
    },
    Claim {
        padding: Option<String>,
    },
    Compound {
        padding: Option<String>,
    },
    EndRewardPool {
        id: Uint128,
        force: Option<bool>,
        padding: Option<String>,
    },
    AddTransferWhitelist {
        user: String,
        padding: Option<String>,
    },
    RemoveTransferWhitelist {
        user: String,
        padding: Option<String>,
    },
    TransferStake {
        amount: Uint128,
        recipient: String,
        compound: Option<bool>,
        padding: Option<String>,
    },
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
        staked: Uint128,
        status: ResponseStatus,
    },
    // Receive Response
    Rewards {
        status: ResponseStatus,
    },
    Claim {
        //TODO multiple denoms?
        // claimed: Uint128,
        status: ResponseStatus,
    },
    Unbond {
        id: Uint128,
        unbonded: Uint128,
        status: ResponseStatus,
    },
    Withdraw {
        withdrawn: Uint128,
        status: ResponseStatus,
    },
    Compound {
        compounded: Uint128,
        status: ResponseStatus,
    },
    RegisterRewards {
        status: ResponseStatus,
    },
    EndRewardPool {
        deleted: bool,
        extracted: Uint128,
        status: ResponseStatus,
    },
    RemoveTransferWhitelist {
        status: ResponseStatus,
    },
    AddTransferWhitelist {
        status: ResponseStatus,
    },
    TransferStake {
        transferred: Uint128,
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
    StakeToken {},
    StakingInfo {},
    TotalStaked {},
    RewardTokens {},
    // All reward pools in progress
    RewardPools {},

    Balance {
        auth: Auth,
        unbonding_ids: Option<Vec<Uint128>>,
    },
    Staked {
        auth: Auth,
    },
    Rewards {
        auth: Auth,
    },
    Unbonding {
        auth: Auth,
        ids: Option<Vec<Uint128>>,
    },
    TransferWhitelist {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config {
        config: Config,
    },
    StakeToken {
        token: Addr,
    },
    StakingInfo {
        info: StakingInfo,
    },
    TotalStaked {
        amount: Uint128,
    },
    RewardTokens {
        tokens: Vec<Addr>,
    },
    RewardPools {
        rewards: Vec<RewardPool>,
    },
    Balance {
        staked: Uint128,
        rewards: Vec<Reward>,
        unbondings: Vec<Unbonding>,
    },
    Staked {
        amount: Uint128,
    },
    Rewards {
        rewards: Vec<Reward>,
    },
    Unbonding {
        unbondings: Vec<Unbonding>,
    },
    TransferWhitelist {
        whitelist: Vec<Addr>,
    },
}
