//! ### Liquidity Book Constants Library
//! Author: Kent
//!
//! Set of constants for Liquidity Book contracts.

use ethnum::U256;

pub static SCALE_OFFSET: u8 = 128;
pub static SCALE: U256 = U256::from_words(1, 0);

pub static PRECISION: u128 = 1_000_000_000_000_000_000; // 1e18
pub static SQUARED_PRECISION: u128 = PRECISION * PRECISION;

pub static MAX_FEE: u128 = 100_000_000_000_000_000; // 10% of 1e18
pub static MAX_PROTOCOL_SHARE: u16 = 2_500; // 25% of the fee

pub static BASIS_POINT_MAX: u16 = 10_000;
