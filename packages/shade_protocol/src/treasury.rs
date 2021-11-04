use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128, Decimal, Binary};
use crate::{
    asset::Contract,
    snip20::Snip20Asset,
    generic_response::ResponseStatus,
};
use secret_toolkit::{
    snip20, 
    utils::{InitCallback, HandleCallback, Query},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Application {
    pub contract: Contract,
    pub allocation: Decimal,
    pub amount_allocated: Uint128,
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
        admin: Option<HumanAddr>,
    },
    RegisterAsset {
        contract: Contract,
        reserves: Option<Decimal>,
    },
    /* List of contracts/users given an allowance based on a percentage of the asset balance
    * e.g. governance, LP, SKY
    */
    RegisterApp {
        contract: Contract,
        //'staked' asset
        asset: HumanAddr,
        // % of balance allocated to app
        allocation: Decimal,
        // TODO: pool token
        //token: Option<Contract>,
    },

    // Trigger to re-allocate asset (all if none)
    //Rebalance { asset: Option<HumanAddr> },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init { status: ResponseStatus, address: HumanAddr },
    UpdateConfig { status: ResponseStatus },
    Receive { status: ResponseStatus },
    RegisterAsset { status: ResponseStatus },
    RegisterApp { status: ResponseStatus },
    //Rebalance { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Allocations { asset: HumanAddr },
    //Balance { asset: HumanAddr },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    Allocations { allocations: Vec<Application> },
    //Balance { possible: bool },
}
