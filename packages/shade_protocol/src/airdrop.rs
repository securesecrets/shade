use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::utils::{InitCallback, HandleCallback, Query};
use secretcli::secretcli::{TestInit, TestHandle, TestQuery};
use cosmwasm_std::{HumanAddr, Uint128};
use crate::asset::Contract;
use crate::generic_response::ResponseStatus;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: HumanAddr,
    // The snip20 to be minted
    pub airdrop_snip20: Contract,
    pub prefered_validator: HumanAddr,
    // Checks if airdrop has started / ended
    pub start_date: u64,
    pub end_date: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub airdrop_snip20: Contract,
    pub prefered_validator: HumanAddr,
    pub start_date: Option<u64>,
    pub end_date: u64,

    // Secret network delegators snapshot
    pub sn_snapshot: Vec<Delegator>,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

impl TestInit for InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        admin: Option<HumanAddr>,
        airdrop_snip20: Option<Contract>,
        prefered_validator: Option<HumanAddr>,
        start_date: Option<u64>,
        end_date: Option<u64>,
    },
    Redeem {}
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

impl TestHandle for HandleMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init { status: ResponseStatus },
    UpdateConfig { status: ResponseStatus },
    Redeem { status: ResponseStatus }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetDates { },
    GetEligibility { address: HumanAddr }
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

impl TestQuery<QueryAnswer> for QueryMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Dates { start: u64, end: u64 },
    Eligibility { amount: Uint128 }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Delegator {
    pub address: HumanAddr,
    pub delegations: Vec<Delegation>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StoredDelegator {
    pub address: HumanAddr,
    pub delegations: Vec<Delegation>.
    pub redeemed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Delegation {
    pub validator_address: HumanAddr,
    pub amount: Uint128
}