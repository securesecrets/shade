use cosmwasm_std::{StdResult, Storage};
use secret_cosmwasm_math_compat::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::governance::stored_id::ID;

#[cfg(feature = "governance-impl")]
use crate::utils::storage::BucketStorage;
#[cfg(feature = "governance-impl")]
use crate::utils::storage::NaiveBucketStorage;

/// Allow better control over the safety and privacy features that proposals will need if
/// Assemblys are implemented. If a profile is disabled then its assembly will also be disabled.
/// All percentages are taken as follows 100000 = 100%
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Profile {
    pub name: String,
    // State of the current profile and its subsequent assemblies
    pub enabled: bool,
    // Require assembly voting
    pub assembly: Option<VoteProfile>,
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
    const COMMITTEE_PROFILE_KEY: &'static [u8] = b"assembly_vote_profile-";
    const TOKEN_PROFILE_KEY: &'static [u8] = b"token_vote_profile-";

    pub fn load<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Self> {
        let data = Self::data(storage, id)?;

        Ok(Self {
            name: data.name,
            enabled: data.enabled,
            assembly: Self::assembly_voting(storage, &id)?,
            funding: Self::funding(storage, &id)?,
            token: Self::public_voting(storage, &id)?,
            cancel_deadline: data.cancel_deadline
        })
    }

    pub fn may_load<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Option<Self>> {
        if id > &ID::profile(storage)? {
            return Ok(None)
        }
        Ok(Some(Self::load(storage, id)?))
    }

    pub fn save<S: Storage>(&self, storage: &mut S, id: &Uint128) -> StdResult<()> {
        ProfileData {
            name: self.name.clone(),
            enabled: self.enabled,
            cancel_deadline: self.cancel_deadline
        }.save(storage, &id.to_be_bytes())?;

        Self::save_assembly_voting(storage, &id, self.assembly.clone())?;

        Self::save_public_voting(storage, &id, self.token.clone())?;

        Self::save_funding(storage, &id, self.funding.clone())?;

        Ok(())
    }

    pub fn data<S: Storage>(storage: &S, id: &Uint128) -> StdResult<ProfileData> {
        ProfileData::load(storage, &id.to_be_bytes())
    }

    pub fn save_data<S: Storage>(storage: &mut S, id: &Uint128, data: ProfileData) -> StdResult<()> {
        data.save(storage, &id.to_be_bytes())
    }

    pub fn assembly_voting<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Option<VoteProfile>> {
        Ok(VoteProfileType::load(storage, COMMITTEE_PROFILE_KEY, &id.to_be_bytes())?.0)
    }

    pub fn save_assembly_voting<S: Storage>(storage: &mut S, id: &Uint128, assembly: Option<VoteProfile>) -> StdResult<()> {
        VoteProfileType(assembly).save(storage, COMMITTEE_PROFILE_KEY, &id.to_be_bytes())
    }

    pub fn public_voting<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Option<VoteProfile>> {
        Ok(VoteProfileType::load(storage, TOKEN_PROFILE_KEY, &id.to_be_bytes())?.0)
    }

    pub fn save_public_voting<S: Storage>(storage: &mut S, id: &Uint128, token: Option<VoteProfile>) -> StdResult<()> {
        VoteProfileType(token).save(storage, TOKEN_PROFILE_KEY, &id.to_be_bytes())
    }

    pub fn funding<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Option<FundProfile>> {
        Ok(FundProfileType::load(storage, &id.to_be_bytes())?.0)
    }

    pub fn save_funding<S: Storage>(storage: &mut S, id: &Uint128, funding: Option<FundProfile>) -> StdResult<()> {
        FundProfileType(funding).save(storage, &id.to_be_bytes())
    }

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UpdateProfile {
    pub name: Option<String>,
    // State of the current profile and its subsequent assemblys
    pub enabled: Option<bool>,
    // Assembly status
    pub disable_assembly: bool,
    // Require assembly voting
    pub assembly: Option<UpdateVoteProfile>,
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
// NOTE: 100% = Uint128(10000)
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