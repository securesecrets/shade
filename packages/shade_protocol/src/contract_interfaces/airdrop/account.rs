use crate::contract_interfaces::airdrop::errors::permit_rejected;
use crate::c_std::Uint128;
use crate::c_std::{Addr, StdResult, Api};
use crate::query_authentication::{
    permit::{bech32_to_canonical, Permit},
    viewing_keys::ViewingKey,
};

use cosmwasm_schema::{cw_serde};

#[cw_serde]
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
#[cw_serde]
pub struct AccountPermitMsg {
    pub contract: Addr,
    pub key: String,
}

#[remain::sorted]
#[cw_serde]
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
#[cw_serde]
pub struct EmptyMsg {}

// Used to prove ownership over IBC addresses
pub type AddressProofPermit = Permit<FillerMsg>;

pub fn authenticate_ownership(api: &dyn Api, permit: &AddressProofPermit, permit_address: &str) -> StdResult<()> {
    let signer_address = permit
        .validate(api, Some("wasm/MsgExecuteContract".to_string()))?
        .as_canonical();

    if signer_address != bech32_to_canonical(permit_address) {
        return Err(permit_rejected());
    }

    Ok(())
}

#[remain::sorted]
#[cw_serde]
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

#[cw_serde]
pub struct AccountKey(pub String);

impl ToString for AccountKey {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl ViewingKey<32> for AccountKey {}
