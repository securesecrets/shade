use crate::impl_into_u8;
use crate::utils::errors::{build_string, CodeType, DetailedError};
use cosmwasm_std::{StdError, Uint128, HumanAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug, JsonSchema)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum Error{
    BondEnded,
    BondNotStarted,
    BondLimitReached,
    GlobalLimitReached,
    MintExceedsLimit,
    ContractNotActive,
    NoBondFound,
    NoPendingBonds,
    IncorrectViewingKey,
    PermitContractMismatch,
    PermitKeyRevoked,
    PermitRejected,
    BondLimitExceedsGlobalLimit,
    BondingPeriodBelowMinimumTime,
    BondDiscountAboveMaximumRate,
    BondIssuanceExceedsAllowance,
    NotLimitAdmin,
    CollateralPriceExceedsLimit,
    IssuedPriceBelowMinimum,
    SlippageToleranceExceeded,
    Blacklisted,
    IssuedAssetDeposit,
    NotTreasuryBond,
    NoBondsClaimable,
}

impl_into_u8!(Error);

impl CodeType for Error {
    fn to_verbose(&self, context: &Vec<&str>) -> String{
        match self{
            Error::BondEnded => {
                build_string("Bond ended on {}, it is currently {}", context)
            }
            Error::BondNotStarted => {
                build_string("Bond begins on {}, it is currently {}", context)
            }
            Error::BondLimitReached => {
                build_string("Bond opportunity is not available due to issuance limit of {} being reached", context)
            }
            Error::GlobalLimitReached => {
                build_string("Bond issuance limit of {} has been reached", context)
            }
            Error::MintExceedsLimit => {
                build_string("Mint amount of {} exceeds available mint of {}", context)
            }
            Error::ContractNotActive => {
                build_string("Bonds contract is currently not active. Governance must activate the contract before functionality can resume.", context)
            }
            Error::NoBondFound => {
                build_string("No bond opportunity found for collateral contract {}", context)
            }
            Error::NoPendingBonds => {
                build_string("No pending bonds for user address {}", context)
            }
            Error::IncorrectViewingKey => {
                build_string("Provided viewing key is incorrect", context)
            }
            Error::BondLimitExceedsGlobalLimit => {
                build_string("Proposed bond issuance limit of {} exceeds available bond limit of {}", context)
            }
            Error::BondingPeriodBelowMinimumTime => {
                build_string("Bonding period of {} is below minimum limit of {}", context)
            }
            Error::BondDiscountAboveMaximumRate => {
                build_string("Bond discount of {} is above maximum limit of {}", context)
            }
            Error::BondIssuanceExceedsAllowance => {
                build_string("Bond issuance limit of {} exceeds available allowance of {}", context)
            }
            Error::NotLimitAdmin => {
                build_string("Global limit parameters can only be changed by the limit admin", context)
            }
            Error::CollateralPriceExceedsLimit => {
                build_string("Collateral asset price of {} exceeds limit price of {}, cannot enter bond opportunity", context)
            }
            Error::IssuedPriceBelowMinimum => {
                build_string("Issued asset price of {} is below minimum value of {}, cannot enter opportunity", context)
            }
            Error::SlippageToleranceExceeded => {
                build_string("Calculated issuance amount of {} is below minimum accepted value of {}", context)
            }
            Error::PermitContractMismatch => {
                build_string("Permit is valid for {}, expected {}", context)
            }
            Error::PermitKeyRevoked => {
                build_string("Permit key {} revoked", context)
            }
            Error::PermitRejected => {
                build_string("Permit was rejected", context)
            }
            Error::Blacklisted => {
                build_string("Cannot enter bond opportunity, sender address of {} is blacklisted", context)
            }
            Error::IssuedAssetDeposit => {
                build_string("Cannot deposit using this contract's issued asset", context)
            }
            Error::NotTreasuryBond => {
                build_string("Cannot perform function since this is not a treasury bond", context)
            }
            Error::NoBondsClaimable => {
                build_string("Pending bonds not redeemable, nothing claimed", context)
            }
        }
    }
}

const BOND_TARGET: &str = "bond";


pub fn bond_not_started(start: u64, current: u64) -> StdError {
    DetailedError::from_code(
        BOND_TARGET,
        Error::BondNotStarted,
        vec![&start.to_string(), &current.to_string()],
    )
    .to_error()
}

pub fn bond_ended(end: u64, current: u64) -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::BondEnded, vec![&end.to_string(), &current.to_string()]).to_error()
}

pub fn bond_limit_reached(limit: Uint128) -> StdError {
    let limit_string: String = limit.into();
    let limit_str: &str = limit_string.as_str();
    DetailedError::from_code(BOND_TARGET, Error::BondLimitReached, vec![limit_str]).to_error()
}

pub fn global_limit_reached(limit: Uint128) -> StdError {
    let limit_string: String = limit.into();
    let limit_str: &str = limit_string.as_str();
    DetailedError::from_code(BOND_TARGET, Error::GlobalLimitReached, vec![limit_str]).to_error()
}

pub fn mint_exceeds_limit(mint_amount: Uint128, available: Uint128) -> StdError{
    let mint_string: String = mint_amount.into();
    let mint_str= mint_string.as_str();
    let available_string: String = available.into();
    let available_str: &str = available_string.as_str();
    DetailedError::from_code(BOND_TARGET, Error::MintExceedsLimit, vec![mint_str, available_str]).to_error()
}

pub fn contract_not_active() -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::ContractNotActive, vec![""]).to_error()
}

pub fn no_bond_found(collateral_asset_address: &str) -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::NoBondFound, vec![collateral_asset_address]).to_error()
}

pub fn no_pending_bonds(account_address: &str) -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::NoPendingBonds, vec![account_address]).to_error()
}

pub fn incorrect_viewing_key() -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::IncorrectViewingKey, vec![]).to_error()
}

pub fn bond_limit_exceeds_global_limit(global_issuance_limit: Uint128, global_total_issued: Uint128, bond_issuance_limit: Uint128) -> StdError {
    //let global_limit_str = global_issuance_limit.to_string().as_str();
    //let global_issued_str = global_issuance_limit.to_string().as_str();
    let available = (global_issuance_limit - global_total_issued).unwrap();
    let available_string = available.to_string();
    let available_str = available_string.as_str();
    let bond_limit_string = bond_issuance_limit.to_string();
    let bond_limit_str = bond_limit_string.as_str();
    DetailedError::from_code(BOND_TARGET, Error::BondLimitExceedsGlobalLimit, vec![bond_limit_str, available_str]).to_error()
}

pub fn bonding_period_below_minimum_time(bond_period: u64, global_minimum_bonding_period: u64) -> StdError {
    let bond_period_string = bond_period.to_string();
    let bond_period_str = bond_period_string.as_str();
    let global_minimum_bonding_period_string = global_minimum_bonding_period.to_string();
    let global_minimum_bonding_period_str = global_minimum_bonding_period_string.as_str();
    DetailedError::from_code(BOND_TARGET, Error::BondingPeriodBelowMinimumTime, vec![bond_period_str, global_minimum_bonding_period_str]).to_error()
}

pub fn bond_discount_above_maximum_rate(bond_discount: Uint128, global_maximum_discount: Uint128) -> StdError {
    let bond_discount_string = bond_discount.to_string();
    let bond_discount_str = bond_discount_string.as_str();
    let global_maximum_discount_string = global_maximum_discount.to_string();
    let global_maximum_discount_str = global_maximum_discount_string.as_str();
    DetailedError::from_code(BOND_TARGET, Error::BondDiscountAboveMaximumRate, vec![bond_discount_str, global_maximum_discount_str]).to_error()
}

pub fn bond_issuance_exceeds_allowance(snip20_allowance: Uint128, allocated_allowance: Uint128, bond_limit: Uint128) -> StdError {
    //let snip20_allowance_string = snip20_allowance.to_string();
    //let snip20_allowance_str = snip20_allowance_string.as_str();
    //let allocated_allowance_string = allocated_allowance.to_string();
    //let allocated_allowance_str = allocated_allowance_string.as_str();
    let available = (snip20_allowance - allocated_allowance).unwrap();
    let available_string = available.to_string();
    let available_str = available_string.as_str();
    let bond_limit_string = bond_limit.to_string();
    let bond_limit_str = bond_limit_string.as_str();
    DetailedError::from_code(BOND_TARGET, Error::BondIssuanceExceedsAllowance, vec![bond_limit_str, available_str]).to_error()
}

pub fn not_limit_admin() -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::NotLimitAdmin, vec![]).to_error()
}

pub fn collateral_price_exceeds_limit(collateral_price: Uint128, limit: Uint128) -> StdError {
    let collateral_string = collateral_price.to_string();
    let collateral_str = collateral_string.as_str();
    let limit_string = limit.to_string();
    let limit_str = limit_string.as_str();
    DetailedError::from_code(BOND_TARGET, Error::CollateralPriceExceedsLimit, vec![collateral_str, limit_str]).to_error()
}

pub fn issued_price_below_minimum(issued_price: Uint128, limit: Uint128) -> StdError {
    let issued_string = issued_price.to_string();
    let issued_str = issued_string.as_str();
    let limit_string = limit.to_string();
    let limit_str = limit_string.as_str();
    DetailedError::from_code(BOND_TARGET, Error::IssuedPriceBelowMinimum, vec![issued_str, limit_str]).to_error()
}

pub fn slippage_tolerance_exceeded(amount_to_issue: Uint128, min_expected_amount: Uint128) -> StdError {
    let issue_string = amount_to_issue.to_string();
    let issue_str = issue_string.as_str();
    let min_amount_string = min_expected_amount.to_string();
    let min_amount_str = min_amount_string.as_str(); 
    DetailedError::from_code(BOND_TARGET, Error::SlippageToleranceExceeded, vec![issue_str, min_amount_str]).to_error()
}

pub fn permit_contract_mismatch(contract: &str, expected: &str) -> StdError {
    DetailedError::from_code(
        BOND_TARGET,
        Error::PermitContractMismatch,
        vec![contract, expected],
    )
    .to_error()
}

pub fn permit_key_revoked(key: &str) -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::PermitKeyRevoked, vec![key]).to_error()
}

pub fn permit_rejected() -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::PermitRejected, vec![]).to_error()
}

pub fn blacklisted(address: HumanAddr) -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::Blacklisted, vec![address.as_str()]).to_error()
}

pub fn issued_asset_deposit() -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::IssuedAssetDeposit, vec![]).to_error()
}

pub fn not_treasury_bond() -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::NotTreasuryBond, vec![]).to_error()
}

pub fn no_bonds_claimable() -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::NoBondsClaimable, vec![]).to_error()
}