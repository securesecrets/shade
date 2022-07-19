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
use cosmwasm_std::{DepsMut, Env, CosmosMsg};
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

#[cfg(feature = "snip20")]
pub fn set_allowance(
    deps: DepsMut,
    env: &Env,
    spender: Addr,
    amount: Uint128,
    key: String,
    asset: Contract,
    cur_allowance: Option<Uint128>,
) -> StdResult<Vec<CosmosMsg>> {
    use secret_toolkit::snip20::allowance_query;

    use crate::snip20::helpers::{decrease_allowance_msg, increase_allowance_msg};


    let mut allowance = match cur_allowance {
        Some(cur) => cur,
        None => allowance_query(
                    &deps.querier,
                    env.contract.address.clone(),
                    spender.clone(),
                    key,
                    1,
                    asset.code_hash.clone(),
                    asset.address.clone(),
                )?.allowance,
    };

    match amount.cmp(&allowance) {
        // Decrease Allowance
        std::cmp::Ordering::Less => Ok(vec![decrease_allowance_msg(
            spender.clone(),
            (allowance - amount)?,
            None,
            None,
            1,
            asset.code_hash.clone(),
            asset.address.clone(),
        )?]),
        // Increase Allowance
        std::cmp::Ordering::Greater => Ok(vec![increase_allowance_msg(
            spender.clone(),
            (amount - allowance)?,
            None,
            None,
            1,
            asset.code_hash.clone(),
            asset.address.clone(),
        )?]),
        _ => Ok(vec![]),
    }
}
