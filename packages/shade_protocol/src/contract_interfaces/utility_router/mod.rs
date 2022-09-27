pub mod error;
pub mod helpers;

use crate::{
    c_std::{Binary, CosmosMsg, QuerierWrapper, StdError},
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
use std::{fmt, str::FromStr};

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
pub enum ExecuteAnswer {
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
    GetKeys { start: usize, limit: usize },
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
    GetKeys { keys: Vec<String> },
}

#[cw_serde]
pub enum UtilityKey {
    Multisig,
    AdminAuth,
    QueryAuth,
    Treasury,
    OracleRouter,
}

impl fmt::Display for UtilityKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UtilityKey::AdminAuth => "ADMIN_AUTH",
                UtilityKey::Multisig => "MULTISIG",
                UtilityKey::OracleRouter => "ORACLE_ROUTER",
                UtilityKey::QueryAuth => "QUERY_AUTH",
                UtilityKey::Treasury => "TREASURY",
            }
            .to_string()
        )
    }
}

impl FromStr for UtilityKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ADMIN_AUTH" => Ok(UtilityKey::AdminAuth),
            "MULTISIG" => Ok(UtilityKey::Multisig),
            "ORACLE_ROUTER" => Ok(UtilityKey::OracleRouter),
            "QUERY_AUTH" => Ok(UtilityKey::QueryAuth),
            "TREASURY" => Ok(UtilityKey::Treasury),
            _ => Err(()),
        }
    }
}
