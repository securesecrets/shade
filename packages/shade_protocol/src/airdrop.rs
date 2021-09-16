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
    pub airdrop_decimals: u8,
    pub sn_validator_weights: Vec<ValidatorWeight>,
    pub sn_banned_validators: Vec<HumanAddr>,
    pub sn_whale_cap: Option<Uint128>,
    // Checks if airdrop has started / ended
    pub start_date: u64,
    pub end_date: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub airdrop_snip20: Contract,
    // These are weight modifiers that will inflate the amount of staked token
    pub sn_validator_weights: Option<Vec<ValidatorWeight>>,
    // These validators will not be counted
    pub sn_banned_validators: Option<Vec<HumanAddr>>,
    // Values greater than this will be ignored
    pub sn_whale_cap: Option<Uint128>,
    // The airdrop time limit
    pub start_date: Option<u64>,
    // Can be set to never end
    pub end_date: Option<u64>,
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
        sn_validator_weights: Option<Vec<ValidatorWeight>>,
        sn_banned_validators: Option<Vec<HumanAddr>>,
        sn_whale_cap: Option<Uint128>,
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
    GetConfig { },
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
    Config { config: Config },
    Dates { start: u64, end: Option<u64> },
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
    pub delegations: Vec<Delegation>,
    pub redeemed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Delegation {
    pub validator_address: HumanAddr,
    pub amount: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// Uint128 has weight of 2 decimals
pub struct ValidatorWeight {
    pub validator_address: HumanAddr,
    pub weight: Uint128
}