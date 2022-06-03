use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cosmwasm_std::{Querier, StdError, StdResult};
use secret_toolkit::snip20::{token_config_query, token_info_query, TokenConfig, TokenInfo};
use secret_toolkit::utils::Query;
use crate::contract_interfaces::snip20::{QueryAnswer, QueryMsg};
use crate::utils::asset::Contract;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Snip20Asset {
    pub contract: Contract,
    pub token_info: TokenInfo,
    pub token_config: Option<TokenConfig>,
}

pub fn fetch_snip20<Q: Querier>(contract: &Contract, querier: &Q) -> StdResult<Snip20Asset> {
    Ok(Snip20Asset {
        contract: contract.clone(),
        token_info: token_info_query(
            querier,
            1,
            contract.code_hash.clone(),
            contract.address.clone(),
        )?,
        token_config: Some(token_config_query(querier, 256, contract.code_hash.clone(), contract.address.clone())?),
    })
}