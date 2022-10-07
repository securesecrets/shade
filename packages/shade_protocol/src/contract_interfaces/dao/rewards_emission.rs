use crate::{
    c_std::{Addr, Binary, Decimal, Delegation, Uint128, Validator},
    contract_interfaces::dao::adapter,
    utils::{asset::Contract, generic_response::ResponseStatus},
};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Reward {
    pub asset: Addr,
    pub amount: Uint128,
}

#[cw_serde]
pub struct Config {
    pub admins: Vec<Addr>,
    pub treasury: Addr,
    pub asset: Contract,
    pub distributor: Addr,
    pub rewards: Vec<Reward>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub config: Config,
    pub viewing_key: String,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    RefillRewards {
        rewards: Vec<Reward>,
    },
    UpdateConfig {
        config: Config,
    },
    RegisterAsset {
        asset: Contract,
    },
    Adapter(adapter::SubHandleMsg),
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
    RegisterAsset {
        status: ResponseStatus,
    },
    RefillRewards {
        status: ResponseStatus,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    PendingAllowance { asset: Addr },
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
    PendingAllowance { amount: Uint128 },
}
