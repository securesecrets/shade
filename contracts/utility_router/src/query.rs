use shade_protocol::{
    c_std::{Deps, StdResult},
    contract_interfaces::utility_router::{error::*, *},
};

use crate::storage::{ADDRESSES, CONTRACTS, KEYS, STATUS};

pub fn get_status(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Status {
        contract_status: STATUS.load(deps.storage)?,
    })
}

pub fn get_contract(deps: Deps, key: String) -> StdResult<QueryAnswer> {
    if let Some(contract) = CONTRACTS.may_load(deps.storage, key.clone())? {
        Ok(QueryAnswer::GetContract { contract })
    } else {
        Err(no_contract_found(key))
    }
}

pub fn get_contracts(deps: Deps, keys: Vec<String>) -> StdResult<QueryAnswer> {
    let mut contracts = vec![];
    for key in keys {
        if let Some(contract) = CONTRACTS.may_load(deps.storage, key.clone())? {
            contracts.push(contract);
        } else {
            return Err(no_contract_found(key));
        }
    }

    Ok(QueryAnswer::GetContracts { contracts })
}

pub fn get_address(deps: Deps, key: String) -> StdResult<QueryAnswer> {
    if let Some(contract) = CONTRACTS.may_load(deps.storage, key.clone())? {
        Ok(QueryAnswer::GetAddress {
            address: contract.address,
        })
    } else if let Some(address) = ADDRESSES.may_load(deps.storage, key.clone())? {
        Ok(QueryAnswer::GetAddress { address })
    } else {
        Err(no_address_found(key))
    }
}

pub fn get_addresses(deps: Deps, keys: Vec<String>) -> StdResult<QueryAnswer> {
    let mut addresses = vec![];
    for key in keys {
        if let Some(contract) = CONTRACTS.may_load(deps.storage, key.clone())? {
            addresses.push(contract.address);
        } else if let Some(address) = ADDRESSES.may_load(deps.storage, key.clone())? {
            addresses.push(address);
        } else {
            return Err(no_address_found(key));
        }
    }
    Ok(QueryAnswer::GetAddresses { addresses })
}

pub fn get_keys(deps: Deps, start: usize, limit: usize) -> StdResult<QueryAnswer> {
    let keys = KEYS.load(deps.storage)?;
    if start + limit > keys.len() {
        Ok(QueryAnswer::GetKeys {
            keys: keys[start..].to_vec(),
        })
    } else {
        Ok(QueryAnswer::GetKeys {
            keys: keys[start..start + limit].to_vec(),
        })
    }
}
