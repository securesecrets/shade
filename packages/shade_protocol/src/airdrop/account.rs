use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128, StdResult, StdError};
use flexible_permits::permit::{bech32_to_canonical, Permit};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub addresses: Vec<HumanAddr>,
    pub total_claimable: Uint128,
}

// Used for querying account information
pub type AccountPermit = Permit<AccountPermitMsg>;

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AccountPermitMsg {
    pub contract: HumanAddr,
    pub key: String,
}

// Used to prove ownership over IBC addresses
pub type AddressProofPermit = Permit<AddressProofMsg>;

pub fn authenticate_ownership(permit: &AddressProofPermit) -> StdResult<HumanAddr> {
    let permit_address = permit.params.address.clone();
    let signer_address = permit.validate()?.as_canonical();
    if signer_address != bech32_to_canonical(permit_address.as_str()) {
        return Err(StdError::generic_err(
            format!("{:?} is not the message signer", permit_address.as_str())))
    }
    Ok(permit_address)
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AddressProofMsg {
    // Address is necessary since we have other network permits present
    pub address: HumanAddr,
    // Reward amount
    pub amount: Uint128,
    // Used to prevent permits from being used elsewhere
    pub contract: HumanAddr,
    // Index of the address in the leaves array
    pub index: u32,
    // Used to identify permits
    pub key: String,
}