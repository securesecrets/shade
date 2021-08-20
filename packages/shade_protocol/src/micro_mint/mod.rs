use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128, Binary};
use crate::{
    snip20::Snip20Asset,
    asset::Contract,
    generic_response::ResponseStatus,
    msg_traits::{
        Init, Handle, Query
    },
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintConfig {
    pub owner: HumanAddr,
    pub oracle: Contract,
    // Both treasury & Commission must be set to function
    pub treasury: Option<Contract>,
    // Commission percentage * 100 e.g. 5 == .05 == 5%
    pub commission: Option<Uint128>,
    pub activated: bool,
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
    // Commission * 100 e.g. 5 == .05 == 5%
    pub commission: Option<Uint128>,
}

impl Init<'_> for InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        owner: Option<HumanAddr>,
        oracle: Option<Contract>,
        treasury: Option<Contract>,
        commission: Option<Uint128>,
    },
    RegisterAsset {
        contract: Contract,
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MintType {
    CoinToSilk {},
    CoinToShade {},
    ConvertToShade {},
    ConvertToSilk {},
}

impl Handle<'_> for HandleMsg{}

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
    GetSupportedAssets {},
    GetAsset {
        contract: String,
    },
    GetConfig {},
}

impl Query for QueryMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    SupportedAssets { assets: Vec<String>, },
    Asset { asset: Snip20Asset, burned: Uint128},
    Config { config: MintConfig },
}

