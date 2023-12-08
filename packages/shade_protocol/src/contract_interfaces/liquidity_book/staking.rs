use cosmwasm_std::Uint256;

use crate::{
    c_std::{Binary, Uint128},
    cosmwasm_schema::cw_serde,
    snip20::Snip20ReceiveMsg,
    swap::{core::TokenType, staking::RewardTokenInfo},
    utils::asset::RawContract,
};

use super::lb_pair::RewardsDistribution;

#[cw_serde]
pub struct InstantiateMsg {
    pub amm_pair: String,
    pub lb_token: RawContract,
    pub admin_auth: RawContract,
    pub query_auth: Option<RawContract>,
    pub first_reward_token: Option<RewardTokenCreate>,
}

#[cw_serde]
pub enum ExecuteMsg {
    ClaimRewards {
        padding: Option<String>,
    },
    Unstake {
        amount: Uint128,
        remove_liquidity: Option<bool>,
        padding: Option<String>,
    },
    Receive(Snip20ReceiveMsg),
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
pub struct RewardTokenCreate {
    pub reward_token: RawContract,
    pub reward_amount_per_epoch: Uint128,
    pub valid_to_epoch: u64,
}

#[cw_serde]
pub struct RewardTokenUpdate {
    pub reward_token: RawContract,
    pub index: u64,
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
