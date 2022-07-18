use std::vec;

use crate::c_std::{
    BalanceResponse,
    BankQuery,
    Addr,
    StdResult,
    Uint128,
    Deps,
    Api, ContractInfo, StdError
};
use cosmwasm_schema::{cw_serde};
#[cfg(feature = "ensemble")]
use fadroma::prelude::ContractLink;

/// Validates an optional address.
pub fn optional_addr_validate(api: &dyn Api, addr: Option<String>) -> StdResult<Option<Addr>> {
    let addr = if let Some(addr) = addr {
        Some(api.addr_validate(&addr)?)
    } else {
        None
    };

    Ok(addr)
}

/// Validates an optional RawContract.
pub fn optional_raw_contract_validate(api: &dyn Api, contract: Option<RawContract>) -> StdResult<Option<Contract>> {
    let contract = if let Some(contract) = contract {
        Some(contract.into_valid(api)?)
    } else {
        None
    };

    Ok(contract)
}

/// A contract that does not contain a validated address.
/// Should be accepted as user input because we shouldn't assume addresses are verified Addrs.
/// https://docs.rs/cosmwasm-std/latest/cosmwasm_std/struct.Addr.html
#[derive(Hash, Eq)]
#[cw_serde]
pub struct RawContract {
    pub address: String,
    pub code_hash: String,
}

impl RawContract {
    #[allow(clippy::ptr_arg)]
    pub fn new (address: &String, code_hash: &String) -> Self {
        RawContract { address: address.clone(), code_hash: code_hash.clone() }
    }
    pub fn into_valid(self, api: &dyn Api) -> StdResult<Contract> {
        let valid_addr = api.addr_validate(self.address.as_str())?;
        Ok(Contract::new(&valid_addr, &self.code_hash))
    }
}

impl From<Contract> for RawContract {
    fn from (item: Contract) -> Self {
        RawContract { address: item.address.into(), code_hash: item.code_hash }
    }
}

impl From<ContractInfo> for RawContract {
    fn from (item: ContractInfo) -> Self {
        RawContract { address: item.address.into(), code_hash: item.code_hash }
    }
}

#[cfg(feature = "ensemble")]
impl From<ContractLink<Addr>> for RawContract {
    fn from(item: ContractLink<Addr>) -> Self {
        RawContract { address: item.address.into(), code_hash: item.code_hash, }    
    }
}

#[derive(Hash, Eq)]
#[cw_serde]
pub struct Contract {
    pub address: Addr,
    pub code_hash: String,
}

impl Contract {
    #[allow(clippy::ptr_arg)]
    pub fn new(address: &Addr, code_hash: &String) -> Self {
        Contract { address: address.clone(), code_hash: code_hash.clone() }
    }

    pub fn validate_new(deps: Deps, address: &str, code_hash: &String) -> StdResult<Self> {
        let valid_addr = deps.api.addr_validate(address)?;
        Ok(Contract::new(&valid_addr, code_hash))
    }

}

impl From<ContractInfo> for Contract {
    fn from(item: ContractInfo) -> Self {
        Contract {
            address: item.address,
            code_hash: item.code_hash,
        }    
    }
}

#[cfg(feature = "ensemble")]
impl From<ContractLink<Addr>> for Contract {
    fn from(item: ContractLink<Addr>) -> Self {
        Contract {
            address: item.address,
            code_hash: item.code_hash,
        }    
    }
}

// In case we ever want to store a dynamic list of dependencies and refer to them in the contract by name.
// Helpful if we ever create a generic interface for something (i.e. oracles).

#[cw_serde]
pub struct RawDependency {
    pub name: String,
    pub contract: RawContract,
}

impl RawDependency {
    pub fn new(name: String, contract: RawContract) -> Self {
        RawDependency { name, contract }
    }

    pub fn into_valid(&self, api: &dyn Api) ->  StdResult<Dependency> {
        Ok(Dependency::new(self.name.clone(), self.contract.clone().into_valid(api)?))
    }
}

#[derive(Default)]
#[cw_serde]
pub struct RawDependencies(pub Vec<RawDependency>);

impl RawDependencies {
    pub fn into_valid(&self, api: &dyn Api) -> StdResult<Dependencies> {
        let deps = self.0.clone().into_iter().map(|d| d.into_valid(api)).collect::<StdResult<Vec<_>>>()?;
        Ok(Dependencies(deps))
    }
}

#[cw_serde]
pub struct Dependency {
    pub name: String,
    pub contract: Contract,
}

impl Dependency {
    pub fn new(name: String, contract: Contract) -> Self {
        Dependency { name, contract }
    }
}

#[derive(Default)]
#[cw_serde]
pub struct Dependencies(pub Vec<Dependency>);

impl Dependencies {
    pub fn get_dep(&self, name: &String) -> StdResult<Contract> {
        let item = self.0.as_slice().iter().find(|c| c.name.eq(name));
        match item {
            Some(item) => Ok(item.contract.clone()),
            None => Err(StdError::generic_err(format!("Could not find dependency named {}", name))),
        }
    }
}

//TODO:  move away from here
pub fn scrt_balance(
    deps: Deps,
    address: Addr,
) -> StdResult<Uint128> {
    let resp: BalanceResponse = deps.querier.query(
        &BankQuery::Balance {
            address: address.into(),
            denom: "uscrt".to_string(),
        }
        .into(),
    )?;

    Ok(resp.amount.amount)
}
