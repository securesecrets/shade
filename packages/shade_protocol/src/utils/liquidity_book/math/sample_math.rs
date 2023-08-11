//! ### Liquidity Book Sample Math Library
//! Author: Kent
//!
//! This library contains functions to encode and decode a sample into a single Bytes32
//! and interact with the encoded Bytes32.
//!
//! The sample is encoded as follows:
//! * 0 - 16: oracle length (16 bits)
//! * 16 - 80: cumulative id (64 bits)
//! * 80 - 144: cumulative volatility accumulator (64 bits)
//! * 144 - 208: cumulative bin crossed (64 bits)
//! * 208 - 216: sample lifetime (8 bits)
//! * 216 - 256: sample creation timestamp (40 bits)

use cosmwasm_schema::cw_serde;

use super::encoded_sample::*;

pub const OFFSET_ORACLE_LENGTH: u8 = 0;
pub const OFFSET_CUMULATIVE_ID: u8 = 16;
pub const OFFSET_CUMULATIVE_VOLATILITY: u8 = 80;
pub const OFFSET_CUMULATIVE_BIN_CROSSED: u8 = 144;
pub const OFFSET_SAMPLE_LIFETIME: u8 = 208;
pub const OFFSET_SAMPLE_CREATION: u8 = 216;

#[cw_serde]
#[derive(Copy)]
pub struct OracleSample(pub EncodedSample);

impl OracleSample {
    /// Encodes a sample.
    ///
    /// # Arguments
    ///
    /// * `oracle_length` - The oracle length
    /// * `cumulative_id` - The cumulative id
    /// * `cumulative_volatility` - The cumulative volatility
    /// * `cumulative_bin_crossed` - The cumulative bin crossed
    /// * `sample_lifetime` - The sample lifetime
    /// * `created_at` - The sample creation timestamp
    pub fn encode(
        oracle_length: u16,
        cumulative_id: u64,
        cumulative_volatility: u64,
        cumulative_bin_crossed: u64,
        sample_lifetime: u8,
        // TODOL create a uint40 type?
        created_at: u64,
    ) -> OracleSample {
        let mut sample = EncodedSample([0u8; 32]);

        // TODO: are all these .into() really necessary?
        sample = sample.set(oracle_length.into(), MASK_UINT16, OFFSET_ORACLE_LENGTH);
        sample = sample.set(cumulative_id.into(), MASK_UINT64, OFFSET_CUMULATIVE_ID);
        sample = sample.set(
            cumulative_volatility.into(),
            MASK_UINT64,
            OFFSET_CUMULATIVE_VOLATILITY,
        );
        sample = sample.set(
            cumulative_bin_crossed.into(),
            MASK_UINT64,
            OFFSET_CUMULATIVE_BIN_CROSSED,
        );
        sample = sample.set(sample_lifetime.into(), MASK_UINT8, OFFSET_SAMPLE_LIFETIME);
        sample = sample.set(created_at.into(), MASK_UINT40, OFFSET_SAMPLE_CREATION);

        OracleSample(sample)
    }

    /// Gets the oracle length from an encoded sample.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [0 - 16[: oracle length (16 bits)
    ///     * [16 - 256[: any (240 bits)
    pub fn get_oracle_length(&self) -> u16 {
        self.0.decode_uint16(0)
    }

    /// Gets the cumulative id from an encoded sample.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [0 - 16[: any (16 bits)
    ///     * [16 - 80[: cumulative id (64 bits)
    ///     * [80 - 256[: any (176 bits)
    pub fn get_cumulative_id(&self) -> u64 {
        self.0.decode_uint64(OFFSET_CUMULATIVE_ID)
    }

    /// Gets the cumulative volatility accumulator from an encoded sample.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [0 - 80[: any (80 bits)
    ///     * [80 - 144[: cumulative volatility accumulator (64 bits)
    ///     * [144 - 256[: any (112 bits)
    pub fn get_cumulative_volatility(&self) -> u64 {
        self.0.decode_uint64(OFFSET_CUMULATIVE_VOLATILITY)
    }

    /// Gets the cumulative bin crossed from an encoded sample.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [0 - 144[: any (144 bits)
    ///     * [144 - 208[: cumulative bin crossed (64 bits)
    ///     * [208 - 256[: any (48 bits)
    pub fn get_cumulative_bin_crossed(&self) -> u64 {
        self.0.decode_uint64(OFFSET_CUMULATIVE_BIN_CROSSED)
    }

    /// Gets the sample lifetime from an encoded sample.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [0 - 208[: any (208 bits)
    ///     * [208 - 216[: sample lifetime (8 bits)
    ///     * [216 - 256[: any (40 bits)
    pub fn get_sample_lifetime(&self) -> u8 {
        self.0.decode_uint8(OFFSET_SAMPLE_LIFETIME)
    }

    /// Gets the sample creation timestamp from an encoded sample.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [0 - 216[: any (216 bits)
    ///     * [216 - 256[: sample creation timestamp (40 bits)
    pub fn get_sample_creation(&self) -> u64 {
        self.0.decode_uint64(OFFSET_SAMPLE_CREATION)
    }

    /// Gets the sample last update timestamp from an encoded sample.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [0 - 216[: any (216 bits)
    ///     * [216 - 256[: sample creation timestamp (40 bits)
    pub fn get_sample_last_update(&self) -> u64 {
        self.get_sample_creation() + self.get_sample_lifetime() as u64
    }

    /// Gets the weighted average of two samples and their respective weights.
    ///
    /// # Arguments
    ///
    /// * `sample1` - The first encoded sample
    /// * `sample2` - The second encoded sample
    /// * `weight1` - The weight of the first sample
    /// * `weight2` - The weight of the second sample
    ///
    /// # Returns
    ///
    /// * `weighted_average_id` - The weighted average id
    /// * `weighted_average_volatility` - The weighted average volatility
    /// * `weighted_average_bin_crossed` - The weighted average bin crossed
    pub fn get_weighted_average(
        sample1: OracleSample,
        sample2: OracleSample,
        weight1: u64,
        weight2: u64,
    ) -> (u64, u64, u64) {
        let c_id1 = sample1.get_cumulative_id();
        let c_volatility1 = sample1.get_cumulative_volatility();
        let c_bin_crossed1 = sample1.get_cumulative_bin_crossed();

        if weight2 == 0 {
            return (c_id1, c_volatility1, c_bin_crossed1);
        }

        let c_id2 = sample2.get_cumulative_id();
        let c_volatility2 = sample2.get_cumulative_volatility();
        let c_bin_crossed2 = sample2.get_cumulative_bin_crossed();

        if weight1 == 0 {
            return (c_id2, c_volatility2, c_bin_crossed2);
        }

        let total_weight = weight1 + weight2;

        let weighted_average_id = (c_id1 * weight1 + c_id2 * weight2) / total_weight;
        let weighted_average_volatility =
            (c_volatility1 * weight1 + c_volatility2 * weight2) / total_weight;
        let weighted_average_bin_crossed =
            (c_bin_crossed1 * weight1 + c_bin_crossed2 * weight2) / total_weight;

        (
            weighted_average_id,
            weighted_average_volatility,
            weighted_average_bin_crossed,
        )
    }

    /// Updates a sample with the given values.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample
    /// * `delta_time` - The time elapsed since the last update
    /// * `active_id` - The active id
    /// * `volatility_accumulator` - The volatility accumulator
    /// * `bin_crossed` - The bin crossed
    ///
    /// # Returns
    ///
    /// * `cumulative_id` - The cumulative id
    /// * `cumulative_volatility` - The cumulative volatility
    /// * `cumulative_bin_crossed` - The cumulative bin crossed
    pub fn update(
        self,
        delta_time: u64,
        active_id: u32,
        volatility_accumulator: u32,
        bin_crossed: u32,
    ) -> (u64, u64, u64) {
        let cumulative_id = u64::from(active_id) * delta_time;
        let cumulative_volatility = u64::from(volatility_accumulator) * delta_time;
        let cumulative_bin_crossed = u64::from(bin_crossed) * delta_time;

        let cumulative_id = cumulative_id + self.get_cumulative_id();
        let cumulative_volatility = cumulative_volatility + self.get_cumulative_volatility();
        let cumulative_bin_crossed = cumulative_bin_crossed + self.get_cumulative_bin_crossed();

        (cumulative_id, cumulative_volatility, cumulative_bin_crossed)
    }
}
