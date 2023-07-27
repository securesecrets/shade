use crate::{
    c_std::{Addr, Binary, Uint128},
    utils::{
        asset::{Contract, RawContract},
        cycle::Cycle,
        generic_response::ResponseStatus,
    },
};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Reward {
    //pub token: Addr,
    pub distributor: Contract,
    pub amount: Uint128,
    pub cycle: Cycle,
    pub last_refresh: String,
    // datetime string
    pub expiration: Option<String>,
}

#[cw_serde]
pub struct Config {
    pub admins: Vec<Addr>,
    pub treasury: Addr,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
    pub viewing_key: String,
    pub treasury: String,
    pub token: RawContract,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        config: Config,
    },
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    RefillRewards {},
    RegisterRewards {
        token: Addr, // Just for verification
        distributor: Contract,
        amount: Uint128,
        cycle: Cycle,
        expiration: Option<String>,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
    Init {
        status: ResponseStatus,
        address: Addr,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    Receive {
        status: ResponseStatus,
    },
    RegisterReward {
        status: ResponseStatus,
    },
    RefillRewards {
        status: ResponseStatus,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    //PendingAllowance { asset: Addr },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
    //PendingAllowance { amount: Uint128 },
}
