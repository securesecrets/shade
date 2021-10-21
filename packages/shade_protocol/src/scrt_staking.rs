use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{
    HumanAddr, Binary,
    Uint128, Decimal,
    Validator, FullDelegation,
    Coin,
};
use crate::asset::Contract;
use crate::generic_response::ResponseStatus;
use secret_toolkit::{
    snip20,
    utils::{
        InitCallback,
        HandleCallback,
        Query,
    }
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: HumanAddr,
    pub treasury: HumanAddr,
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
    pub admin: Option<HumanAddr>,
    pub treasury: HumanAddr,
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
    UpdateConfig {
        owner: Option<HumanAddr>,
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    Unbond {
        validator: HumanAddr,
    },
    Collect {
        validator: HumanAddr,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init { status: ResponseStatus, address: HumanAddr },
    UpdateConfig { status: ResponseStatus },
    Receive { 
        status: ResponseStatus,
        validator: Validator,
    },
    //Collect { amount: Uint128 },
    Collect { status: ResponseStatus },
    Unbond { 
        status: ResponseStatus,
        delegation: FullDelegation,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    Delegations { },
    Delegation { validator: HumanAddr },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config},
    Balance { amount: Uint128 },
}
