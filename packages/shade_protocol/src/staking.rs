use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128};
use secret_toolkit::utils::{HandleCallback, Query};
use crate::{
    asset::Contract,
    generic_response::ResponseStatus,
};
use std::cmp::Ordering;
use crate::governance::UserVote;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admin: Contract,
    // Time to unbond
    pub unbond_time: u64,
    // Supported staking token
    pub staked_token: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StakeState{
    pub total_shares: Uint128,
    pub total_tokens: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UserStakeState{
    pub shares: Uint128,
    // This is used to derive the actual value to recover
    pub tokens_staked: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Unbonding {
    pub amount: Uint128,
    pub unbond_time: u64,
}

impl Ord for Unbonding {
    fn cmp(&self, other: &Unbonding) -> Ordering { self.unbond_time.cmp(&other.unbond_time) }
}

impl PartialOrd for Unbonding {
    fn partial_cmp(&self, other: &Unbonding) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub admin: Option<Contract>,
    pub unbond_time: u64,
    pub staked_token: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {admin: Option<Contract>, unbond_time: Option<u64>},
    // Stake
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
    },
    Unbond { amount: Uint128 },
    // While secure querying is resolved
    Vote { proposal_id: Uint128, votes: Vec<UserVote> },
    ClaimUnbond {},
    ClaimRewards {},
    SetViewingKey { key: String },
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
    Vote { status: ResponseStatus },
    ClaimUnbond { status: ResponseStatus },
    ClaimRewards { status: ResponseStatus },
    SetViewingKey { status: ResponseStatus }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    TotalStaked {},
    TotalUnbonding { start: Option<u64>, end: Option<u64> },
    UserStake { address: HumanAddr, key: String, time: u64},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    TotalStaked { total: Uint128 },
    TotalUnbonding { total: Uint128 },
    UserStake { staked: Uint128, pending_rewards: Uint128, unbonding: Uint128, unbonded: Uint128 },
}
