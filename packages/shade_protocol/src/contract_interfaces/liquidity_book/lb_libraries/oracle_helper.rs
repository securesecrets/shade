//! ### Liquidity Book Oracle Helper Library
//! Author: Kent
//!
//! This library contains functions to manage the oracle.
//! The oracle samples are stored in a HashMap with 65535 possible entries.
//!
//! Each sample is encoded as follows:
//! * 0 - 16: oracle length (16 bits)
//! * 16 - 80: cumulative id (64 bits)
//! * 80 - 144: cumulative volatility accumulator (64 bits)
//! * 144 - 208: cumulative bin crossed (64 bits)
//! * 208 - 216: sample lifetime (8 bits)
//! * 216 - 256: sample creation timestamp (40 bits)

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Timestamp;

use super::{
    math::sample_math::OracleSample,
    pair_parameter_helper::PairParameters,
    types::Bytes32,
};

#[cw_serde]
pub struct Oracle(
    /// This array represents a fixed-size storage for 65535 samples,
    /// where each sample is a 32-byte (256-bit) value.
    pub OracleSample,
);

pub const MAX_SAMPLE_LIFETIME: u8 = 120; //seconds

#[cw_serde]
#[derive(thiserror::Error)]
pub enum OracleError {
    #[error("Oracle Error: Invalid Oracle ID")]
    InvalidOracleId,
    #[error("Oracle Error: New length too small")]
    NewLengthTooSmall,
    #[error("Oracle Error: Lookup timestamp too old")]
    LookUpTimestampTooOld,
}

impl Oracle {
    /// Updates the oracle and returns the updated pair parameters.
    pub fn update(
        &mut self,
        time: &Timestamp,
        mut parameters: PairParameters,
        active_id: u32,
        new_volume: Option<Bytes32>,
        new_fee: Option<Bytes32>,
        length: u16,
    ) -> Result<(PairParameters, Option<OracleSample>), OracleError> {
        let mut oracle_id = parameters.get_oracle_id();
        if oracle_id == 0 {
            return Ok((parameters, None));
        };

        let new_vol = new_volume.unwrap_or_default();
        let new_fee = new_fee.unwrap_or_default();

        let mut created_at = self.0.get_sample_creation();
        let last_updated_at = created_at + self.0.get_sample_lifetime() as u64;

        if time.seconds() > last_updated_at {
            let (
                mut cumulative_txns,
                cumulative_id,
                cumulative_volatility,
                cumulative_bin_crossed,
                cumulative_vol,
                cumulative_fee,
            ) = OracleSample::update(
                self.0,
                time.seconds() - last_updated_at,
                active_id,
                parameters.get_volatility_accumulator(),
                parameters.get_delta_id(active_id),
                new_vol,
                new_fee,
            );

            let mut lifetime = time.seconds() - created_at;

            if lifetime > MAX_SAMPLE_LIFETIME as u64 {
                cumulative_txns = 1;
                oracle_id = (oracle_id % length) + 1;
                lifetime = 0;
                created_at = time.seconds();
                parameters.set_oracle_id(oracle_id);
            }

            let new_sample = OracleSample::encode(
                cumulative_txns,
                cumulative_id,
                cumulative_volatility,
                cumulative_bin_crossed,
                lifetime as u8,
                created_at,
                cumulative_vol,
                cumulative_fee,
            );

            return Ok((parameters, Some(new_sample)));
        }

        return Ok((parameters, None));
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::liquidity_book::lb_libraries::math::encoded_sample::MASK_UINT20;
//     use std::collections::HashMap;

//     // Helper function to bound a value within a range
//     fn bound<T: Ord>(value: T, min: T, max: T) -> T {
//         if value < min {
//             min
//         } else if value > max {
//             max
//         } else {
//             value
//         }
//     }

//     #[test]
//     fn test_set_and_get_sample() {
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         // Replace with random values for fuzz testing
//         let oracle_id: u16 = 1;
//         let sample = OracleSample(EncodedSample([0u8; 32]));

//         oracle.set_sample(oracle_id, sample).unwrap();

//         let retrieved_sample = oracle.get_sample(oracle_id).unwrap();
//         assert_eq!(retrieved_sample, sample, "test_SetSample::1");

//         let internal_sample = oracle.samples.get(&(oracle_id - 1)).unwrap();
//         assert_eq!(*internal_sample, sample, "test_SetSample::2");
//     }

//     #[test]
//     fn test_revert_set_and_get_sample() {
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         let oracle_id: u16 = 0;
//         let sample = OracleSample(EncodedSample([0u8; 32]));

//         match oracle.set_sample(oracle_id, sample) {
//             Err(OracleError::InvalidOracleId) => {} // Expected error
//             _ => panic!("test_revert_SetSample failed"),
//         }

//         match oracle.get_sample(oracle_id) {
//             Err(OracleError::InvalidOracleId) => {} // Expected error
//             _ => panic!("test_revert_GetSample failed"),
//         }
//     }

//     #[test]
//     fn test_set_and_get_sample_edge_cases() {
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         // Test with maximum oracle_id value for u16
//         let max_oracle_id: u16 = u16::MAX;
//         let sample = OracleSample(EncodedSample([1u8; 32]));

//         // Set sample with maximum oracle_id
//         oracle.set_sample(max_oracle_id, sample).unwrap();

//         // Retrieve and validate
//         let retrieved_sample = oracle.get_sample(max_oracle_id).unwrap();
//         assert_eq!(
//             retrieved_sample, sample,
//             "test_set_and_get_sample_edge_cases::MaxOracleId"
//         );

//         // Test with minimum valid oracle_id (1, since 0 is considered invalid)
//         let min_valid_oracle_id: u16 = 1;
//         oracle.set_sample(min_valid_oracle_id, sample).unwrap();

//         // Retrieve and validate
//         let retrieved_sample = oracle.get_sample(min_valid_oracle_id).unwrap();
//         assert_eq!(
//             retrieved_sample, sample,
//             "test_set_and_get_sample_edge_cases::MinValidOracleId"
//         );

//         // Test with an empty sample ([0u8; 32])
//         let empty_sample = OracleSample(EncodedSample([0u8; 32]));
//         oracle
//             .set_sample(min_valid_oracle_id, empty_sample)
//             .unwrap();

//         // Retrieve and validate
//         let retrieved_sample = oracle.get_sample(min_valid_oracle_id).unwrap();
//         assert_eq!(
//             retrieved_sample, empty_sample,
//             "test_set_and_get_sample_edge_cases::EmptySample"
//         );
//     }

//     #[test]
//     fn test_binary_search_simple() {
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         let sample1 = OracleSample::encode(3, 1, 2, 3, 0, 0);
//         let sample2 = OracleSample::encode(3, 2, 3, 4, 0, 10);
//         let sample3 = OracleSample::encode(3, 3, 4, 5, 0, 20);

//         oracle.set_sample(1, sample1).unwrap();
//         oracle.set_sample(2, sample2).unwrap();
//         oracle.set_sample(3, sample3).unwrap();

//         let (previous, next) = oracle.binary_search(3, 0, 3).unwrap();
//         assert_eq!(previous, sample1, "test_binarySearch::1");
//         assert_eq!(next, sample1, "test_binarySearch::2");

//         let (previous, next) = oracle.binary_search(3, 1, 3).unwrap();
//         assert_eq!(previous, sample1, "test_binarySearch::3");
//         assert_eq!(next, sample2, "test_binarySearch::4");

//         let (previous, next) = oracle.binary_search(3, 9, 3).unwrap();
//         assert_eq!(previous, sample1, "test_binarySearch::5");
//         assert_eq!(next, sample2, "test_binarySearch::6");

//         let (previous, next) = oracle.binary_search(3, 10, 3).unwrap();
//         assert_eq!(previous, sample2, "test_binarySearch::7");
//         assert_eq!(next, sample2, "test_binarySearch::8");

//         let (previous, next) = oracle.binary_search(3, 11, 3).unwrap();
//         assert_eq!(previous, sample2, "test_binarySearch::9");
//         assert_eq!(next, sample3, "test_binarySearch::10");

//         let (previous, next) = oracle.binary_search(3, 20, 3).unwrap();
//         assert_eq!(previous, sample3, "test_binarySearch::11");
//         assert_eq!(next, sample3, "test_binarySearch::12");
//     }

//     #[test]
//     fn test_binary_search_circular() {
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         let sample1 = OracleSample::encode(3, 1, 2, 3, 3, 30);
//         let sample2 = OracleSample::encode(3, 2, 3, 4, 9, 10);
//         let sample3 = OracleSample::encode(3, 3, 4, 5, 9, 20);

//         oracle.set_sample(1, sample1).unwrap();
//         oracle.set_sample(2, sample2).unwrap();
//         oracle.set_sample(3, sample3).unwrap();

//         let (previous, next) = oracle.binary_search(1, 19, 3).unwrap();
//         assert_eq!(previous, sample2, "test_binarySearch::1");
//         assert_eq!(next, sample2, "test_binarySearch::2");

//         let (previous, next) = oracle.binary_search(1, 24, 3).unwrap();
//         assert_eq!(previous, sample2, "test_binarySearch::3");
//         assert_eq!(next, sample3, "test_binarySearch::4");

//         let (previous, next) = oracle.binary_search(1, 29, 3).unwrap();
//         assert_eq!(previous, sample3, "test_binarySearch::5");
//         assert_eq!(next, sample3, "test_binarySearch::6");

//         let (previous, next) = oracle.binary_search(1, 30, 3).unwrap();
//         assert_eq!(previous, sample3, "test_binarySearch::7");
//         assert_eq!(next, sample1, "test_binarySearch::8");

//         let (previous, next) = oracle.binary_search(1, 33, 3).unwrap();
//         assert_eq!(previous, sample1, "test_binarySearch::9");
//         assert_eq!(next, sample1, "test_binarySearch::10");
//     }

//     #[test]
//     #[should_panic]
//     fn test_revert_binary_search() {
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         let sample1 = OracleSample::encode(3, 1, 2, 3, 0, 30);
//         let sample2 = OracleSample::encode(3, 2, 3, 4, 5, 10);

//         // Invalid oracleId
//         match oracle.binary_search(0, 20, 3) {
//             Err(OracleError::InvalidOracleId) => {}
//             _ => panic!("test_revert_BinarySearch::1 failed"),
//         }

//         // Invalid length
//         match oracle.binary_search(1, 20, 0) {
//             Err(OracleError::InvalidOracleId) => {}
//             _ => panic!("test_revert_BinarySearch::2 failed"),
//         }

//         oracle.set_sample(1, sample1).unwrap();
//         oracle.set_sample(2, sample2).unwrap();

//         // Invalid oracleId
//         match oracle.binary_search(0, 20, 3) {
//             Err(OracleError::InvalidOracleId) => {}
//             _ => panic!("test_revert_BinarySearch::3 failed"),
//         }

//         // Invalid length
//         match oracle.binary_search(1, 20, 0) {
//             Err(OracleError::InvalidOracleId) => {}
//             _ => panic!("test_revert_BinarySearch::4 failed"),
//         }

//         // Invalid timestamp
//         match oracle.binary_search(1, 9, 2) {
//             Err(OracleError::LookUpTimestampTooOld) => {}
//             _ => panic!("test_revert_BinarySearch::5 failed"),
//         }

//         // Invalid timestamp
//         match oracle.binary_search(1, 31, 2) {
//             Err(OracleError::LookUpTimestampTooOld) => {}
//             _ => panic!("test_revert_BinarySearch::6 failed"),
//         }
//     }

//     #[test]
//     fn test_binary_search_simple_edge_cases() {
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         // 1. Minimum Length
//         let sample_min = OracleSample::encode(1, 1, 2, 3, 0, 0);
//         oracle.set_sample(1, sample_min).unwrap();

//         let (previous, next) = oracle.binary_search(1, 0, 1).unwrap();
//         assert_eq!(
//             previous, sample_min,
//             "test_binary_search_simple_edge_cases::MinLength1"
//         );
//         assert_eq!(
//             next, sample_min,
//             "test_binary_search_simple_edge_cases::MinLength2"
//         );

//         // 2. Maximum Timestamp
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };
//         let max_timestamp: u64 = u64::MAX;
//         let sample_max = OracleSample::encode(u16::MAX, 1, 2, 3, 0, 0);
//         oracle.set_sample(u16::MAX - 2, sample_max).unwrap();
//         oracle.set_sample(u16::MAX - 1, sample_max).unwrap();
//         oracle.set_sample(u16::MAX, sample_max).unwrap();

//         let (previous, next) = oracle
//             .binary_search(u16::MAX - 1, max_timestamp, u16::MAX)
//             .unwrap();
//         assert_eq!(
//             previous, sample_max,
//             "test_binary_search_simple_edge_cases::MaxTimestamp1"
//         );
//         assert_eq!(
//             next, sample_max,
//             "test_binary_search_simple_edge_cases::MaxTimestamp2"
//         );

//         // 3. Minimum Timestamp
//         let min_timestamp: u64 = 0;
//         let sample_min_ts = OracleSample::encode(2, 1, 2, 3, 0, 0);
//         oracle.set_sample(1, sample_min_ts).unwrap();
//         oracle.set_sample(2, sample_min_ts).unwrap();

//         let (previous, next) = oracle.binary_search(1, min_timestamp, 2).unwrap();
//         assert_eq!(
//             previous, sample_min_ts,
//             "test_binary_search_simple_edge_cases::MinTimestamp1"
//         );
//         assert_eq!(
//             next, sample_min_ts,
//             "test_binary_search_simple_edge_cases::MinTimestamp2"
//         );
//     }

//     #[test]
//     fn test_get_sample_at_fully_initialized() {
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         let sample1 = OracleSample::encode(3, 40, 50, 60, 3, 30);
//         let sample2 = OracleSample::encode(3, 20, 30, 40, 5, 10);
//         let sample3 = OracleSample::encode(3, 30, 40, 50, 5, 20);

//         oracle.set_sample(1, sample1).unwrap();
//         oracle.set_sample(2, sample2).unwrap();
//         oracle.set_sample(3, sample3).unwrap();

//         let (last_update, cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
//             oracle.get_sample_at(1, 15).unwrap();

//         assert_eq!(last_update, 15, "test_GetSampleAt::1");
//         assert_eq!(cumulative_id, 20, "test_GetSampleAt::2");
//         assert_eq!(cumulative_volatility, 30, "test_GetSampleAt::3");
//         assert_eq!(cumulative_bin_crossed, 40, "test_GetSampleAt::4");

//         let (last_update, cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
//             oracle.get_sample_at(1, 20).unwrap();

//         assert_eq!(last_update, 20, "test_GetSampleAt::5");
//         assert_eq!(cumulative_id, 25, "test_GetSampleAt::6");
//         assert_eq!(cumulative_volatility, 35, "test_GetSampleAt::7");
//         assert_eq!(cumulative_bin_crossed, 45, "test_GetSampleAt::8");

//         let (last_update, cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
//             oracle.get_sample_at(1, 25).unwrap();

//         assert_eq!(last_update, 25, "test_GetSampleAt::9");
//         assert_eq!(cumulative_id, 30, "test_GetSampleAt::10");
//         assert_eq!(cumulative_volatility, 40, "test_GetSampleAt::11");
//         assert_eq!(cumulative_bin_crossed, 50, "test_GetSampleAt::12");

//         let (last_update, cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
//             oracle.get_sample_at(1, 30).unwrap();

//         assert_eq!(last_update, 30, "test_GetSampleAt::13");
//         assert_eq!(cumulative_id, 36, "test_GetSampleAt::14");
//         assert_eq!(cumulative_volatility, 46, "test_GetSampleAt::15");
//         assert_eq!(cumulative_bin_crossed, 56, "test_GetSampleAt::16");

//         let (last_update, cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
//             oracle.get_sample_at(1, 40).unwrap();

//         assert_eq!(last_update, 33, "test_GetSampleAt::17");
//         assert_eq!(cumulative_id, 40, "test_GetSampleAt::18");
//         assert_eq!(cumulative_volatility, 50, "test_GetSampleAt::19");
//         assert_eq!(cumulative_bin_crossed, 60, "test_GetSampleAt::20");
//     }

//     struct UpdateInputs {
//         pub oracle_length: u16,
//         pub oracle_id: u16,
//         pub previous_active_id: u32, // u24 is not a native Rust type, so we use u32
//         pub active_id: u32,          // u24 is not a native Rust type, so we use u32
//         pub previous_volatility: u32, // u24 is not a native Rust type, so we use u32
//         pub volatility: u32,         // u24 is not a native Rust type, so we use u32
//         pub previous_bin_crossed: u32, // u24 is not a native Rust type, so we use u32
//         pub created_at: u64,         // u40 is not a native Rust type, so we use u64
//         pub timestamp: u64,          // u40 is not a native Rust type, so we use u64
//     }

//     #[test]
//     fn test_update_delta_ts_lower_than_2_minutes() {
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         // Populate inputs struct (you may want to fuzz these values)
//         let mut inputs = UpdateInputs {
//             oracle_length: 3,
//             oracle_id: 2,
//             previous_active_id: 100,
//             active_id: 200,
//             previous_volatility: 50,
//             volatility: 60,
//             previous_bin_crossed: 1,
//             created_at: 10,
//             timestamp: 20,
//         };

//         inputs.oracle_id = bound(inputs.oracle_id, 1, u16::MAX);
//         inputs.oracle_length = bound(inputs.oracle_length, inputs.oracle_id, u16::MAX);
//         inputs.created_at = bound(
//             inputs.created_at,
//             if inputs.timestamp > 120 {
//                 inputs.timestamp - 120
//             } else {
//                 0
//             },
//             inputs.timestamp,
//         );
//         inputs.volatility = bound(inputs.volatility, 1, MASK_UINT20.as_u32());
//         inputs.previous_volatility = bound(inputs.previous_volatility, 1, MASK_UINT20.as_u32());

//         let sample = OracleSample::encode(
//             inputs.oracle_length,
//             inputs.previous_active_id as u64 * inputs.created_at,
//             inputs.previous_volatility as u64 * inputs.created_at,
//             inputs.previous_bin_crossed as u64 * inputs.created_at,
//             0,
//             inputs.created_at,
//         );

//         oracle.set_sample(inputs.oracle_id, sample).unwrap();

//         let mut parameters = PairParameters(EncodedSample([0u8; 32]));

//         parameters.set_oracle_id(inputs.oracle_id);
//         parameters.set_active_id(inputs.previous_active_id).unwrap();
//         parameters
//             .set_volatility_accumulator(inputs.volatility)
//             .unwrap();

//         let new_params = oracle
//             .update(
//                 &Timestamp::from_seconds(inputs.timestamp),
//                 parameters,
//                 inputs.active_id,
//             )
//             .unwrap();

//         assert_eq!(new_params, parameters, "test_Update::1");

//         let sample = oracle.get_sample(inputs.oracle_id).unwrap();

//         let dt = inputs.timestamp - inputs.created_at;

//         let d_id = if inputs.active_id > inputs.previous_active_id {
//             inputs.active_id - inputs.previous_active_id
//         } else {
//             inputs.previous_active_id - inputs.active_id
//         } as u64;

//         let cumulative_id =
//             (inputs.previous_active_id as u64 * inputs.created_at) + (inputs.active_id as u64 * dt);
//         let cumulative_volatility = (inputs.previous_volatility as u64 * inputs.created_at)
//             + (inputs.volatility as u64 * dt);
//         let cumulative_bin_crossed =
//             (inputs.previous_bin_crossed as u64 * inputs.created_at) + (d_id * dt);

//         assert_eq!(
//             sample.get_oracle_length(),
//             inputs.oracle_length,
//             "test_Update::3"
//         );
//         assert_eq!(sample.get_cumulative_id(), cumulative_id, "test_Update::4");
//         assert_eq!(
//             sample.get_cumulative_volatility(),
//             cumulative_volatility,
//             "test_Update::5"
//         );
//         assert_eq!(
//             sample.get_cumulative_bin_crossed(),
//             cumulative_bin_crossed,
//             "test_Update::6"
//         );
//     }

//     #[test]
//     fn test_update_delta_ts_greater_than_2_minutes() {
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         // Populate inputs struct (you may want to fuzz these values)
//         let inputs = UpdateInputs {
//             oracle_length: 3,
//             oracle_id: 2,
//             previous_active_id: 100,
//             active_id: 200,
//             previous_volatility: 50,
//             volatility: 60,
//             previous_bin_crossed: 1,
//             created_at: 10,
//             timestamp: 140,
//         };

//         // Your "vm.assume" logic goes here as assertions
//         assert!(
//             inputs.oracle_id > 0
//                 && inputs.oracle_length >= inputs.oracle_id
//                 && inputs.created_at <= inputs.timestamp
//                 && inputs.timestamp - inputs.created_at > 120
//                 && inputs.volatility <= MASK_UINT20.as_u32()
//                 && inputs.previous_volatility <= MASK_UINT20.as_u32()
//         );

//         // Your "vm.warp" logic should be implemented if needed

//         let sample = OracleSample::encode(
//             inputs.oracle_length,
//             inputs.previous_active_id as u64 * inputs.created_at,
//             inputs.previous_volatility as u64 * inputs.created_at,
//             inputs.previous_bin_crossed as u64 * inputs.created_at,
//             0,
//             inputs.created_at,
//         );

//         oracle.set_sample(inputs.oracle_id, sample).unwrap();

//         let mut parameters = PairParameters(EncodedSample([0u8; 32]));

//         parameters.set_oracle_id(inputs.oracle_id);
//         parameters.set_active_id(inputs.previous_active_id).unwrap();
//         parameters
//             .set_volatility_accumulator(inputs.volatility)
//             .unwrap();

//         // Your "vm.warp" logic should be implemented if needed

//         let mut new_params = oracle
//             .update(
//                 &Timestamp::from_seconds(inputs.timestamp),
//                 parameters,
//                 inputs.active_id,
//             )
//             .unwrap();

//         let next_id = ((inputs.oracle_id as usize % inputs.oracle_length as usize) + 1) as u16;

//         assert_eq!(
//             new_params.set_oracle_id(next_id).clone(),
//             new_params,
//             "test_Update::1"
//         );

//         if inputs.oracle_length > 1 {
//             assert_eq!(
//                 oracle.get_sample(inputs.oracle_id).unwrap(),
//                 sample,
//                 "test_Update::2"
//             );
//         }

//         let sample = oracle.get_sample(next_id).unwrap();

//         let dt = inputs.timestamp - inputs.created_at;

//         let d_id = if inputs.active_id > inputs.previous_active_id {
//             inputs.active_id - inputs.previous_active_id
//         } else {
//             inputs.previous_active_id - inputs.active_id
//         } as u64;

//         let cumulative_id =
//             (inputs.previous_active_id as u64 * inputs.created_at) + (inputs.active_id as u64 * dt);
//         let cumulative_volatility = (inputs.previous_volatility as u64 * inputs.created_at)
//             + (inputs.volatility as u64 * dt);
//         let cumulative_bin_crossed =
//             (inputs.previous_bin_crossed as u64 * inputs.created_at) + (d_id * dt);

//         assert_eq!(
//             sample.get_oracle_length(),
//             inputs.oracle_length,
//             "test_Update::3"
//         );
//         assert_eq!(sample.get_cumulative_id(), cumulative_id, "test_Update::4");
//         assert_eq!(
//             sample.get_cumulative_volatility(),
//             cumulative_volatility,
//             "test_Update::5"
//         );
//         assert_eq!(
//             sample.get_cumulative_bin_crossed(),
//             cumulative_bin_crossed,
//             "test_Update::6"
//         );
//     }

//     #[test]
//     fn test_increase_oracle_length() {
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         // Random lengths, you may want to fuzz these values.
//         let length = 3;
//         let new_length = 4;

//         let oracle_id = 1;

//         println!(
//             "{:#?}",
//             oracle.get_sample(oracle_id).unwrap().get_oracle_length()
//         );

//         oracle.increase_length(oracle_id, length).unwrap();

//         println!(
//             "{:#?}",
//             oracle.get_sample(oracle_id).unwrap().get_oracle_length()
//         );

//         oracle.increase_length(oracle_id, new_length).unwrap();

//         println!(
//             "{:#?}",
//             oracle.get_sample(oracle_id).unwrap().get_oracle_length()
//         );

//         assert_eq!(
//             oracle.get_sample(oracle_id).unwrap().get_oracle_length(),
//             new_length,
//             "test_IncreaseOracleLength::1"
//         );

//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         // Random lengths, you may want to fuzz these values.
//         let length = u16::MAX - 1;
//         let new_length = u16::MAX;

//         let oracle_id = 1;

//         oracle.increase_length(oracle_id, length).unwrap();
//         oracle.increase_length(oracle_id, new_length).unwrap();

//         assert_eq!(
//             oracle.get_sample(oracle_id).unwrap().get_oracle_length(),
//             new_length,
//             "test_IncreaseOracleLength::2"
//         );
//     }

//     #[test]
//     fn test_revert_increase_oracle_length() {
//         let mut oracle = Oracle {
//             samples: HashMap::new(),
//         };

//         // Random lengths, you may want to fuzz these values.
//         let length = 3;
//         let new_length = 2;

//         // Equivalent to vm.assume in Solidity
//         assert!(new_length <= length && length > 0);

//         let oracle_id = 1;

//         oracle.increase_length(oracle_id, length).unwrap();

//         // Equivalent to vm.expectRevert in Solidity.
//         // Replace with your own logic.
//         assert!(oracle.increase_length(oracle_id, new_length).is_err());
//     }
// }
