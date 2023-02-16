use shade_protocol::{
    c_std::{
        shd_entry_point,
        from_binary,
        to_binary,
        Addr,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Uint128,
    },
    contract_interfaces::{
        dex::{
            dex::pool_take_amount,
            sienna::{
                self,
                Pair,
                PairInfoResponse,
                PairQuery,
                SimulationResponse,
                TokenType,
            },
        },
        mock::mock_sienna::{
            ExecuteMsg, InstantiateMsg, PairInfo,
            ReceiverCallbackMsg,
        },
        snip20::helpers::send_msg,
    },
    utils::{
        asset::Contract,
        storage::plus::ItemStorage,
    },
};

#[shd_entry_point]
pub fn instantiate(
    _deps: DepsMut, 
    _env: Env, 
    _info: MessageInfo,
    _msg: InstantiateMsg
) -> StdResult<Response> {
    Ok(Response::default())
}

#[shd_entry_point]
pub fn execute(
    deps: DepsMut, 
    _env: Env, 
    info: MessageInfo, 
    msg: ExecuteMsg
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::MockPool {
            token_a,
            amount_a,
            token_b,
            amount_b,
        } => {
            let pair_info = PairInfo {
                /*liquidity_token: Contract {
                    address: Addr::unchecked("".to_string()),
                    code_hash: "".to_string(),
                },
                factory: Contract {
                    address: Addr::unchecked("".to_string()),
                    code_hash: "".to_string(),
                },*/
                pair: Pair {
                    token_0: TokenType::CustomToken {
                        contract_addr: token_a.address,
                        token_code_hash: token_a.code_hash,
                    },
                    token_1: TokenType::CustomToken {
                        contract_addr: token_b.address,
                        token_code_hash: token_b.code_hash,
                    },
                },
                amount_0: amount_a,
                amount_1: amount_b,
                //total_liquidity: Uint128::zero(),
                //contract_version: 0,
            };

            pair_info.save(deps.storage)?;
            Ok(Response::default())
        }

        // Swap
        ExecuteMsg::Receive {
            from,
            msg,
            amount,
            ..
        } => {
            let msg = msg.ok_or_else(|| {
                StdError::generic_err("Receiver callback \"msg\" parameter cannot be empty.")
            })?;

            match from_binary(&msg)? {
                ReceiverCallbackMsg::Swap { expected_return, to } => {
                    let pair_info = PairInfo::load(deps.storage)?;
                    match pair_info.pair.token_0.clone() {
                        TokenType::CustomToken { contract_addr, .. } => {
                            if contract_addr == info.sender {
                                
                                let return_amount = pool_take_amount(
                                    amount,
                                    pair_info.amount_0,
                                    pair_info.amount_1,
                                );
                                let commission = return_amount.multiply_ratio(Uint128::new(3), Uint128::new(1000));

                                if return_amount > expected_return.unwrap_or(Uint128::MAX) {
                                    return Err(StdError::generic_err(
                                            "Operation fell short of expected_return
                                    "));
                                }

                                PairInfo {
                                    pair: pair_info.pair.clone(),
                                    amount_0: pair_info.amount_0 - amount,
                                    amount_1: pair_info.amount_1 + return_amount,
                                }.save(deps.storage)?;
                                                
                                // send tokens
                                let return_addr = to.unwrap_or(from);
                                let return_token = match pair_info.pair.token_1 {
                                    TokenType::CustomToken { contract_addr, token_code_hash} =>
                                        (contract_addr, token_code_hash),
                                    TokenType::NativeToken { .. } => {
                                        return Err(StdError::generic_err(
                                                "Native tokens not supported"
                                        ))
                                    },
                                };

                                return Ok(Response::default()
                                    .add_message(send_msg(
                                            return_addr,
                                            return_amount - commission,
                                            None,
                                            None,
                                            None,
                                            &Contract {
                                                address: return_token.0,
                                                code_hash: return_token.1,
                                            },
                                    )?))
                            }
                        }
                        _ => { },
                    }

                    match pair_info.pair.token_1.clone() {
                        TokenType::CustomToken { contract_addr, .. } => {
                            if contract_addr == info.sender {
                                let return_amount = pool_take_amount(
                                    amount,
                                    pair_info.amount_1,
                                    pair_info.amount_0,
                                );
                                let commission = return_amount.multiply_ratio(Uint128::new(3), Uint128::new(1000));

                                if return_amount > expected_return.unwrap_or(Uint128::MAX) {
                                    return Err(StdError::generic_err(
                                            "Operation fell short of expected_return
                                    "));
                                }

                                PairInfo {
                                    pair: pair_info.pair.clone(),
                                    amount_0: pair_info.amount_0 - return_amount,
                                    amount_1: pair_info.amount_1 + amount,
                                }.save(deps.storage)?;
                                                
                                // send tokens
                                let return_addr = to.unwrap_or(from);
                                let return_token = match pair_info.pair.token_0 {
                                    TokenType::CustomToken { contract_addr, token_code_hash} =>
                                        (contract_addr, token_code_hash),
                                    TokenType::NativeToken { .. } => {
                                        return Err(StdError::generic_err(
                                                "Native tokens not supported"
                                        ))
                                    },
                                };

                                return Ok(Response::default()
                                    .add_message(send_msg(
                                            return_addr,
                                            return_amount - commission,
                                            None,
                                            None,
                                            None,
                                            &Contract {
                                                address: return_token.0,
                                                code_hash: return_token.1,
                                            },
                                    )?))
                            }
                        }
                        _ => { },
                    }
                }
            }

            Err(StdError::generic_err("unauthorized"))
        }
    }

}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: PairQuery) -> StdResult<Binary> {
    match msg {
        PairQuery::PairInfo => {
            let pair_info = PairInfo::load(deps.storage)?;
            to_binary(&PairInfoResponse {
                pair_info: sienna::PairInfo {
                    liquidity_token: Contract::new(&Addr::unchecked("lp_token"), &"hash".to_string()),
                    factory: Contract::new(&Addr::unchecked("factory"), &"hash".to_string()),
                    pair: pair_info.pair,
                    amount_0: pair_info.amount_0,
                    amount_1: pair_info.amount_1,
                    total_liquidity: pair_info.amount_0 + pair_info.amount_1,
                    contract_version: 0,
                },
            })
        },
        PairQuery::SwapSimulation { offer } => {
            //TODO: check swap doesnt exceed pool size

            let in_token = match offer.token {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } => Contract {
                    address: contract_addr,
                    code_hash: token_code_hash,
                },
                _ => {
                    return Err(StdError::generic_err("Only CustomToken supported"));
                }
            };

            let pair_info = PairInfo::load(deps.storage)?;
            let commission = offer.amount.multiply_ratio(Uint128::new(3), Uint128::new(1_000));
            let swap_amount = offer.amount - commission;

            match pair_info.pair.token_0 {
                TokenType::CustomToken { contract_addr, .. } => {
                    if in_token.address == contract_addr {
                        return to_binary(&SimulationResponse {
                            return_amount: pool_take_amount(
                                swap_amount,
                                pair_info.amount_0,
                                pair_info.amount_1,
                            ),
                            spread_amount: Uint128::zero(),
                            commission_amount: commission,
                        });
                    }
                }
                _ => {
                    return Err(StdError::generic_err("Only CustomToken supported"));
                }
            };

            match pair_info.pair.token_1 {
                TokenType::CustomToken { contract_addr, .. } => {
                    if in_token.address == contract_addr {
                        return to_binary(&SimulationResponse {
                            return_amount: pool_take_amount(
                                swap_amount,
                                pair_info.amount_1,
                                pair_info.amount_0,
                            ),
                            spread_amount: Uint128::zero(),
                            commission_amount: commission,
                        });
                    }
                }
                _ => {
                    return Err(StdError::generic_err("Only CustomToken supported"));
                }
            };

            return Err(StdError::generic_err("Failed to match offer token"));
        }
    }
}
