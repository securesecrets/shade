use cosmwasm_std::{StdResult, Storage};
use cosmwasm_math_compat::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "governance-impl")]
use crate::utils::storage::{NaiveBucketStorage};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Vote {
    pub yes: Uint128,
    pub no: Uint128,
    pub no_with_veto: Uint128,
    pub abstain: Uint128,
}

#[cfg(feature = "governance-impl")]
impl NaiveBucketStorage for Vote {
}

impl Default for Vote {
    fn default() -> Self {
        Self {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero()
        }
    }
}

#[cfg(feature = "governance-impl")]
impl Vote {
    // Load votes related to staking
    fn load_token<'a, S: Storage>(storage: &'a S, key: &'a [u8]) -> StdResult<Option<Self>> {
        Vote::read(storage, b"vote_tally_token-").may_load(key)
    }

    fn save_token<'a, S: Storage>(&self, storage: &'a mut S, key: &'a [u8]) -> StdResult<()> {
        Vote::write(storage, b"vote_tally_token-").save(key, self)
    }
}

pub struct TalliedVotes {
    pub yes: Uint128,
    pub no: Uint128,
    pub veto: Uint128,
    pub total: Uint128,
}

impl TalliedVotes {
    pub fn tally(votes: Vote) -> Self {
        Self {
            yes: votes.yes,
            no: votes.no + votes.no_with_veto,
            veto: votes.no_with_veto,
            total: votes.yes + votes.no + votes.no_with_veto + votes.abstain
        }
    }
}
