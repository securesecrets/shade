use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::utils::flexible_msg::FlexibleMsg;

#[cfg(feature = "governance-impl")]
use crate::utils::storage::BucketStorage;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Committee {
    // Readable name
    pub name: String,
    // Description of the committee, preferably in base64
    pub metadata: String,
    // List of members in committee
    pub members: Vec<HumanAddr>,
    // Selected profile
    pub profile: Uint128,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for Committee {
    const NAMESPACE: &'static [u8] = b"committee-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// A generic msg is created at init, its a black msg where the variable is the start
pub struct CommitteeMsg {
    pub name: String,
    // Committees allowed to call this msg
    pub committees: Vec<Uint128>,
    // HandleMsg template
    pub msg: FlexibleMsg
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for CommitteeMsg {
    const NAMESPACE: &'static [u8] = b"committee_msg-";
}