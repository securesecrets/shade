use crate::{
    c_std::{Addr, Binary, Uint128},
    contract_interfaces::snip20::helpers::Snip20Asset,
    utils::{asset::Contract, generic_response::ResponseStatus},
};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::cw_serde;
use std::convert::TryFrom;

#[cw_serde]
pub struct Config {
    //TODO shade_admin contract
    pub admin: Addr,
    pub token: Contract,
    pub debt_ratio: Uint128,
    pub oracle: Contract,
    pub treasury: Contract,
    //pub interest: Uint128 <- probs not
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    pub token: Contract,
    pub debt_ratio: Uint128,
    pub oracle: Contract,
    pub treasury: Contract,
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
        // Contract?
        address: Addr,
    },
    AddWhitelist {
        // Contract?
        address: Addr,
    },
    AddCollateral {
        asset: Contract,
    },
    RemoveCollateral {
        asset: Contract,
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
pub enum ExecuteAnswer {
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
    RemoveCollateral {
        status: ResponseStatus,
    },
    AddCollateral {
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
    //TODO add once moved to storage
    //Collateral {},
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
