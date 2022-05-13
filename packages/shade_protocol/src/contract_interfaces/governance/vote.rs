use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{StdResult, Storage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "governance-impl")]
use crate::utils::storage::default::NaiveBucketStorage;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ReceiveBalanceMsg {
    pub vote: Vote,
    pub proposal: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Vote {
    pub yes: Uint128,
    pub no: Uint128,
    pub no_with_veto: Uint128,
    pub abstain: Uint128,
}

#[cfg(feature = "governance-impl")]
impl NaiveBucketStorage for Vote {}

impl Default for Vote {
    fn default() -> Self {
        Self {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            no_with_veto: Uint128::zero(),
            abstain: Uint128::zero(),
        }
    }
}

impl Vote {
    pub fn total_count(&self) -> StdResult<Uint128> {
        Ok(self.yes.checked_add(
            self.no
                .checked_add(self.no_with_veto.checked_add(self.abstain)?)?,
        )?)
    }

    pub fn checked_sub(&self, vote: &Self) -> StdResult<Self> {
        Ok(Self {
            yes: self.yes.checked_sub(vote.yes)?,
            no: self.no.checked_sub(vote.no)?,
            no_with_veto: self.no_with_veto.checked_sub(vote.no_with_veto)?,
            abstain: self.abstain.checked_sub(vote.abstain)?,
        })
    }

    pub fn checked_add(&self, vote: &Self) -> StdResult<Self> {
        Ok(Self {
            yes: self.yes.checked_add(vote.yes)?,
            no: self.no.checked_add(vote.no)?,
            no_with_veto: self.no_with_veto.checked_add(vote.no_with_veto)?,
            abstain: self.abstain.checked_add(vote.abstain)?,
        })
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
            total: votes.yes + votes.no + votes.no_with_veto + votes.abstain,
        }
    }
}
