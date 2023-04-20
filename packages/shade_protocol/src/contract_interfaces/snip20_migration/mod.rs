use crate::utils::{
    asset::{Contract, RawContract},
    generic_response::ResponseStatus,
    ExecuteCallback,
    InstantiateCallback,
    Query,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint128};

#[cw_serde]
pub struct RegisteredToken {
    pub burn_token: Contract,
    pub mint_token: Contract,
    pub burnable: Option<bool>,
}

#[cw_serde]
pub struct Config {
    pub admin: Contract,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Contract,
    pub tokens: Option<RegisteredToken>,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        admin: RawContract,
        padding: Option<String>,
    },
    RegisterMigrationTokens {
        burn_token: RawContract,
        mint_token: RawContract,
        burnable: Option<bool>,
        padding: Option<String>,
    },
    Receive {
        sender: String,
        from: String,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
    SetConfig {
        status: ResponseStatus,
        config: Config,
    },
    RegisterMigrationTokens {
        status: ResponseStatus,
    },
    Receive {
        status: ResponseStatus,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Metrics { token: String },
    RegistrationStatus { token: String },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
    Metrics { amount_minted: Uint128 },
    RegistrationStatus { status: RegisteredToken },
}
