use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128, Binary};
use secret_toolkit::utils::{InitCallback, HandleCallback, Query};
use crate::{
    snip20::Snip20Asset,
    asset::Contract,
    generic_response::ResponseStatus,
};
#[cfg(test)]
use secretcli::secretcli::{TestInit, TestHandle, TestQuery};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: HumanAddr,
    pub oracle: Contract,
    // Both treasury & Commission must be set to function
    pub treasury: Option<Contract>,
    pub secondary_burn: Option<HumanAddr>,
    pub activated: bool,
}


/// Used to store the assets allowed to be burned
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SupportedAsset {
    pub asset: Snip20Asset,
    // Commission percentage * 100 e.g. 5 == .05 == 5%
    pub capture: Uint128,
}

// Used to keep track of the cap
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintLimit {
    pub frequency: u64,
    pub mint_capacity: Uint128,
    pub total_minted: Uint128,
    pub next_epoch: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub native_asset: Contract,
    pub oracle: Contract,
    //Symbol to peg to, default to snip20 symbol
    pub peg: Option<String>,
    // Both treasury & capture must be set to function
    pub treasury: Option<Contract>,
    // This is where the non-burnable assets will go, if not defined they will stay in this contract
    pub secondary_burn: Option<HumanAddr>,
    // If left blank no limit will be enforced
    pub start_epoch: Option<Uint128>,
    pub epoch_frequency: Option<Uint128>,
    pub epoch_mint_limit: Option<Uint128>,
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
        owner: Option<HumanAddr>,
        oracle: Option<Contract>,
        treasury: Option<Contract>,
        secondary_burn: Option<HumanAddr>,
    },
    UpdateMintLimit {
        start_epoch: Option<Uint128>,
        epoch_frequency: Option<Uint128>,
        epoch_limit: Option<Uint128>,
    },
    RegisterAsset {
        contract: Contract,
        // Commission * 100 e.g. 5 == .05 == 5%
        capture: Option<Uint128>,
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

#[cfg(test)]
impl TestHandle for HandleMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init { status: ResponseStatus, address: HumanAddr },
    UpdateConfig { status: ResponseStatus },
    UpdateMintLimit { status: ResponseStatus },
    RegisterAsset { status: ResponseStatus },
    RemoveAsset { status: ResponseStatus },
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
    GetMintLimit {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cfg(test)]
impl TestQuery<QueryAnswer> for QueryMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    NativeAsset { asset: Snip20Asset, peg: String },
    SupportedAssets { assets: Vec<String>, },
    Asset { asset: SupportedAsset, burned: Uint128},
    Config { config: Config },
    MintLimit { limit: MintLimit },
}

