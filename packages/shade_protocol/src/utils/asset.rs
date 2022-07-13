use crate::c_std::{
    Api,
    BalanceResponse,
    BankQuery,
    Extern,
    Addr,
    Querier,
    StdResult,
    Storage,
    Uint128,
};

use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Contract {
    pub address: Addr,
    pub code_hash: String,
}

//TODO:  move away from here
pub fn scrt_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
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
