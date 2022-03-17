pub mod stake;
use crate::{utils::{asset::Contract, generic_response::ResponseStatus}};
use crate::snip20::permit::Snip20Permit;
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, Query};
use serde::{Deserialize, Serialize};
use crate::shd_staking::stake::{QueueItem, StakeConfig, VecQueue};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Default, Debug)]
#[serde(rename_all = "snake_case")]
pub struct InitConfig {
    /// Indicates whether the total supply is public or should be kept secret.
    /// default: False
    public_total_supply: Option<bool>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub name: String,
    pub admin: Option<HumanAddr>,
    pub symbol: String,
    // Will default to staked token decimals if not set
    pub decimals: Option<u8>,
    pub share_decimals: u8,
    pub prng_seed: Binary,
    pub config: Option<InitConfig>,

    // Stake
    pub unbond_time: u64,
    pub staked_token: Contract,
    pub treasury: Option<HumanAddr>,
    pub treasury_code_hash: Option<String>,

    // Distributors
    pub limit_transfer: bool,
    pub distributors: Option<Vec<HumanAddr>>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveType {
    // User staking, users can pick between using the sender or fund allower
    Bond { useFrom: Option<bool> },
    // Adding staker rewards
    Reward,
    // Funding unbonds
    Unbond
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatusLevel {
    NormalRun,
    StopBonding,
    StopAllButUnbond, //Can set time to 0 for instant unbond
    StopAll,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Staking
    UpdateStakeConfig {
        unbond_time: Option<u64>,
        disable_treasury: bool,
        treasury: Option<HumanAddr>,
        padding: Option<String>
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>
    },
    Unbond {
        amount: Uint128,
        padding: Option<String>
    },
    ClaimUnbond {
        padding: Option<String>
    },
    ClaimRewards {
        padding: Option<String>
    },
    StakeRewards {
        padding: Option<String>
    },

    // Balance
    ExposeBalance {
        recipient: HumanAddr,
        code_hash: Option<String>,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>
    },

    ExposeBalanceWithCooldown {
        recipient: HumanAddr,
        code_hash: Option<String>,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>
    },

    // Distributors
    SetDistributorsStatus {
        enabled: bool,
        padding: Option<String>
    },
    AddDistributors {
        distributors: Vec<HumanAddr>,
        padding: Option<String>
    },
    SetDistributors {
        distributors: Vec<HumanAddr>,
        padding: Option<String>
    },

    ContractStatus {
        status: ContractStatusLevel,
    },

    // Implement this to receive balance information
    // ReceiveBalance {
    //      sender: HumanAddr,
    //      msg: Option<Binary>,
    //      balance: Uint128
    //      memo: Option<String>
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
    SetDistributorsStatus { status: ResponseStatus },
    AddDistributors { status: ResponseStatus },
    SetDistributors { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Staking
    StakeConfig {},
    TotalStaked {},
    // Total token shares per token
    StakeRate {},
    Unbonding {},
    Unfunded {
        start: u64,
        total: u64
    },
    Staked {
        address: HumanAddr,
        key: String,
        time: Option<u64>,
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
        time: Option<u64>,
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
    // Shares per token
    StakeRate {
        shares: Uint128
    },
    Staked {
        tokens: Uint128,
        shares: Uint128,
        pending_rewards: Uint128,
        unbonding: Uint128,
        unbonded: Option<Uint128>,
        cooldown: VecQueue<QueueItem>
    },
    Unbonding {
        total: Uint128
    },
    Unfunded {
        total: Uint128
    },

    // Distributors
    Distributors {
        distributors: Option<Vec<HumanAddr>>
    },
}