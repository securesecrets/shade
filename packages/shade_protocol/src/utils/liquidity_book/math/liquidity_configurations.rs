//! ### Liquidity Book Liquidity Configurations Library
//! Author: Kent
//!
//! This library contains functions to encode and decode the config of a pool and interact with the encoded Bytes32.

use cosmwasm_schema::cw_serde;
use ethnum::U256;

use crate::utils::liquidity_book::types::Bytes32;

use super::encoded_sample::*;
use super::packed_u128_math::{Decode, Encode};

pub const OFFSET_ID: u8 = 0;
pub const OFFSET_DISTRIBUTION_Y: u8 = 24;
pub const OFFSET_DISTRIBUTION_X: u8 = 88;

pub const PRECISION: u64 = 1_000_000_000_000_000_000; // 1e18

#[derive(thiserror::Error, Debug)]
pub enum LiquidityConfigurationsError {
    #[error("Liquidity Configurations Error: Invalid Config")]
    InvalidConfig,
}

#[cw_serde]
pub struct LiquidityConfigurations(pub EncodedSample);

impl LiquidityConfigurations {
    /// Encode the distributionX, distributionY and id into a single Bytes32.
    ///
    /// # Arguments
    ///
    /// * `distribution_x` - The distribution of the first token
    /// * `distribution_y` - The distribution of the second token
    /// * `id` - The id of the pool
    ///
    /// # Returns
    ///
    /// * `config` - The encoded config as follows:
    ///     * `[`0 - 24[: id
    ///     * `[`24 - 88[: distributionY
    ///     * `[`88 - 152[: distributionX
    ///     * `[`152 - 256[: empty
    pub fn encode_params(distribution_x: u64, distribution_y: u64, id: u32) -> Bytes32 {
        let mut config = EncodedSample([0u8; 32]);
        config = config.set(distribution_x.into(), MASK_UINT64, OFFSET_DISTRIBUTION_X);
        config = config.set(distribution_y.into(), MASK_UINT64, OFFSET_DISTRIBUTION_Y);
        config = config.set(id.into(), MASK_UINT24, OFFSET_ID);

        config.0
    }

    /// Decode the distributionX, distributionY and id from a single Bytes32.
    ///
    /// # Arguments
    ///
    /// * `config` - The encoded config as follows:
    ///     * [0 - 24[: id
    ///     * [24 - 88[: distributionY
    ///     * [88 - 152[: distributionX
    ///     * [152 - 256[: empty
    ///
    /// # Returns
    ///
    /// * `distribution_x` - The distribution of the first token
    /// * `distribution_y` - The distribution of the second token
    /// * `id` - The id of the bin to add the liquidity to
    /// * `LiquidityConfigurationsError` - An error type for invalid config
    pub fn decode_params(
        config: EncodedSample,
    ) -> Result<(u64, u64, u32), LiquidityConfigurationsError> {
        let distribution_x = config.decode_uint64(OFFSET_DISTRIBUTION_X);
        let distribution_y = config.decode_uint64(OFFSET_DISTRIBUTION_Y);
        let id = config.decode_uint24(OFFSET_ID);

        let config_value = U256::from_le_bytes(config.0);
        // config_value must be less than 152 bits - see encoding in function doc
        if config_value > U256::ONE << 151
            || distribution_x > PRECISION
            || distribution_y > PRECISION
        {
            Err(LiquidityConfigurationsError::InvalidConfig)
        } else {
            Ok((distribution_x, distribution_y, id))
        }
    }

    /// Get the amounts and id from a config and amounts_in.
    ///
    /// # Arguments
    ///
    /// * `config` -
    /// The encoded config as follows:
    ///     * [0 - 24[: id
    ///     * [24 - 88[: distributionY
    ///     * [88 - 152[: distributionX
    ///     * [152 - 256[: empty
    /// * `amounts_in` - The amounts to distribute as follows:
    ///     * [0 - 128[: x1
    ///     * [128 - 256[: x2
    ///
    /// # Returns
    ///
    /// * `amounts` - The distributed amounts as follows:
    ///     * [0 - 128[: x1
    ///     * [128 - 256[: x2
    /// * `id` - The id of the bin to add the liquidity to
    /// * `LiquidityConfigurationsError` - An error type for invalid config
    pub fn get_amounts_and_id(
        config: EncodedSample,
        amounts_in: Bytes32,
    ) -> Result<(Bytes32, u32), LiquidityConfigurationsError> {
        let (distribution_x, distribution_y, id) = Self::decode_params(config)?;

        let (x1, x2) = amounts_in.decode();

        let x1_distributed = (U256::from(x1) * U256::from(distribution_x)) / U256::from(PRECISION);
        let x2_distributed = (U256::from(x2) * U256::from(distribution_y)) / U256::from(PRECISION);

        let amounts = Bytes32::encode(x1_distributed.as_u128(), x2_distributed.as_u128());

        Ok((amounts, id))
    }
}
