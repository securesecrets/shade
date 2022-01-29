use cosmwasm_std::{from_binary, Binary, HumanAddr, StdError, StdResult, Uint128};
use flexible_permits::{
    permit::{bech32_to_canonical, Permit},
    transaction::SignedTx,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EmptyMsg {}

// Used to prove ownership over IBC addresses
pub type AddressProofPermit = Permit<FillerMsg>;

pub fn authenticate_ownership(permit: &AddressProofPermit) -> StdResult<HumanAddr> {
    if let Some(memo) = permit.memo.clone() {
        let params: AddressProofMsg = from_binary(&Binary::from_base64(&memo)?)?;
        let permit_address = params.address.clone();

        let signer_address = permit
            .validate(Some("wasm/MsgExecuteContract".to_string()))?
            .as_canonical();

        if signer_address != bech32_to_canonical(permit_address.as_str()) {
            return Err(StdError::generic_err(format!(
                "{:?} is not the message signer",
                permit_address.as_str()
            )));
        }
        return Ok(permit_address);
    }
    Err(StdError::generic_err("Expected a memo"))
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
