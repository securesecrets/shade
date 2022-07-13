use crate::c_std::{
    BalanceResponse,
    BankQuery,
    Addr,
    StdResult,
    Uint128,
    Deps,
};

use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Contract {
    pub address: Addr,
    pub code_hash: String,
}

//TODO:  move away from here
pub fn scrt_balance(
    deps: Deps,
    address: Addr,
) -> StdResult<Uint128> {
    let resp: BalanceResponse = deps.querier.query(
        &BankQuery::Balance {
            address,
            denom: "uscrt".to_string(),
        }
        .into(),
    )?;

    Ok(resp.amount.amount)
}
