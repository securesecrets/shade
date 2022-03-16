pub mod errors;

use crate::utils::generic_response::ResponseStatus;
use crate::utils::asset::Contract;
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::snip20::Snip20Asset;
use secret_toolkit::utils::{HandleCallback};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    pub oracle: Contract,
    pub treasury: HumanAddr,
    pub activated: bool,
    pub issuance_cap: Uint128,
    pub start_date: Option<u64>,
    pub end_date: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub oracle: Contract,
    pub treasury: HumanAddr,
    pub issuance_cap: Uint128,
    pub minted_asset: Contract,
    pub start_date: Option<u64>,
    pub end_date: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        config: Config,
    },
    RegisterAsset {
        contract: Contract,
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        msg: Option<Binary>,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateConfig {
        status: ResponseStatus,
    },
    Deposit {
        status: ResponseStatus,
        amount: Uint128,
    },
    Claim {
        status: ResponseStatus,
        amount: Uint128,
    },
    RegisterAsset {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    IssuanceCap {},
    TotalMinted {},
    CollateralAsset {},
}   

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config {
        config: Config,
    },
    IssuanceCap {
        issuance_cap: Uint128,
    },
    TotalMinted {
        total_minted: Uint128,
    },
    CollateralAsset {
        collateral_asset: Snip20Asset,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub address: HumanAddr,
    pub deposited_amount: Uint128,
    pub deposit_date: u64,
    pub claimable_amount: Uint128,
    pub is_claimable_status: bool,
    pub claimed_status: bool,
    pub claimed_date: u64,
}