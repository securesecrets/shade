//! ### Liquidity Book Liquidity Configurations Library
//! Author: Kent and Haseeb
//!
//! This library contains functions to encode and decode the config of a pool and interact with the encoded Bytes32.

use cosmwasm_schema::cw_serde;
use ethnum::U256;

use crate::liquidity_book::lb_libraries::types::Bytes32;

use super::packed_u128_math::PackedUint128Math;

pub const PRECISION: u64 = 1_000_000_000_000_000_000; // 1e18

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum LiquidityConfigurationsError {
    #[error("Liquidity Configurations Error: Distribution must be less than PRECISION")]
    InvalidConfig,
}

#[cw_serde]
pub struct LiquidityConfigurations {
    pub distribution_x: u64,
    pub distribution_y: u64,
    pub id: u32,
}

impl LiquidityConfigurations {
    /// Get the amounts and id from a config and amounts_in.
    ///
    /// # Arguments
    ///
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
        &self,
        amounts_in: Bytes32,
    ) -> Result<(Bytes32, u32), LiquidityConfigurationsError> {
        let (x1, x2) = amounts_in.decode();

        // Cannot overflow as
        // max x1 or x2 = 2^128.
        // max distribution value= 10^18
        // PRECISION = 10^18

        // (2^128 * 10^18)/10^18 = 3.4 * 10^38 <  1.157 * 10^77

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

    impl LiquidityConfigurations {
        pub fn new(
            distribution_x: u64,
            distribution_y: u64,
            id: u32,
        ) -> Result<Self, LiquidityConfigurationsError> {
            if (distribution_x > PRECISION) || (distribution_y > PRECISION) {
                Err(LiquidityConfigurationsError::InvalidConfig)
            } else {
                Ok(LiquidityConfigurations {
                    distribution_x,
                    distribution_y,
                    id,
                })
            }
        }

        pub fn update_distribution(
            &mut self,
            distribution_x: u64,
            distribution_y: u64,
        ) -> Result<(), LiquidityConfigurationsError> {
            if (distribution_x > PRECISION) || (distribution_y > PRECISION) {
                Err(LiquidityConfigurationsError::InvalidConfig)
            } else {
                self.distribution_x = distribution_x;
                self.distribution_y = distribution_y;
                Ok(())
            }
        }
    }

    #[test]
    fn test_get_amounts_and_id_normal_case() {
        let lc: LiquidityConfigurations = LiquidityConfigurations {
            distribution_x: (0.1 * PRECISION as f64) as u64,
            distribution_y: (0.1 * PRECISION as f64) as u64,
            id: 1,
        };

        let amounts_in = Bytes32::encode(1000, 2000);

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
        let amounts_in = Bytes32::encode(0, 0);
        let result = lc.get_amounts_and_id(amounts_in).unwrap();

        let expected_amounts = Bytes32::encode(0, 0);

        assert_eq!(result, (expected_amounts, 0));
    }

    #[test]
    fn test_new_valid_config() {
        let lc = LiquidityConfigurations::new(500_000_000_000_000_000, 500_000_000_000_000_000, 1);
        assert!(lc.is_ok());
    }

    #[test]
    fn test_new_invalid_config_x() {
        let lc = LiquidityConfigurations::new(PRECISION + 1, 500_000_000_000_000_000, 1);
        assert_eq!(lc, Err(LiquidityConfigurationsError::InvalidConfig));
    }

    #[test]
    fn test_new_invalid_config_y() {
        let lc = LiquidityConfigurations::new(500_000_000_000_000_000, PRECISION + 1, 1);
        assert_eq!(lc, Err(LiquidityConfigurationsError::InvalidConfig));
    }

    #[test]
    fn test_update_distribution_valid() {
        let mut lc =
            LiquidityConfigurations::new(300_000_000_000_000_000, 300_000_000_000_000_000, 1)
                .unwrap();
        let result = lc.update_distribution(400_000_000_000_000_000, 400_000_000_000_000_000);
        assert!(result.is_ok());
        assert_eq!(lc.distribution_x, 400_000_000_000_000_000);
        assert_eq!(lc.distribution_y, 400_000_000_000_000_000);
    }

    #[test]
    fn test_update_distribution_invalid_x() {
        let mut lc =
            LiquidityConfigurations::new(300_000_000_000_000_000, 300_000_000_000_000_000, 1)
                .unwrap();
        let result = lc.update_distribution(PRECISION + 1, 400_000_000_000_000_000);
        assert_eq!(result, Err(LiquidityConfigurationsError::InvalidConfig));
    }

    #[test]
    fn test_update_distribution_invalid_y() {
        let mut lc =
            LiquidityConfigurations::new(300_000_000_000_000_000, 300_000_000_000_000_000, 1)
                .unwrap();
        let result = lc.update_distribution(400_000_000_000_000_000, PRECISION + 1);
        assert_eq!(result, Err(LiquidityConfigurationsError::InvalidConfig));
    }
    #[test]
    fn test_equality() {
        let config1 = LiquidityConfigurations::new(100, 200, 1).unwrap();
        let config2 = LiquidityConfigurations::new(100, 200, 1).unwrap();
        assert_eq!(config1, config2);
    }

    #[test]
    fn test_debug_format() {
        let config = LiquidityConfigurations::new(100, 200, 1).unwrap();
        let debug_string = format!("{:?}", config);
        assert!(!debug_string.is_empty());
    }

    #[test]
    fn test_clone() {
        let config = LiquidityConfigurations::new(100, 200, 1).unwrap();
        let cloned_config = config.clone();
        assert_eq!(config, cloned_config);
    }
}
