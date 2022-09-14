use crate::{
    c_std::StdError,
    impl_into_u8,
    utils::errors::{build_string, CodeType, DetailedError},
};

use cosmwasm_schema::cw_serde;

#[cw_serde]
#[repr(u8)]
pub enum Error {
    InvalidTaskPercentage,
    InvalidDates,
    PermitContractMismatch,
    PermitKeyRevoked,
    PermitRejected,
    NotAdmin,
    AccountAlreadyCreated,
    AccountDoesntExist,
    NothingToClaim,
    DecayClaimed,
    NoDecaySet,
    ClaimAmountTooHigh,
    AddressInAccount,
    ExpectedMemo,
    InvalidPartialTree,
    AirdropNotStarted,
    AirdropEnded,
    InvalidViewingKey,
    UnexpectedError,
}

impl_into_u8!(Error);

impl CodeType for Error {
    fn to_verbose(&self, context: &Vec<&str>) -> String {
        match self {
            Error::InvalidTaskPercentage => {
                build_string("Task total exceeds maximum of 100%, got {}", context)
            }
            Error::InvalidDates => build_string("{} ({}) cannot happen {} {} ({})", context),
            Error::PermitContractMismatch => {
                build_string("Permit is valid for {}, expected {}", context)
            }
            Error::PermitKeyRevoked => build_string("Permit key {} revoked", context),
            Error::PermitRejected => build_string("Permit was rejected", context),
            Error::NotAdmin => build_string("Can only be accessed by {}", context),
            Error::AccountAlreadyCreated => build_string("Account already exists", context),
            Error::AccountDoesntExist => build_string("Account does not exist", context),
            Error::NothingToClaim => build_string("Amount to claim is 0", context),
            Error::DecayClaimed => build_string("Decay already claimed", context),
            Error::NoDecaySet => build_string("Decay has not been set", context),
            Error::ClaimAmountTooHigh => {
                build_string("Claim {} is higher than the maximum claim of {}", context)
            }
            Error::AddressInAccount => build_string("{} has already been claimed", context),
            Error::ExpectedMemo => build_string("Expected a memo", context),
            Error::InvalidPartialTree => build_string("Partial tree is not valid", context),
            Error::AirdropNotStarted => {
                build_string("Airdrop starts in {}, its currently {}", context)
            }
            Error::AirdropEnded => build_string("Airdrop ended on {}, its currently {}", context),
            Error::InvalidViewingKey => build_string("Provided viewing key is invalid", context),
            Error::UnexpectedError => build_string("Something unexpected happened", context),
        }
    }
}

const AIRDROP_TARGET: &str = "airdrop";

pub fn invalid_task_percentage(percentage: &str) -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::InvalidTaskPercentage, vec![
        percentage,
    ])
    .to_error()
}

pub fn invalid_dates(
    item_a: &str,
    item_a_amount: &str,
    precedence: &str,
    item_b: &str,
    item_b_amount: &str,
) -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::InvalidDates, vec![
        item_a,
        item_a_amount,
        precedence,
        item_b,
        item_b_amount,
    ])
    .to_error()
}

pub fn permit_contract_mismatch(contract: &str, expected: &str) -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::PermitContractMismatch, vec![
        contract, expected,
    ])
    .to_error()
}

pub fn permit_key_revoked(key: &str) -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::PermitKeyRevoked, vec![key]).to_error()
}

pub fn permit_rejected() -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::PermitRejected, vec![]).to_error()
}

pub fn not_admin(admin: &str) -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::NotAdmin, vec![admin]).to_error()
}

pub fn account_already_created() -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::AccountAlreadyCreated, vec![]).to_error()
}

pub fn account_does_not_exist() -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::AccountDoesntExist, vec![]).to_error()
}

pub fn nothing_to_claim() -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::NothingToClaim, vec![]).to_error()
}

pub fn decay_claimed() -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::DecayClaimed, vec![]).to_error()
}

pub fn decay_not_set() -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::NoDecaySet, vec![]).to_error()
}

pub fn claim_too_high(claim: &str, max: &str) -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::ClaimAmountTooHigh, vec![claim, max]).to_error()
}

pub fn address_already_in_account(address: &str) -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::AddressInAccount, vec![address]).to_error()
}

pub fn expected_memo() -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::ExpectedMemo, vec![]).to_error()
}

pub fn invalid_partial_tree() -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::InvalidPartialTree, vec![]).to_error()
}

pub fn airdrop_not_started(start: &str, current: &str) -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::AirdropNotStarted, vec![
        start, current,
    ])
    .to_error()
}

pub fn airdrop_ended(end: &str, current: &str) -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::AirdropEnded, vec![end, current]).to_error()
}

pub fn invalid_viewing_key() -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::InvalidViewingKey, vec![]).to_error()
}

pub fn unexpected_error() -> StdError {
    DetailedError::from_code(AIRDROP_TARGET, Error::UnexpectedError, vec![]).to_error()
}
