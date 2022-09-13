pub mod error;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdResult};
use serde::Serialize;
use crate::c_std::Binary;
use crate::utils::{InstantiateCallback, ExecuteCallback, Query};
use crate::utils::generic_response::ResponseStatus;
use crate::utils::storage::plus::{ItemStorage, Item};
use crate::utils::asset::Contract;

#[cw_serde]
pub struct InstantiateMsg{
    pub admin_auth: Contract,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum RouterStatus {
    Running,
    UnderMaintenance,
}

#[cw_serde]
pub enum ExecuteMsg {
    ToggleStatus {
        status: RouterStatus,
        padding: Option<String>,
    },
    SetContract {
        utility_contract_name: String,
        contract: Contract,
        query: Option<Binary>,
        padding: Option<String>,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum HandleAnswer {
    SetStatus {
        status: ResponseStatus
    },
    SetContract {
        status: ResponseStatus
    }
}

#[cw_serde]
pub enum QueryMsg {
    Status {},
    ForwardQuery {
        utility_name: String,
        query: Binary
    },
    GetContract {
        utility_name: String,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Status {
        contract_status: RouterStatus
    },
    ForwardQuery {
        status: ResponseStatus,
        result: Binary
    },
    GetContract {
        status: ResponseStatus,
        contract: Contract
    }
}

#[derive(Clone)]
pub enum UtilityContracts {
    AdminAuth,
    QueryAuth,
    Treasury,
    TreasuryManager,
    OracleRouter,
}

// NOTE: SHADE_{CONTRACT_NAME}_{POTENTIAL_VERSION}

impl UtilityContracts {
    pub fn into_string(self) -> String {
        match self {
            UtilityContracts::AdminAuth => "SHADE_ADMIN_AUTH",
            UtilityContracts::OracleRouter => "SHADE_ORACLE_ROUTER",
            UtilityContracts::QueryAuth => "SHADE_QUERY_AUTH",
            UtilityContracts::Treasury => "SHADE_TREASURY",
            UtilityContracts::TreasuryManager => "SHADE_TREASURY_MANAGER",
        }
        .to_string()
    }
}