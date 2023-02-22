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
        QuerierWrapper,
        Uint128,
    },
    contract_interfaces::{
        dex::{
            dex::pool_take_amount,
            sienna::{
                self,
                Pair,
                TokenType,
            },
        },
        snip20::helpers::{balance_query, send_msg, set_viewing_key_msg},
    },
    cosmwasm_schema::cw_serde,
    utils::{
        asset::Contract, ExecuteCallback, InstantiateCallback,
        storage::plus::{Item, ItemStorage},
    },
};
pub use shade_protocol::dex::sienna::{
    PairQuery as QueryMsg,
    PairInfoResponse,
    SimulationResponse,
};

#[cw_serde]
pub struct Config {
    pub address: Addr,
    pub viewing_key: String,
}

impl ItemStorage for Config {
    const ITEM: Item<'static, Self> = Item::new("item-config");
}

#[cw_serde]
pub struct PairInfo {
    pub token_0: Contract,
    pub token_1: Contract,
}

impl ItemStorage for PairInfo {
    const ITEM: Item<'static, Self> = Item::new("item-pair");
}

#[cw_serde]
pub enum ReceiverCallbackMsg {
    Swap {
        expected_return: Option<Uint128>,
        to: Option<Addr>,
    },
}

#[cw_serde]
pub struct InstantiateMsg {
    pub token_0: Contract,
    pub token_1: Contract,
    pub viewing_key: String,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut, 
    env: Env, 
    _info: MessageInfo,
    msg: InstantiateMsg
) -> StdResult<Response> {
    let pair_info = PairInfo {
        token_0: msg.token_0.clone(),
        token_1: msg.token_1.clone(),
    };
    pair_info.save(deps.storage)?;

    let config = Config {
        address: env.contract.address,
        viewing_key: msg.viewing_key.clone(),
    };
    config.save(deps.storage)?;
    
    let messages = vec![
        set_viewing_key_msg(
            msg.viewing_key.clone(),
            None,
            &msg.token_0,
        )?,
        set_viewing_key_msg(
            msg.viewing_key,
            None,
            &msg.token_1,
        )?,
    ];
    Ok(Response::default()
       .add_messages(messages))
}

#[cw_serde]
pub enum ExecuteMsg {
    MockPool {
        token_a: Contract,
        token_b: Contract,
    },
    // SNIP20 receiver interface
    Receive {
        sender: Addr,
        from: Addr,
        msg: Option<Binary>,
        amount: Uint128,
    },   
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
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
            token_b,
        } => {
            let pair_info = PairInfo {
                token_0: token_a, 
                token_1: token_b,
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
                    let config = Config::load(deps.storage)?;
                    let pair = PairInfo::load(deps.storage)?;
                    let token_0 = pair.token_0;
                    let token_1 = pair.token_1;
                    let (amount_0, amount_1) = query_pool_amounts(
                        &deps.querier, 
                        config, 
                        token_0.clone(),
                        token_1.clone(),
                    )?;

                    if token_0.address == info.sender {
                        let return_amount = pool_take_amount(
                            amount,
                            amount_0,
                            amount_1,
                        );
                        let commission = return_amount.multiply_ratio(Uint128::new(3), Uint128::new(1000));

                        if return_amount > expected_return.unwrap_or(Uint128::MAX) {
                            return Err(StdError::generic_err(
                                    "Operation fell short of expected_return"
                            ));
                        }

                        // send tokens
                        let return_addr = to.unwrap_or(from);
                        return Ok(Response::default()
                            .add_message(send_msg(
                                    return_addr,
                                    return_amount - commission,
                                    None,
                                    None,
                                    None,
                                    &token_1,
                            )?))
                    }
                    
                    if token_1.address == info.sender {
                        let return_amount = pool_take_amount(
                            amount,
                            amount_1,
                            amount_0,
                        );
                        let commission = return_amount.multiply_ratio(Uint128::new(3), Uint128::new(1000));

                        if return_amount > expected_return.unwrap_or(Uint128::MAX) {
                            return Err(StdError::generic_err(
                                    "Operation fell short of expected_return
                            "));
                        }

                        // send tokens
                        let return_addr = to.unwrap_or(from);
                        return Ok(Response::default()
                            .add_message(send_msg(
                                    return_addr,
                                    return_amount - commission,
                                    None,
                                    None,
                                    None,
                                    &token_0,
                            )?))
                    }
                }
            }

            Err(StdError::generic_err("unauthorized"))
        }
    }

}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PairInfo => {
            let config = Config::load(deps.storage)?;
            let pair_info = PairInfo::load(deps.storage)?;
            let (amount_0, amount_1) = query_pool_amounts(
                &deps.querier, 
                config, 
                pair_info.token_0.clone(),
                pair_info.token_1.clone(),
            )?;

            to_binary(&PairInfoResponse {
                pair_info: sienna::PairInfo {
                    liquidity_token: Contract::new(&Addr::unchecked("lp_token"), &"hash".to_string()),
                    factory: Contract::new(&Addr::unchecked("factory"), &"hash".to_string()),
                    pair: Pair {
                        token_0: TokenType::CustomToken {
                            contract_addr: pair_info.token_0.address,
                            token_code_hash: pair_info.token_0.code_hash,
                        },
                        token_1: TokenType::CustomToken {
                            contract_addr: pair_info.token_1.address,
                            token_code_hash: pair_info.token_1.code_hash,
                        }
                    },
                    amount_0,
                    amount_1,
                    total_liquidity: amount_0 + amount_1,
                    contract_version: 0,
                },
            })
        },
        QueryMsg::SwapSimulation { offer } => {
            //TODO: check swap doesnt exceed pool size
            let config = Config::load(deps.storage)?;
            let pair = PairInfo::load(deps.storage)?;
            let token_0 = pair.token_0;
            let token_1 = pair.token_1;
            let (amount_0, amount_1) = query_pool_amounts(
                &deps.querier, 
                config, 
                token_0.clone(),
                token_1.clone(),
            )?;

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

            let commission = offer.amount.multiply_ratio(Uint128::new(3), Uint128::new(1_000));
            let swap_amount = offer.amount - commission;

            if in_token.address == token_0.address {
                return to_binary(&SimulationResponse {
                    return_amount: pool_take_amount(
                        swap_amount,
                        amount_0,
                        amount_1,
                    ),
                    spread_amount: Uint128::zero(),
                    commission_amount: commission,
                });
            }
            if in_token.address == token_1.address {
                return to_binary(&SimulationResponse {
                    return_amount: pool_take_amount(
                        swap_amount,
                        amount_1,
                        amount_0,
                    ),
                    spread_amount: Uint128::zero(),
                    commission_amount: commission,
                });
            }

            return Err(StdError::generic_err("Failed to match offer token"));
        }
    }
}

fn query_pool_amounts(
    querier: &QuerierWrapper,
    config: Config,
    token_0: Contract,
    token_1: Contract,
) -> StdResult<(Uint128, Uint128)> {
    Ok((
        balance_query(querier, config.address.clone(), config.viewing_key.clone(), &token_0)?,
        balance_query(querier, config.address, config.viewing_key, &token_1)?,
    ))
}
