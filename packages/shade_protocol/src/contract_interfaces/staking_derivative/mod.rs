// Types imported from staking-derivatives private repo used for interfacing with stkd-SCRT
// and updated to v1. Types copied as needed.

use cosmwasm_std::Addr;
use cosmwasm_std::Uint128;

use crate::utils::{ExecuteCallback, Query};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum HandleMsg {
    /// stake the sent SCRT
    Stake {},
    /// Unbond SCRT
    Unbond {
        /// amount of derivative tokens to redeem
        redeem_amount: Uint128,
    },
}

impl ExecuteCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryMsg {
    /// display the validator addresses, amount of bonded SCRT, amount of available SCRT not
    /// reserved for mature unbondings, amount of pending staking rewards not yet claimed,
    /// the derivative token supply, and the price of the derivative token in SCRT to 6 decimals
    StakingInfo {
        /// time in seconds since 01/01/1970.
        time: u64,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    /// displays staking info
    StakingInfo {
        /// validator addresses and their weights
        validators: Vec<WeightedValidator>,
        /// unbonding time
        unbonding_time: u32,
        /// minimum number of seconds between unbonding batches
        unbonding_batch_interval: u32,
        /// earliest time of next batch unbonding
        next_unbonding_batch_time: u64,
        /// amount of SCRT that will unbond in the next batch
        unbond_amount_of_next_batch: Uint128,
        /// true if a batch unbonding is in progress
        batch_unbond_in_progress: bool,
        /// amount of bonded SCRT
        bonded_scrt: Uint128,
        /// amount of SCRT reserved for mature unbondings
        reserved_scrt: Uint128,
        /// amount of available SCRT not reserved for mature unbondings
        available_scrt: Uint128,
        /// unclaimed staking rewards
        rewards: Uint128,
        /// total supply of derivative token
        total_derivative_token_supply: Uint128,
        /// price of derivative token in SCRT to 6 decimals
        price: Uint128,
    }
}

// validators and their weights
#[cw_serde]
pub struct WeightedValidator {
    /// the validator's address
    pub validator: Addr,
    /// the validator's weight in whole percents
    pub weight: u8,
}
