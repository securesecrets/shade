use super::*;
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

pub fn set_status(
    status: RouterStatus,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::SetStatus { status }.to_cosmos_msg(contract, vec![])
}

pub fn set_contract(
    key: UtilityKey,
    set_contract: Contract,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::SetContract {
        key: key.to_string(),
        contract: set_contract.into(),
    }
    .to_cosmos_msg(contract, vec![])
}

pub fn set_address(
    key: UtilityKey,
    address: Addr,
    padding: Option<String>,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    ExecuteMsg::SetAddress {
        key: key.to_string(),
        address: address.to_string(),
    }
    .to_cosmos_msg(contract, vec![])
}

pub fn get_contract(
    querier: &QuerierWrapper,
    key: UtilityKey,
    contract: &Contract,
) -> StdResult<Contract> {
    match (QueryMsg::GetContract {
        key: key.to_string(),
    }
    .query(querier, contract)?)
    {
        QueryAnswer::GetContract { contract } => Ok(contract),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub fn get_contracts(
    querier: &QuerierWrapper,
    keys: Vec<UtilityKey>,
    contract: &Contract,
) -> StdResult<Vec<Contract>> {
    match (QueryMsg::GetContracts {
        keys: keys.iter().map(|k| k.to_string()).collect(),
    }
    .query(querier, contract)?)
    {
        QueryAnswer::GetContracts { contracts } => Ok(contracts),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub fn get_address(
    querier: &QuerierWrapper,
    key: UtilityKey,
    contract: &Contract,
) -> StdResult<Addr> {
    match (QueryMsg::GetAddress {
        key: key.to_string(),
    }
    .query(querier, contract)?)
    {
        QueryAnswer::GetAddress { address } => Ok(address),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub fn get_addresses(
    querier: &QuerierWrapper,
    keys: Vec<UtilityKey>,
    contract: &Contract,
) -> StdResult<Vec<Addr>> {
    match (QueryMsg::GetAddresses {
        keys: keys.iter().map(|k| k.to_string()).collect(),
    }
    .query(querier, contract)?)
    {
        QueryAnswer::GetAddresses { addresses } => Ok(addresses),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub fn get_keys(
    querier: &QuerierWrapper,
    start: usize,
    limit: usize,
    contract: &Contract,
) -> StdResult<Vec<UtilityKey>> {
    match (QueryMsg::GetKeys { start, limit }.query(querier, contract)?) {
        QueryAnswer::GetKeys { keys } => Ok(keys
            .iter()
            .map(|k| UtilityKey::from_str(k).unwrap())
            .collect::<Vec<UtilityKey>>()),
        _ => Err(StdError::generic_err("Query failed")),
    }
}
