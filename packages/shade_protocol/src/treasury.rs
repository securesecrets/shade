use crate::utils::generic_response::ResponseStatus;
use cosmwasm_std::{Binary, Decimal, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::{
    snip20,
    utils::{HandleCallback, InitCallback, Query},
};
use serde::{Deserialize, Serialize};
use crate::utils::asset::Contract;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Asset {
    pub contract: Contract,
    pub token_info: snip20::TokenInfo,
    pub allocations: Option<Vec<Allocation>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Allocation {
    pub contract: Contract,
    pub portion: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub viewing_key: String,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    UpdateConfig {
        owner: Option<HumanAddr>,
    },
    RegisterAsset {
        contract: Contract,
        /* List of contracts/users given an allowance based on a percentage of the asset balance
         * e.g. governance, LP, SKY
         */
        allocations: Option<Vec<Allocation>>,
    },

    // Trigger to re-calc asset allocations
    Rebalance {},
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init {
        status: ResponseStatus,
        address: HumanAddr,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    RegisterAsset {
        status: ResponseStatus,
    },
    Receive {
        status: ResponseStatus,
    },
    Rebalance {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetBalance { contract: HumanAddr },
    CanRebalance {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    Balance { amount: Uint128 },
    CanRebalance { possible: bool },
}
