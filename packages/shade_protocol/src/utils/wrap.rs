use cosmwasm_std::SubMsg;

use crate::utils::{asset::Contract};
use crate::c_std::{
    Binary,
    CosmosMsg,
    Addr,
    StdResult,
    Uint128,
};
use crate::snip20::helpers::{deposit_msg, redeem_msg, send_msg};

pub fn wrap(
    amount: Uint128,
    token: Contract,
    //denom: Option<String>,
) -> StdResult<SubMsg> {
    Ok(deposit_msg(
        amount,
        None,
        &token
    )?)
}

pub fn wrap_and_send(
    amount: Uint128,
    recipient: Addr,
    token: Contract,
    //denom: Option<String>,
    msg: Option<Binary>,
) -> StdResult<Vec<SubMsg>> {
    Ok(vec![
        wrap(amount, token.clone())?,
        send_msg(
            recipient,
            amount,
            msg,
            None,
            None,
            &token
        )?,
    ])
}

pub fn unwrap(
    amount: Uint128,
    token: Contract,
    //denom: Option<String>,
) -> StdResult<SubMsg> {
    Ok(redeem_msg(
        amount,
        None,
        None,
        &token
    )?)
}
