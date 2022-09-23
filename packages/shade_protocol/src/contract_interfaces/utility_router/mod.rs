pub mod error;

use crate::{
    c_std::Binary,
    utils::{
        asset::{Contract, RawContract},
        generic_response::ResponseStatus,
        ExecuteCallback,
        InstantiateCallback,
        Query,
    },
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdResult};
use serde::Serialize;
use std::fmt;

#[cw_serde]
pub struct InstantiateMsg {
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
    SetStatus { status: RouterStatus },
    SetContract { key: String, contract: RawContract },
    SetAddress { key: String, address: String },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum HandleAnswer {
    SetStatus { status: ResponseStatus },
    SetContract { status: ResponseStatus },
    SetAddress { status: ResponseStatus },
}

#[cw_serde]
pub enum QueryMsg {
    Status {},
    GetContract { key: String },
    GetContracts { keys: Vec<String> },
    GetAddress { key: String },
    GetAddresses { keys: Vec<String> },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Status { contract_status: RouterStatus },
    GetContract { contract: Contract },
    GetContracts { contracts: Vec<Contract> },
    GetAddress { address: Addr },
    GetAddresses { addresses: Vec<Addr> },
}

#[derive(Clone)]
pub enum UtilityContract {
    AdminAuth,
    QueryAuth,
    Treasury,
    OracleRouter,
}

impl fmt::Display for UtilityContract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UtilityContract::AdminAuth => "ADMIN_AUTH",
                UtilityContract::OracleRouter => "ORACLE_ROUTER",
                UtilityContract::QueryAuth => "QUERY_AUTH",
                UtilityContract::Treasury => "TREASURY",
            }
            .to_string()
        )
    }
}

#[derive(Clone)]
pub enum UtilityAddresses {
    Multisig,
}

impl UtilityAddresses {
    pub fn into_string(self) -> String {
        match self {
            UtilityAddresses::Multisig => "MULTISIG",
        }
        .to_string()
    }
}
