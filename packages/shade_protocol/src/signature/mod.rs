pub mod transaction;

use cosmwasm_std::{Binary, CanonicalAddr, Extern, HumanAddr, StdError, StdResult, to_binary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ripemd160::{Digest, Ripemd160};
use secp256k1::Secp256k1;
use secret_toolkit::crypto::sha_256;
use crate::signature::transaction::{SignedTx, TxMsg, PermitSignature};

// NOTE: Struct order is very important for signatures

// Signature idea taken from https://github.com/scrtlabs/secret-toolkit/blob/token-permits/packages/permit/src/funcs.rs

/// Where the information will be stored
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Permit<T: Clone + Serialize> {
    pub params: T,
    pub chain_id: String,
    pub signature: PermitSignature,
}

impl<T: Clone + Serialize> Permit<T> {
    pub fn create_signed_tx(&self) -> SignedTx<T> {
        SignedTx::from_msg(
            TxMsg::from_permit(self), self.chain_id.clone())
    }

    /// Returns the permit signer
    pub fn validate(&self) -> StdResult<HumanAddr> {
        let pubkey = &self.signature.pub_key.value;
        let account = HumanAddr(pubkey_to_account(pubkey).to_string());

        // Validate signature
        let signed_bytes = to_binary(&self.create_signed_tx())?;
        let signed_bytes_hash = sha_256(signed_bytes.as_slice());
        let secp256k1_msg = secp256k1::Message::from_slice(&signed_bytes_hash).map_err(
            |err| {
                StdError::generic_err(
                    format!(
                        "Failed to create a secp256k1 message from signed_bytes: {:?}", err))
            }
        )?;

        let secp256k1_verifier = Secp256k1::verification_only();

        let secp256k1_signature = secp256k1::Signature::from_compact(&self.signature.signature.0)
            .map_err(|err| StdError::generic_err(format!("Malformed signature: {:?}", err)))?;
        let secp256k1_pubkey = secp256k1::PublicKey::from_slice(pubkey.0.as_slice())
            .map_err(|err| StdError::generic_err(format!("Malformed pubkey: {:?}", err)))?;

        secp256k1_verifier
            .verify(&secp256k1_msg, &secp256k1_signature, &secp256k1_pubkey)
            .map_err(|err| {
                StdError::generic_err(format!(
                    "Failed to verify signatures for the given permit: {:?}",
                    err
                ))
            })?;

        Ok(account)
    }
}

pub fn pubkey_to_account(pubkey: &Binary) -> CanonicalAddr {
    let mut hasher = Ripemd160::new();
    hasher.update(sha_256(&pubkey.0));
    CanonicalAddr(Binary(hasher.finalize().to_vec()))
}