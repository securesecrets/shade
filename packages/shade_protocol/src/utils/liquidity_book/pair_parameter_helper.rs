//! ### Liquidity Book Pair Parameter Helper Library
//! Author: Kent
//!
//! This library contains functions to get and set parameters of a pair.
//!
//! The parameters are stored in a single bytes32 variable in the following format:
//! * [0 - 16[: base factor (16 bits)
//! * [16 - 28[: filter period (12 bits)
//! * [28 - 40[: decay period (12 bits)
//! * [40 - 54[: reduction factor (14 bits)
//! * [54 - 78[: variable fee control (24 bits)
//! * [78 - 92[: protocol share (14 bits)
//! * [92 - 112[: max volatility accumulator (20 bits)
//! * [112 - 132[: volatility accumulator (20 bits)
//! * [132 - 152[: volatility reference (20 bits)
//! * [152 - 176[: index reference (24 bits)
//! * [176 - 216[: time of last update (40 bits)
//! * [216 - 232[: oracle index (16 bits)
//! * [232 - 256[: active index (24 bits)

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Timestamp;
use ethnum::U256;

use crate::utils::liquidity_book::{constants::*, math::encoded_sample::*};

const OFFSET_BASE_FACTOR: u8 = 0;
const OFFSET_FILTER_PERIOD: u8 = 16;
const OFFSET_DECAY_PERIOD: u8 = 28;
const OFFSET_REDUCTION_FACTOR: u8 = 40;
const OFFSET_VAR_FEE_CONTROL: u8 = 54;
const OFFSET_PROTOCOL_SHARE: u8 = 78;
const OFFSET_MAX_VOL_ACC: u8 = 92;
const OFFSET_VOL_ACC: u8 = 112;
const OFFSET_VOL_REF: u8 = 132;
const OFFSET_ID_REF: u8 = 152;
const OFFSET_TIME_LAST_UPDATE: u8 = 176;
const OFFSET_ORACLE_ID: u8 = 216;
const OFFSET_ACTIVE_ID: u8 = 232;

const MASK_STATIC_PARAMETER: u128 = 0xffffffffffffffffffffffffffffu128;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum PairParametersError {
    #[error("Pair Parameters Error: Invalid Parameter")]
    InvalidParameter,
}

#[cw_serde]
#[derive(Copy, Default)]
pub struct PairParameters(pub EncodedSample);

impl PairParameters {
    /// Get the base factor from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 16[: base factor (16 bits)
    ///     * [16 - 256[: other parameters
    pub fn get_base_factor(&self) -> u16 {
        self.0.decode_uint16(OFFSET_BASE_FACTOR)
    }

    /// Get the filter period from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 16[: other parameters
    ///     * [16 - 28[: filter period (12 bits)
    ///     * [28 - 256[: other parameters
    pub fn get_filter_period(&self) -> u16 {
        self.0.decode_uint12(OFFSET_FILTER_PERIOD)
    }

    /// Get the decay period from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 16[: other parameters
    ///     * [28 - 40[: decay period (12 bits)
    ///     * [40 - 256[: other parameters
    pub fn get_decay_period(&self) -> u16 {
        self.0.decode_uint12(OFFSET_DECAY_PERIOD)
    }

    /// Get the reduction factor from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 40[: other parameters
    ///     * [40 - 54[: reduction factor (14 bits)
    ///     * [54 - 256[: other parameters
    pub fn get_reduction_factor(&self) -> u16 {
        self.0.decode_uint14(OFFSET_REDUCTION_FACTOR)
    }

    /// Get the variable fee control from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 54[: other parameters
    ///     * [54 - 78[: variable fee control (24 bits)
    ///     * [78 - 256[: other parameters
    pub fn get_variable_fee_control(&self) -> u32 {
        self.0.decode_uint24(OFFSET_VAR_FEE_CONTROL)
    }

    /// Get the protocol share from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 78[: other parameters
    ///     * [78 - 92[: protocol share (14 bits)
    ///     * [92 - 256[: other parameters
    pub fn get_protocol_share(&self) -> u16 {
        self.0.decode_uint14(OFFSET_PROTOCOL_SHARE)
    }

    /// Get the max volatility accumulator from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 92[: other parameters
    ///     * [92 - 112[: max volatility accumulator (20 bits)
    ///     * [112 - 256[: other parameters
    pub fn get_max_volatility_accumulator(&self) -> u32 {
        self.0.decode_uint20(OFFSET_MAX_VOL_ACC)
    }

    /// Get the volatility accumulator from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 112[: other parameters
    ///     * [112 - 132[: volatility accumulator (20 bits)
    ///     * [132 - 256[: other parameters
    pub fn get_volatility_accumulator(&self) -> u32 {
        self.0.decode_uint20(OFFSET_VOL_ACC)
    }

    /// Get the volatility reference from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 132[: other parameters
    ///     * [132 - 152[: volatility reference (20 bits)
    ///     * [152 - 256[: other parameters
    pub fn get_volatility_reference(&self) -> u32 {
        self.0.decode_uint20(OFFSET_VOL_REF)
    }

    /// Get the index reference from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 152[: other parameters
    ///     * [152 - 176[: index reference (24 bits)
    ///     * [176 - 256[: other parameters
    pub fn get_id_reference(&self) -> u32 {
        self.0.decode_uint24(OFFSET_ID_REF)
    }

    /// Get the time of last update from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 176[: other parameters
    ///     * [176 - 216[: time of last update (40 bits)
    ///     * [216 - 256[: other parameters
    pub fn get_time_of_last_update(&self) -> u64 {
        self.0.decode_uint40(OFFSET_TIME_LAST_UPDATE)
    }

    /// Get the oracle id from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 216[: other parameters
    ///     * [216 - 232[: oracle id (16 bits)
    ///     * [232 - 256[: other parameters
    pub fn get_oracle_id(&self) -> u16 {
        self.0.decode_uint16(OFFSET_ORACLE_ID)
    }

    /// Get the active index from the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 232[: other parameters
    ///     * [232 - 256[: active index (24 bits)
    pub fn get_active_id(&self) -> u32 {
        self.0.decode_uint24(OFFSET_ACTIVE_ID)
    }

    /// Get the delta between the current active index and the cached active index.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters, as follows:
    ///     * [0 - 232[: other parameters
    ///     * [232 - 256[: active index (24 bits)
    /// * `active_id` - The current active index
    pub fn get_delta_id(&self, active_id: u32) -> u32 {
        let id = Self::get_active_id(self);
        if active_id > id {
            active_id - id
        } else {
            id - active_id
        }
    }

    /// Calculates the base fee, with 18 decimals.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `bin_step` - The bin step (in basis points)
    pub fn get_base_fee(&self, bin_step: u16) -> u128 {
        let base_factor = Self::get_base_factor(self) as u128;
        base_factor * (bin_step as u128) * 10_000_000_000
    }

    /// Calculates the variable fee.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `bin_step` - The bin step (in basis points)
    pub fn get_variable_fee(&self, bin_step: u16) -> u128 {
        let variable_fee_control = Self::get_variable_fee_control(self) as u128;

        if variable_fee_control != 0 {
            let vol_accumulator = Self::get_volatility_accumulator(self) as u128;
            let prod = vol_accumulator * (bin_step as u128);
            (prod * prod * variable_fee_control + 99) / 100
        } else {
            0
        }
    }

    /// Calculates the total fee, which is the sum of the base fee and the variable fee.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `bin_step` - The bin step (in basis points)
    pub fn get_total_fee(&self, bin_step: u16) -> u128 {
        let base_fee = Self::get_base_fee(self, bin_step);
        let variable_fee = Self::get_variable_fee(self, bin_step);
        base_fee + variable_fee
    }

    /// Set the oracle id in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `oracle_id` - The oracle id
    pub fn set_oracle_id(&mut self, oracle_id: u16) -> &mut Self {
        self.0.set(oracle_id.into(), MASK_UINT16, OFFSET_ORACLE_ID);
        self
    }

    /// Set the volatility reference in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `vol_ref` - The volatility reference
    pub fn set_volatility_reference(
        &mut self,
        vol_ref: u32,
    ) -> Result<&mut Self, PairParametersError> {
        if vol_ref > MASK_UINT20.as_u32() {
            Err(PairParametersError::InvalidParameter)
        } else {
            self.0.set(vol_ref.into(), MASK_UINT20, OFFSET_VOL_REF);
            Ok(self)
        }
    }

    /// Set the volatility accumulator in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `vol_acc` - The volatility accumulator
    pub fn set_volatility_accumulator(
        &mut self,
        vol_acc: u32,
    ) -> Result<&mut Self, PairParametersError> {
        if vol_acc > MASK_UINT20.as_u32() {
            Err(PairParametersError::InvalidParameter)
        } else {
            self.0.set(vol_acc.into(), MASK_UINT20, OFFSET_VOL_ACC);
            Ok(self)
        }
    }

    /// Set the active id in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `active_id` - The active id
    pub fn set_active_id(&mut self, active_id: u32) -> Result<&mut Self, PairParametersError> {
        if active_id > MASK_UINT24.as_u32() {
            return Err(PairParametersError::InvalidParameter);
        }
        self.0.set(active_id.into(), MASK_UINT24, OFFSET_ACTIVE_ID);
        Ok(self)
    }

    /// Sets the static fee parameters in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `base_factor` - The base factor
    /// * `filter_period` - The filter period
    /// * `decay_period` - The decay period
    /// * `reduction_factor` - The reduction factor
    /// * `variable_fee_control` - The variable fee control
    /// * `protocol_share` - The protocol share
    /// * `max_volatility_accumulator` - The max volatility accumulator
    #[allow(clippy::too_many_arguments)]
    pub fn set_static_fee_parameters(
        &mut self,
        base_factor: u16,
        filter_period: u16,
        decay_period: u16,
        reduction_factor: u16,
        variable_fee_control: u32,
        protocol_share: u16,
        max_volatility_accumulator: u32,
    ) -> Result<&mut Self, PairParametersError> {
        if (filter_period > decay_period) | (decay_period > MASK_UINT12.as_u16())
            || reduction_factor > BASIS_POINT_MAX
            || protocol_share > MAX_PROTOCOL_SHARE
            || max_volatility_accumulator > MASK_UINT20.as_u32()
        {
            return Err(PairParametersError::InvalidParameter);
        }

        let mut new_parameters = EncodedSample([0u8; 32]);

        // TODO: all of these needing to be turned into U256 seems like a waste
        new_parameters.set(base_factor.into(), MASK_UINT16, OFFSET_BASE_FACTOR);
        new_parameters.set(filter_period.into(), MASK_UINT12, OFFSET_FILTER_PERIOD);
        new_parameters.set(decay_period.into(), MASK_UINT12, OFFSET_DECAY_PERIOD);
        new_parameters.set(
            reduction_factor.into(),
            MASK_UINT14,
            OFFSET_REDUCTION_FACTOR,
        );
        new_parameters.set(
            variable_fee_control.into(),
            MASK_UINT24,
            OFFSET_VAR_FEE_CONTROL,
        );
        new_parameters.set(protocol_share.into(), MASK_UINT14, OFFSET_PROTOCOL_SHARE);
        new_parameters.set(
            max_volatility_accumulator.into(),
            MASK_UINT20,
            OFFSET_MAX_VOL_ACC,
        );

        self.0.set(
            U256::from_le_bytes(new_parameters.0),
            MASK_STATIC_PARAMETER.into(),
            0,
        );
        Ok(self)
    }

    /// Updates the index reference in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    pub fn update_id_reference(&mut self) -> &mut Self {
        let active_id = self.get_active_id();
        self.0.set(active_id.into(), MASK_UINT24, OFFSET_ID_REF);
        self
    }

    /// Updates the time of last update in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `current_time` - The current timestamp
    pub fn update_time_of_last_update(
        &mut self,
        time: &Timestamp,
    ) -> Result<&mut Self, PairParametersError> {
        let current_time = time.seconds();

        if current_time > MASK_UINT40.as_u64() {
            Err(PairParametersError::InvalidParameter)
        } else {
            self.0
                .set(current_time.into(), MASK_UINT40, OFFSET_TIME_LAST_UPDATE);
            Ok(self)
        }
    }

    /// Updates the volatility reference in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    pub fn update_volatility_reference(&mut self) -> Result<&mut Self, PairParametersError> {
        let vol_acc = self.get_volatility_accumulator();
        let reduction_factor = self.get_reduction_factor();
        let vol_ref = vol_acc * reduction_factor as u32 / BASIS_POINT_MAX as u32;

        self.set_volatility_reference(vol_ref)?;
        Ok(self)
    }

    /// Updates the volatility accumulator in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `active_id` - The active id
    pub fn update_volatility_accumulator(
        &mut self,
        active_id: u32,
    ) -> Result<&mut Self, PairParametersError> {
        let id_reference = self.get_id_reference();
        let delta_id = match active_id > id_reference {
            true => active_id - id_reference,
            false => id_reference - active_id,
        };
        let vol_acc = self.get_volatility_reference() + delta_id * BASIS_POINT_MAX as u32;
        let max_vol_acc = self.get_max_volatility_accumulator();
        let vol_acc = std::cmp::min(vol_acc, max_vol_acc);

        self.set_volatility_accumulator(vol_acc)?;
        Ok(self)
    }

    /// Updates the volatility reference and the volatility accumulator in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    pub fn update_references(
        &mut self,
        time: &Timestamp,
    ) -> Result<&mut Self, PairParametersError> {
        let dt = time.seconds() - self.get_time_of_last_update();

        if dt >= self.get_filter_period().into() {
            self.update_id_reference();
            if dt < self.get_decay_period().into() {
                self.update_volatility_reference()?
            } else {
                self.set_volatility_reference(0)?
            };
        }

        self.update_time_of_last_update(time)?;
        Ok(self)
    }

    /// Updates the volatility reference and the volatility accumulator in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `active_id` - The active id
    pub fn update_volatility_parameters(
        &mut self,
        active_id: u32,
        time: &Timestamp,
    ) -> Result<&mut Self, PairParametersError> {
        self.update_references(time)?
            .update_volatility_accumulator(active_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lb_libraries::types::StaticFeeParameters;
    use cosmwasm_std::testing::mock_env;

    static MAX_STATIC_FEE_PARAMETER: StaticFeeParameters = StaticFeeParameters {
        base_factor: MASK_UINT16.as_u16(),
        filter_period: MASK_UINT12.as_u16(),
        decay_period: MASK_UINT12.as_u16(),
        reduction_factor: BASIS_POINT_MAX,
        variable_fee_control: MASK_UINT24.as_u32(),
        protocol_share: MAX_PROTOCOL_SHARE,
        max_volatility_accumulator: MASK_UINT20.as_u32(),
    };

    #[test]
    fn test_static_fee_parameters() {
        let mut params = PairParameters::default();

        //Error because values over the limit are used.
        let raw_new_param = params.set_static_fee_parameters(
            MAX_STATIC_FEE_PARAMETER.base_factor,
            MAX_STATIC_FEE_PARAMETER.filter_period + 1,
            MAX_STATIC_FEE_PARAMETER.decay_period + 1,
            MAX_STATIC_FEE_PARAMETER.reduction_factor,
            MAX_STATIC_FEE_PARAMETER.variable_fee_control,
            MAX_STATIC_FEE_PARAMETER.protocol_share,
            MAX_STATIC_FEE_PARAMETER.max_volatility_accumulator,
        );
        assert_eq!(
            raw_new_param.unwrap_err(),
            PairParametersError::InvalidParameter
        );

        //Working fine because value within limit are used.
        let result = params.set_static_fee_parameters(
            MAX_STATIC_FEE_PARAMETER.base_factor,
            MAX_STATIC_FEE_PARAMETER.filter_period,
            MAX_STATIC_FEE_PARAMETER.decay_period,
            MAX_STATIC_FEE_PARAMETER.reduction_factor,
            MAX_STATIC_FEE_PARAMETER.variable_fee_control,
            MAX_STATIC_FEE_PARAMETER.protocol_share,
            MAX_STATIC_FEE_PARAMETER.max_volatility_accumulator,
        );

        assert!(result.is_ok(), "Invalid Parameters");

        match result {
            Ok(new_params) => {
                let pair_params = new_params;

                // Now test the parameters to make sure they were set correctly
                assert_eq!(
                    pair_params.get_base_factor(),
                    MAX_STATIC_FEE_PARAMETER.base_factor
                );
                assert_eq!(
                    pair_params.get_filter_period(),
                    MAX_STATIC_FEE_PARAMETER.filter_period
                );
                assert_eq!(
                    pair_params.get_decay_period(),
                    MAX_STATIC_FEE_PARAMETER.decay_period
                );
                assert_eq!(
                    pair_params.get_reduction_factor(),
                    MAX_STATIC_FEE_PARAMETER.reduction_factor
                );
                assert_eq!(
                    pair_params.get_variable_fee_control(),
                    MAX_STATIC_FEE_PARAMETER.variable_fee_control
                );
                assert_eq!(
                    pair_params.get_protocol_share(),
                    MAX_STATIC_FEE_PARAMETER.protocol_share
                );
                assert_eq!(
                    pair_params.get_max_volatility_accumulator(),
                    MAX_STATIC_FEE_PARAMETER.max_volatility_accumulator
                );
            }
            Err(_) => panic!("Setting static fee parameters failed"),
        }
    }

    #[test]
    fn test_set_oracle_id() {
        // Initialize the PairParameters object with some encoded sample.
        let mut pair_params = PairParameters::default();

        // Set the Oracle ID
        let oracle_id: u16 = 42;
        pair_params.set_oracle_id(oracle_id);

        // Test the Oracle ID to make sure it was set correctly
        assert_eq!(pair_params.get_oracle_id(), oracle_id);

        // For the second assertion, we'll mimic the bitwise operations
        let shifted_mask = MASK_UINT16 << OFFSET_ORACLE_ID;
        let new_params_bits = U256::from_le_bytes(pair_params.0 .0);
        let original_params_bits = U256::from_le_bytes(EncodedSample([0u8; 32]).0);

        assert_eq!(
            new_params_bits & !shifted_mask,
            original_params_bits & !shifted_mask
        );
    }

    #[test]
    fn test_set_volatility_reference() {
        // Initialize the PairParameters object with some encoded sample.
        let mut pair_params = PairParameters::default();

        let mut volatility_reference: u32 = MASK_UINT20.as_u32() + 1;
        let err_result = pair_params.set_volatility_reference(volatility_reference);
        assert!(err_result.is_err(), "set_volatility_reference failed");

        // Make sure volatility_reference is within bounds
        volatility_reference = 1024;
        if volatility_reference <= MASK_UINT20.as_u32() {
            // Set the Volatility Reference
            let result = pair_params.set_volatility_reference(volatility_reference);

            match result {
                Ok(pair_params) => {
                    // Test the Volatility Reference to make sure it was set correctly
                    assert_eq!(pair_params.get_volatility_reference(), volatility_reference);

                    let shifted_mask = MASK_UINT20 << OFFSET_VOL_REF;
                    let new_params_bits = U256::from_le_bytes(pair_params.0 .0);
                    let original_params_bits = U256::from_le_bytes(EncodedSample([0u8; 32]).0);

                    assert_eq!(
                        new_params_bits & !shifted_mask,
                        original_params_bits & !shifted_mask
                    );
                }
                Err(_) => panic!("Setting Volatility Reference failed"),
            }
        } else {
            panic!("Volatility Reference is out of bounds");
        }
    }

    #[test]
    fn test_set_volatility_accumulator() {
        // Initialize the PairParameters object with some encoded sample.
        let mut pair_params = PairParameters::default();

        let mut volatility_accumulator: u32 = MASK_UINT20.as_u32() + 1;
        let err_result = pair_params.set_volatility_accumulator(volatility_accumulator);
        assert!(err_result.is_err(), "set_volatility_accumulator failed");
        // Make sure volatility_accumulator is within bounds
        volatility_accumulator = 1024;
        let mask_uint20 = MASK_UINT20;
        if U256::from(volatility_accumulator) <= mask_uint20 {
            // Set the Volatility Accumulator
            let result = pair_params.set_volatility_accumulator(volatility_accumulator);

            match result {
                Ok(pair_params) => {
                    assert_eq!(
                        pair_params.get_volatility_accumulator(),
                        volatility_accumulator
                    );

                    let mask_not_uint20 = !mask_uint20;
                    let shifted_mask = mask_not_uint20 << OFFSET_VOL_ACC;
                    let new_params_bits = U256::from_le_bytes(pair_params.0 .0);
                    let original_params_bits = U256::from_le_bytes(EncodedSample([0u8; 32]).0);

                    assert_eq!(
                        new_params_bits & shifted_mask,
                        original_params_bits & shifted_mask
                    );
                }
                Err(_) => panic!("Setting Volatility Accumulator failed"),
            }
        } else {
            panic!("Volatility Accumulator is out of bounds");
        }
    }

    #[test]
    fn test_set_active_id() {
        // Initialize the PairParameters object with some encoded sample.
        let mut pair_params = PairParameters::default();

        // Test the max limit
        let mut active_id: u32 = MASK_UINT24.as_u32() + 1;
        let err_result = pair_params.set_active_id(active_id);
        assert!(err_result.is_err(), "set_active_id failed");

        //Custom example
        active_id = 1024;
        let previous_active_id = pair_params.get_active_id();
        //Checking Delta Id
        let delta_id = if previous_active_id > active_id {
            previous_active_id - active_id
        } else {
            active_id - previous_active_id
        };
        assert_eq!(pair_params.get_delta_id(active_id), delta_id);

        // Set the Active ID
        let result = pair_params.set_active_id(active_id);

        match result {
            Ok(pair_params) => {
                assert_eq!(pair_params.get_active_id(), active_id);
                assert_eq!(pair_params.get_delta_id(active_id), 0);
                assert_eq!(pair_params.get_delta_id(previous_active_id), delta_id);

                // For the final assertion, we'll mimic the bitwise operations
                let mask_not_uint24 = !MASK_UINT24;
                let shifted_mask = mask_not_uint24 << OFFSET_ACTIVE_ID;
                let new_params_bits = U256::from_le_bytes(pair_params.0 .0);
                let original_params_bits = U256::from_le_bytes(EncodedSample([0u8; 32]).0);

                assert_eq!(
                    new_params_bits & shifted_mask,
                    original_params_bits & shifted_mask
                );
            }
            Err(_) => panic!("Setting Active ID failed"),
        }
    }

    #[test]
    fn test_get_base_and_variable_fees() {
        let mut pair_params = PairParameters::default();
        pair_params
            .set_static_fee_parameters(
                MAX_STATIC_FEE_PARAMETER.base_factor,
                MAX_STATIC_FEE_PARAMETER.filter_period,
                MAX_STATIC_FEE_PARAMETER.decay_period,
                MAX_STATIC_FEE_PARAMETER.reduction_factor,
                MAX_STATIC_FEE_PARAMETER.variable_fee_control,
                MAX_STATIC_FEE_PARAMETER.protocol_share,
                MAX_STATIC_FEE_PARAMETER.max_volatility_accumulator,
            )
            .unwrap();
        let bin_step: u16 = 100;

        let base_fee = pair_params.get_base_fee(bin_step);
        let variable_fee = pair_params.get_variable_fee(bin_step);

        let base_factor = U256::from(pair_params.get_base_factor());
        let expected_base_fee = base_factor * U256::from(bin_step) * U256::from(1e10 as u64);
        assert_eq!(base_fee, expected_base_fee);

        let volatility_accumulator = U256::from(pair_params.get_volatility_accumulator());
        let variable_fee_control = U256::from(pair_params.get_variable_fee_control());
        let prod = volatility_accumulator * U256::from(bin_step);
        let expected_variable_fee =
            (prod * prod * variable_fee_control + U256::from(99u128)) / U256::from(100u128);
        assert_eq!(variable_fee, expected_variable_fee);

        if base_fee + variable_fee < U256::from(u128::MAX) {
            let total_fee = pair_params.get_total_fee(bin_step);
            assert_eq!(total_fee, base_fee + variable_fee);
        } else {
            panic!("Exceeds 128 bits");
        }
    }

    #[test]
    fn test_update_id_reference() {
        let mut pair_params = PairParameters::default();

        pair_params.set_active_id(1024).unwrap();

        let active_id: u32 = pair_params.get_active_id();

        pair_params.update_id_reference();

        assert_eq!(pair_params.get_id_reference(), active_id);

        let mask_not_uint24 = !MASK_UINT24;
        let shifted_mask = mask_not_uint24 << OFFSET_ACTIVE_ID;

        let new_params_bits = U256::from_le_bytes(pair_params.0 .0);
        let original_params_bits = U256::from_le_bytes(EncodedSample([0u8; 32]).0);

        assert_eq!(
            new_params_bits & shifted_mask,
            original_params_bits & shifted_mask
        );
    }

    #[test]
    fn test_update_time_of_last_update() {
        let mut pair_params = PairParameters::default();
        let env = mock_env();

        let current_timestamp = env.block.time;
        let result = pair_params.update_time_of_last_update(&current_timestamp);

        match result {
            Ok(pair_params) => {
                assert_eq!(
                    pair_params.get_time_of_last_update(),
                    current_timestamp.seconds()
                );

                let mask_not_uint40 = !MASK_UINT40;
                let shifted_mask = mask_not_uint40 << OFFSET_TIME_LAST_UPDATE;
                let new_params_bits = U256::from_le_bytes(pair_params.0 .0);
                let original_params_bits = U256::from_le_bytes(EncodedSample([0u8; 32]).0);

                assert_eq!(
                    new_params_bits & shifted_mask,
                    original_params_bits & shifted_mask
                );
            }
            Err(_) => panic!("Update Time Of Last Update failed"),
        }
    }

    #[test]
    fn test_update_volatility_reference() {
        let mut pair_params = PairParameters::default();

        let vol_accumulator = pair_params.get_volatility_accumulator();
        let reduction_factor = pair_params.get_reduction_factor();
        let new_vol_accumulator =
            (vol_accumulator * reduction_factor as u32) / BASIS_POINT_MAX as u32;

        if new_vol_accumulator > MASK_UINT20.as_u32() {
            let result = pair_params.update_volatility_reference();
            assert!(
                result.is_err(),
                "Update Volatility Reference should have failed."
            );
        } else {
            let result = pair_params.update_volatility_reference();
            match result {
                Ok(pair_params) => {
                    assert_eq!(pair_params.get_volatility_reference(), new_vol_accumulator);

                    let mask_not_uint20 = !MASK_UINT20;
                    let shifted_mask = mask_not_uint20 << OFFSET_VOL_REF;
                    let new_params_bits = U256::from_le_bytes(pair_params.0 .0);
                    let original_params_bits = U256::from_le_bytes(EncodedSample([0u8; 32]).0);

                    assert_eq!(
                        new_params_bits & shifted_mask,
                        original_params_bits & shifted_mask
                    );
                }
                Err(_) => panic!("Update Volatility Reference failed"),
            }
        }
    }
    #[test]
    fn test_update_volatility_accumulator() {
        let mut pair_params = PairParameters::default();

        let active_id: u32 = 1500; // Replace this with your own value
        let id_reference = pair_params.get_id_reference();
        let delta_id = if active_id > id_reference {
            active_id - id_reference
        } else {
            id_reference - active_id
        };

        let mut vol_accumulator =
            pair_params.get_volatility_reference() + delta_id * BASIS_POINT_MAX as u32;
        let max_vol_accumulator = pair_params.get_max_volatility_accumulator();
        vol_accumulator = if vol_accumulator > max_vol_accumulator {
            max_vol_accumulator
        } else {
            vol_accumulator
        };

        let result = pair_params.update_volatility_accumulator(active_id);
        match result {
            Ok(pair_params) => {
                assert_eq!(pair_params.get_volatility_accumulator(), vol_accumulator);

                let mask_not_uint20 = !MASK_UINT20;
                let shifted_mask = mask_not_uint20 << OFFSET_VOL_ACC;
                let new_params_bits = U256::from_le_bytes(pair_params.0 .0);
                let original_params_bits = U256::from_le_bytes(EncodedSample([0u8; 32]).0);

                assert_eq!(
                    new_params_bits & shifted_mask,
                    original_params_bits & shifted_mask
                );
            }
            Err(_) => panic!("Update Volatility Accumulator failed"),
        }
    }

    #[test]
    fn test_update_references() {
        let mut pair_params = PairParameters::default();

        let previous_time: u64 = 1000; // Replace with your value
        let time: u64 = 2000; // Replace with your value
        let sfp = &MAX_STATIC_FEE_PARAMETER;

        let env = mock_env();

        let current_timestamp = env.block.time;

        if previous_time <= time {
            pair_params
                .set_static_fee_parameters(
                    sfp.base_factor,
                    sfp.filter_period,
                    sfp.decay_period,
                    sfp.reduction_factor,
                    sfp.variable_fee_control,
                    sfp.protocol_share,
                    sfp.max_volatility_accumulator,
                )
                .unwrap()
                .update_time_of_last_update(&current_timestamp)
                .unwrap();

            let delta_time = time - previous_time;

            let id_reference = if delta_time >= sfp.filter_period.into() {
                pair_params.get_active_id()
            } else {
                pair_params.get_id_reference()
            };

            let mut vol_reference = pair_params.get_volatility_reference();
            if delta_time >= sfp.filter_period.into() {
                vol_reference = if delta_time >= sfp.decay_period.into() {
                    0
                } else {
                    pair_params
                        .update_volatility_reference()
                        .unwrap()
                        .get_volatility_reference()
                };
            }

            let result = pair_params.update_references(&current_timestamp);
            match result {
                Ok(pair_params) => {
                    assert_eq!(pair_params.get_id_reference(), id_reference);
                    assert_eq!(pair_params.get_volatility_reference(), vol_reference);
                    assert_eq!(
                        pair_params.get_time_of_last_update(),
                        current_timestamp.seconds()
                    );

                    let mask = !(U256::from(1u128 << 84u128) - 1u128) << OFFSET_VOL_REF;
                    let new_params_bits = U256::from_le_bytes(pair_params.0 .0);
                    let original_params_bits = U256::from_le_bytes(EncodedSample([0u8; 32]).0);

                    assert_eq!(new_params_bits & mask, original_params_bits & mask);
                }
                Err(_) => panic!("Update References failed"),
            }
        } else {
            panic!("Time condition not met");
        }
    }

    #[test]
    fn test_update_volatility_parameters() {
        let mut pair_params = PairParameters::default();

        let previous_time: u64 = 1000;
        let time: u64 = 2000;
        let active_id: u32 = 10;

        let sfp = &MAX_STATIC_FEE_PARAMETER;
        let env = mock_env();
        let current_timestamp = env.block.time;

        if previous_time <= time {
            pair_params
                .set_static_fee_parameters(
                    sfp.base_factor,
                    sfp.filter_period,
                    sfp.decay_period,
                    sfp.reduction_factor,
                    sfp.variable_fee_control,
                    sfp.protocol_share,
                    sfp.max_volatility_accumulator,
                )
                .unwrap()
                .update_time_of_last_update(&current_timestamp)
                .unwrap();

            let trusted_params = *pair_params
                .update_references(&current_timestamp)
                .unwrap()
                .update_volatility_accumulator(active_id)
                .unwrap();

            let new_params = *pair_params
                .update_volatility_parameters(active_id, &current_timestamp)
                .unwrap();

            assert_eq!(
                new_params.get_id_reference(),
                trusted_params.get_id_reference()
            );
            assert_eq!(
                new_params.get_volatility_reference(),
                trusted_params.get_volatility_reference()
            );
            assert_eq!(
                new_params.get_volatility_accumulator(),
                trusted_params.get_volatility_accumulator()
            );
            assert_eq!(
                new_params.get_time_of_last_update(),
                current_timestamp.seconds()
            );

            let mask = !(U256::from(1u128 << 104u128) - 1u128) << OFFSET_VOL_ACC;
            let new_params_bits = U256::from_le_bytes(new_params.0 .0);
            let original_params_bits = U256::from_le_bytes(pair_params.0 .0);

            assert_eq!(new_params_bits & mask, original_params_bits & mask);
        } else {
            panic!("Time condition not met");
        }
    }
}
