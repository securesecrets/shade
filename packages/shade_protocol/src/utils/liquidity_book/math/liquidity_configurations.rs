//! ### Liquidity Book Liquidity Configurations Library
//! Author: Kent and Haseeb
//!
//! This library contains functions to encode and decode the config of a pool and interact with the encoded Bytes32.

use cosmwasm_schema::cw_serde;
use ethnum::U256;

use crate::utils::liquidity_book::types::Bytes32;

use super::packed_u128_math::{Decode, Encode};
use crate::utils::liquidity_book::constants::*;

pub const OFFSET_ID: u8 = 0;
pub const OFFSET_DISTRIBUTION_Y: u8 = 24;
pub const OFFSET_DISTRIBUTION_X: u8 = 88;

#[derive(thiserror::Error, Debug)]
pub enum LiquidityConfigurationsError {
    #[error("Liquidity Configurations Error: Invalid Config")]
    InvalidConfig,
}

#[cw_serde]
pub struct LiquidityConfigurations {
    pub distribution_x: u64,
    pub distribution_y: u64,
    pub id: u32,
}

impl LiquidityConfigurations {
    /// Get the amounts and id from a config and amounts_in
    /// # Returns
    ///
    /// * `amounts` - The distributed amounts as follows:
    /// * `id` - The id of the bin to add the liquidity to
    /// * `LiquidityConfigurationsError` - An error type for invalid config
    pub fn get_amounts_and_id(
        &self,
        amounts_in: Bytes32,
    ) -> Result<(Bytes32, u32), LiquidityConfigurationsError> {
        let (x1, x2) = amounts_in.decode();

        let x1_distributed =
            (U256::from(x1) * U256::from(self.distribution_x)) / U256::from(PRECISION);
        let x2_distributed =
            (U256::from(x2) * U256::from(self.distribution_y)) / U256::from(PRECISION);

        let amounts = Bytes32::encode(x1_distributed.as_u128(), x2_distributed.as_u128());

        Ok((amounts, self.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethnum::U256;

    #[test]
    fn test_get_amounts_and_id_normal_case() {
        let lc: LiquidityConfigurations = LiquidityConfigurations {
            distribution_x: (0.1 * PRECISION as f64) as u64,
            distribution_y: (0.1 * PRECISION as f64) as u64,
            id: 1,
        };

        let amounts_in = Bytes32::encode(1000, 2000); // Assuming you have such a method

        let result = lc.get_amounts_and_id(amounts_in).unwrap();

        let expected_x1_distributed =
            (U256::from(1000u128) * U256::from(lc.distribution_x)) / U256::from(PRECISION);
        let expected_x2_distributed =
            U256::from(2000u128) * U256::from(lc.distribution_y) / U256::from(PRECISION);

        let expected_amounts = Bytes32::encode(
            expected_x1_distributed.as_u128(),
            expected_x2_distributed.as_u128(),
        );

        assert_eq!(result, (expected_amounts, 1));
    }

    #[test]
    fn test_get_amounts_and_id_zero_case() {
        let lc = LiquidityConfigurations {
            distribution_x: 0,
            distribution_y: 0,
            id: 0,
        };
        let amounts_in = Bytes32::encode(0, 0); // Assuming you have such a method
        let result = lc.get_amounts_and_id(amounts_in).unwrap();

        let expected_amounts = Bytes32::encode(0, 0);

        assert_eq!(result, (expected_amounts, 0));
    }
}
