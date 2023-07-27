use std::vec;

use crate::{
    c_std::{Addr, Api, BalanceResponse, BankQuery, ContractInfo, Deps, StdResult, Uint128},
    BLOCK_SIZE,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CosmosMsg, DepsMut, Env};

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
pub fn optional_raw_contract_validate(
    api: &dyn Api,
    contract: Option<RawContract>,
) -> StdResult<Option<Contract>> {
    let contract = if let Some(contract) = contract {
        Some(contract.into_valid(api)?)
    } else {
        None
    };

    Ok(contract)
}

/// Validates an optional RawContract.
pub fn optional_validate(
    api: &dyn Api,
    contract: Option<RawContract>,
) -> StdResult<Option<ContractInfo>> {
    let contract = if let Some(contract) = contract {
        Some(contract.valid(api)?)
    } else {
        None
    };

    Ok(contract)
}

/// Validates a vector of Strings as Addrs
pub fn validate_vec(api: &dyn Api, unvalidated_addresses: Vec<String>) -> StdResult<Vec<Addr>> {
    let items: Result<Vec<_>, _> = unvalidated_addresses
        .iter()
        .map(|f| api.addr_validate(f.as_str()))
        .collect();
    Ok(items?)
}

/// A contract that does not contain a validated address.
/// Should be accepted as user input because we shouldn't assume addresses are verified Addrs.
/// https://docs.rs/cosmwasm-std/latest/cosmwasm_std/struct.Addr.html
#[derive(Hash, Eq, Default)]
#[cw_serde]
pub struct RawContract {
    pub address: String,
    pub code_hash: String,
}

impl RawContract {
    #[allow(clippy::ptr_arg)]
    pub fn new(address: &String, code_hash: &String) -> Self {
        RawContract {
            address: address.clone(),
            code_hash: code_hash.clone(),
        }
    }

    /// Being deprecated in favor of `valid` which turns this into ContractInfo
    /// instead of a Contract (which we are getting rid of)
    pub fn into_valid(self, api: &dyn Api) -> StdResult<Contract> {
        let valid_addr = api.addr_validate(self.address.as_str())?;
        Ok(Contract::new(&valid_addr, &self.code_hash))
    }

    pub fn valid(self, api: &dyn Api) -> StdResult<ContractInfo> {
        let valid_addr = api.addr_validate(self.address.as_str())?;
        Ok(ContractInfo {
            address: valid_addr,
            code_hash: self.code_hash.clone(),
        })
    }
}

impl From<Contract> for RawContract {
    fn from(item: Contract) -> Self {
        RawContract {
            address: item.address.into(),
            code_hash: item.code_hash,
        }
    }
}

impl From<ContractInfo> for RawContract {
    fn from(item: ContractInfo) -> Self {
        RawContract {
            address: item.address.into(),
            code_hash: item.code_hash,
        }
    }
}

#[derive(Hash, Eq)]
#[cw_serde]
/// In the process of being deprecated for [cosmwasm_std::ContractInfo] so use that
/// instead when possible.
pub struct Contract {
    pub address: Addr,
    pub code_hash: String,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            address: Addr::unchecked(String::default()),
            code_hash: Default::default(),
        }
    }
}

impl Contract {
    #[allow(clippy::ptr_arg)]
    pub fn new(address: &Addr, code_hash: &String) -> Self {
        Contract {
            address: address.clone(),
            code_hash: code_hash.clone(),
        }
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

impl Into<ContractInfo> for Contract {
    fn into(self) -> ContractInfo {
        ContractInfo {
            address: self.address,
            code_hash: self.code_hash,
        }
    }
}

//TODO:  move away from here
pub fn scrt_balance(deps: Deps, address: Addr) -> StdResult<Uint128> {
    let resp: BalanceResponse = deps.querier.query(
        &BankQuery::Balance {
            address: address.into(),
            denom: "uscrt".to_string(),
        }
        .into(),
    )?;

    Ok(resp.amount.amount)
}

#[cfg(feature = "snip20")]
pub fn set_allowance(
    deps: DepsMut,
    env: &Env,
    spender: Addr,
    amount: Uint128,
    key: String,
    asset: &Contract,
    cur_allowance: Option<Uint128>,
) -> StdResult<Vec<CosmosMsg>> {
    use crate::snip20::helpers::{allowance_query, decrease_allowance_msg, increase_allowance_msg};

    let allowance = match cur_allowance {
        Some(cur) => cur,
        None => {
            allowance_query(
                &deps.querier,
                env.contract.address.clone(),
                spender.clone(),
                key,
                1,
                asset,
            )?
            .allowance
        }
    };

    match amount.cmp(&allowance) {
        // Decrease Allowance
        std::cmp::Ordering::Less => Ok(vec![decrease_allowance_msg(
            spender,
            allowance.checked_sub(amount)?,
            None,
            None,
            BLOCK_SIZE,
            asset,
            vec![],
        )?]),
        // Increase Allowance
        std::cmp::Ordering::Greater => Ok(vec![increase_allowance_msg(
            spender,
            amount.checked_sub(amount)?,
            None,
            None,
            BLOCK_SIZE,
            asset,
            vec![],
        )?]),
        _ => Ok(vec![]),
    }
}
