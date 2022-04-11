use cosmwasm_std::{StdResult, Storage};
use secret_cosmwasm_math_compat::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "governance-impl")]
use crate::utils::storage::{NaiveBucketStorage};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VoteTally {
    pub yes: Uint128,
    pub no: Uint128,
    pub no_with_veto: Uint128,
    pub abstain: Uint128,
}

#[cfg(feature = "governance-impl")]
impl NaiveBucketStorage for VoteTally {
}

#[cfg(feature = "governance-impl")]
impl VoteTally {
    // Load votes related to staking
    fn load_token<'a, S: Storage>(storage: &'a S, key: &'a [u8]) -> StdResult<Option<Self>> {
        VoteTally::read(storage, b"vote_tally_token-").may_load(key)
    }

    fn save_token<'a, S: Storage>(&self, storage: &'a mut S, key: &'a [u8]) -> StdResult<()> {
        VoteTally::write(storage, b"vote_tally_token-").save(key, self)
    }

    // Load votes related to assembly
    fn load_assembly<'a, S: Storage>(storage: &'a S, key: &'a [u8]) -> StdResult<Option<Self>> {
        VoteTally::read(storage, b"vote_tally_assembly-").may_load(key)
    }

    fn save_assembly<'a, S: Storage>(&self, storage: &'a mut S, key: &'a [u8]) -> StdResult<()> {
        VoteTally::write(storage, b"vote_tally_assembly-").save(key, self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Vote {
    Yes,
    No,
    NoWithVeto,
    Abstain,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Used to give weight to votes per user
pub struct UserVote {
    pub vote: Vote,
    pub weight: u8,
}
