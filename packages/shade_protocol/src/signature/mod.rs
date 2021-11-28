pub mod transaction;

use cosmwasm_std::{Binary, CanonicalAddr, Extern, HumanAddr, StdError, StdResult, to_binary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ripemd160::{Digest, Ripemd160};
use secp256k1::Secp256k1;

// NOTE: Struct order is very important for signatures

// Signature idea taken from https://github.com/scrtlabs/secret-toolkit/blob/token-permits/packages/permit/src/funcs.rs

/// Where the information will be stored
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Permit<T> {
    pub params: T,
    pub chain_id: String,
    pub signature: transaction::PermitSignature,
}

impl<T: Clone> Permit<T> {
    pub fn create_signed_tx(permit: &Self) -> transaction::SignedTx<T> {
        transaction::SignedTx::from_msg(
            transaction::TxMsg::from_permit(permit), permit.chain_id.clone())
    }

    /// Returns the permit signer
    pub fn validate(permit: &Self, deps: &Extern<S, A, Q>,) -> StdResult<HumanAddr> {
        let pubkey = &permit.signature.pub_key.value;
        let account = HumanAddr(pubkey_to_account(pubkey).to_string());

        // Validate signature
        let signed_bytes = to_binary(&SignedPermit::from_params(&permit.params))?;
        let signed_bytes_hash = secret_toolkit_crypto::sha_256(signed_bytes.as_slice());
        let secp256k1_msg = secp256k1::Message::from_slice(&signed_bytes_hash).map_err(
            |err| {
                StdError::generic_err(
                    format!(
                        "Failed to create a secp256k1 message from signed_bytes: {:?}", err))
            }
        )?;

        let secp256k1_verifier = Secp256k1::verification_only();

        let secp256k1_signature = secp256k1::Signature::from_compact(&permit.signature.signature.0)
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
    hasher.update(secret_toolkit_crypto::sha_256(&pubkey.0));
    CanonicalAddr(Binary(hasher.finalize().to_vec()))
}