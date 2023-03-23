// Types imported from staking-derivatives private repo used for interfacing with stkd-SCRT
// and updated to v1. Types copied as needed, feel free to add.
// Types also included for mock_stkd contract

use cosmwasm_std::{Addr, Binary};
use cosmwasm_std::Uint128;

use crate::utils::{
    ExecuteCallback, Query, 
};
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
    /// claim matured unbondings
    Claim {},

    SetViewingKey {
        key: String,
        padding: Option<String>,
    },
    Send {
        recipient: Addr,
        recipient_code_hash: Option<String>,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },

    // For mock_stkd contract, to simulate passage of time for unbondings
    MockFastForward {
        steps: u32,
    },
}

#[cw_serde]
pub enum HandleAnswer {
    Stake {
        /// amount of uSCRT staked
        scrt_staked: Uint128,
        /// amount of derivative token minted
        tokens_returned: Uint128,
    },
    Unbond {
        /// amount of derivative tokens redeemed
        tokens_redeemed: Uint128,
        /// amount of scrt to be unbonded (available in 21 days after the batch processes)
        scrt_to_be_received: Uint128,
        /// estimated time of maturity
        estimated_time_of_maturity: u64,
    },
    Claim {
        /// amount of SCRT claimed
        withdrawn: Uint128,
        /// fees collected
        fees: Uint128,
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
    Unbonding {
        /// address whose unclaimed unbondings to display
        address: Addr,
        /// the address' viewing key
        key: String,
        /// optional page number to display
        page: Option<u32>,
        /// optional page size
        page_size: Option<u32>,
        /// optional time in seconds since 01/01/1970.  If provided, response will
        /// include the amount of SCRT that can be withdrawn at that time
        time: Option<u64>,
    },
    Balance {
        address: Addr,
        key: String,
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
    },
    /// displays user's unclaimed unbondings
    Unbonding {
        /// number of unclaimed unbondings
        count: u64,
        /// amount of claimable SCRT at the specified time (if given)
        claimable_scrt: Option<Uint128>,
        /// unclaimed unbondings
        unbondings: Vec<Unbond>,
        /// total amount of pending unbondings that will begin maturing in the next batch
        unbond_amount_in_next_batch: Uint128,
        /// optional estimated time the next batch of unbondings will mature.  Only provided
        /// if the user has SCRT waiting to be unbonded in the next batch
        estimated_time_of_maturity_for_next_batch: Option<u64>,
    },
    Balance {
        amount: Uint128,
    },
}

/// validators and their weights
#[cw_serde]
pub struct WeightedValidator {
    /// the validator's address
    pub validator: Addr,
    /// the validator's weight in whole percents
    pub weight: u8,
}

/// Unbonding data
#[cw_serde]
pub struct Unbond {
    /// amount of SCRT unbonding
    pub amount: Uint128,
    /// time of maturation in seconds since 01/01/1970
    pub unbonds_at: u64,
    /// optional bool if time was supplied, which is true if the unbonding is mature
    pub is_mature: Option<bool>,
}
