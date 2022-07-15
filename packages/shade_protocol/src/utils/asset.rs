use crate::c_std::{
    BalanceResponse,
    BankQuery,
    Addr,
    StdResult,
    Uint128,
    Deps,
};

use cosmwasm_schema::{cw_serde};
use cosmwasm_std::ContractInfo;
#[cfg(feature = "ensemble")]
use fadroma::prelude::ContractLink;

/// Should be accepted as user input because we shouldn't assume addresses are verified Addrs.
/// https://docs.rs/cosmwasm-std/latest/cosmwasm_std/struct.Addr.html
#[derive(Hash, Eq)]
#[cw_serde]
pub struct UnvalidatedContract {
    pub address: String,
    pub code_hash: String,
}

impl UnvalidatedContract {
    pub fn validate(self, deps: Deps) -> StdResult<Contract> {
        let valid_addr = deps.api.addr_validate(self.address.as_str())?;
        Ok(Contract::new(&valid_addr, &self.code_hash))
    }
}

#[derive(Hash, Eq)]
#[cw_serde]
pub struct Contract {
    pub address: Addr,
    pub code_hash: String,
}

impl Contract {
    pub fn new(address: &Addr, code_hash: &String) -> Self {
        Contract {
            address: address.clone(),
            code_hash: code_hash.to_string(),
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
