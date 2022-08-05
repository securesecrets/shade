use crate::{
    contract_interfaces::snip20::helpers::Snip20Asset,
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use crate::c_std::Uint128;
use crate::c_std::{Binary, Addr};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::{cw_serde};
use std::convert::TryFrom;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub limit: Uint128,
    //pub token: Contract,
    //pub treasury: Addr,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    pub token: Contract,
    pub limit: Uint128,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        config: Config,
    },
    RemoveWhitelist {
        // contract?
        address: Addr,
    },
    AddWhitelist {
        // contract?
        address: Addr,
    },
    Mint {
        amount: Uint128,
    },
    // Receive config.token to pay back liabilities
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum HandleAnswer {
    Init {
        status: ResponseStatus,
        address: Addr,
    },
    RemoveWhitelist {
        status: ResponseStatus,
    },
    AddWhitelist {
        status: ResponseStatus,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    Mint {
        status: ResponseStatus,
        amount: Uint128,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Whitelist {},
    Liabilities {},
    Token {},
    Config {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Whitelist {
        whitelist: Vec<Addr>,
    },
    Liabilities {
        outstanding: Uint128,
        limit: Uint128,
    },
    Token {
        token: Snip20Asset,
    },
    Config {
        config: Config,
    },
}
