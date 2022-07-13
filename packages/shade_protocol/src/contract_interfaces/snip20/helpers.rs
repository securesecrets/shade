
use crate::serde::{Deserialize, Serialize};
use crate::c_std::{Querier, StdError, StdResult, Addr, Uint128, Binary, CosmosMsg, QuerierWrapper};
use crate::utils::{HandleCallback, Query};
use super::{QueryAnswer, QueryMsg, HandleMsg};
use crate::utils::asset::Contract;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Snip20Asset {
    pub contract: Contract,
    pub token_info: TokenInfo,
    pub token_config: Option<TokenConfig>,
}

pub fn fetch_snip20<Q: Querier>(contract: &Contract, querier: &Q) -> StdResult<Snip20Asset> {
    Ok(Snip20Asset {
        contract: contract.clone(),
        token_info: token_info(
            querier,
            1,
            contract.clone(),
        )?,
        token_config: Some(token_config_query(querier, 256, contract.code_hash.clone(), contract.address.clone())?),
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
    block_size: usize,
    contract: Contract,
) -> StdResult<CosmosMsg> {
    HandleMsg::Send {
        recipient,
        recipient_code_hash: None,
        amount,
        msg,
        memo,
        padding
    }.to_cosmos_msg(
        contract.address.into(),
        contract.code_hash,
        vec![]
    )
}


/// TokenInfo response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    block_size: usize,
    contract: Contract,
) -> StdResult<TokenInfo> {
    let answer: QueryAnswer =
        QueryMsg::TokenInfo {}.query(querier, contract.address.into(), contract.code_hash, block_size)?;

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