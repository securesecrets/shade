use crate::{
    c_std::{
        Addr,
        Binary,
        CosmosMsg,
        Decimal256,
        OverflowError,
        QuerierWrapper,
        StdError,
        StdResult,
        Storage,
        Uint128,
        Uint256,
    },
    cosmwasm_schema::{cw_serde, QueryResponses},
    liquidity_book::{lb_pair::RewardsDistribution, lb_token::Snip1155ReceiveMsg},
    query_auth::QueryPermit,
    secret_storage_plus::{Bincode2, Item, ItemStorage, Map},
    snip20::{ExecuteMsg as Snip20ExecuteMsg, Snip20ReceiveMsg},
    swap::core::{ContractInstantiationInfo, TokenType},
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, Query},
    Contract,
    BLOCK_SIZE,
};
use std::{cmp::min, collections::HashMap};

/*
use crate::swap::core::{
    ClaimRewardResponse, ClaimableRewardsResponse, ConfigResponse, PermitQueryResponse,
    RewardTokenInfo,
};
*/

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub struct StakingContractInstantiateInfo {
    pub staking_contract_info: ContractInstantiationInfo,
    pub custom_label: Option<String>,
    pub first_reward_token: Option<RewardTokenCreate>,
    pub query_auth: Option<RawContract>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub amm_pair: String,
    pub lb_token: RawContract,
    pub admin_auth: RawContract,
    pub query_auth: Option<RawContract>,
    pub epoch_index: u64,
    pub epoch_duration: u64,
    pub expiry_duration: Option<u64>,
    pub first_reward_token: Option<RewardTokenCreate>,
}

#[cw_serde]
pub enum ExecuteMsg {
    ClaimRewards {},
    EndEpoch {
        rewards_distribution: RewardsDistribution,
    },
    Unstake {
        amount: Uint128,
        remove_liquidity: Option<bool>,
        padding: Option<String>,
    },
    Snip1155Receive(Snip1155ReceiveMsg),
    UpdateRewardTokens(Vec<RewardTokenUpdate>),
    CreateRewardTokens(Vec<RewardTokenCreate>),
    UpdateConfig {
        admin_auth: Option<RawContract>,
        query_auth: Option<RawContract>,
        padding: Option<String>,
    },
    RecoverFunds {
        token: TokenType,
        amount: Uint128,
        to: String,
        msg: Option<Binary>,
        padding: Option<String>,
    },
}

#[cw_serde]
pub enum InvokeMsg {
    /// From is used to determine the staker since this can be called by the AMMPair when auto staking.
    Stake {
        from: Option<String>,
        padding: Option<String>,
    },
}

#[cw_serde]
pub struct RewardTokenUpdate {
    pub reward_token: RawContract,
    pub index: u64,
    pub valid_to: u64,
}

#[cw_serde]
pub struct RewardTokenCreate {
    pub reward_token: RawContract,
    pub daily_reward_amount: Uint128,
    pub valid_to: u64,
}

#[allow(clippy::large_enum_variant)]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    GetConfig {},

    #[returns(PermitQueryResponse)]
    WithPermit {
        permit: QueryPermit,
        query: AuthQuery,
    },
}

#[cw_serde]
pub struct QueryPermitData {}

#[cw_serde]
pub enum AuthQuery {
    GetStakerInfo {},
}

#[derive(PartialEq, Debug, Clone)]
pub struct ClaimRewardResponse {
    pub token: Contract,
    pub amount: Uint128,
}

// RESPONSE TYPES

#[cw_serde]
pub struct ConfigResponse {
    pub lp_token: Contract,
    pub amm_pair: Addr,
    pub admin_auth: Contract,
    pub query_auth: Option<Contract>,
    pub total_amount_staked: Uint128,
    pub reward_tokens: Vec<RewardTokenInfo>,
}

#[cw_serde]
pub enum PermitQueryResponse {
    StakerInfo {
        /// Amount normally staked.
        staked: Uint128,
        /// Staked
        total_staked: Uint128,
        claimable_rewards: Vec<ClaimableRewardsResponse>,
    },
}

#[cw_serde]
pub struct ClaimableRewardsResponse {
    pub token: Contract,
    pub amount: Uint128,
}

#[cw_serde]
pub struct RewardTokenInfo {
    pub token: Contract,
    pub decimals: u8,
    pub reward_per_epoch: Uint128,
    pub valid_to: u64,
}

/// Manages the global state of the staking contract.
#[cw_serde]
pub struct State {
    pub lp_token: Contract,
    pub lb_pair: Addr,
    pub admin_auth: Contract,
    pub query_auth: Option<Contract>,
    pub epoch_index: u64,
    pub epoch_durations: u64,
    pub expiry_durations: Option<u64>,
    pub total_amount_staked: Uint128,
}

#[cw_serde]
pub struct RewardTokenSet(pub Vec<Addr>);
impl RewardTokenSet {
    pub fn insert(&mut self, addr: &Addr) {
        if !self.0.contains(addr) {
            self.0.push(addr.clone());
        }
    }

    pub fn get(&self) -> &[Addr] {
        self.0.as_slice()
    }
}

#[cw_serde]
pub struct Staker {
    pub addr: Addr,
    pub staked: Uint128,
    pub claimable_rewards: HashMap<Addr, Vec<ClaimableRewardsInfo>>,
}

#[cw_serde]
pub struct ClaimableRewardsInfo {
    pub info: RewardTokenInfo,
    pub amount: Uint128,
    pub last_reward_per_staked_token_paid: Uint256,
}
