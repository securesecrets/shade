use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Binary, Uint128};
use crate::signature::Permit;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PermitSignature {
    pub pub_key: PubKey,
    pub signature: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PubKey {
    /// ignored, but must be "tendermint/PubKeySecp256k1" otherwise the verification will fail
    pub r#type: String,
    /// Secp256k1 PubKey
    pub value: Binary,
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TxMsg<T> {
    // Must be "tendermint/PubKeySecp256k1"
    pub r#type: String,
    pub value: T,
}

impl<T: Clone + Serialize> TxMsg<T> {
    pub fn from_permit(permit: &Permit<T>) -> Self {
        Self {
            r#type: "signature_proof".to_string(),
            value: permit.params.clone(),
        }
    }
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SignedTx<T> {
    /// ignored
    pub account_number: Uint128,
    /// ignored, no Env in query
    pub chain_id: String,
    /// ignored
    pub fee: Fee,
    /// ignored
    pub memo: String,
    /// the signed message
    pub msgs: Vec<TxMsg<T>>,
    /// ignored
    pub sequence: Uint128,
}

impl<T: Clone + Serialize> SignedTx<T> {
    pub fn from_msg(item: TxMsg<T>, chain_id: String) -> Self {
        Self {
            account_number: Uint128::zero(),
            chain_id,
            fee: Fee::new(),
            memo: String::new(),
            msgs: vec![item],
            sequence: Uint128::zero(),
        }
    }
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Fee {
    pub amount: Vec<Coin>,
    pub gas: Uint128,
}

impl Fee {
    pub fn new() -> Self {
        Self {
            amount: vec![Coin::new()],
            gas: Uint128(1),
        }
    }
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Coin {
    pub amount: Uint128,
    pub denom: String,
}

impl Coin {
    pub fn new() -> Self {
        Self {
            amount: Uint128::zero(),
            denom: "uscrt".to_string(),
        }
    }
}