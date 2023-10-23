use crate::{
    operations::{next_swap, refresh_tokens, swap_tokens_for_exact_tokens},
    query,
    state::{config_r, config_w, registered_tokens_list_r, registered_tokens_list_w, Config},
};
use cosmwasm_std::{
    entry_point,
    from_binary,
    to_binary,
    Addr,
    BankMsg,
    Binary,
    Coin,
    CosmosMsg,
    Deps,
    DepsMut,
    Env,
    MessageInfo,
    Reply,
    Response,
    StdError,
    StdResult,
    Uint128,
};
use shade_protocol::{utils::liquidity_book::tokens::TokenType, Contract};
use shadeswap_shared::{
    admin::helpers::{validate_admin, AdminPermissions},
    amm_pair::QueryMsgResponse as AMMPairQueryReponse,
    core::TokenAmount,
    router::{ExecuteMsg, InitMsg, InvokeMsg, QueryMsg, QueryMsgResponse},
    snip20::helpers::send_msg,
    utils::{pad_handle_result, pad_query_result},
};

/// Pad handle responses and log attributes to blocks
/// of 256 bytes to prevent leaking info based on response size
const BLOCK_SIZE: usize = 256;
pub const SHADE_ROUTER_KEY: &str = "SHADE_ROUTER_KEY";
pub const SWAP_REPLY_ID: u64 = 1u64;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    config_w(deps.storage).save(&Config {
        viewing_key: SHADE_ROUTER_KEY.to_string(),
        admin_auth: msg.admin_auth,
        airdrop_address: msg.airdrop_address,
    })?;
    registered_tokens_list_w(deps.storage).save(&vec![])?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    pad_handle_result(
        match msg {
            ExecuteMsg::SetConfig {
                admin_auth,
                padding: _,
            } => {
                let mut config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                if let Some(admin_auth) = admin_auth {
                    config.admin_auth = admin_auth;
                }
                config_w(deps.storage).save(&config)?;
                Ok(Response::new())
            }
            ExecuteMsg::Receive(msg) => {
                let checked_address = deps.api.addr_validate(&msg.from)?;
                receiver_callback(deps, env, info, checked_address, msg.amount, msg.msg)
            }
            ExecuteMsg::SwapTokensForExact {
                offer,
                expected_return,
                path,
                recipient,
                padding: _,
            } => {
                if !offer.token.is_native_token() {
                    return Err(StdError::generic_err(
                        "Sent a non-native token. Should use the receive interface in SNIP20.",
                    ));
                }
                offer.assert_sent_native_token_balance(&info)?;
                let sender = info.sender.clone();
                let checked_address = match recipient {
                    Some(x) => Some(deps.api.addr_validate(&x)?),
                    None => None,
                };
                let response = Response::new();
                Ok(swap_tokens_for_exact_tokens(
                    deps,
                    env,
                    offer,
                    expected_return,
                    &path,
                    sender,
                    checked_address,
                    response,
                )?)
            }
            ExecuteMsg::RegisterSNIP20Token {
                token_addr,
                token_code_hash,
                oracle_key,
                padding: _,
            } => {
                let checked_token_addr = deps.api.addr_validate(&token_addr)?;
                refresh_tokens(deps, env, checked_token_addr, token_code_hash)
            }
            ExecuteMsg::RecoverFunds {
                token,
                amount,
                to,
                msg,
                padding: _,
            } => {
                let config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                let send_msg = match token {
                    TokenType::CustomToken {
                        contract_addr,
                        token_code_hash,
                        ..
                    } => vec![send_msg(
                        deps.api.addr_validate(&to)?,
                        amount,
                        msg,
                        None,
                        None,
                        &Contract {
                            address: contract_addr,
                            code_hash: token_code_hash,
                        },
                    )?],
                    TokenType::NativeToken { denom, .. } => vec![CosmosMsg::Bank(BankMsg::Send {
                        to_address: to.to_string(),
                        amount: vec![Coin::new(amount.u128(), denom)],
                    })],
                };

                Ok(Response::new().add_messages(send_msg))
            }
        },
        BLOCK_SIZE,
    )
}

fn receiver_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {
    pad_handle_result(
        if let Some(content) = msg {
            match from_binary::<InvokeMsg>(&content)? {
                InvokeMsg::SwapTokensForExact {
                    expected_return,
                    path,
                    recipient,
                } => {
                    let pair_contract_config =
                        query::pair_contract_config(&deps.querier, Contract {
                            address: deps.api.addr_validate(&path[0].addr.to_string())?,
                            code_hash: path[0].code_hash.clone(),
                        })?;

                    match pair_contract_config {
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
                            for token in pair.into_iter() {
                                match token {
                                    TokenType::CustomToken { contract_addr, .. } => {
                                        if *contract_addr == info.sender {
                                            let offer = TokenAmount {
                                                token: token.clone(),
                                                amount,
                                            };

                                            let checked_address = match recipient {
                                                Some(x) => Some(deps.api.addr_validate(&x)?),
                                                None => None,
                                            };

                                            let response = Response::new();
                                            return Ok(swap_tokens_for_exact_tokens(
                                                deps,
                                                env,
                                                offer,
                                                expected_return,
                                                &path,
                                                from,
                                                checked_address,
                                                response,
                                            )?);
                                        }
                                    }
                                    _ => continue,
                                }
                            }
                            return Err(StdError::generic_err("No matching token in pair"));
                        }
                        _ => {
                            return Err(StdError::generic_err(format!(
                                "Could not retrieve PairInfo from {}",
                                &path[0].addr
                            )));
                        }
                    }
                }
            }
        } else {
            //Cannot err here because swap returns are sent to this contract without a msg
            Ok(Response::default())
        },
        BLOCK_SIZE,
    )
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::SwapSimulation {
                offer,
                path,
                exclude_fee,
            } => query::swap_simulation(deps, path, offer, exclude_fee),
            QueryMsg::GetConfig {} => {
                let config = config_r(deps.storage).load()?;
                return Ok(to_binary(&QueryMsgResponse::GetConfig {
                    admin_auth: config.admin_auth,
                    airdrop_address: config.airdrop_address,
                })?);
            }
            QueryMsg::RegisteredTokens {} => to_binary(&QueryMsgResponse::RegisteredTokens {
                tokens: registered_tokens_list_r(deps.storage).load()?,
            }),
        },
        BLOCK_SIZE,
    )
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    pad_handle_result(
        match msg.id {
            SWAP_REPLY_ID => match msg.result {
                cosmwasm_std::SubMsgResult::Ok(_) => {
                    return next_swap(deps, env, Response::new());
                }
                cosmwasm_std::SubMsgResult::Err(e) => Err(StdError::generic_err(format!(
                    "Swap failed with message: {e}"
                ))),
            },
            _ => Ok(Response::default()),
        },
        BLOCK_SIZE,
    )
}
