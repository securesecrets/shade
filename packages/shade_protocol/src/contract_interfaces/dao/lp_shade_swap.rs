use crate::{
    contract_interfaces::dao::adapter,
    utils::{
        asset::Contract, 
        generic_response::ResponseStatus
    },
};
use cosmwasm_std::{Binary, Decimal, Delegation, HumanAddr, Uint128, Validator};

use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SplitMethod {
    Conversion {
        contract: Contract,
    },
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admin: HumanAddr,
    pub treasury: HumanAddr,
    pub pair: Contract,
    pub token_a: Contract,
    pub token_b: Contract,
    pub liquidity_token: Contract,
    pub staking_contract: Option<Contract>,
    pub reward_token: Option<Contract>,
    pub split: Option<SplitMethod>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub treasury: HumanAddr,
    pub viewing_key: String,
    pub pair: Contract,
    pub token_a: Contract,
    pub token_b: Contract,
    pub staking_contract: Option<Contract>,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /* token_a || token_b
     * - check and provide as much as you can based on balances
     * 
     * LP share token
     * - Bond the share token, to be used when unbonding
     */
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
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
    Adapter(adapter::SubHandleMsg),
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init {
        status: ResponseStatus,
        address: HumanAddr,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    Receive {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    //Ratio {},
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
pub fn is_supported_asset(config: &Config, asset: &HumanAddr) -> bool {
    if let Some(reward_token) = config.reward_token {
        if reward_token.address == *asset {
            return true;
        }
    }

    vec![
        config.token_a.address.clone(),
        config.token_b.address.clone(),
        config.liquidity_token.address.clone(),
    ].contains(asset)
}

pub fn get_supported_asset(
    config: &Config, 
    asset: &HumanAddr
) -> Contract {
    vec![
        config.token_a.clone(),
        config.token_b.clone(),
        config.liquidity_token.clone(),
    ].into_iter().find(|a| a.address == *asset).unwrap()
}
