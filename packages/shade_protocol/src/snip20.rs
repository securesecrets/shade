use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::asset::Contract;
use secret_toolkit::snip20::TokenInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Snip20Asset {
    pub contract: Contract,
    pub token_info: TokenInfo,
    pub burnable: Option<bool>,
}
