use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128, Binary};
use secret_toolkit::utils::{InitCallback, HandleCallback, Query};
use crate::{
    snip20::Snip20Asset,
    asset::Contract,
    generic_response::ResponseStatus,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: HumanAddr,
    pub oracle: Contract,
    // Both treasury & Commission must be set to function
    pub treasury: Option<Contract>,
    pub activated: bool,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SupportedAsset {
    pub asset: Snip20Asset,
    // Commission percentage * 100 e.g. 5 == .05 == 5%
    pub commission: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub native_asset: Contract,
    pub oracle: Contract,
    //Symbol to peg to, default to snip20 symbol
    pub peg: Option<String>,
    // Both treasury & commission must be set to function
    pub treasury: Option<Contract>,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        owner: Option<HumanAddr>,
        oracle: Option<Contract>,
        treasury: Option<Contract>,
    },
    RegisterAsset {
        contract: Contract,
        // Commission * 100 e.g. 5 == .05 == 5%
        commission: Option<Uint128>,
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init { status: ResponseStatus, address: HumanAddr },
    UpdateConfig { status: ResponseStatus},
    RegisterAsset { status: ResponseStatus},
    Burn { status: ResponseStatus, mint_amount: Uint128 }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetNativeAsset {},
    GetSupportedAssets {},
    GetAsset {
        contract: String,
    },
    GetConfig {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    NativeAsset { asset: Snip20Asset, peg: String },
    SupportedAssets { assets: Vec<String>, },
    Asset { asset: SupportedAsset, burned: Uint128},
    Config { config: Config },
}

