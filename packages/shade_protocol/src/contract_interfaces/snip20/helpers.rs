use super::{batch, manager::AllowanceResponse, ExecuteMsg, QueryAnswer, QueryMsg};
use crate::{
    c_std::{Addr, Binary, CosmosMsg, QuerierWrapper, StdError, StdResult, Uint128},
    utils::{asset::Contract, ExecuteCallback, Query},
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

#[cw_serde]
pub struct Snip20Asset {
    pub contract: Contract,
    pub token_info: TokenInfo,
    pub token_config: Option<TokenConfig>,
}

pub fn fetch_snip20(contract: &Contract, querier: &QuerierWrapper) -> StdResult<Snip20Asset> {
    Ok(Snip20Asset {
        contract: contract.clone(),
        token_info: token_info(querier, contract)?,
        token_config: Some(token_config(querier, contract)?),
    })
}

/// Returns a StdResult<CosmosMsg> used to execute Send
#[allow(clippy::too_many_arguments)]
pub fn send_msg(
    recipient: Addr,
    amount: Uint128,
    msg: Option<Binary>,
    memo: Option<String>,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    Ok(ExecuteMsg::Send {
        recipient: recipient.to_string(),
        recipient_code_hash: None,
        amount,
        msg,
        memo,
        padding,
    }
    .to_cosmos_msg(contract, vec![])?)
}

/// Returns a StdResult<CosmosMsg> used to execute Redeem
pub fn redeem_msg(
    amount: Uint128,
    denom: Option<String>,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::Redeem {
        amount,
        denom,
        padding,
    }
    .to_cosmos_msg(contract, vec![])
}

/// Returns a StdResult<CosmosMsg> used to execute Deposit
pub fn deposit_msg(
    amount: Uint128,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::Deposit { padding }.to_cosmos_msg(contract, vec![Coin {
        denom: "uscrt".to_string(),
        amount,
    }])
}

/// Returns a StdResult<CosmosMsg> used to execute Mint
pub fn mint_msg(
    recipient: Addr,
    amount: Uint128,
    memo: Option<String>,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::Mint {
        recipient: recipient.to_string(),
        amount,
        memo,
        padding,
    }
    .to_cosmos_msg(contract, vec![])
}

/// Returns a StdResult<CosmosMsg> used to execute Burn
pub fn burn_msg(
    amount: Uint128,
    memo: Option<String>,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::Burn {
        amount,
        memo,
        padding,
    }
    .to_cosmos_msg(contract, vec![])
}

/// Returns a StdResult<CosmosMsg> used to execute RegisterReceive
pub fn register_receive(
    register_hash: String,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::RegisterReceive {
        code_hash: register_hash,
        padding,
    }
    .to_cosmos_msg(contract, vec![])
}

pub fn set_viewing_key_msg(
    viewing_key: String,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::SetViewingKey {
        key: viewing_key,
        padding,
    }
    .to_cosmos_msg(contract, vec![])
}

pub fn batch_send_msg(
    actions: Vec<batch::SendAction>,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::BatchSend { actions, padding }.to_cosmos_msg(contract, vec![])
}

pub fn batch_send_from_msg(
    actions: Vec<batch::SendFromAction>,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::BatchSendFrom { actions, padding }.to_cosmos_msg(contract, vec![])
}

/// TokenInfo response
#[cw_serde]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_supply: Option<Uint128>,
}
/// Returns a StdResult<TokenInfo> from performing TokenInfo query
pub fn token_info(querier: &QuerierWrapper, contract: &Contract) -> StdResult<TokenInfo> {
    let answer: QueryAnswer = QueryMsg::TokenInfo {}.query(querier, contract)?;

    match answer {
        QueryAnswer::TokenInfo {
            name,
            symbol,
            decimals,
            total_supply,
        } => Ok(TokenInfo {
            name,
            symbol,
            decimals,
            total_supply,
        }),
        _ => Err(StdError::generic_err("Wrong answer")), //TODO: better error
    }
}

/// Returns a StdResult<Uint128> from performing a Balance query
pub fn balance_query(
    querier: &QuerierWrapper,
    address: Addr,
    key: String,
    contract: &Contract,
) -> StdResult<Uint128> {
    let answer: QueryAnswer = QueryMsg::Balance {
        address: address.to_string(),
        key,
    }
    .query(querier, contract)?;

    match answer {
        QueryAnswer::Balance { amount, .. } => Ok(amount),
        _ => Err(StdError::generic_err("Invalid Balance Response")), //TODO: better error
    }
}

/// TokenConfig response
#[cw_serde]
pub struct TokenConfig {
    pub public_total_supply: bool,
    pub deposit_enabled: bool,
    pub redeem_enabled: bool,
    pub mint_enabled: bool,
    pub burn_enabled: bool,
    // Optionals only relevant to some snip20a
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer_enabled: Option<bool>,
}
/// Returns a StdResult<TokenConfig> from performing TokenConfig query
pub fn token_config(querier: &QuerierWrapper, contract: &Contract) -> StdResult<TokenConfig> {
    let answer: QueryAnswer = QueryMsg::TokenConfig {}.query(querier, contract)?;

    match answer {
        QueryAnswer::TokenConfig {
            public_total_supply,
            deposit_enabled,
            redeem_enabled,
            mint_enabled,
            burn_enabled,
            ..
        } => Ok(TokenConfig {
            public_total_supply,
            deposit_enabled,
            redeem_enabled,
            mint_enabled,
            burn_enabled,
            transfer_enabled: None,
        }),
        _ => Err(StdError::generic_err("Wrong answer")), //TODO: better error
    }
}

/// Returns a StdResult<CosmosMsg> used to execute IncreaseAllowance
///
/// # Arguments
///
/// * `spender` - the address of the allowed spender
/// * `amount` - Uint128 additional amount the spender is allowed to send/burn
/// * `expiration` - Optional u64 denoting the epoch time in seconds that the allowance will expire
/// * `padding` - Optional String used as padding if you don't want to use block padding
/// * `block_size` - pad the message to blocks of this size
/// * `callback_code_hash` - String holding the code hash of the contract being called
/// * `contract_addr` - address of the contract being called
pub fn increase_allowance_msg(
    spender: Addr,
    amount: Uint128,
    expiration: Option<u64>,
    padding: Option<String>,
    block_size: usize,
    contract: &Contract,
    funds: Vec<Coin>,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::IncreaseAllowance {
        spender: spender.to_string(),
        amount,
        expiration,
        padding,
    }
    .to_cosmos_msg(contract, funds)
}

/// Returns a StdResult<CosmosMsg> used to execute DecreaseAllowance
///
/// # Arguments
///
/// * `spender` - the address of the allowed spender
/// * `amount` - Uint128 amount the spender is no longer allowed to send/burn
/// * `expiration` - Optional u64 denoting the epoch time in seconds that the allowance will expire
/// * `padding` - Optional String used as padding if you don't want to use block padding
/// * `block_size` - pad the message to blocks of this size
/// * `callback_code_hash` - String holding the code hash of the contract being called
/// * `contract_addr` - address of the contract being called
pub fn decrease_allowance_msg(
    spender: Addr,
    amount: Uint128,
    expiration: Option<u64>,
    padding: Option<String>,
    block_size: usize,
    contract: &Contract,
    funds: Vec<Coin>,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::DecreaseAllowance {
        spender: spender.to_string(),
        amount,
        expiration,
        padding,
    }
    .to_cosmos_msg(contract, funds)
}

/// Returns a StdResult<Allowance> from performing Allowance query
///
/// # Arguments
///
/// * `querier` - a reference to the Querier dependency of the querying contract
/// * `owner` - the address that owns the tokens
/// * `spender` - the address allowed to send/burn tokens
/// * `key` - String holding the authentication key needed to view the allowance
/// * `block_size` - pad the message to blocks of this size
/// * `callback_code_hash` - String holding the code hash of the contract being queried
/// * `contract_addr` - address of the contract being queried
#[allow(clippy::too_many_arguments)]
pub fn allowance_query(
    querier: &QuerierWrapper,
    owner: Addr,
    spender: Addr,
    key: String,
    block_size: usize,
    contract: &Contract,
) -> StdResult<AllowanceResponse> {
    let answer: QueryAnswer = QueryMsg::Allowance {
        owner: owner.to_string(),
        spender: spender.to_string(),
        key,
    }
    .query(querier, contract)?;
    match answer {
        QueryAnswer::Allowance {
            spender,
            owner,
            allowance,
            expiration,
        } => Ok(AllowanceResponse {
            spender,
            owner,
            allowance,
            expiration,
        }),
        QueryAnswer::ViewingKeyError { .. } => Err(StdError::generic_err("Unauthorized")),
        _ => Err(StdError::generic_err("Invalid Allowance query response")),
    }
}

pub fn transfer_from_msg(
    owner: String,
    recipient: String,
    amount: Uint128,
    memo: Option<String>,
    padding: Option<String>,
    contract: &Contract
) -> StdResult<CosmosMsg> {
    ExecuteMsg::TransferFrom { 
        owner,
        recipient,
        amount,
        memo,
        padding,
    }.to_cosmos_msg(contract, vec![])
}