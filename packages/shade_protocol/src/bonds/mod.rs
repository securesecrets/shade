pub mod errors;

use chrono::prelude::*;
use crate::utils::generic_response::ResponseStatus;
use crate::utils::asset::Contract;
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::snip20::Snip20Asset;
use secret_toolkit::utils::{HandleCallback};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub limit_admin: HumanAddr,
    pub admin: HumanAddr,
    pub oracle: Contract,
    pub treasury: HumanAddr,
    pub issued_asset: Contract,
    pub activated: bool,
    pub minting_bond: bool,
    pub bond_issuance_limit: Uint128,
    pub bonding_period: Uint128,
    pub discount: Uint128,
    pub global_issuance_limit: Uint128,
    pub global_minimum_bonding_period: Uint128,
    pub global_maximum_discount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub limit_admin: HumanAddr,
    pub global_issuance_limit: Uint128,
    pub global_minimum_bonding_period: Uint128,
    pub global_maximum_discount: Uint128,
    pub admin: HumanAddr,
    pub oracle: Contract,
    pub treasury: HumanAddr,
    pub issued_asset: Contract,
    pub activated: bool,
    pub minting_bond: bool,
    pub bond_issuance_limit: Uint128,
    pub bonding_period: Uint128,
    pub discount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateLimitConfig {
        limit_admin: Option<HumanAddr>,
        global_issuance_limit: Option<Uint128>,
        global_minimum_bonding_period: Option<Uint128>,
        global_maximum_discount: Option<Uint128>,
    },
    UpdateConfig {
        admin: Option<HumanAddr>,
        oracle: Option<Contract>,
        treasury: Option<HumanAddr>,
        issued_asset: Option<Contract>,
        activated: Option<bool>,
        minting_bond: Option<bool>,
        bond_issuance_limit: Option<Uint128>,
        bonding_period: Option<Uint128>,
        discount: Option<Uint128>,
    },
    OpenBond {
        collateral_asset: Option<Contract>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        bond_issuance_limit: Option<Uint128>,
        bonding_period: Option<Uint128>,
        discount: Option<Uint128>,
    },
    RegisterCollateralAsset {
        collateral_asset: Contract,
    },
    RemoveCollateralAsset {
        collateral_asset: Contract,
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
    UpdateLimitConfig {
        status: ResponseStatus,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    Deposit {
        status: ResponseStatus,
        deposit_amount: Uint128,
        pending_claim_amount: Uint128,
        end_date: u64,
    },
    Claim {
        status: ResponseStatus,
        amount: Uint128,
    },
    RegisterCollateralAsset {
        status: ResponseStatus,
    },
    RemoveCollateralAsset {
        status: ResponseStatus,
    },
    OpenBond {
        status: ResponseStatus,
        deposit_contract: Contract,
        start_time: u64,
        end_time: u64,
        bond_issuance_limit: Uint128,
        bonding_period: Uint128,
        discount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    IssuanceCap {},
    TotalIssued {},
    CollateralAsset {},
}   

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config {
        config: Config,
    },
    BondingPeriod {
        bonding_period: Uint128,
    },
    BondIssuanceLimit {
        bond_issuance_limit: Uint128,
    },
    GlobalTotalIssued {
        global_total_issued: Uint128,
    },
    CollateralAsset {
        collateral_asset: Snip20Asset,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub address: HumanAddr,
    pub pending_bonds: Vec<PendingBond>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PendingBond {
    pub claim_amount: Uint128,
    pub end: u64, // Will be turned into a time via block time calculations
    pub deposit_denom: Snip20Asset,
    pub deposit_amount: Uint128,
}

// When users deposit and try to use the bond, a Bond Opportunity is selected via deposit denom
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BondOpportunity {
    pub issuance_limit: Uint128,
    pub amount_issued: Uint128,
    pub deposit_denom: Snip20Asset,
    pub start_time: u64,
    pub end_time: u64,
    pub bonding_period: Uint128,
    pub discount: Uint128,
}