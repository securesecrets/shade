use crate::{
    c_std::{Addr, Binary, Decimal, Delegation, Uint128, Validator},
    utils::{
        asset::{Contract, RawContract},
        generic_response::ResponseStatus,
    },
};

use crate::contract_interfaces::dao::adapter;

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Config {
    pub admin_auth: Contract,
    //pub treasury: Addr,
    // This is the contract that will "unbond" funds
    pub owner: Addr,
    pub sscrt: Contract,
    pub staking_derivatives: Contract,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: RawContract,
    pub owner: String,
    pub sscrt: RawContract,
    pub viewing_key: String,
    pub staking_derivatives: RawContract,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive {
        sender: String,
        from: String,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    UpdateConfig {
        config: Config,
    },
    Adapter(adapter::SubExecuteMsg),
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
    Init {
        status: ResponseStatus,
        address: String,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    Receive {
        status: ResponseStatus,
        validator: Validator,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
}

// STAKING DERIVATIVES INTERFACE
// TODO move to common location
pub mod staking_derivatives {

    #[cw_serde]
    pub struct ExecuteMsg {
        Stake {},
        Unbond {
            redeem_amount: Uint128,
        },
        Claim {},
    }

    impl ExecuteCallback for ExecuteMsg {
        const BLOCK_SIZE: usize = 256;
    }

    #[cw_serde]
    pub struct QueryMsg {
        Unbonding { 
            address: Addr,
            key: String,
            page: Option<u32>,
            page_size: Option<u32>,
            time: Option<u64>,
        },
        Holdings {
            address: Addr,
            key: String,
            time: u64,
        },
    }

    #[cw_serde]
    pub struct Unbond {
        pub amount: Uint128,
        pub unbonds_at: u64,
        pub is_mature: Option<bool>,
    }

    #[cw_serde]
    pub struct WeightedValidator {
        pub validator: HumanAddr,
        pub weight: u8,
    }

    #[cw_serde]
    pub struct QueryAnswer {
        Unbonding {
            count: u64,
            claimable_scrt: Option<Uint128>,
            unbondings: Vec<Unbond>,
            unbond_amount_in_next_batch: Uint128,
            estimated_time_of_maturity_for_next_batch: Option<u64>,
        },
        Holdings {
            claimable_scrt: Uint128,
            unbonding_scrt: Uint128,
            token_balance: Uint128,
            token_balance_value_in_scrt: Uint128,
        },
    }

    impl Query for QueryMsg {
        const BLOCK_SIZE: usize = 256;
    }
}
