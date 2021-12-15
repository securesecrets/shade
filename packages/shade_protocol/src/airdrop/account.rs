use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128, StdResult, StdError};
use crate::signature::{Permit, bech32_to_canonical};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub addresses: Vec<HumanAddr>,
    pub total_claimable: Uint128,
}

pub type AddressProofPermit = Permit<AddressProofMsg>;

impl AddressProofPermit {
    /// Will check if signer is the same as the given address
    pub fn authenticate(&self) -> StdResult<HumanAddr> {
        let permit_address = self.params.address.clone();
        let signer_address = self.validate()?;
        if signer_address != bech32_to_canonical(permit_address.as_str()) {
            return Err(StdError::generic_err(
                format!("{:?} is not the message signer", permit_address.as_str())))
        }
        Ok(permit_address)
    }
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AddressProofMsg {
    pub address: HumanAddr,
    // Reward amount
    pub amount: Uint128,
    // Used to prevent permits from being used elsewhere
    pub contract: HumanAddr,
    // Index of the address in the leafs array
    pub index: u32,
    // Used to ban permits
    pub key: String,
}