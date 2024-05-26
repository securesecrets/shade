//! ### Liquidity Book Type Library
//! Author: Kent and Haseeb
//!
//! This library contains common types used throughout the project.

use cosmwasm_schema::cw_serde;

pub use crate::math::{liquidity_configurations::LiquidityConfigurations, tree_math::TreeUint24};

pub type Bytes32 = [u8; 32];

// TODO: move this type somewhere else?

#[cw_serde]
#[derive(Default)]
pub struct ContractImplementation {
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
