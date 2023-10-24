use cosmwasm_std::{
    to_binary,
    Addr,
    Coin,
    CosmosMsg,
    DepsMut,
    Env,
    Response,
    StdError,
    StdResult,
    Storage,
    SubMsg,
    Uint128,
    WasmMsg,
};
use shade_protocol::{utils::liquidity_book::tokens::TokenType, Contract};
use shadeswap_shared::{
    core::TokenAmount,
    msg::amm_pair::{
        ExecuteMsg as AMMPairExecuteMsg,
        InvokeMsg as AMMPairInvokeMsg,
        QueryMsgResponse as AMMPairQueryReponse,
    },
    router::{ExecuteMsgResponse, Hop},
    snip20::{
        self,
        helpers::{register_receive, set_viewing_key_msg},
    },
};

use crate::{
    contract::{SHADE_ROUTER_KEY, SWAP_REPLY_ID},
    query,
    state::{
        config_r,
        config_w,
        epheral_storage_r,
        epheral_storage_w,
        registered_tokens_list_r,
        registered_tokens_list_w,
        registered_tokens_r,
        registered_tokens_w,
        CurrentSwapInfo,
    },
};

/// Set Viewing Key for Router & register pair token.
pub fn refresh_tokens(
    deps: DepsMut,
    env: Env,
    token_address: Addr,
    token_code_hash: String,
) -> StdResult<Response> {
    let mut msg = vec![];
    let config = config_r(deps.storage).load()?;
    registered_tokens_w(deps.storage)
        .save(token_address.to_string().as_bytes(), &token_code_hash)?;
    register_pair_token(
        deps,
        &env,
        &mut msg,
        &TokenType::CustomToken {
            contract_addr: token_address,
            token_code_hash,
        },
        config.viewing_key,
    )?;

    Ok(Response::new().add_messages(msg))
}

/// Execute Next Swap
pub fn next_swap(deps: DepsMut, env: Env, mut response: Response) -> StdResult<Response> {
    let current_trade_info: Option<CurrentSwapInfo> = epheral_storage_r(deps.storage).may_load()?;
    if let Some(mut info) = current_trade_info {
        let token_in: TokenAmount = TokenAmount {
            token: info.next_token_in.clone(),
            amount: info.next_token_in.query_balance(
                deps.as_ref(),
                env.contract.address.to_string(),
                SHADE_ROUTER_KEY.to_owned(),
            )?,
        };

        if info.path.len() > (info.current_index + 1) as usize {
            let next_pair_contract = query::pair_contract_config(&deps.querier, Contract {
                address: deps
                    .api
                    .addr_validate(&info.path[info.current_index as usize + 1].addr.clone())?,
                code_hash: info.path[info.current_index as usize + 1].code_hash.clone(),
            })?;

            match next_pair_contract {
                AMMPairQueryReponse::GetPairInfo {
                    liquidity_token: _,
                    factory: _,
                    pair,
                    amount_0: _,
                    amount_1: _,
                    total_liquidity: _,
                    contract_version: _,
                    fee_info: _,
                    stable_info: _,
                } => {
                    // Check tokens are registered
                    for token in vec![pair.0.clone(), pair.1.clone()] {
                        match token {
                            TokenType::CustomToken {
                                contract_addr,
                                token_code_hash,
                            } => {
                                if let Some(code_hash) = registered_tokens_r(deps.storage)
                                    .may_load(contract_addr.to_string().as_bytes())?
                                {
                                    if code_hash != token_code_hash {
                                        return Err(StdError::generic_err(format!(
                                            "Registered code hash for {} does not match pair. Pair: {} Stored: {}",
                                            contract_addr, token_code_hash, code_hash
                                        )));
                                    }
                                } else {
                                    return Err(StdError::generic_err(format!(
                                        "{} is not registered",
                                        contract_addr
                                    )));
                                }
                            }
                            _ => {}
                        }
                    }
                    info.current_index = info.current_index + 1;

                    if pair.0 == info.next_token_in {
                        info.next_token_in = pair.1;
                    } else {
                        info.next_token_in = pair.0;
                    }
                    epheral_storage_w(deps.storage).save(&info)?;
                    response = get_trade_with_callback(
                        env,
                        token_in,
                        info.path[(info.current_index) as usize].clone(),
                        response,
                    )?;
                    Ok(response)
                }
                _ => Err(StdError::generic_err("Contract not found.")),
            }
        } else {
            if let Some(min_out) = info.amount_out_min {
                if token_in.amount.lt(&min_out) {
                    return Err(StdError::generic_err(
                        "Operation fell short of expected_return. Actual: ".to_owned()
                            + &token_in.amount.to_string().to_owned()
                            + ", Expected: "
                            + &min_out.to_string().to_owned(),
                    ));
                }
            }

            epheral_storage_w(deps.storage).remove();
            response = response
                .add_messages(vec![
                    token_in
                        .token
                        .create_send_msg(info.recipient.to_string(), token_in.amount)?,
                ])
                .set_data(to_binary(&ExecuteMsgResponse::SwapResult {
                    amount_in: info.amount.amount,
                    amount_out: token_in.amount,
                })?);

            Ok(response)
        }
    } else {
        Err(StdError::generic_err(
            "There is currently no trade in progress.",
        ))
    }
}

/// Execute Swap for Exact Token
pub fn swap_tokens_for_exact_tokens(
    deps: DepsMut,
    env: Env,
    amount_in: TokenAmount,
    amount_out_min: Option<Uint128>,
    path: &Vec<Hop>,
    sender: Addr,
    recipient: Option<Addr>,
    mut response: Response,
) -> StdResult<Response> {
    //Validates whether the amount received is greater then the amount_out_min

    let next_pair_contract = query::pair_contract_config(&deps.querier, Contract {
        address: deps.api.addr_validate(&path[0].addr.clone())?,
        code_hash: path[0].code_hash.clone(),
    })?;

    match next_pair_contract {
        AMMPairQueryReponse::GetPairInfo {
            liquidity_token: _,
            factory: _,
            pair,
            amount_0: _,
            amount_1: _,
            total_liquidity: _,
            contract_version: _,
            fee_info: _,
            stable_info: _,
        } => {
            // Check tokens are registered
            for token in vec![pair.0.clone(), pair.1.clone()] {
                match token {
                    TokenType::CustomToken {
                        contract_addr,
                        token_code_hash,
                    } => {
                        if let Some(code_hash) = registered_tokens_r(deps.storage)
                            .may_load(contract_addr.to_string().as_bytes())?
                        {
                            if code_hash != token_code_hash {
                                return Err(StdError::generic_err(format!(
                                    "Registered code hash for {} does not match pair. Pair: {} Stored: {}",
                                    contract_addr, token_code_hash, code_hash
                                )));
                            }
                        } else {
                            return Err(StdError::generic_err(format!(
                                "{} is not registered",
                                contract_addr
                            )));
                        }
                    }
                    _ => {}
                }
            }

            let next_token_in;
            if pair.0 == amount_in.token {
                next_token_in = pair.1;
            } else {
                next_token_in = pair.0;
            }

            epheral_storage_w(deps.storage).save(&CurrentSwapInfo {
                amount: amount_in.clone(),
                amount_out_min,
                path: path.clone(),
                recipient: recipient.unwrap_or(sender),
                current_index: 0,
                next_token_in,
            })?;

            get_trade_with_callback(env, amount_in, path[0].clone(), response)
        }
        _ => Err(StdError::generic_err("Pair Contract not found.")),
    }
}

/// Update Viewing Key
pub fn update_viewing_key(storage: &mut dyn Storage, viewing_key: String) -> StdResult<Response> {
    let mut config = config_w(storage).load()?;
    config.viewing_key = viewing_key;
    config_w(storage).save(&config)?;
    Ok(Response::default())
}

/// Get Trade from AMMPairs
fn get_trade_with_callback(
    env: Env,
    token_in: TokenAmount,
    hop: Hop,
    mut response: Response,
) -> StdResult<Response> {
    match &token_in.token {
        TokenType::NativeToken { denom, .. } => {
            let msg = to_binary(&AMMPairExecuteMsg::SwapTokens {
                expected_return: None,
                to: None,
                offer: token_in.clone(),
                padding: None,
            })?;

            response = response.add_submessage(SubMsg::reply_always(
                WasmMsg::Execute {
                    contract_addr: hop.addr.to_string(),
                    code_hash: hop.code_hash,
                    msg,
                    funds: vec![Coin {
                        denom: denom.clone(),
                        amount: token_in.amount,
                    }],
                },
                SWAP_REPLY_ID,
            ));
        }
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
            ..
        } => {
            let msg = to_binary(&snip20::ExecuteMsg::Send {
                recipient: hop.addr.to_string(),
                amount: token_in.amount,
                msg: Some(to_binary(&AMMPairInvokeMsg::SwapTokens {
                    expected_return: None,
                    to: Some(env.contract.address.to_string()),
                    padding: None,
                })?),
                padding: None,
                recipient_code_hash: None,
                memo: None,
            })?;

            response = response.add_submessage(SubMsg::reply_always(
                WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    code_hash: token_code_hash.clone(),
                    msg,
                    funds: vec![],
                },
                SWAP_REPLY_ID,
            ));
        }
    };
    return Ok(response);
}

/// Register Pair Token in Router
fn register_pair_token(
    deps: DepsMut,
    env: &Env,
    messages: &mut Vec<CosmosMsg>,
    token: &TokenType,
    viewing_key: String,
) -> StdResult<()> {
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = token
    {
        let mut tokens = registered_tokens_list_r(deps.storage).load()?;
        tokens.push(contract_addr.clone());
        registered_tokens_list_w(deps.storage).save(&tokens)?;
        messages.push(set_viewing_key_msg(viewing_key.clone(), None, &Contract {
            address: contract_addr.clone(),
            code_hash: token_code_hash.clone(),
        })?);
        messages.push(register_receive(
            env.contract.code_hash.clone(),
            None,
            &Contract {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            },
        )?);
    }

    Ok(())
}
