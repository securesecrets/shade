use crate::utils::{asset::Contract};
use cosmwasm_std::{
    Binary,
    CosmosMsg,
    HumanAddr,
    StdResult,
    Uint128,
};
use secret_toolkit::snip20::{deposit_msg, redeem_msg, send_msg};

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
