use cosmwasm_std::{StdResult, Storage, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "governance-impl")]
use crate::utils::storage::BucketStorage;
use crate::utils::storage::NaiveBucketStorage;

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
impl Profile {
    const COMMITTEE_PROFILE_KEY: &'static [u8] = b"committee_vote_profile-";
    const TOKEN_PROFILE_KEY: &'static [u8] = b"token_vote_profile-";

    pub fn load<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Self> {
        let data = Self::data(storage, id)?;

        Ok(Self {
            name: data.name,
            enabled: data.enabled,
            committee: Self::load_committee(storage, &id)?,
            funding: Self::load_funding(storage, &id)?,
            token: Self::load_token(storage, &id)?,
            cancel_deadline: data.cancel_deadline
        })
    }

    pub fn save<S: Storage>(&self, storage: &mut S, id: &Uint128) -> StdResult<()> {
        ProfileData {
            name: self.name.clone(),
            enabled: self.enabled,
            cancel_deadline: self.cancel_deadline
        }.save(storage, id.to_string().as_bytes())?;

        Self::save_committee(storage, &id, self.committee.clone())?;

        Self::save_token(storage, &id, self.token.clone())?;

        Self::save_funding(storage, &id, self.funding.clone())?;

        Ok(())
    }

    pub fn data<S: Storage>(storage: &S, id: &Uint128) -> StdResult<ProfileData> {
        ProfileData::load(storage, id.to_string().as_bytes())
    }

    pub fn save_data<S: Storage>(storage: &mut S, id: &Uint128, data: ProfileData) -> StdResult<()> {
        data.save(storage, id.to_string().as_bytes())
    }

    pub fn load_committee<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Option<VoteProfile>> {
        Ok(VoteProfileType::load(storage, COMMITTEE_PROFILE_KEY, id.to_string().as_bytes())?.0)
    }

    pub fn save_committee<S: Storage>(storage: &mut S, id: &Uint128, committee: Option<VoteProfile>) -> StdResult<()> {
        VoteProfileType(committee).save(storage, COMMITTEE_PROFILE_KEY, id.to_string().as_bytes())
    }

    pub fn load_token<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Option<VoteProfile>> {
        Ok(VoteProfileType::load(storage, TOKEN_PROFILE_KEY, id.to_string().as_bytes())?.0)
    }

    pub fn save_token<S: Storage>(storage: &mut S, id: &Uint128, token: Option<VoteProfile>) -> StdResult<()> {
        VoteProfileType(token).save(storage, TOKEN_PROFILE_KEY, id.to_string().as_bytes())
    }

    pub fn load_funding<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Option<FundProfile>> {
        Ok(FundProfileType::load(storage, id.to_string().as_bytes())?.0)
    }

    pub fn save_funding<S: Storage>(storage: &mut S, id: &Uint128, funding: Option<FundProfile>) -> StdResult<()> {
        FundProfileType(funding).save(storage, id.to_string().as_bytes())
    }

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UpdateProfile {
    pub name: Option<String>,
    // State of the current profile and its subsequent committees
    pub enabled: Option<bool>,
    // Committee status
    pub disable_committee: bool,
    // Require committee voting
    pub committee: Option<UpdateVoteProfile>,
    // Funding status
    pub disable_funding: bool,
    // Require funding
    pub funding: Option<UpdateFundProfile>,
    // Require token voting
    pub disable_token: bool,
    // Require token voting
    pub token: Option<UpdateVoteProfile>,
    // Once the contract is approved, theres a deadline for the tx to be executed and completed
    // else it will just be canceled and assume that the tx failed
    pub cancel_deadline: Option<u64>
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProfileData {
    pub name: String,
    pub enabled: bool,
    pub cancel_deadline: u64
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for ProfileData {
    const NAMESPACE: &'static [u8] = b"profile_data-";
}

#[cfg(feature = "governance-impl")]
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

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VoteProfileType(pub Option<VoteProfile>);

#[cfg(feature = "governance-impl")]
impl NaiveBucketStorage for VoteProfileType {
}

#[cfg(feature = "governance-impl")]
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

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct FundProfileType(pub Option<FundProfile>);


#[cfg(feature = "governance-impl")]
impl BucketStorage for FundProfile {
    const NAMESPACE: &'static [u8] = b"fund_profile-";
}

/// Helps simplify the given limits
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Count {
    Percentage { percent: u16 },
    LiteralCount { count: Uint128 }
}