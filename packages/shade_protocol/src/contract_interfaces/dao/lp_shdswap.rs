use crate::{
    c_std::{Addr, Binary, Uint128},
    contract_interfaces::dao::adapter,
    utils::{
        asset::Contract,
        generic_response::ResponseStatus,
        ExecuteCallback,
        InstantiateCallback,
        Query,
    },
};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum SplitMethod {
    Conversion { contract: Contract },
    //TODO implement
    /*
    Market {
        // "market_buy" contract
        contract: Contract,
    },
    Lend {
        overseer: Contract,
    },
    */
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub treasury: Addr,
    pub pair: Contract,
    pub token_a: Contract,
    pub token_b: Contract,
    pub liquidity_token: Contract,
    pub staking_contract: Option<Contract>,
    pub reward_token: Option<Contract>,
    pub split: Option<SplitMethod>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    pub treasury: Addr,
    pub viewing_key: String,
    pub pair: Contract,
    pub token_a: Contract,
    pub token_b: Contract,
    pub staking_contract: Option<Contract>,
}

impl InstantiateCallback for InstantiateMsg {
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
    // TODO Refresh approvals to max
    // admin only
    RefreshApprovals,
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
        address: Addr,
    },
    UpdateConfig {
        status: ResponseStatus,
        config: Config,
    },
    RefreshApprovals {
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
    if let Some(reward_token) = &config.reward_token {
        if reward_token.address == *asset {
            return true;
        }
    }

    vec![
        config.token_a.address.clone(),
        config.token_b.address.clone(),
        config.liquidity_token.address.clone(),
    ]
    .contains(asset)
}

pub fn get_supported_asset(config: &Config, asset: &Addr) -> Contract {
    vec![
        config.token_a.clone(),
        config.token_b.clone(),
        config.liquidity_token.clone(),
    ]
    .into_iter()
    .find(|a| a.address == *asset)
    .unwrap()
}
