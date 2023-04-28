use crate::{msg::ResponseStatus, state::RESPONSE_BLOCK_SIZE};
use core::fmt;
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, CustomQuery, QuerierWrapper, QueryRequest, StdError, StdResult,
    Uint128, WasmMsg, WasmQuery,
};
use schemars::JsonSchema;
use secret_toolkit::utils::space_pad;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// HANDLES
#[derive(Serialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // Messages allowed when sending SHD to Staking contract
    // Deposit rewards to be distributed
    Stake { compound: Option<bool> },
}

// Staking contract handles
#[derive(Serialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StakingMsg {
    // Claims rewards generated
    Claim {},
    // Unbond X amount
    Unbond {
        amount: Uint128,
        compound: Option<bool>,
    },
    // Claims mature unbondings
    Withdraw {
        ids: Option<Vec<Uint128>>,
    },
    // Claims available rewards and re-stake them
    Compound {},
    TransferStake {
        amount: Uint128,
        recipient: String,
        compound: Option<bool>,
    },
}

impl StakingMsg {
    /// Returns a StdResult<CosmosMsg> used to execute a contract function
    ///
    /// # Arguments
    ///
    /// * `callback_code_hash` - String holding the code hash of the contract being called
    /// * `contract_addr` - address of the contract being called
    /// * `send_amount` - Optional Uint128 amount of native coin to send with the callback message
    ///                 NOTE: Only a Deposit message should have an amount sent with it
    pub fn to_cosmos_msg(&self, code_hash: String, contract_addr: String) -> StdResult<CosmosMsg> {
        let mut msg = to_binary(self)?;
        space_pad(&mut msg.0, RESPONSE_BLOCK_SIZE);
        let funds = Vec::new();
        let execute = WasmMsg::Execute {
            contract_addr,
            code_hash,
            msg,
            funds,
        };
        Ok(execute.into())
    }
}

pub fn transfer_staked_msg(
    callback_code_hash: String,
    contract_addr: String,
    amount: Uint128,
    recipient: String,
    compound: Option<bool>,
) -> StdResult<CosmosMsg> {
    StakingMsg::TransferStake {
        amount,
        recipient,
        compound,
    }
    .to_cosmos_msg(callback_code_hash, contract_addr)
}
/// Returns a StdResult<CosmosMsg> used to execute Claim
/// Claims all rewards generated on the Staking Contract
/// # Arguments
/// * `callback_code_hash` - String holding the code hash of the contract being called
/// * `contract_addr` - address of the contract being called
pub fn claim_rewards_msg(
    callback_code_hash: String,
    contract_addr: String,
) -> StdResult<CosmosMsg> {
    StakingMsg::Claim {}.to_cosmos_msg(callback_code_hash, contract_addr)
}

/// Returns a StdResult<CosmosMsg> used to execute Unbond
/// Unbonds X amount; This can later be claim with a "withdraw" message
/// # Arguments
/// * `amount` - amount of SHD to unbond
/// * `callback_code_hash` - String holding the code hash of the contract being called
/// * `contract_addr` - address of the contract being called
pub fn unbond_msg(
    amount: Uint128,
    callback_code_hash: String,
    contract_addr: String,
    compound: Option<bool>,
) -> StdResult<CosmosMsg> {
    StakingMsg::Unbond { amount, compound }.to_cosmos_msg(callback_code_hash, contract_addr)
}

/// Returns a StdResult<CosmosMsg> used to execute Withdraw
/// Claims any mature unbondings
/// # Arguments
/// * `callback_code_hash` - String holding the code hash of the contract being called
/// * `contract_addr` - address of the contract being called
pub fn withdraw_msg(
    callback_code_hash: String,
    contract_addr: String,
    ids: Option<Vec<Uint128>>,
) -> StdResult<CosmosMsg> {
    StakingMsg::Withdraw { ids }.to_cosmos_msg(callback_code_hash, contract_addr)
}

/// Returns a StdResult<CosmosMsg> used to execute Compound
/// Claims available rewards and re-stake them
/// # Arguments
/// * `callback_code_hash` - String holding the code hash of the contract being called
/// * `contract_addr` - address of the contract being called
pub fn compound_msg(callback_code_hash: String, contract_addr: String) -> StdResult<CosmosMsg> {
    StakingMsg::Compound {}.to_cosmos_msg(callback_code_hash, contract_addr)
}

// QUERIES

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct RawContract {
    pub address: String,
    pub code_hash: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct StakingConfig {
    pub admin_auth: RawContract,
    pub query_auth: RawContract,
    pub unbond_period: Uint128,
    pub max_user_pools: Uint128,
    pub reward_cancel_threshold: Uint128,
}

#[derive(Serialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ViewingKey {
    key: String,
    address: String,
}
#[derive(Serialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Auth {
    viewing_key: ViewingKey,
    // Removed since contract's don't support permits
    // to communicate between them
    // Permit(QueryPermit),
}

#[derive(Serialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StakingQuery {
    Config {},
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
}

impl fmt::Display for StakingQuery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StakingQuery::Config {} => write!(f, "Config"),
            StakingQuery::Staked { .. } => write!(f, "Staked"),
            StakingQuery::Rewards { .. } => write!(f, "Rewards"),
            StakingQuery::Unbonding { .. } => write!(f, "Unbonding"),
        }
    }
}

impl StakingQuery {
    /// Returns a StdResult<T>, where T is the "Response" type that wraps the query answer
    ///
    /// # Arguments
    ///
    /// * `querier` - a reference to the Querier dependency of the querying contract
    /// * `callback_code_hash` - String holding the code hash of the contract being queried
    /// * `contract_addr` - address of the contract being queried
    pub fn query<C: CustomQuery, T: DeserializeOwned>(
        &self,
        querier: QuerierWrapper<C>,
        code_hash: String,
        contract_addr: String,
    ) -> StdResult<T> {
        let mut msg = to_binary(self)?;
        space_pad(&mut msg.0, RESPONSE_BLOCK_SIZE);
        querier
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                code_hash,
                msg,
            }))
            .map_err(|err| {
                StdError::generic_err(format!("Error performing {} query: {}", self, err))
            })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, PartialEq, Eq)]
pub struct Unbonding {
    pub id: Uint128,
    pub amount: Uint128,
    pub complete: Uint128,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct RewardToken {
    pub address: Addr,
    pub code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Token {
    pub address: Addr,
    pub code_hash: String,
    pub viewing_key: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Reward {
    pub token: RewardToken,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryResponse {
    Config { config: StakingConfig },
    Staked { amount: Uint128 },
    Rewards { rewards: Vec<Reward> },
    Unbonding { unbondings: Vec<Unbonding> },
    ViewingKeyError { msg: String },
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Staked {
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Rewards {
    pub rewards: Vec<Reward>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Unbondings {
    pub unbondings: Vec<Unbonding>,
}

/// Returns a StdResult<Balance> from performing Balance query
///
/// # Arguments
/// * `address` - the address whose balance should be displayed
/// * `key` - String holding the authentication key needed to view the balance
/// * `querier` - a reference to the Querier dependency of the querying contract
/// * `callback_code_hash` - String holding the code hash of the contract being queried
/// * `contract_addr` - address of the contract being queried
pub fn balance_query<C: CustomQuery>(
    address: String,
    key: String,
    querier: QuerierWrapper<C>,
    callback_code_hash: String,
    contract_addr: String,
) -> StdResult<Staked> {
    let auth = Auth {
        viewing_key: ViewingKey { address, key },
    };
    let answer: QueryResponse =
        StakingQuery::Staked { auth }.query(querier, callback_code_hash, contract_addr)?;
    match answer {
        QueryResponse::Staked { amount } => Ok(Staked { amount }),
        QueryResponse::ViewingKeyError { .. } => Err(StdError::generic_err("unauthorized")),
        _ => Err(StdError::generic_err("Invalid Balance query response")),
    }
}

/// Returns a StdResult<Rewards> from performing rewards query
///
/// # Arguments
/// * `address` - the address whose balance should be displayed
/// * `key` - String holding the authentication key needed to view the balance
/// * `querier` - a reference to the Querier dependency of the querying contract
/// * `callback_code_hash` - String holding the code hash of the contract being queried
/// * `contract_addr` - address of the contract being queried
pub fn rewards_query<C: CustomQuery>(
    address: String,
    key: String,
    querier: QuerierWrapper<C>,
    callback_code_hash: String,
    contract_addr: String,
) -> StdResult<Rewards> {
    let auth = Auth {
        viewing_key: ViewingKey { address, key },
    };
    let answer: QueryResponse =
        StakingQuery::Rewards { auth }.query(querier, callback_code_hash, contract_addr)?;
    match answer {
        QueryResponse::Rewards { rewards } => Ok(Rewards { rewards }),
        QueryResponse::ViewingKeyError { .. } => Err(StdError::generic_err("unauthorized")),
        _ => Err(StdError::generic_err("Invalid Rewards query response")),
    }
}

/// Returns a StdResult<Rewards> from performing rewards query
///
/// # Arguments
/// * `address` - the address whose balance should be displayed
/// * `key` - String holding the authentication key needed to view the balance
/// * `querier` - a reference to the Querier dependency of the querying contract
/// * `callback_code_hash` - String holding the code hash of the contract being queried
/// * `contract_addr` - address of the contract being queried
pub fn unbondings_query<C: CustomQuery>(
    address: String,
    key: String,
    ids: Option<Vec<Uint128>>,
    querier: QuerierWrapper<C>,
    callback_code_hash: String,
    contract_addr: String,
) -> StdResult<Unbondings> {
    let auth = Auth {
        viewing_key: ViewingKey { address, key },
    };
    let answer: QueryResponse =
        StakingQuery::Unbonding { auth, ids }.query(querier, callback_code_hash, contract_addr)?;
    match answer {
        QueryResponse::Unbonding { unbondings } => Ok(Unbondings { unbondings }),
        QueryResponse::ViewingKeyError { .. } => Err(StdError::generic_err("unauthorized")),
        _ => Err(StdError::generic_err("Invalid Unbonding query response")),
    }
}

pub fn config_query<C: CustomQuery>(
    querier: QuerierWrapper<C>,
    callback_code_hash: String,
    contract_addr: String,
) -> StdResult<StakingConfig> {
    let answer: QueryResponse =
        StakingQuery::Config {}.query(querier, callback_code_hash, contract_addr)?;
    match answer {
        QueryResponse::Config { config } => Ok(config),
        _ => Err(StdError::generic_err("Invalid Rewards query response")),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Unbond {
    pub id: Uint128,
    pub status: ResponseStatus,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct UnbondResponse {
    pub unbond: Unbond,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Withdraw {
    pub withdrawn: Uint128,
    pub status: ResponseStatus,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct WithdrawResponse {
    pub withdraw: Withdraw,
}
