use crate::c_std::{
    BalanceResponse,
    BankQuery,
    Addr,
    StdResult,
    Uint128,
    Deps,
};

use cosmwasm_schema::{cw_serde};
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
            code_hash: code_hash.to_string().clone(),
        }
    }

    pub fn validate_new(deps: Deps, address: &String, code_hash: &String) -> StdResult<Self> {
        let valid_addr = deps.api.addr_validate(address.as_str())?;
        Ok(Contract::new(&valid_addr, code_hash))
    }

    pub fn new_link(link: ContractLink<Addr>) -> Self {
        Contract {
            address: link.address,
            code_hash: link.code_hash,
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
