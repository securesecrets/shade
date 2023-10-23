use cosmwasm_std::{
    Addr, Decimal256, Uint256
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::core::{ContractLink, TokenPair, Fee, TokenType};
use shade_protocol::Contract;

/// Represents the address of an exchange and the pair that it manages
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
pub struct AMMPair {
    /// The pair that the contract manages.
    pub pair: TokenPair,
    /// Address of the contract that manages the exchange.
    pub address: Addr,
    /// Used to enable or disable the AMMPair
    pub enabled: bool
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StableParams {
    pub a: Decimal256,
    pub gamma1: Uint256,
    pub gamma2: Uint256,
    pub oracle: Contract,
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug,Clone)]
pub struct AMMSettings {
    pub lp_fee: Fee,
    pub shade_dao_fee: Fee,
    pub stable_lp_fee: Fee,
    pub stable_shade_dao_fee: Fee,
    pub shade_dao_address: ContractLink
}

pub fn generate_pair_key(pair: &TokenPair) -> Vec<u8> {
    let mut bytes: Vec<&[u8]> = Vec::new();

    match &pair.0 {
        TokenType::NativeToken { denom, ..} => bytes.push(denom.as_bytes()),
        TokenType::CustomToken { contract_addr, .. } => bytes.push(contract_addr.as_bytes())
    }

    match &pair.1 {
        TokenType::NativeToken { denom, .. } => bytes.push(denom.as_bytes()),
        TokenType::CustomToken { contract_addr, .. } => bytes.push(contract_addr.as_bytes())
    }

    bytes.sort();

    bytes.concat()
}