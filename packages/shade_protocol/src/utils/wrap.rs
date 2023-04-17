use crate::{
    c_std::{Addr, Binary, Coin, CosmosMsg, StdResult, Uint128},
    contract_interfaces::snip20,
    snip20::helpers::{deposit_msg, redeem_msg, send_msg},
    utils::{asset::Contract, callback::ExecuteCallback},
};

pub fn wrap(amount: Uint128, token: Contract) -> StdResult<CosmosMsg> {
    Ok(deposit_msg(amount, None, &token)?)
}

pub fn wrap_coin(coin: Coin, token: Contract) -> StdResult<CosmosMsg> {
    snip20::ExecuteMsg::Deposit { padding: None }.to_cosmos_msg(&token, vec![coin])
}

pub fn wrap_and_send(
    amount: Uint128,
    recipient: Addr,
    token: Contract,
    //denom: Option<String>,
    msg: Option<Binary>,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![
        wrap(amount, token.clone())?,
        send_msg(recipient, amount, msg, None, None, &token)?,
    ])
}

pub fn unwrap(
    amount: Uint128,
    token: Contract,
    //denom: Option<String>,
) -> StdResult<CosmosMsg> {
    Ok(redeem_msg(amount, None, None, &token)?)
}
