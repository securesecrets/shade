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
    pub owner: HumanAddr,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Application {
    pub contract: Contract,
    pub allocation: Decimal,
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
    },
    /* List of contracts/users given an allowance based on a percentage of the asset balance
    * e.g. governance, LP, SKY
    */
    RegisterApp {
        application: Contract,
        //'staked' asset
        asset: HumanAddr,
        // % of balance allocated to app
        allocation: Decimal,
        // TODO: pool token
        //token: Option<Contract>,
    },

    // Trigger to re-calc asset allocations
    //Rebalance { },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init { status: ResponseStatus, address: HumanAddr },
    UpdateConfig { status: ResponseStatus },
    RegisterAsset { status: ResponseStatus },
    RegisterApp { status: ResponseStatus },
    Receive { status: ResponseStatus },
    //Rebalance { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetBalance {
        contract: HumanAddr,
    },
    //CanRebalance { },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    Balance { amount: Uint128 },
    CanRebalance { possible: bool},
}
