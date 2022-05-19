use crate::utils::{asset::Contract, generic_response::ResponseStatus};
use cosmwasm_std::{Binary, Decimal, Delegation, HumanAddr, Uint128, Validator};

use crate::contract_interfaces::dao::adapter;
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admins: Vec<HumanAddr>,
    //pub treasury: HumanAddr,
    // This is the contract that will "unbond" funds
    pub owner: HumanAddr,
    pub sscrt: Contract,
    pub validator_bounds: Option<ValidatorBounds>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorBounds {
    pub min_commission: Decimal,
    pub max_commission: Decimal,
    pub top_position: Uint128,
    pub bottom_position: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admins: Option<Vec<HumanAddr>>,
    pub owner: HumanAddr,
    pub sscrt: Contract,
    pub validator_bounds: Option<ValidatorBounds>,
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
        config: Config,
    },
    Adapter(adapter::SubHandleMsg),
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
    Receive {
        status: ResponseStatus,
        validator: Validator,
    },
    /*
    Claim {
        status: ResponseStatus,
    },
    Unbond {
        status: ResponseStatus,
        delegations: Vec<HumanAddr>,
    },
    */
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Delegations {},
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    //Balance { amount: Uint128 },
}
