use cosmwasm_std::{HumanAddr, Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::utils::{InitCallback, HandleCallback, Query};
use crate::{
    asset::Contract,
    generic_response::ResponseStatus,
    snip20::Snip20Asset,
    governance,
    micro_mint,
    staking,
};

#[cfg(test)]
use secretcli::secretcli::{TestInit, TestHandle, TestQuery};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaskMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    pub airdrop: Contract,
    pub governance: Option<Contract>,
    pub mint: Option<Contract>,
    pub staking: Option<Contract>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub config: Config,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cfg(test)]
impl TestInit for InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        config: Config,
    },
    /*
    Vote {
        tx: governance::HandleMsg::Vote,
    },
    */
    Mint {
        tx: micro_mint::HandleMsg::Receive,
    },
    Stake {
        tx: staking::HandleMsg::Receive,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cfg(test)]
impl TestHandle for HandleMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateConfig { status: ResponseStatus},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cfg(test)]
impl TestQuery<QueryAnswer> for QueryMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
}

