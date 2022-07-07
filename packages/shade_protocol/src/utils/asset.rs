use cosmwasm_std::{
    Addr,
    Api,
    BalanceResponse,
    BankQuery,
    Deps,
    Querier,
    StdResult,
    Storage,
    Uint128,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Contract {
    pub address: Addr,
    pub code_hash: String,
}

pub fn scrt_balance<S: Storage, A: Api, Q: Querier>(
    deps: Deps,
    address: Addr,
) -> StdResult<Uint128> {
    let resp: BalanceResponse = deps.querier.query(
        &BankQuery::Balance {
            address: address.into_string(),
            denom: "uscrt".to_string(),
        }
        .into(),
    )?;

    Ok(resp.amount.amount)
}
