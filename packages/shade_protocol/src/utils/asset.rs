use cosmwasm_std::{
    Api,
    Env,
    BalanceResponse,
    BankQuery,
    Extern,
    HumanAddr,
    Querier,
    StdResult,
    Storage,
    Uint128,
    CosmosMsg,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::snip20::{
    allowance_query, increase_allowance_msg, decrease_allowance_msg
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Contract {
    pub address: HumanAddr,
    pub code_hash: String,
}

pub fn scrt_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
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

pub fn set_allowance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
    spender: HumanAddr,
    amount: Uint128,
    key: String,
    asset: Contract,
    cur_allowance: Option<Uint128>,
) -> StdResult<Vec<CosmosMsg>> {

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
