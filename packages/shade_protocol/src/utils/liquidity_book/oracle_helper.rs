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

use std::collections::HashMap;

use cosmwasm_std::Timestamp;
use ethnum::U256;
use serde::{Deserialize, Serialize};

use super::pair_parameter_helper::PairParameters;
use crate::utils::liquidity_book::math::{
    encoded_sample::EncodedSample, sample_math::OracleSample,
};

// TODO: consider creating a different type of storage for this.
#[derive(Serialize, Deserialize)]
pub struct Oracle {
    /// This array represents a fixed-size storage for 65535 samples,
    /// where each sample is a 32-byte (256-bit) value.
    pub samples: HashMap<u16, OracleSample>,
}

pub const MAX_SAMPLE_LIFETIME: u8 = 120; //seconds

#[derive(thiserror::Error, Debug)]
pub enum OracleError {
    #[error("Oracle Error: Invalid Oracle ID")]
    InvalidOracleId,
    #[error("Oracle Error: New length too small")]
    NewLengthTooSmall,
    #[error("Oracle Error: Lookup timestamp too old")]
    LookUpTimestampTooOld,
}

impl Oracle {
    /// Modifier to check that the oracle id is valid.
    fn check_oracle_id(oracle_id: u16) -> Result<(), OracleError> {
        if oracle_id == 0 {
            return Err(OracleError::InvalidOracleId);
        }

        Ok(())
    }

    /// Returns the sample at the given oracleId.
    pub fn get_sample(&self, oracle_id: u16) -> Result<OracleSample, OracleError> {
        Self::check_oracle_id(oracle_id)?;

        Ok(self.samples[&(oracle_id - 1)])
    }

    /// Returns the active sample (Bytes32) and the active size (u16) of the oracle.
    pub fn get_active_sample_and_size(
        &self,
        oracle_id: u16,
    ) -> Result<(OracleSample, u16), OracleError> {
        let active_sample = self.get_sample(oracle_id)?;
        let mut active_size = OracleSample::get_oracle_length(&active_sample);

        if oracle_id != active_size {
            active_size = OracleSample::get_oracle_length(&self.get_sample(active_size)?);
            active_size = if oracle_id > active_size {
                oracle_id
            } else {
                active_size
            };
        }

        Ok((active_sample, active_size))
    }

    /// Returns the sample at the given timestamp. If the timestamp is not in the oracle, it returns the closest sample.
    ///
    /// # Arguments
    ///
    /// * `oracle_id` - The oracle id
    /// * `look_up_timestamp` - The timestamp to look up
    ///
    /// # Returns
    ///
    /// * `last_update` - The last update timestamp
    /// * `cumulative_id` - The cumulative id
    /// * `cumulative_volatility` - The cumulative volatility
    /// * `cumulative_bin_crossed` - The cumulative bin crossed
    pub fn get_sample_at(
        &self,
        oracle_id: u16,
        look_up_timestamp: u64,
    ) -> Result<(u64, u64, u64, u64), OracleError> {
        let (active_sample, active_size) = self.get_active_sample_and_size(oracle_id)?;

        if OracleSample::get_sample_last_update(&self.samples[&(oracle_id % active_size)])
            > look_up_timestamp
        {
            return Err(OracleError::LookUpTimestampTooOld);
        }

        let mut last_update = OracleSample::get_sample_last_update(&active_sample);
        if last_update <= look_up_timestamp {
            return Ok((
                last_update,
                OracleSample::get_cumulative_id(&active_sample),
                OracleSample::get_cumulative_volatility(&active_sample),
                OracleSample::get_cumulative_bin_crossed(&active_sample),
            ));
        } else {
            last_update = look_up_timestamp;
        }
        let (prev_sample, next_sample) =
            self.binary_search(oracle_id, look_up_timestamp, active_size)?;
        let weight_prev = next_sample.get_sample_last_update() - look_up_timestamp;
        let weight_next = look_up_timestamp - prev_sample.get_sample_last_update();

        let (cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
            OracleSample::get_weighted_average(prev_sample, next_sample, weight_prev, weight_next);

        Ok((
            last_update,
            cumulative_id,
            cumulative_volatility,
            cumulative_bin_crossed,
        ))
    }

    /// Binary search to find the 2 samples surrounding the given timestamp.
    ///
    /// # Arguments
    ///
    /// * `oracle` - The oracle
    /// * `oracleId` - The oracle id
    /// * `look_up_timestamp` - The timestamp to look up
    /// * `length` - The oracle length
    ///
    /// # Returns
    ///
    /// * `prev_sample` - The previous sample
    /// * `next_sample` - The next sample
    // TODO: make lookUpTimestamp a uint40? what if cosmos block time doesn't fit in uint40?
    pub fn binary_search(
        &self,
        oracle_id: u16,
        look_up_timestamp: u64,
        length: u16,
    ) -> Result<(OracleSample, OracleSample), OracleError> {
        let mut oracle_id = oracle_id;
        let mut low = 0;
        let mut high = length - 1;

        // TODO: not sure if it's ok to initialize these at 0
        let mut sample = OracleSample(EncodedSample([0u8; 32]));
        let mut sample_last_update = 0u64;

        let start_id = oracle_id; // oracleId is 1-based
        while low <= high {
            let mid = (low + high) >> 1;

            oracle_id = (start_id + mid) % length;

            sample = self.samples[&oracle_id];
            sample_last_update = sample.get_sample_last_update();

            if sample_last_update > look_up_timestamp {
                high = mid - 1;
            } else if sample_last_update < look_up_timestamp {
                low = mid + 1;
            } else {
                return Ok((sample, sample));
            }
        }

        if look_up_timestamp < sample_last_update {
            if oracle_id == 0 {
                oracle_id = length;
            }

            Ok((self.samples[&(oracle_id - 1)], sample))
        } else {
            oracle_id = (oracle_id + 1) % length;

            Ok((sample, self.samples[&oracle_id]))
        }
    }

    /// Sets the sample at the given oracle_id.
    pub fn set_sample(&mut self, oracle_id: u16, sample: OracleSample) -> Result<(), OracleError> {
        Self::check_oracle_id(oracle_id)?;

        self.samples.insert(oracle_id - 1, sample);

        Ok(())
    }

    /// Updates the oracle and returns the updated pair parameters.
    pub fn update(
        &mut self,
        time: &Timestamp,
        parameters: PairParameters,
        active_id: u32,
    ) -> Result<PairParameters, OracleError> {
        let oracle_id = parameters.get_oracle_id();
        if oracle_id == 0 {
            return Ok(parameters);
        }

        let sample = self.get_sample(oracle_id)?;

        let created_at = sample.get_sample_creation();
        let last_updated_at = created_at + sample.get_sample_lifetime() as u64;

        if time.seconds() > last_updated_at {
            let (cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
                OracleSample::update(
                    sample,
                    time.seconds() - last_updated_at,
                    active_id,
                    parameters.get_volatility_accumulator(),
                    parameters.get_delta_id(active_id),
                );

            let length = sample.get_oracle_length();
            let lifetime = time.seconds() - created_at;

            let oracle_id = if lifetime > MAX_SAMPLE_LIFETIME as u64 {
                (oracle_id % length) + 1
            } else {
                oracle_id
            };

            let created_at = if lifetime > MAX_SAMPLE_LIFETIME as u64 {
                time.seconds()
            } else {
                created_at
            };

            let new_sample = OracleSample::encode(
                length,
                cumulative_id,
                cumulative_volatility,
                cumulative_bin_crossed,
                lifetime as u8,
                created_at,
            );

            self.set_sample(oracle_id, new_sample)?;

            let new_parameters = parameters.set_oracle_id(oracle_id);

            return Ok(new_parameters);
        }

        Ok(parameters)
    }

    /// Increases the oracle length.
    pub fn increase_length(mut self, oracle_id: u16, new_length: u16) -> Result<Self, OracleError> {
        let sample = self.get_sample(oracle_id)?;
        let length = sample.get_oracle_length();

        if length >= new_length {
            return Err(OracleError::NewLengthTooSmall);
        }

        let last_sample = if length == oracle_id {
            sample
        } else if length == 0 {
            OracleSample(EncodedSample([0u8; 32]))
        } else {
            self.get_sample(length)?
        };

        let mut active_size = last_sample.get_oracle_length();
        active_size = if oracle_id > active_size {
            oracle_id
        } else {
            active_size
        };

        for i in length..new_length {
            // NOTE: I think what this does is encode the active_size as the oracle_length (16 bits)
            // in each of the newly added samples... the rest of the sample values are empty.
            self.samples.insert(
                i,
                OracleSample(EncodedSample(U256::from(active_size).to_le_bytes())),
            );
        }

        // I think this is a fancy way of changing the length of the current sample.
        // It's confusing looking because we don't have methods for pow or bitOR for bytes32,
        // so we have to convert to U256 and back.
        let new_sample = (U256::from_le_bytes(sample.0 .0).pow(length as u32)) | new_length as u128;
        self.set_sample(
            oracle_id,
            OracleSample(EncodedSample(new_sample.to_le_bytes())),
        )?;

        Ok(self)
    }
}
