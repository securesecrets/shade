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

use crate::utils::liquidity_book::constants::*;
use crate::utils::liquidity_book::math::encoded_sample::*;

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

#[derive(thiserror::Error, Debug)]
pub enum PairParametersError {
    #[error("Pair Parameters Error: Invalid Parameter")]
    InvalidParameter,
}

#[cw_serde]
#[derive(Copy)]
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
        let base_factor = Self::get_base_factor(&self) as u128;
        base_factor * (bin_step as u128) * 10_000_000_000
    }

    /// Calculates the variable fee.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `bin_step` - The bin step (in basis points)
    pub fn get_variable_fee(&self, bin_step: u16) -> u128 {
        let variable_fee_control = Self::get_variable_fee_control(&self) as u128;

        if variable_fee_control != 0 {
            let vol_accumulator = Self::get_volatility_accumulator(&self) as u128;
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
        let base_fee = Self::get_base_fee(&self, bin_step);
        let variable_fee = Self::get_variable_fee(&self, bin_step);
        base_fee + variable_fee
    }

    /// Set the oracle id in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `oracle_id` - The oracle id
    pub fn set_oracle_id(self, oracle_id: u16) -> PairParameters {
        //No need to add a check oracle_id == u16
        PairParameters(self.0.set(oracle_id.into(), MASK_UINT16, OFFSET_ORACLE_ID))
    }

    /// Set the volatility reference in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `vol_ref` - The volatility reference
    pub fn set_volatility_reference(
        self,
        vol_ref: u32,
    ) -> Result<PairParameters, PairParametersError> {
        if vol_ref > MASK_UINT20.as_u32() {
            Err(PairParametersError::InvalidParameter)
        } else {
            Ok(PairParameters(self.0.set(
                vol_ref.into(),
                MASK_UINT20,
                OFFSET_VOL_REF,
            )))
        }
    }

    /// Set the volatility accumulator in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `vol_acc` - The volatility accumulator
    pub fn set_volatility_accumulator(
        self,
        vol_acc: u32,
    ) -> Result<PairParameters, PairParametersError> {
        if vol_acc > MASK_UINT20.as_u32() {
            Err(PairParametersError::InvalidParameter)
        } else {
            Ok(PairParameters(self.0.set(
                vol_acc.into(),
                MASK_UINT20,
                OFFSET_VOL_ACC,
            )))
        }
    }

    /// Set the active id in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `active_id` - The active id
    pub fn set_active_id(self, active_id: u32) -> Result<PairParameters, PairParametersError> {
        if active_id > MASK_UINT24.as_u32() {
            Err(PairParametersError::InvalidParameter)
        } else {
            Ok(PairParameters(self.0.set(
                active_id.into(),
                MASK_UINT24,
                OFFSET_ACTIVE_ID,
            )))
        }
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
    pub fn set_static_fee_parameters(
        self,
        base_factor: u16,                //u16
        filter_period: u16,              //u12
        decay_period: u16,               //u12
        reduction_factor: u16,           //u14
        variable_fee_control: u32,       //u24
        protocol_share: u16,             //u14
        max_volatility_accumulator: u32, //u20
    ) -> Result<PairParameters, PairParametersError> {
        if (filter_period > decay_period) | (decay_period > MASK_UINT12.as_u16())
            || reduction_factor > BASIS_POINT_MAX as u16
            || protocol_share > MAX_PROTOCOL_SHARE as u16
            || max_volatility_accumulator > MASK_UINT20.as_u32()
        {
            return Err(PairParametersError::InvalidParameter);
        }

        let mut new_parameters = EncodedSample([0u8; 32]);
        // TODO: all of these needing to be turned into U256 seems like a waste
        new_parameters = new_parameters.set(base_factor.into(), MASK_UINT16, OFFSET_BASE_FACTOR);
        new_parameters =
            new_parameters.set(filter_period.into(), MASK_UINT12, OFFSET_FILTER_PERIOD);
        new_parameters = new_parameters.set(decay_period.into(), MASK_UINT12, OFFSET_DECAY_PERIOD);
        new_parameters = new_parameters.set(
            reduction_factor.into(),
            MASK_UINT14,
            OFFSET_REDUCTION_FACTOR,
        );
        new_parameters = new_parameters.set(
            variable_fee_control.into(),
            MASK_UINT24,
            OFFSET_VAR_FEE_CONTROL,
        );
        new_parameters =
            new_parameters.set(protocol_share.into(), MASK_UINT14, OFFSET_PROTOCOL_SHARE);
        new_parameters = new_parameters.set(
            max_volatility_accumulator.into(),
            MASK_UINT20,
            OFFSET_MAX_VOL_ACC,
        );

        Ok(PairParameters(self.0.set(
            U256::from_le_bytes(new_parameters.0),
            MASK_STATIC_PARAMETER.into(),
            0,
        )))
    }

    /// Updates the index reference in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    pub fn update_id_reference(self) -> PairParameters {
        //No need to add check because we already have a check on setting active id
        let active_id = Self::get_active_id(&self);
        PairParameters(self.0.set(active_id.into(), MASK_UINT24, OFFSET_ID_REF))
    }

    /// Updates the time of last update in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `current_time` - The current timestamp
    pub fn update_time_of_last_update(
        self,
        time: &Timestamp,
    ) -> Result<PairParameters, PairParametersError> {
        let current_time = time.seconds();
        if current_time > MASK_UINT40.as_u64() {
            // If not, return an error (you can define a custom error type for this)
            return Err(PairParametersError::InvalidParameter);
        } else {
            Ok(PairParameters(self.0.set(
                current_time.into(),
                MASK_UINT40,
                OFFSET_TIME_LAST_UPDATE,
            )))
        }
    }

    /// Updates the volatility reference in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    pub fn update_volatility_reference(self) -> Result<PairParameters, PairParametersError> {
        let vol_acc = self.get_volatility_accumulator();
        let reduction_factor = self.get_reduction_factor();

        // TODO make a uint24() function to wrap this in?
        let vol_ref = vol_acc * reduction_factor as u32 / BASIS_POINT_MAX;

        self.set_volatility_reference(vol_ref)
    }

    /// Updates the volatility accumulator in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `active_id` - The active id
    pub fn update_volatility_accumulator(
        self,
        active_id: u32,
    ) -> Result<PairParameters, PairParametersError> {
        let id_reference = self.get_id_reference();

        let delta_id = if active_id > id_reference {
            active_id - id_reference
        } else {
            id_reference - active_id
        };
        let vol_acc = self.get_volatility_reference() + delta_id * BASIS_POINT_MAX;

        let max_vol_acc = self.get_max_volatility_accumulator();

        let vol_acc = if vol_acc > max_vol_acc {
            max_vol_acc
        } else {
            vol_acc
        };
        //Check done in volatility accumulator
        self.set_volatility_accumulator(vol_acc)
    }

    /// Updates the volatility reference and the volatility accumulator in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    pub fn update_references(
        mut self,
        time: &Timestamp,
    ) -> Result<PairParameters, PairParametersError> {
        let dt = time.seconds() - self.get_time_of_last_update();

        if dt > MASK_UINT40.as_u64() {
            // If not, return an error (you can define a custom error type for this)
            return Err(PairParametersError::InvalidParameter);
        } else {
            if dt >= self.get_filter_period().into() {
                self = self.update_id_reference();
                self = if dt < self.get_decay_period().into() {
                    self.update_volatility_reference()?
                } else {
                    self.set_volatility_reference(0)?
                };
            }
            self.update_time_of_last_update(time)
        }
    }

    /// Updates the volatility reference and the volatility accumulator in the encoded pair parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The encoded pair parameters
    /// * `active_id` - The active id
    pub fn update_volatility_parameters(
        self,
        active_id: u32,
        time: &Timestamp,
    ) -> Result<PairParameters, PairParametersError> {
        self.update_references(time)?
            .update_volatility_accumulator(active_id)
    }
}
