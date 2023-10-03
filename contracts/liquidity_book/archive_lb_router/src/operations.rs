use crate::{
    contract::{query, SHADE_ROUTER_KEY, SWAP_REPLY_ID},
    error::LBRouterError,
    msg::ExecuteMsgResponse,
    state::{CurrentSwapInfo, CONFIG, EPHEMERAL_STORAGE},
};
use cosmwasm_std::{
    to_binary, Addr, Coin, ContractInfo, CosmosMsg, DepsMut, Env, Response, StdResult, SubMsg,
    Uint128, WasmMsg,
};
use interfaces::ILBPair;
use libraries::{
    tokens::TokenType,
    viewing_keys::{register_receive, set_viewing_key_msg},
};

use crate::msg::{Hop, TokenAmount};

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
) -> Result<Response, LBRouterError> {
    //Validates whether the amount received is greater then the amount_out_min

    let next_pair_contract = crate::query::pair_contract_config(
        &deps.querier,
        ContractInfo {
            address: deps.api.addr_validate(&path[0].addr.clone())?,
            code_hash: path[0].code_hash.clone(),
        },
    )?;

    match next_pair_contract {
        ILBPair::TokensResponse { token_x, token_y } => {
            let next_token_in;
            let mut swap_for_y = false;
            if token_x == amount_in.token {
                next_token_in = token_y;
                swap_for_y = true;
            } else {
                next_token_in = token_x;
            }

            EPHEMERAL_STORAGE.save(
                deps.storage,
                &CurrentSwapInfo {
                    amount: amount_in.clone(),
                    amount_out_min: amount_out_min,
                    path: path.clone(),
                    recipient: recipient.unwrap_or(sender),
                    current_index: 0,
                    next_token_in: next_token_in,
                },
            )?;

            response =
                get_trade_with_callback(env, swap_for_y, amount_in, path[0].clone(), response)?;

            Ok(response)
        }
        _ => Err(LBRouterError::PairNotFound),
    }
}

/// Get Trade from AMMPairs
fn get_trade_with_callback(
    env: Env,
    swap_for_y: bool,
    token_in: TokenAmount,
    hop: Hop,
    mut response: Response,
) -> Result<Response, LBRouterError> {
    match &token_in.token {
        TokenType::NativeToken { denom } => {
            let msg = to_binary(&ILBPair::ExecuteMsg::Swap {
                swap_for_y,
                to: env.contract.address,
                amount_received: token_in.amount,
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
        } => {
            let msg = to_binary(&libraries::transfer::HandleMsg::Send {
                recipient: hop.addr.to_string(),
                amount: token_in.amount,
                msg: Some(to_binary(&&ILBPair::ExecuteMsg::Swap {
                    swap_for_y,
                    to: env.contract.address,
                    amount_received: token_in.amount,
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

/// Execute Next Swap
pub fn next_swap(
    deps: DepsMut,
    env: Env,
    mut response: Response,
) -> Result<Response, LBRouterError> {
    let current_trade_info: Option<CurrentSwapInfo> = EPHEMERAL_STORAGE.may_load(deps.storage)?;
    if let Some(mut info) = current_trade_info {
        let amount_in: TokenAmount = TokenAmount {
            token: info.next_token_in.clone(),
            amount: info.next_token_in.query_balance(
                deps.as_ref(),
                env.contract.address.to_string(),
                SHADE_ROUTER_KEY.to_owned(),
            )?,
        };

        if info.path.len() > (info.current_index + 1) as usize {
            let next_pair_contract = crate::query::pair_contract_config(
                &deps.querier,
                ContractInfo {
                    address: deps
                        .api
                        .addr_validate(&info.path[info.current_index as usize + 1].addr.clone())?,
                    code_hash: info.path[info.current_index as usize + 1].code_hash.clone(),
                },
            )?;

            match next_pair_contract {
                ILBPair::TokensResponse { token_x, token_y } => {
                    info.current_index = info.current_index + 1;

                    let mut swap_for_y = false;
                    if token_x == amount_in.token {
                        info.next_token_in = token_y;
                        swap_for_y = true;
                    } else {
                        info.next_token_in = token_x;
                    }

                    EPHEMERAL_STORAGE.save(deps.storage, &info)?;

                    response = get_trade_with_callback(
                        env,
                        swap_for_y,
                        amount_in,
                        info.path[(info.current_index) as usize].clone(),
                        response,
                    )?;

                    Ok(response)
                }
                _ => Err(LBRouterError::PairNotFound), //TODO CHECK this error
            }
        } else {
            if let Some(min_out) = info.amount_out_min {
                if amount_in.amount.lt(&min_out) {
                    return Err(LBRouterError::InsufficientAmountOut {
                        amount_out_min: min_out,
                        amount_out: amount_in.amount,
                    });
                }
            }

            EPHEMERAL_STORAGE.remove(deps.storage);
            response = response
                .add_messages(vec![amount_in.token.create_send_msg(
                    env.contract.address.to_string(),
                    info.recipient.to_string(),
                    amount_in.amount,
                )?])
                .set_data(to_binary(&ExecuteMsgResponse::SwapResult {
                    amount_in: info.amount.amount,
                    amount_out: amount_in.amount,
                })?);

            Ok(response)
        }
    } else {
        Err(LBRouterError::NoTradeInProgress)
    }
}
/// Update Viewing Key
///
///
pub fn update_viewing_key(deps: DepsMut, env: Env, viewing_key: String) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;
    config.viewing_key = viewing_key;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}

/// Set Viewing Key for Router & register pair token.
pub fn refresh_tokens(
    deps: DepsMut,
    env: Env,
    token_address: Addr,
    token_code_hash: String,
) -> StdResult<Response> {
    let mut msg = vec![];
    let config = CONFIG.load(deps.storage)?;
    set_viewing_key_msg(
        SHADE_ROUTER_KEY.to_string(),
        None,
        &ContractInfo {
            address: token_address.clone(),
            code_hash: token_code_hash.clone(),
        },
    )?;
    register_pair_token(
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
/// Register Pair Token in Router
fn register_pair_token(
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
        messages.push(set_viewing_key_msg(
            viewing_key.clone(),
            None,
            &ContractInfo {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            },
        )?);
        messages.push(register_receive(
            env.contract.code_hash.clone(),
            None,
            &ContractInfo {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            },
        )?);
    }

    Ok(())
}
