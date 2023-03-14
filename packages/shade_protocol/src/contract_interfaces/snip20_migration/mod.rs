use crate::utils::{
    asset::Contract,
    storage::plus::MapStorage,
    ExecuteCallback,
    InstantiateCallback,
    Query,
};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct AmountMinted(pub Uint128);

impl MapStorage<'static, u16> for AmountMinted {
    const MAP: Map<'static, u16, Self> = Map::new("amount_minted-");
}

#[cw_serde]
pub struct RegisteredToken {
    pub burn_token: Contract,
    pub mint_token: Contract,
}

impl MapStorage<'static, u16> for RegisteredToken {
    const MAP: Map<'static, u16, Self> = Map::new("registered_tokens-");
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
        admin: contract,
        padding: Option<String>,
    },
    RegisterMigrationTokens {
        BurnToken: Contract,
        MintToken: Contract,
    },
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint123,
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
    Metrics { token: Addr },
    RegistragionStatus { token: Addr },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
    Metrics { AmountMinted: Uint128 },
    RegistrationStatus { status: Option<RegisteredToken> },
}
