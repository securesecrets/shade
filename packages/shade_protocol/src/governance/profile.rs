use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "governance-impl")]
use crate::utils::storage::BucketStorage;

/// Allow better control over the safety and privacy features that proposals will need if
/// Committees are implemented. If a profile is disabled then its committee will also be disabled.
/// All percentages are taken as follows 100000 = 100%
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Profile {
    pub name: String,
    // State of the current profile and its subsequent committees
    pub enabled: bool,
    // Require committee voting
    pub committee: Option<VoteProfile>,
    // Require funding
    pub funding: Option<FundProfile>,
    // Require token voting
    pub token: Option<VoteProfile>,
    // Once the contract is approved, theres a deadline for the tx to be executed and completed
    // else it will just be canceled and assume that the tx failed
    pub cancel_deadline: u64
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for Profile {
    const NAMESPACE: &'static [u8] = b"profile-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VoteProfile {
    // Deadline for voting
    pub deadline: u64,
    // Expected participation threshold
    pub threshold: Count,
    // Expected yes votes
    pub yes_threshold: Count,
    // Expected veto votes
    pub veto_threshold: Count
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct FundProfile {
    // Deadline for funding
    pub deadline: u64,
    // Amount required to fund
    pub required: Uint128,
    // Display voter information
    pub privacy: bool,
    // Deposit loss on failed proposal
    pub failed_deposit_loss: Count,
    // Deposit loss on vetoed proposal
    pub veto_deposit_loss: Count,
}

/// Helps simplify the given limits
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Count {
    Percentage { percent: u16 },
    LiteralCount { count: Uint128 }
}