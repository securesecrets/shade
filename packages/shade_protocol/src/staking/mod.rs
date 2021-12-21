pub mod stake;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128};
use secret_toolkit::utils::{HandleCallback, Query};
use crate::{
    asset::Contract,
    generic_response::ResponseStatus,
};
use crate::governance::vote::UserVote;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admin: Contract,
    // Time to unbond
    pub unbond_time: u64,
    // Supported staking token
    pub staked_token: Contract,
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
