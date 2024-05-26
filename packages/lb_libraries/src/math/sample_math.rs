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

use super::{encoded_sample::*, packed_u128_math::PackedUint128Math};
use crate::types::Bytes32;
use cosmwasm_schema::cw_serde;

pub const OFFSET_CUMULATIVE_TXNS: u8 = 0;
pub const OFFSET_CUMULATIVE_ID: u8 = 16;
pub const OFFSET_CUMULATIVE_VOLATILITY: u8 = 80;
pub const OFFSET_CUMULATIVE_BIN_CROSSED: u8 = 144;
pub const OFFSET_SAMPLE_LIFETIME: u8 = 208;
pub const OFFSET_SAMPLE_CREATION: u8 = 216;

#[cw_serde]
#[derive(Copy, Default)]
pub struct OracleSample {
    pub data: EncodedSample,
    pub volume: EncodedSample,
    pub fee: EncodedSample,
}

impl OracleSample {
    /// Encodes a sample.
    ///
    /// # Arguments
    ///
    /// * `cumulative_txns` - The number of transactions
    /// * `cumulative_id` - The cumulative id
    /// * `cumulative_volatility` - The cumulative volatility
    /// * `cumulative_bin_crossed` - The cumulative bin crossed
    /// * `sample_lifetime` - The sample lifetime
    /// * `created_at` - The sample creation timestamp
    pub fn encode(
        cumulative_txns: u16,
        cumulative_id: u64,
        cumulative_volatility: u64,
        cumulative_bin_crossed: u64,
        sample_lifetime: u8,
        created_at: u64,
        cumulative_vol: Bytes32,
        cumulative_fee: Bytes32,
    ) -> OracleSample {
        let mut sample = EncodedSample::default();

        sample.set(cumulative_txns.into(), MASK_UINT16, OFFSET_CUMULATIVE_TXNS);
        sample.set(cumulative_id.into(), MASK_UINT64, OFFSET_CUMULATIVE_ID);
        sample.set(
            cumulative_volatility.into(),
            MASK_UINT64,
            OFFSET_CUMULATIVE_VOLATILITY,
        );
        sample.set(
            cumulative_bin_crossed.into(),
            MASK_UINT64,
            OFFSET_CUMULATIVE_BIN_CROSSED,
        );
        sample.set(sample_lifetime.into(), MASK_UINT8, OFFSET_SAMPLE_LIFETIME);
        sample.set(created_at.into(), MASK_UINT40, OFFSET_SAMPLE_CREATION);

        OracleSample {
            data: sample,
            volume: EncodedSample(cumulative_vol),
            fee: EncodedSample(cumulative_fee),
        }
    }

    /// Gets the cumulative txns from encoded sample
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [0 - 16[: oracle length (16 bits)
    ///     * [16 - 256[: any (240 bits)
    pub fn get_cumulative_txns(&self) -> u16 {
        self.data.decode_uint16(0)
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
        self.data.decode_uint64(OFFSET_CUMULATIVE_ID)
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
        self.data.decode_uint64(OFFSET_CUMULATIVE_VOLATILITY)
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
        self.data.decode_uint64(OFFSET_CUMULATIVE_BIN_CROSSED)
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
        self.data.decode_uint8(OFFSET_SAMPLE_LIFETIME)
    }

    /// Gets the sample creation timestamp from an encoded sample.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [0 - 216[: any (216 bits)
    ///     * [216 - 256[: sample creation timestamp (40 bits)
    pub fn get_sample_creation(&self) -> u64 {
        self.data.decode_uint40(OFFSET_SAMPLE_CREATION)
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

    /// Gets the sample creation timestamp from an encoded sample.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [0 - 128[: any (128 bits)
    pub fn get_vol_token_x(&self) -> u128 {
        self.volume.decode_uint128(0)
    }

    /// Gets the sample creation timestamp from an encoded sample.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [128 - 256[: any (128 bits)
    pub fn get_vol_token_y(&self) -> u128 {
        self.volume.decode_uint128(128)
    }

    /// Gets the sample creation timestamp from an encoded sample.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [0 - 128[: any (128 bits)
    pub fn get_fee_token_x(&self) -> u128 {
        self.fee.decode_uint128(0)
    }

    /// Gets the sample creation timestamp from an encoded sample.
    ///
    /// # Arguments
    ///
    /// * `sample` - The encoded sample as follows:
    ///     * [128 - 256[: any (128 bits)
    pub fn get_fee_token_y(&self) -> u128 {
        self.fee.decode_uint128(128)
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
        vol: Bytes32,
        fee: Bytes32,
    ) -> (u16, u64, u64, u64, Bytes32, Bytes32) {
        let mut cumulative_txns = self.get_cumulative_txns();
        let cumulative_id = u64::from(active_id) * delta_time;
        let cumulative_volatility = u64::from(volatility_accumulator) * delta_time;
        let cumulative_bin_crossed = u64::from(bin_crossed) * delta_time;

        let cumulative_id = cumulative_id + self.get_cumulative_id();
        let cumulative_volatility = cumulative_volatility + self.get_cumulative_volatility();
        let cumulative_bin_crossed = cumulative_bin_crossed + self.get_cumulative_bin_crossed();

        if !(vol == [0u8; 32]) && !(fee == [0u8; 32]) {
            cumulative_txns += 1;
        }

        let cumm_vol = vol.add(self.volume.0);
        let cumm_fee = fee.add(self.fee.0);
        (
            cumulative_txns,
            cumulative_id,
            cumulative_volatility,
            cumulative_bin_crossed,
            cumm_vol,
            cumm_fee,
        )
    }

    /// Set the creation_time in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `created_at` - Time of the creation
    pub fn set_created_at(&mut self, created_at: u64) -> &mut Self {
        self.data
            .set(created_at.into(), MASK_UINT40, OFFSET_SAMPLE_CREATION);
        self
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Uint256;

    use super::*;

    #[test]
    fn test_encode() {
        let vol = Uint256::from(123u128).to_le_bytes();
        let fee = Uint256::from(123u128).to_le_bytes();
        let sample = OracleSample::encode(3, 1000, 2000, 3000, 4, 123456, vol, fee);
        assert_eq!(sample.get_cumulative_txns(), 3);
        assert_eq!(sample.get_cumulative_id(), 1000);
        assert_eq!(sample.get_cumulative_volatility(), 2000);
        assert_eq!(sample.get_cumulative_bin_crossed(), 3000);
        assert_eq!(sample.get_sample_lifetime(), 4);
        assert_eq!(sample.get_sample_creation(), 123456);
    }

    #[test]
    fn test_get_cumulative_txns() {
        let vol = Uint256::from(123u128).to_le_bytes();
        let fee = Uint256::from(123u128).to_le_bytes();
        let sample = OracleSample::encode(3, 1000, 2000, 3000, 4, 123456, vol, fee);
        assert_eq!(sample.get_cumulative_txns(), 3);
    }

    #[test]
    fn test_get_cumulative_id() {
        let vol = Uint256::from(123u128).to_le_bytes();
        let fee = Uint256::from(123u128).to_le_bytes();
        let sample = OracleSample::encode(3, 1000, 2000, 3000, 4, 123456, vol, fee);
        assert_eq!(sample.get_cumulative_id(), 1000);
    }

    #[test]
    fn test_get_cumulative_volatility() {
        let vol = Uint256::from(123u128).to_le_bytes();
        let fee = Uint256::from(123u128).to_le_bytes();
        let sample = OracleSample::encode(3, 1000, 2000, 3000, 4, 123456, vol, fee);
        assert_eq!(sample.get_cumulative_volatility(), 2000);
    }

    #[test]
    fn test_get_cumulative_bin_crossed() {
        let vol = Uint256::from(123u128).to_le_bytes();
        let fee = Uint256::from(123u128).to_le_bytes();
        let sample = OracleSample::encode(3, 1000, 2000, 3000, 4, 123456, vol, fee);
        assert_eq!(sample.get_cumulative_bin_crossed(), 3000);
    }

    #[test]
    fn test_get_sample_lifetime() {
        let vol = Uint256::from(123u128).to_le_bytes();
        let fee = Uint256::from(123u128).to_le_bytes();
        let sample = OracleSample::encode(3, 1000, 2000, 3000, 4, 123456, vol, fee);
        assert_eq!(sample.get_sample_lifetime(), 4);
    }

    #[test]
    fn test_get_sample_creation() {
        let vol = Uint256::from(123u128).to_le_bytes();
        let fee = Uint256::from(123u128).to_le_bytes();
        let sample = OracleSample::encode(3, 1000, 2000, 3000, 4, 123456, vol, fee);
        assert_eq!(sample.get_sample_creation(), 123456);
    }

    #[test]
    fn test_get_sample_last_update() {
        let vol = Uint256::from(123u128).to_le_bytes();
        let fee = Uint256::from(123u128).to_le_bytes();
        let sample = OracleSample::encode(3, 1000, 2000, 3000, 4, 123456, vol, fee);
        assert_eq!(sample.get_sample_last_update(), 123460); // 123456 + 4
    }

    #[test]
    fn test_get_weighted_average() {
        let vol = Uint256::from(123u128).to_le_bytes();
        let fee = Uint256::from(123u128).to_le_bytes();
        let sample1 = OracleSample::encode(3, 1000, 2000, 3000, 4, 123456, vol, fee);
        let sample2 = OracleSample::encode(3, 2000, 4000, 6000, 4, 123456, vol, fee);
        let (avg_id, avg_vol, avg_bin) = OracleSample::get_weighted_average(sample1, sample2, 1, 1);
        assert_eq!(avg_id, 1500);
        assert_eq!(avg_vol, 3000);
        assert_eq!(avg_bin, 4500);
    }

    #[test]
    fn test_update() {
        let vol = Uint256::from(123u128).to_le_bytes();
        let fee = Uint256::from(123u128).to_le_bytes();

        let sample = OracleSample::encode(3, 1000, 2000, 3000, 4, 123456, vol, fee);
        let (
            cumulative_txns,
            cumulative_id,
            cumulative_volatility,
            cumulative_bin_crossed,
            cumm_vol,
            cumm_fee,
        ) = sample.update(1, 1000, 2000, 3000, vol, fee);

        assert_eq!(cumulative_txns, 4);
        assert_eq!(cumulative_id, 2000);
        assert_eq!(cumulative_volatility, 4000);
        assert_eq!(cumulative_bin_crossed, 6000);
        assert_eq!(cumm_vol, vol.add(vol));
        assert_eq!(cumm_fee, fee.add(fee));
    }
}
