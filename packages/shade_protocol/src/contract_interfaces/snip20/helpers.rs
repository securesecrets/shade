use cosmwasm_std::{Coin, SubMsg};
use cosmwasm_schema::{cw_serde};
use crate::c_std::{StdError, StdResult, Addr, Uint128, Binary, CosmosMsg, QuerierWrapper};
use crate::utils::{HandleCallback, Query};
use super::{QueryAnswer, QueryMsg, ExecuteMsg};
use crate::utils::asset::Contract;

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
///
/// # Arguments
///
/// * `recipient` - the address tokens are to be sent to
/// * `amount` - Uint128 amount of tokens to send
/// * `msg` - Optional base64 encoded string to pass to the recipient contract's
///           Receive function
/// * `memo` - A message to include in transaction
/// * `padding` - Optional String used as padding if you don't want to use block padding
/// * `block_size` - pad the message to blocks of this size
/// * `callback_code_hash` - String holding the code hash of the contract being called
/// * `contract_addr` - address of the contract being called
#[allow(clippy::too_many_arguments)]
pub fn send_msg(
    recipient: Addr,
    amount: Uint128,
    msg: Option<Binary>,
    memo: Option<String>,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<SubMsg> {
    Ok(SubMsg::new(ExecuteMsg::Send {
        recipient,
        recipient_code_hash: None,
        amount,
        msg,
        memo,
        padding
    }.to_cosmos_msg(
        contract,
        vec![]
    )?))
}

/// Returns a StdResult<CosmosMsg> used to execute Redeem
///
/// # Arguments
///
/// * `amount` - Uint128 amount of token to redeem for SCRT
/// * `denom` - Optional String to hold the denomination of tokens to redeem
/// * `padding` - Optional String used as padding if you don't want to use block padding
/// * `block_size` - pad the message to blocks of this size
/// * `callback_code_hash` - String holding the code hash of the contract being called
/// * `contract_addr` - address of the contract being called
pub fn redeem_msg(
    amount: Uint128,
    denom: Option<String>,
    padding: Option<String>,
    contract: &Contract
) -> StdResult<SubMsg> {
    Ok(SubMsg::new(ExecuteMsg::Redeem {
        amount,
        denom,
        padding,
    }.to_cosmos_msg(contract, vec![])?))
}

/// Returns a StdResult<CosmosMsg> used to execute Deposit
///
/// # Arguments
///
/// * `amount` - Uint128 amount of uSCRT to convert to the SNIP20 token
/// * `padding` - Optional String used as padding if you don't want to use block padding
/// * `block_size` - pad the message to blocks of this size
/// * `callback_code_hash` - String holding the code hash of the contract being called
/// * `contract_addr` - address of the contract being called
pub fn deposit_msg(
    amount: Uint128,
    padding: Option<String>,
    contract: &Contract
) -> StdResult<SubMsg> {
    Ok(SubMsg::new(ExecuteMsg::Deposit { padding }.to_cosmos_msg(
        contract,
        vec![Coin {
            denom: "uscrt".to_string(),
            amount
        }],
    )?))
}

/// Returns a StdResult<CosmosMsg> used to execute RegisterReceive
///
/// # Arguments
///
/// * `your_contracts_code_hash` - String holding the code hash of your contract
/// * `padding` - Optional String used as padding if you don't want to use block padding
/// * `block_size` - pad the message to blocks of this size
/// * `callback_code_hash` - String holding the code hash of the contract being called
/// * `contract_addr` - address of the contract being called
pub fn register_receive(
    register_hash: String,
    padding: Option<String>,
    contract: &Contract
) -> StdResult<SubMsg> {
    Ok(SubMsg::new(ExecuteMsg::RegisterReceive {
        code_hash: register_hash,
        padding,
    }
        .to_cosmos_msg(contract, vec![])?))
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
///
/// # Arguments
///
/// * `querier` - a reference to the Querier dependency of the querying contract
/// * `block_size` - pad the message to blocks of this size
/// * `callback_code_hash` - String holding the code hash of the contract being queried
/// * `contract_addr` - address of the contract being queried
pub fn token_info(
    querier: &QuerierWrapper,
    contract: &Contract,
) -> StdResult<TokenInfo> {
    let answer: QueryAnswer =
        QueryMsg::TokenInfo {}.query(querier, contract)?;

    match answer {
        QueryAnswer::TokenInfo { name, symbol, decimals, total_supply } => Ok(TokenInfo {
            name,
            symbol,
            decimals,
            total_supply
        }),
        _ => Err(StdError::generic_err("Wrong answer")) //TODO: better error
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
    pub transfer_enabled: Option<bool>
}
/// Returns a StdResult<TokenConfig> from performing TokenConfig query
///
/// # Arguments
///
/// * `querier` - a reference to the Querier dependency of the querying contract
/// * `block_size` - pad the message to blocks of this size
/// * `callback_code_hash` - String holding the code hash of the contract being queried
/// * `contract_addr` - address of the contract being queried
pub fn token_config(
    querier: &QuerierWrapper,
    contract: &Contract,
) -> StdResult<TokenConfig> {
    let answer: QueryAnswer =
        QueryMsg::TokenConfig {}.query(querier, contract)?;

    match answer {
        QueryAnswer::TokenConfig { public_total_supply, deposit_enabled, redeem_enabled, mint_enabled, burn_enabled, .. } => Ok(TokenConfig {
            public_total_supply,
            deposit_enabled,
            redeem_enabled,
            mint_enabled,
            burn_enabled,
            transfer_enabled: None
        }),
        _ => Err(StdError::generic_err("Wrong answer")) //TODO: better error
    }
}