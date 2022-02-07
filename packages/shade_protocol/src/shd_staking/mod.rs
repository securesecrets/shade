pub mod stake;
use crate::{utils::{asset::Contract, generic_response::ResponseStatus}};
use crate::snip20::permit::Snip20Permit;
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, Query};
use serde::{Deserialize, Serialize};
use crate::snip20::{InitConfig, InitialBalance};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StakeConfig {
    pub unbond_time: u64,
    pub staked_token: Contract,
    pub treasury: Option<HumanAddr>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub name: String,
    pub admin: Option<HumanAddr>,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Option<Vec<InitialBalance>>,
    pub prng_seed: Binary,
    pub config: Option<InitConfig>,

    // Stake
    pub unbond_time: u64,
    pub staked_token: Contract,
    pub treasury: Option<HumanAddr>,
    pub treasury_code_hash: Option<String>,

    // Distributors
    pub limit_transfer: bool,
    pub distributors: Option<HumanAddr>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveType {
    Bond,
    Reward
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Staking
    UpdateStakeConfig {
        unbond_time: Option<u64>,
        staked_token: Option<Contract>,
        disable_treasury: bool,
        treasury: Option<HumanAddr>,
        treasury_code_hash: Option<String>,
        padding: String
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        msg: Option<String>,
        padding: String
    },
    Unbond {
        amount: Uint128,
        padding: String
    },
    ClaimUnbond {
        padding: String
    },
    ClaimRewards {
        padding: String
    },
    StakeRewards {
        padding: String
    },

    // Balance
    ExposeBalance {
        recipient: HumanAddr,
        code_hash: String,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: String
    },

    // Distributors
    AddDistributors {
        distributors: Vec<String>,
        padding: String
    },
    SetDistributors {
        distributors: Vec<String>,
        padding: String
    },

    // Implement this to receive balance information
    // ReceiveBalance {
    //      sender: HumanAddr,
    //      msg: Option<String>,
    //      balance: Uint128
    // }
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateStakeConfig { status: ResponseStatus },
    Receive { status: ResponseStatus },
    Unbond { status: ResponseStatus },
    ClaimUnbond { status: ResponseStatus },
    ClaimRewards { status: ResponseStatus },
    StakeRewards { status: ResponseStatus },
    ExposeBalance { status: ResponseStatus },
    AddDistributors { status: ResponseStatus },
    SetDistributors { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Staking
    StakeConfig {},
    TotalStaked {},
    Unbonding {
        start: u64,
        end: u64
    },
    Staked {
        address: HumanAddr,
        key: String,
        time: u64,
    },

    // Distributors
    Distributors {},
    WithPermit {
        permit: Snip20Permit,
        query: QueryWithPermit,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryWithPermit {
    Staked {
        time: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    // Stake
    StakedConfig {
        config: StakeConfig,
    },
    TotalStaked {
        tokens: Uint128,
        shares: Uint128
    },
    Staked {
        tokens: Uint128,
        shares: Uint128,
        pending_rewards: Uint128,
        unbonding: Uint128,
        unbonded: Uint128
    },
    Unbonding {
        total: Uint128
    },

    // Distributors
    Distributors {
        distributors: Vec<HumanAddr>
    },
}