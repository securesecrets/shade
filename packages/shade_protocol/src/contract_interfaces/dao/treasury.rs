use crate::utils::{
    asset::{Contract, RawContract},
    cycle::Cycle,
    generic_response::ResponseStatus,
};

use crate::c_std::{Addr, Api, Binary, Coin, StdResult, Uint128};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::cw_serde;

use crate::utils::storage::plus::period_storage::Period;

/// The permission referenced in the Admin Auth contract to give a user
/// admin permissions for the Shade Treasury
//pub const SHADE_TREASURY_ADMIN: &str = "SHADE_TREASURY_ADMIN";

#[cw_serde]
pub struct Config {
    pub admin_auth: Contract,
    pub multisig: Addr,
}

#[cw_serde]
pub enum RunLevel {
    Normal,
    Deactivated,
    Migrating,
}

#[cw_serde]
pub enum Context {
    Receive,
    Rebalance,
    Migration,
    Unbond,
    Wrap,
}

#[cw_serde]
pub enum Action {
    IncreaseAllowance,
    DecreaseAllowance,
    Unbond,
    Claim,
    FundsReceived,
    SendFunds,
    Wrap,
}

#[cw_serde]
pub struct Metric {
    pub action: Action,
    pub context: Context,
    pub timestamp: u64,
    pub token: Addr,
    pub amount: Uint128,
    pub user: Addr,
}

#[cw_serde]
pub enum AllowanceType {
    Amount,
    Portion,
}

#[cw_serde]
pub struct RawAllowance {
    pub spender: String,
    pub allowance_type: AllowanceType,
    pub cycle: Cycle,
    pub amount: Uint128,
    pub tolerance: Uint128,
}

impl RawAllowance {
    pub fn valid(self, api: &dyn Api) -> StdResult<Allowance> {
        Ok(Allowance {
            spender: api.addr_validate(self.spender.as_str())?,
            allowance_type: self.allowance_type,
            cycle: self.cycle,
            amount: self.amount,
            tolerance: self.tolerance,
        })
    }
}

#[cw_serde]
pub struct Allowance {
    pub spender: Addr,
    pub allowance_type: AllowanceType,
    pub cycle: Cycle,
    pub amount: Uint128,
    pub tolerance: Uint128,
}

#[cw_serde]
pub struct AllowanceMeta {
    pub spender: Addr,
    pub allowance_type: AllowanceType,
    pub cycle: Cycle,
    pub amount: Uint128,
    pub tolerance: Uint128,
    pub last_refresh: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: RawContract,
    pub multisig: String,
    pub viewing_key: String,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive {
        sender: String,
        from: String,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    UpdateConfig {
        admin_auth: Option<RawContract>,
        multisig: Option<String>,
    },
    RegisterAsset {
        contract: RawContract,
    },
    RegisterManager {
        contract: RawContract,
    },
    RegisterWrap {
        denom: String,
        contract: RawContract,
    },
    WrapCoins {},
    // Setup a new allowance
    Allowance {
        asset: String,
        allowance: RawAllowance,
        refresh_now: bool,
    },
    Update {
        asset: String,
    },
    SetRunLevel {
        run_level: RunLevel,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
    Init {
        status: ResponseStatus,
        address: String,
    },
    UpdateConfig {
        config: Config,
        status: ResponseStatus,
    },
    Receive {
        status: ResponseStatus,
    },
    RegisterAsset {
        status: ResponseStatus,
    },
    RegisterManager {
        status: ResponseStatus,
    },
    RegisterWrap {
        status: ResponseStatus,
    },
    Allowance {
        status: ResponseStatus,
    },
    Rebalance {
        status: ResponseStatus,
    },
    Migration {
        status: ResponseStatus,
    },
    Unbond {
        status: ResponseStatus,
    },
    Update {
        status: ResponseStatus,
    },
    RunLevel {
        run_level: RunLevel,
    },
    WrapCoins {
        success: Vec<Coin>,
        failed: Vec<Coin>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Assets {},
    // List of recurring allowances configured
    Allowances {
        asset: String,
    },
    // Current allowance to spender
    Allowance {
        asset: String,
        spender: String,
    },
    RunLevel,
    Metrics {
        date: Option<String>,
        epoch: Option<Uint128>,
        period: Period,
    },
    Balance {
        asset: String,
    },
    BatchBalance {
        assets: Vec<String>,
    },
    Reserves {
        asset: String,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
    Assets { assets: Vec<Addr> },
    Allowances { allowances: Vec<AllowanceMeta> },
    Allowance { amount: Uint128 },
    RunLevel { run_level: RunLevel },
    Metrics { metrics: Vec<Metric> },
    Balance { amount: Uint128 },
    Reserves { amount: Uint128 },
}
