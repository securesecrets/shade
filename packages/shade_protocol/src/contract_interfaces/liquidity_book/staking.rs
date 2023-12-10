use std::collections::HashMap;

use cosmwasm_schema::QueryResponses;

use crate::{
    c_std::{Addr, Binary, ContractInfo, Uint128, Uint256},
    cosmwasm_schema::cw_serde,
    lb_libraries::types::ContractInstantiationInfo,
    query_auth::QueryPermit,
    snip20::Snip20ReceiveMsg,
    swap::core::TokenType,
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, Query},
    BLOCK_SIZE,
};

use super::{lb_pair::RewardsDistribution, lb_token::Snip1155ReceiveMsg};

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
        token_ids: Vec<u32>,
        amounts: Vec<Uint256>,
    },
    Snip1155Receive(Snip1155ReceiveMsg),
    Receive(Snip20ReceiveMsg),
    UpdateRewardTokens(Vec<RewardTokenUpdate>),
    RegisterRewardTokens(Vec<ContractInfo>),
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
    AddRewards {
        start: Option<u64>,
        end: u64,
    },
}

#[cw_serde]
pub struct RewardTokenUpdate {
    pub reward_token: RawContract,
    pub index: u64,
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
    pub token: ContractInfo,
    pub amount: Uint128,
}

// RESPONSE TYPES

#[cw_serde]
pub struct ConfigResponse {
    pub lp_token: ContractInfo,
    pub amm_pair: Addr,
    pub admin_auth: ContractInfo,
    pub query_auth: Option<ContractInfo>,
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
    pub token: ContractInfo,
    pub amount: Uint128,
}

#[cw_serde]
pub struct RewardTokenInfo {
    pub token: ContractInfo,
    pub decimals: u8,
    pub reward_per_epoch: Uint128,
    pub start: u64,
    pub end: u64,
}

/// Manages the global state of the staking contract.
#[cw_serde]
pub struct State {
    pub lp_token: ContractInfo,
    pub lb_pair: Addr,
    pub admin_auth: ContractInfo,
    pub query_auth: Option<ContractInfo>,
    pub epoch_index: u64,
    pub epoch_durations: u64,
    pub expiry_durations: Option<u64>,
    pub total_amount_staked: Uint128,
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

#[cw_serde]
pub struct RewardTokenCreate {
    pub reward_token: RawContract,
    pub daily_reward_amount: Uint128,
    pub valid_to: u64,
}

#[cw_serde]
pub struct EpochInfo {
    pub rewards_distribution: Option<RewardsDistribution>,
    pub reward_tokens: Option<Vec<RewardTokenInfo>>,
    pub start_time: u64,
    pub end_time: u64,
    pub duration: u64,
    pub expired_at: Option<u64>,
}

#[cw_serde]
pub struct StakerInfo {
    pub starting_round: Option<u64>,
    pub total_rewards_earned: Uint128,
    pub last_claim_rewards_round: Option<u64>,
}

#[cw_serde]
pub struct StakerLiquidity {
    pub amount_delegated: Uint256,
    pub amount_withdrawable: Uint256,
    pub amount_unbonding: Uint256,
    pub unbondings: Option<Vec<Unbonding>>,
}

impl Default for StakerLiquidity {
    fn default() -> Self {
        StakerLiquidity {
            amount_delegated: Uint256::zero(),
            amount_withdrawable: Uint256::zero(),
            amount_unbonding: Uint256::zero(),
            unbondings: None,
        }
    }
}
/// State of user liquidity information
#[cw_serde]
pub struct StakerLiquiditySnapshot {
    pub amount_delegated: Uint256,
    pub liquidity: Uint256,
}
impl Default for StakerLiquiditySnapshot {
    fn default() -> Self {
        StakerLiquiditySnapshot {
            amount_delegated: Uint256::zero(),
            liquidity: Uint256::zero(),
        }
    }
}

#[cw_serde]
pub struct Unbonding {
    pub time: u64,
    pub amount: Uint256,
}

#[cw_serde]
pub struct TotalLiquidity {
    pub amount_delegated: Uint256,
    pub last_deposited: Option<u64>,
}

impl Default for TotalLiquidity {
    fn default() -> Self {
        TotalLiquidity {
            amount_delegated: Uint256::zero(),
            last_deposited: None,
        }
    }
}

#[cw_serde]
pub struct TotalLiquiditySnapshot {
    pub amount_delegated: Uint256,
    pub liquidity: Uint256,
}
impl Default for TotalLiquiditySnapshot {
    fn default() -> Self {
        TotalLiquiditySnapshot {
            amount_delegated: Uint256::zero(),
            liquidity: Uint256::zero(),
        }
    }
}
