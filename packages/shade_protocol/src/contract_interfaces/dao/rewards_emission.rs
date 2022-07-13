use crate::{
    contract_interfaces::dao::adapter,
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use crate::c_std::{Binary, Decimal, Delegation, Addr, Uint128, Validator};

use crate::utils::{HandleCallback, InitCallback, Query};
use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Reward {
    pub asset: Addr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admins: Vec<Addr>,
    pub treasury: Addr,
    pub asset: Contract,
    pub distributor: Addr,
    pub rewards: Vec<Reward>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub config: Config,
    pub viewing_key: String,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    RefillRewards {
        rewards: Vec<Reward>,
    },
    UpdateConfig {
        config: Config,
    },
    RegisterAsset {
        asset: Contract,
    },
    Adapter(adapter::SubHandleMsg),
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    RegisterAsset {
        status: ResponseStatus,
    },
    RefillRewards {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    PendingAllowance { asset: Addr },
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    PendingAllowance { amount: Uint128 },
}
