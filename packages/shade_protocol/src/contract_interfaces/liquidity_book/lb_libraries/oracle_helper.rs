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
