//! ### Liquidity Book Type Library
//! Author: Kent and Haseeb
//!
//! This library contains common types used throughout the project.

pub use super::math::{liquidity_configurations::LiquidityConfigurations, tree_math::TreeUint24};
use crate::{c_std::ContractInfo, cosmwasm_schema::cw_serde};
use ethnum::U256;
// TODO - Try to not use this type in the liquidity_book module, because it's gated by the "swap"
// feature.
use crate::contract_interfaces::swap::core::TokenType;

pub type Bytes32 = [u8; 32];

// TODO - This type belongs somewhere else. It's not specific to liquidity_book.
#[cw_serde]
#[derive(Default)]
pub struct ContractInstantiationInfo {
    pub id: u64,
    pub code_hash: String,
}

#[cw_serde]
pub struct StaticFeeParameters {
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub variable_fee_control: u32,
    pub protocol_share: u16,
    pub max_volatility_accumulator: u32,
}

#[derive(Clone, Debug)]
pub struct MintArrays {
    pub ids: Vec<U256>,
    pub amounts: Vec<Bytes32>,
    pub liquidity_minted: Vec<U256>,
}

#[cw_serde]
pub struct LBPair {
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub contract: ContractInfo,
}

#[cw_serde]
pub struct LBPairInformation {
    pub bin_step: u16,
    pub info: LBPair,
    pub created_by_owner: bool,
    pub ignored_for_routing: bool,
}
