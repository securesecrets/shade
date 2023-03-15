use crate::utils::{
    asset::{Contract, RawContract},
    generic_response::ResponseStatus,
    storage::plus::{ItemStorage, MapStorage},
    ExecuteCallback,
    InstantiateCallback,
    Query,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint128};
use secret_storage_plus::{Item, Map};

#[cw_serde]
pub struct AmountMinted(pub Uint128);

impl MapStorage<'static, String> for AmountMinted {
    const MAP: Map<'static, String, Self> = Map::new("amount_minted-");
}

#[cw_serde]
pub struct RegisteredToken {
    pub burn_token: Contract,
    pub mint_token: Contract,
}

impl MapStorage<'static, String> for RegisteredToken {
    const MAP: Map<'static, String, Self> = Map::new("registered_tokens-");
}

#[cw_serde]
pub struct Config {
    pub admin: Contract,
}

impl ItemStorage for Config {
    const ITEM: Item<'static, Config> = Item::new("item_config");
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
    RegistragionStatus { token: String },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
    Metrics { amount_minted: Option<Uint128> },
    RegistrationStatus { status: Option<RegisteredToken> },
}
