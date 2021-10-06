use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128, Binary};
use crate::asset::Contract;
use crate::generic_response::ResponseStatus;
use secret_toolkit::{snip20, utils::{InitCallback, HandleCallback, Query}};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TreasuryConfig {
    pub owner: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Snip20Asset {
    pub contract: Contract,
    pub token_info: snip20::TokenInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Delegation {
    pub validator: HumanAddr,
    pub denom: String,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub viewing_key: String,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    UpdateConfig {
        owner: Option<HumanAddr>,
    },
    RegisterAsset {
        contract: Contract,
        // Percent of asset to have staked on the derivative staking contract
        stake: Option<Uint128>,
    },

    //Trigger to refresh stake amounts, re-staking to reach approprate amounts
    // TODO: Maybe initiate unstaking if balance falls below a threshold 
    RefreshStake { },
    // Trigger to claim all pending staking rewards
    ClaimRewards { },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init { status: ResponseStatus, address: HumanAddr },
    UpdateConfig { status: ResponseStatus },
    RegisterAsset { status: ResponseStatus },
    Receive { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetBalance {
        contract: HumanAddr,
    },
    PendingRewards { 
        denom: String,
    },
    Delegations { },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: TreasuryConfig },
    Balance { amount: Uint128 },
}
