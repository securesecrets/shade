use crate::snip20::Snip20Asset;
use crate::utils::asset::Contract;
use crate::utils::generic_response::ResponseStatus;
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    pub oracle: Contract,
    // Both treasury & Commission must be set to function
    pub treasury: Option<Contract>,
    pub secondary_burn: Option<HumanAddr>,
    pub activated: bool,
    pub limit: Option<Limit>,
}

/// Used to store the assets allowed to be burned
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SupportedAsset {
    pub asset: Snip20Asset,
    // Commission percentage * 100 e.g. 5 == .05 == 5%
    pub capture: Uint128,
    pub unlimited: bool,
}

// Used to keep track of the cap
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Limit {
    Daily {
        annual_limit: Uint128,
        days: Uint128,
    },
    Monthly {
        annual_limit: Uint128,
        months: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MintMsgHook {
    pub minimum_expected_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub oracle: Contract,

    // Asset that is minted
    pub native_asset: Contract,

    //Symbol to peg to, default to snip20 symbol
    pub peg: Option<String>,

    // Both treasury & asset capture must be set to function properly
    pub treasury: Option<Contract>,

    // This is where the non-burnable assets will go, if not defined they will stay in this contract
    pub secondary_burn: Option<HumanAddr>,

    pub limit: Option<Limit>,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        config: Config,
    },
    RegisterAsset {
        contract: Contract,
        // Commission * 100 e.g. 5 == .05 == 5%
        capture: Option<Uint128>,
        unlimited: Option<bool>
    },
    RemoveAsset {
        address: HumanAddr,
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
    RemoveAsset {
        status: ResponseStatus,
    },
    Mint {
        status: ResponseStatus,
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    NativeAsset {},
    SupportedAssets {},
    Asset { contract: String },
    Config {},
    Limit {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    NativeAsset {
        asset: Snip20Asset,
        peg: String,
    },
    SupportedAssets {
        assets: Vec<String>,
    },
    Asset {
        asset: SupportedAsset,
        burned: Uint128,
    },
    Config {
        config: Config,
    },
    Limit {
        minted: Uint128,
        limit: Uint128,
        last_refresh: String,
    },
}
