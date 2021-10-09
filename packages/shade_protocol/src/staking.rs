use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128, Binary};
use secret_toolkit::utils::{InitCallback, HandleCallback, Query};
use crate::{
    asset::Contract,
    generic_response::ResponseStatus,
};
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admin: HumanAddr,
    // Time to unbond
    pub unbond_time: u64,
    // Supported staking token
    pub staked_token: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Unbonding {
    pub account: HumanAddr,
    pub amount: Uint128,
    pub unbond_time: u64,
}

impl Ord for Unbonding {
    fn cmp(&self, other: &Unbonding) -> Ordering {
        other.unbond_time.cmp(&self.unbond_time)
    }
}

impl PartialOrd for Unbonding {
    fn partial_cmp(&self, other: &Unbonding) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub unbond_time: u64,
    pub staked_token: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateUnbondTime { unbond_time: u64 },
    // Stake
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
    },
    Unbond { amount: Uint128 },
    QueryStaker { account: HumanAddr },
    QueryStakers { accounts: Vec<HumanAddr> },
    TriggerUnbonds {},
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateUnbondTime { status: ResponseStatus },
    Stake { status: ResponseStatus },
    Unbond { status: ResponseStatus },
    QueryStaker { status: ResponseStatus, stake: Uint128 },
    QueryStakers { status: ResponseStatus, stake: Vec<Uint128> },
    TriggerUnbonds { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    TotalStaked {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    TotalStaked { total: Uint128 },
}