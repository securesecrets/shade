use crate::utils::{asset::Contract, generic_response::ResponseStatus};
use chrono::prelude::*;
use cosmwasm_std::{
    Api,
    Binary,
    CosmosMsg,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
};
use schemars::JsonSchema;
use secret_toolkit::snip20::{deposit_msg, redeem_msg, send_msg};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

pub fn wrap(
    amount: Uint128,
    token: Contract,
    //denom: Option<String>,
) -> StdResult<CosmosMsg> {
    Ok(deposit_msg(
        amount,
        None,
        256,
        token.code_hash,
        token.address,
    )?)
}

pub fn wrap_and_send(
    amount: Uint128,
    recipient: HumanAddr,
    token: Contract,
    //denom: Option<String>,
    msg: Option<Binary>,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![
        wrap(amount, token.clone())?,
        send_msg(
            recipient,
            amount,
            msg,
            None,
            None,
            256,
            token.code_hash.clone(),
            token.address.clone(),
        )?,
    ])
}

pub fn unwrap(
    amount: Uint128,
    token: Contract,
    //denom: Option<String>,
) -> StdResult<CosmosMsg> {
    Ok(redeem_msg(
        amount,
        None,
        None,
        256,
        token.code_hash.clone(),
        token.address.clone(),
    )?)
}
