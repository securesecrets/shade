pub mod permit;
use crate::utils::asset::Contract;
use cosmwasm_std::{Binary, HumanAddr, Querier, StdResult, Uint128};
use schemars::JsonSchema;
use secret_toolkit::{
    snip20::{token_info_query, Allowance, TokenInfo},
    utils::{HandleCallback, InitCallback, Query},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Snip20Asset {
    pub contract: Contract,
    pub token_info: TokenInfo,
    pub token_config: Option<TokenConfig>,
}

pub fn fetch_snip20<Q: Querier>(contract: &Contract, querier: &Q) -> StdResult<Snip20Asset> {
    Ok(Snip20Asset {
        contract: contract.clone(),
        token_info: token_info_query(
            querier,
            1,
            contract.code_hash.clone(),
            contract.address.clone(),
        )?,
        token_config: Some(token_config_query(querier, contract.clone())?),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenConfig {
    pub public_total_supply: bool,
    pub deposit_enabled: bool,
    pub redeem_enabled: bool,
    pub mint_enabled: bool,
    pub burn_enabled: bool,
}

// Temporary values while secret_toolkit updates
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Snip20Query {
    TokenConfig {},
}

impl Query for Snip20Query {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct TokenConfigResponse {
    pub token_config: TokenConfig,
}

pub fn token_config_query<Q: Querier>(querier: &Q, contract: Contract) -> StdResult<TokenConfig> {
    let answer: TokenConfigResponse =
        Snip20Query::TokenConfig {}.query(querier, contract.code_hash, contract.address)?;
    Ok(answer.token_config)
}

// Snip20 initializer
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitialBalance {
    pub address: HumanAddr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub name: String,
    pub admin: Option<HumanAddr>,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Option<Vec<InitialBalance>>,
    pub prng_seed: Binary,
    pub config: Option<InitConfig>,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Default, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub struct InitConfig {
    pub public_total_supply: Option<bool>,
    pub enable_deposit: Option<bool>,
    pub enable_redeem: Option<bool>,
    pub enable_mint: Option<bool>,
    pub enable_burn: Option<bool>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    ChangeAdmin {
        address: HumanAddr,
        padding: Option<String>,
    },
    // Native coin interactions
    Redeem {
        amount: Uint128,
        denom: Option<String>,
        padding: Option<String>,
    },
    Deposit {
        padding: Option<String>,
    },

    // Base ERC-20 stuff
    Transfer {
        recipient: HumanAddr,
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
    Send {
        recipient: HumanAddr,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
    Burn {
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
    RegisterReceive {
        code_hash: String,
        padding: Option<String>,
    },
    CreateViewingKey {
        entropy: String,
        padding: Option<String>,
    },
    SetViewingKey {
        key: String,
        padding: Option<String>,
    },
    // Mint
    Mint {
        recipient: HumanAddr,
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
    AddMinters {
        minters: Vec<HumanAddr>,
        padding: Option<String>,
    },
    RemoveMinters {
        minters: Vec<HumanAddr>,
        padding: Option<String>,
    },
    SetMinters {
        minters: Vec<HumanAddr>,
        padding: Option<String>,
    },
    IncreaseAllowance {
        owner: HumanAddr,
        spender: HumanAddr,
        amount: Uint128,
    },
    DecreaseAllowance {
        owner: HumanAddr,
        spender: HumanAddr,
        amount: Uint128,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    TokenInfo {},
    TokenConfig {},
    ExchangeRate {},
    Allowance {
        owner: HumanAddr,
        spender: HumanAddr,
        key: String,
    },
    Balance {
        address: HumanAddr,
        key: String,
    },
    Minters {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    TokenInfo {
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: Option<Uint128>,
    },
    TokenConfig {
        public_total_supply: bool,
        deposit_enabled: bool,
        redeem_enabled: bool,
        mint_enabled: bool,
        burn_enabled: bool,
    },
    ExchangeRate {
        rate: Uint128,
        denom: String,
    },
    Allowance {
        allowance: Allowance,
        /*
        spender: HumanAddr,
        owner: HumanAddr,
        allowance: Uint128,
        expiration: Option<u64>,
        */
    },
    Balance {
        amount: Uint128,
    },
    ViewingKeyError {
        msg: String,
    },
    Minters {
        minters: Vec<HumanAddr>,
    },
}
