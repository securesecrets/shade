use crate::serde::{Deserialize, Serialize};
use crate::schemars::JsonSchema;
use crate::c_std::{Querier, StdResult};
use secret_toolkit::snip20::{token_config_query, token_info_query, TokenConfig, TokenInfo};
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
        token_config: {
            let config = token_config_query(querier, 256, contract.code_hash.clone(), contract.address.clone());
            match config {
                Err(_) => None,
                Ok(_) => Some(config.unwrap())
            }
        }
    })
}