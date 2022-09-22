use crate::{
    c_std::{StdError, StdResult, Storage, Uint128},
    contract_interfaces::governance::stored_id::ID,
};

use cosmwasm_schema::cw_serde;
use secret_storage_plus::Map;

#[cfg(feature = "governance-impl")]
use crate::utils::storage::plus::{MapStorage, NaiveMapStorage};

/// Allow better control over the safety and privacy features that proposals will need if
/// Assemblys are implemented. If a profile is disabled then its assembly will also be disabled.
/// All percentages are taken as follows 100000 = 100%
#[cw_serde]
pub struct Profile {
    pub name: String,
    // State of the current profile and its subsequent assemblies
    pub enabled: bool,
    // Require assembly voting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assembly: Option<VoteProfile>,
    // Require funding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funding: Option<FundProfile>,
    // Require token voting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<VoteProfile>,
    // Once the contract is approved, theres a deadline for the tx to be executed and completed
    // else it will just be canceled and assume that the tx failed
    pub cancel_deadline: u64,
}

const COMMITTEE_PROFILE_KEY: Map<u16, VoteProfileType> = Map::new("assembly_vote_profile-");
const TOKEN_PROFILE_KEY: Map<u16, VoteProfileType> = Map::new("token_vote_profile-");

#[cfg(feature = "governance-impl")]
impl Profile {
    pub fn load(storage: &dyn Storage, id: u16) -> StdResult<Self> {
        let data = Self::data(storage, id)?;

        Ok(Self {
            name: data.name,
            enabled: data.enabled,
            assembly: Self::assembly_voting(storage, id)?,
            funding: Self::funding(storage, id)?,
            token: Self::public_voting(storage, id)?,
            cancel_deadline: data.cancel_deadline,
        })
    }

    pub fn may_load(storage: &dyn Storage, id: u16) -> StdResult<Option<Self>> {
        if id > ID::profile(storage)? {
            return Ok(None);
        }
        Ok(Some(Self::load(storage, id)?))
    }

    pub fn save(&self, storage: &mut dyn Storage, id: u16) -> StdResult<()> {
        ProfileData {
            name: self.name.clone(),
            enabled: self.enabled,
            cancel_deadline: self.cancel_deadline,
        }
        .save(storage, id)?;

        Self::save_assembly_voting(storage, id, self.assembly.clone())?;

        Self::save_public_voting(storage, id, self.token.clone())?;

        Self::save_funding(storage, id, self.funding.clone())?;

        Ok(())
    }

    pub fn data(storage: &dyn Storage, id: u16) -> StdResult<ProfileData> {
        ProfileData::load(storage, id)
    }

    pub fn save_data(storage: &mut dyn Storage, id: u16, data: ProfileData) -> StdResult<()> {
        data.save(storage, id)
    }

    pub fn assembly_voting(storage: &dyn Storage, id: u16) -> StdResult<Option<VoteProfile>> {
        Ok(VoteProfileType::load(storage, COMMITTEE_PROFILE_KEY, id)?.0)
    }

    pub fn save_assembly_voting(
        storage: &mut dyn Storage,
        id: u16,
        assembly: Option<VoteProfile>,
    ) -> StdResult<()> {
        VoteProfileType(assembly).save(storage, COMMITTEE_PROFILE_KEY, id)
    }

    pub fn public_voting(storage: &dyn Storage, id: u16) -> StdResult<Option<VoteProfile>> {
        Ok(VoteProfileType::load(storage, TOKEN_PROFILE_KEY, id)?.0)
    }

    pub fn save_public_voting(
        storage: &mut dyn Storage,
        id: u16,
        token: Option<VoteProfile>,
    ) -> StdResult<()> {
        VoteProfileType(token).save(storage, TOKEN_PROFILE_KEY, id)
    }

    pub fn funding(storage: &dyn Storage, id: u16) -> StdResult<Option<FundProfile>> {
        Ok(FundProfileType::load(storage, id)?.0)
    }

    pub fn save_funding(
        storage: &mut dyn Storage,
        id: u16,
        funding: Option<FundProfile>,
    ) -> StdResult<()> {
        FundProfileType(funding).save(storage, id)
    }
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
pub struct ProfileData {
    pub name: String,
    pub enabled: bool,
    pub cancel_deadline: u64,
}

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u16> for ProfileData {
    const MAP: Map<'static, u16, Self> = Map::new("profile_data-");
}

#[cfg(feature = "governance-impl")]
#[cw_serde] // NOTE: 100% = Uint128::new(10000)
pub struct VoteProfile {
    // Deadline for voting
    pub deadline: u64,
    // Expected participation threshold
    pub threshold: Count,
    // Expected yes votes
    pub yes_threshold: Count,
    // Expected veto votes
    pub veto_threshold: Count,
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
struct VoteProfileType(pub Option<VoteProfile>);

#[cfg(feature = "governance-impl")]
impl NaiveMapStorage<'static> for VoteProfileType {}

#[cfg(feature = "governance-impl")]
#[cw_serde]
pub struct FundProfile {
    // Deadline for funding
    pub deadline: u64,
    // Amount required to fund
    pub required: Uint128,
    // Display voter information
    pub privacy: bool,
    // Deposit loss on vetoed proposal
    pub veto_deposit_loss: Uint128,
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
struct FundProfileType(pub Option<FundProfile>);

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u16> for FundProfileType {
    const MAP: Map<'static, u16, Self> = Map::new("fund_profile-");
}

/// Helps simplify the given limits
#[cw_serde]
pub enum Count {
    Percentage { percent: u16 },
    LiteralCount { count: Uint128 },
}

#[cw_serde]
pub struct UpdateProfile {
    pub name: Option<String>,
    // State of the current profile and its subsequent assemblies
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
    pub cancel_deadline: Option<u64>,
}

#[cw_serde]
pub struct UpdateVoteProfile {
    // Deadline for voting
    pub deadline: Option<u64>,
    // Expected participation threshold
    pub threshold: Option<Count>,
    // Expected yes votes
    pub yes_threshold: Option<Count>,
    // Expected veto votes
    pub veto_threshold: Option<Count>,
}

impl UpdateVoteProfile {
    pub fn update_profile(&self, profile: &Option<VoteProfile>) -> StdResult<VoteProfile> {
        let new_profile: VoteProfile;

        if let Some(profile) = profile {
            new_profile = VoteProfile {
                deadline: self.deadline.unwrap_or(profile.deadline),
                threshold: self.threshold.clone().unwrap_or(profile.threshold.clone()),
                yes_threshold: self
                    .yes_threshold
                    .clone()
                    .unwrap_or(profile.yes_threshold.clone()),
                veto_threshold: self
                    .veto_threshold
                    .clone()
                    .unwrap_or(profile.veto_threshold.clone()),
            };
        } else {
            new_profile = VoteProfile {
                deadline: match self.deadline {
                    None => Err(StdError::generic_err("Vote profile must be set")),
                    Some(ret) => Ok(ret),
                }?,
                threshold: match self.threshold.clone() {
                    None => Err(StdError::generic_err("Vote profile must be set")),
                    Some(ret) => Ok(ret),
                }?,
                yes_threshold: match self.yes_threshold.clone() {
                    None => Err(StdError::generic_err("Vote profile must be set")),
                    Some(ret) => Ok(ret),
                }?,
                veto_threshold: match self.veto_threshold.clone() {
                    None => Err(StdError::generic_err("Vote profile must be set")),
                    Some(ret) => Ok(ret),
                }?,
            };
        }

        Ok(new_profile)
    }
}

#[cw_serde]
pub struct UpdateFundProfile {
    // Deadline for funding
    pub deadline: Option<u64>,
    // Amount required to fund
    pub required: Option<Uint128>,
    // Display voter information
    pub privacy: Option<bool>,
    // Deposit loss on vetoed proposal
    pub veto_deposit_loss: Option<Uint128>,
}

impl UpdateFundProfile {
    pub fn update_profile(&self, profile: &Option<FundProfile>) -> StdResult<FundProfile> {
        let new_profile: FundProfile;

        if let Some(profile) = profile {
            new_profile = FundProfile {
                deadline: self.deadline.unwrap_or(profile.deadline),
                required: self.required.unwrap_or(profile.required),
                privacy: self.privacy.unwrap_or(profile.privacy),
                veto_deposit_loss: self
                    .veto_deposit_loss
                    .clone()
                    .unwrap_or(profile.veto_deposit_loss.clone()),
            };
        } else {
            new_profile = FundProfile {
                deadline: match self.deadline {
                    None => Err(StdError::generic_err("Fund profile must be set")),
                    Some(ret) => Ok(ret),
                }?,
                required: match self.required {
                    None => Err(StdError::generic_err("Fund profile must be set")),
                    Some(ret) => Ok(ret),
                }?,
                privacy: match self.privacy {
                    None => Err(StdError::generic_err("Fund profile must be set")),
                    Some(ret) => Ok(ret),
                }?,
                veto_deposit_loss: match self.veto_deposit_loss.clone() {
                    None => Err(StdError::generic_err("Fund profile must be set")),
                    Some(ret) => Ok(ret),
                }?,
            };
        }

        Ok(new_profile)
    }
}
