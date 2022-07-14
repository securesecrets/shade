use crate::{
    contract_interfaces::dao::adapter,
    utils::{
        asset::Contract, 
        generic_response::ResponseStatus
    },
};
use crate::c_std::{Binary, Decimal, Delegation, Addr, Uint128, Validator};


use crate::utils::{HandleCallback, InitCallback, Query};
use cosmwasm_schema::{cw_serde};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub treasury: Addr,
    pub pair: Contract,
    pub token_a: Contract,
    pub token_b: Contract,
    pub liquidity_token: Contract,
    pub reward_token: Option<Contract>,
    pub rewards_contract: Option<Contract>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    pub treasury: Addr,
    pub viewing_key: String,
    pub pair: Contract,
    pub token_a: Contract,
    pub token_b: Contract,
    pub rewards_contract: Option<Contract>,
}

impl InitCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    /* token_a || token_b
     * - check and provide as much as you can based on balances
     * 
     * LP share token
     * - Bond the share token, to be used when unbonding
     */
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    UpdateConfig {
        config: Config,
    },
    Adapter(adapter::SubHandleMsg),
}

impl HandleCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum HandleAnswer {
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
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    //Ratio {},
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
    // Should add to %100
    //Ratio { token_a: Uint128, token_b: Uint128 },
}

/* NOTE
 * 'reward_token' isn't technically supported
 * if it collides with one of the pair tokens 
 * it will be treated as such
 * Otherwise it will be sent straight to treasury on claim
 */
pub fn is_supported_asset(config: &Config, asset: &Addr) -> bool {
    vec![
        config.token_a.address.clone(),
        config.token_b.address.clone(),
        config.liquidity_token.address.clone(),
    ].contains(asset) 
}

pub fn get_supported_asset(
    config: &Config, 
    asset: &Addr
) -> Contract {
    vec![
        config.token_a.clone(),
        config.token_b.clone(),
        config.liquidity_token.clone(),
    ].into_iter().find(|a| a.address == *asset).unwrap()
}
