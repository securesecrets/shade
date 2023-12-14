use std::collections::HashMap;

use cosmwasm_schema::QueryResponses;
use secret_toolkit::permit::Permit;

use crate::{
    c_std::{
        to_binary,
        Addr,
        Api,
        Binary,
        BlockInfo,
        Coin,
        ContractInfo,
        CosmosMsg,
        StdResult,
        Uint128,
        Uint256,
        WasmMsg,
    },
    cosmwasm_schema::cw_serde,
    lb_libraries::types::ContractInstantiationInfo,
    snip20::Snip20ReceiveMsg,
    swap::core::TokenType,
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, Query},
    BLOCK_SIZE,
};

use super::{
    lb_pair::RewardsDistribution,
    lb_token::{space_pad, Snip1155ReceiveMsg},
};

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
    RegisterRewardTokens(Vec<ContractInfo>),
    UpdateConfig {
        admin_auth: Option<RawContract>,
        query_auth: Option<RawContract>,
        epoch_duration: Option<u64>,
        expiry_duration: Option<u64>,
    },
    RecoverFunds {
        token: TokenType,
        amount: Uint128,
        to: String,
        msg: Option<Binary>,
    },
    CreateViewingKey {
        entropy: String,
    },
    SetViewingKey {
        key: String,
    },
    /// disallow the use of a query permit
    RevokePermit {
        permit_name: String,
    },
}

#[cw_serde]
pub enum ExecuteAnswer {
    CreateViewingKey { key: String },
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

// #[allow(clippy::large_enum_variant)]
// #[cw_serde]
// #[derive(QueryResponses)]
// pub enum QueryMsg {
//     #[returns(ConfigResponse)]
//     GetConfig {},

//     #[returns(PermitQueryResponse)]
//     WithPermit {
//         permit: QueryPermit,
//         query: AuthQuery,
//     },
// }

// #[cw_serde]
// pub struct QueryPermitData {}

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
    pub lb_token: ContractInfo,
    pub lb_pair: Addr,
    pub admin_auth: ContractInfo,
    pub query_auth: Option<ContractInfo>,
    pub epoch_index: u64,
    pub epoch_durations: u64,
    pub expiry_durations: Option<u64>,
    pub tx_id: u64,
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

/////////////////////////////////////////////////////////////////////////////////
// Query messages
/////////////////////////////////////////////////////////////////////////////////

/// Query messages to SNIP1155 contract. See [QueryAnswer](crate::msg::QueryAnswer)
/// for the response messages for each variant, which has more detail.
#[cw_serde]
pub enum QueryMsg {
    /// returns public information of the SNIP1155 contract
    ContractInfo {},
    RegisteredTokens {},
    IdTotalBalance {
        id: String,
    },
    Balance {
        owner: Addr,
        key: String,
        token_id: String,
    },
    AllBalances {
        owner: Addr,
        key: String,
        page: Option<u32>,
        page_size: Option<u32>,
    },
    Liquidity {
        owner: Addr,
        key: String,
        round_index: Option<u64>,
        token_ids: Vec<u32>,
    },
    TransactionHistory {
        owner: Addr,
        key: String,
        page: Option<u32>,
        page_size: Option<u32>,
        txn_type: QueryTxnType,
    },
    WithPermit {
        permit: Permit,
        query: QueryWithPermit,
    },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> StdResult<(Vec<&Addr>, String)> {
        match self {
            Self::Balance { owner, key, .. } => Ok((vec![owner], key.clone())),
            Self::AllBalances { owner, key, .. } => Ok((vec![owner], key.clone())),
            Self::Liquidity { owner, key, .. } => Ok((vec![owner], key.clone())),
            Self::TransactionHistory { owner, key, .. } => Ok((vec![owner], key.clone())),
            Self::ContractInfo {}
            | Self::IdTotalBalance { .. }
            | Self::RegisteredTokens { .. }
            | Self::WithPermit { .. } => {
                unreachable!("This query type does not require viewing key authentication")
            }
        }
    }
}

#[cw_serde]
pub enum QueryWithPermit {
    Balance {
        owner: Addr,
        token_id: String,
    },
    AllBalances {
        page: Option<u32>,
        page_size: Option<u32>,
    },
    TransactionHistory {
        page: Option<u32>,
        page_size: u32,
    },
}

/// the query responses for each [QueryMsg](crate::msg::QueryMsg) variant
#[cw_serde]
pub enum QueryAnswer {
    /// returns contract-level information:
    ContractInfo {
        lb_token: ContractInfo,
        lb_pair: Addr,
        admin_auth: ContractInfo,
        query_auth: Option<ContractInfo>,
        epoch_index: u64,
        epoch_durations: u64,
        expiry_durations: Option<u64>,
    },
    RegisteredTokens(Vec<ContractInfo>),
    IdTotalBalance {
        amount: Uint256,
    },
    /// returns balance of a specific token_id. Owners can give permission to other addresses to query their balance
    Balance {
        amount: Uint256,
    },
    /// returns all token_id balances owned by an address. Only owners can use this query
    AllBalances(Vec<OwnerBalance>),

    Liquidity(Vec<Liquidity>),

    TokenIdBalance {
        total_supply: Option<Uint256>,
    },

    TransactionHistory {
        txns: Vec<Tx>,
        count: u64,
    },

    /// returned when an viewing_key-specific errors occur during a user's attempt to
    /// perform an authenticated query
    ViewingKeyError {
        msg: String,
    },
}

#[cw_serde]
pub struct Liquidity {
    pub token_id: String,
    pub user_liquidity: Uint256,
    pub total_liquidity: Uint256,
}

/////////////////////////////////////////////////////////////////////////////////
// Structs, Enums and other functions
/////////////////////////////////////////////////////////////////////////////////

#[cw_serde]
pub enum ResponseStatus {
    Success,
    Failure,
}

#[cw_serde]
pub struct Permission {
    pub view_balance_perm: bool,
}

/// to store all keys to access all permissions for a given `owner`
#[cw_serde]
pub struct PermissionKey {
    pub token_id: String,
    pub allowed_addr: Addr,
}

#[cw_serde]
pub struct OwnerBalance {
    pub token_id: String,
    pub amount: Uint256,
}

impl ExecuteMsg {
    pub fn to_cosmos_msg(
        &self,
        code_hash: String,
        contract_addr: String,
        send_amount: Option<Uint128>,
    ) -> StdResult<CosmosMsg> {
        let mut msg = to_binary(self)?;
        space_pad(256, &mut msg.0);
        let mut funds = Vec::new();
        if let Some(amount) = send_amount {
            funds.push(Coin {
                amount,
                denom: String::from("uscrt"),
            });
        }
        let execute = WasmMsg::Execute {
            contract_addr,
            code_hash,
            msg,
            funds,
        };
        Ok(execute.into())
    }
}

//Txn history

/// tx type and specifics for storage
#[cw_serde]
pub enum TxAction {
    Stake {
        ids: Vec<u32>,
        amounts: Vec<Uint256>,
    },
    UnStake {
        ids: Vec<u32>,
        amounts: Vec<Uint256>,
    },
    ClaimRewards {
        ids: Vec<u32>,
        rewards: Vec<Reward>,
    },
}

#[cw_serde]
pub enum QueryTxnType {
    All,
    Stake,
    UnStake,
    ClaimRewards,
}

/// tx in storage
#[cw_serde]
pub struct Tx {
    /// tx id
    pub tx_id: u64,
    /// the block containing this tx
    pub block_height: u64,
    /// the time (in seconds since 01/01/1970) of the block containing this tx
    pub block_time: u64,
    /// tx type and specifics
    pub action: TxAction,
}

#[cw_serde]
pub struct Reward {
    /// tx id
    pub token: ContractInfo,
    /// the block containing this tx
    pub amounts: Vec<Uint128>,
    pub total_amount: Uint128,
}
