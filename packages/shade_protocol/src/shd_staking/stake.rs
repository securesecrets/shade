use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cosmwasm_std::{HumanAddr, Uint128};
use crate::storage::{BucketStorage, SingletonStorage};
use crate::utils::asset::Contract;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StakeConfig {
    pub unbond_time: u64,
    pub staked_token: Contract,
    pub decimal_difference: u8,
    pub treasury: Option<HumanAddr>
}

impl SingletonStorage for StakeConfig {
    const NAMESPACE: &'static [u8] = b"stake_config";
}

// uint wrappers

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TotalStaked(pub u128);

impl SingletonStorage for TotalStaked {
    const NAMESPACE: &'static [u8] = b"total_Staked";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UserStake(pub u128);

impl BucketStorage for UserStake {
    const NAMESPACE: &'static [u8] = b"user_Staked";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UnsentStakedTokens(pub u128);

impl SingletonStorage for UnsentStakedTokens {
    const NAMESPACE: &'static [u8] = b"total_Staked";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TotalUnbonding(pub u128);

impl SingletonStorage for TotalUnbonding {
    const NAMESPACE: &'static [u8] = b"total_unbonding";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DailyUnbonding(pub u128);

impl BucketStorage for DailyUnbonding {
    const NAMESPACE: &'static [u8] = b"daily_unbonding";
}

// Distributors wrappers

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Distributors(pub Vec<HumanAddr>);

impl SingletonStorage for Distributors {
    const NAMESPACE: &'static [u8] = b"distributors";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DistributorsEnabled(pub bool);

impl SingletonStorage for DistributorsEnabled {
    const NAMESPACE: &'static [u8] = b"distributors_transfer";
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Unbonding {
    pub amount: Uint128,
    pub release: u64,
}

impl Ord for Unbonding {
    fn cmp(&self, other: &Unbonding) -> Ordering {
        self.release.cmp(&other.release)
    }
}

impl PartialOrd for Unbonding {
    fn partial_cmp(&self, other: &Unbonding) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}