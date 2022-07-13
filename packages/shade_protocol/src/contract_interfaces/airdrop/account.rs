use crate::contract_interfaces::airdrop::errors::permit_rejected;
use crate::math_compat::Uint128;
use crate::c_std::{from_binary, Binary, Addr, StdError, StdResult, Api};
use crate::query_authentication::{
    permit::{bech32_to_canonical, Permit},
    transaction::SignedTx,
    viewing_keys::ViewingKey,
};

use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub addresses: Vec<Addr>,
    pub total_claimable: Uint128,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            addresses: vec![],
            total_claimable: Uint128::zero(),
        }
    }
}

// Used for querying account information
pub type AccountPermit = Permit<AccountPermitMsg>;

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AccountPermitMsg {
    pub contract: Addr,
    pub key: String,
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct FillerMsg {
    pub coins: Vec<String>,
    pub contract: String,
    pub execute_msg: EmptyMsg,
    pub sender: String,
}

impl Default for FillerMsg {
    fn default() -> Self {
        Self {
            coins: vec![],
            contract: "".to_string(),
            sender: "".to_string(),
            execute_msg: EmptyMsg {},
        }
    }
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct EmptyMsg {}

// Used to prove ownership over IBC addresses
pub type AddressProofPermit = Permit<FillerMsg>;

pub fn authenticate_ownership<A: Api>(api: &A, permit: &AddressProofPermit, permit_address: &str) -> StdResult<()> {
    let signer_address = permit
        .validate(api, Some("wasm/MsgExecuteContract".to_string()))?
        .as_canonical();

    if signer_address != bech32_to_canonical(permit_address) {
        return Err(permit_rejected());
    }

    Ok(())
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AddressProofMsg {
    // Address is necessary since we have other network permits present
    pub address: Addr,
    // Reward amount
    pub amount: Uint128,
    // Used to prevent permits from being used elsewhere
    pub contract: Addr,
    // Index of the address in the leaves array
    pub index: u32,
    // Used to identify permits
    pub key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AccountKey(pub String);

impl ToString for AccountKey {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl ViewingKey<32> for AccountKey {}
