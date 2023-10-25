//! ### Liquidity Book Type Library
//! Author: Kent
//!
//! This library contains common types used throughout the project.

use cosmwasm_schema::cw_serde;
use cosmwasm_std::ContractInfo;
use ethnum::U256;

pub use crate::utils::liquidity_book::math::{
    liquidity_configurations::LiquidityConfigurations, tree_math::TreeUint24,
};
pub use crate::utils::liquidity_book::tokens::TokenType;

pub type Bytes32 = [u8; 32];

/// Info needed to instantiate a contract.
#[cw_serde]
#[derive(Default)]
pub struct ContractInstantiationInfo {
    pub id: u64,
    pub code_hash: String,
}
/// Pair parameters that don't change.
/// * `base_factor`
/// * `filter_period`
/// * `decay_period`
/// * `reduction_factor`
/// * `variable_fee_control`
/// * `protocol_share`
/// * `max_volatility_accumulator`
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
    pub lb_pair: LBPair,
    pub created_by_owner: bool,
    pub ignored_for_routing: bool,
}
