use shade_protocol::{
    c_std::{
        shd_entry_point, from_binary, to_binary,
        Addr, Binary, Decimal, Deps, DepsMut,
        Env, MessageInfo, Response, StdError,
        StdResult, QuerierWrapper, Uint128,
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
    ReceiverCallbackMsg,
};

#[cw_serde]
pub struct Config {
    pub address: Addr,
    pub viewing_key: String,
    pub commission: Decimal,
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
pub struct InstantiateMsg {
    pub token_0: Contract,
    pub token_1: Contract,
    pub viewing_key: String,
    pub commission: Decimal,
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
        commission: msg.commission,
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

                    let (in_token, out_token) = if info.sender == pair.token_0.address {
                        (pair.token_0, pair.token_1)
                    } else if info.sender == pair.token_1.address {
                        (pair.token_1, pair.token_0)
                    } else {
                        return Err(StdError::generic_err("unauthorized"));
                    };

                    let (in_pool, out_pool) = query_pool_amounts(
                        &deps.querier, 
                        &config, 
                        in_token.clone(),
                        out_token.clone(),
                    )?;
                    
                    // Sienna takes commission before swap
                    let swap_amount = amount - (amount * config.commission);
                    let return_amount = pool_take_amount(
                        swap_amount,
                        in_pool - amount, // amount has already been added to this pool
                        out_pool,
                    );

                    if return_amount < expected_return.unwrap_or(Uint128::zero()) {
                        return Err(StdError::generic_err(
                                "Operation fell short of expected_return"
                        ));
                    }

                    // send tokens
                    let return_addr = to.unwrap_or(from);
                    return Ok(Response::default()
                        .add_message(send_msg(
                                return_addr,
                                return_amount,
                                None,
                                None,
                                None,
                                &out_token,
                        )?))
                },
            }
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
                &config, 
                pair_info.token_0.clone(),
                pair_info.token_1.clone(),
            )?;

            to_binary(&PairInfoResponse {
                pair_info: sienna::PairInfo {
                    liquidity_token: Contract {
                        address: Addr::unchecked("lp_token"), 
                        code_hash: "hash".to_string(),
                    },
                    factory: Contract {
                        address: Addr::unchecked("factory"), 
                        code_hash: "hash".to_string(),
                    },
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
                    total_liquidity: Uint128::zero(),
                    contract_version: 0,
                },
            })
        },
        QueryMsg::SwapSimulation { offer } => {
            let config = Config::load(deps.storage)?;
            let pair = PairInfo::load(deps.storage)?;
            let token_0 = pair.token_0;
            let token_1 = pair.token_1;
            
            let (in_token, out_token) = match offer.token {
                TokenType::CustomToken { contract_addr, .. } => {
                    if contract_addr == token_0.address {
                        (token_0, token_1)
                    } else if contract_addr == token_1.address {
                        (token_1, token_0)
                    } else {
                        return Err(StdError::generic_err(format!(
                                    "The supplied token {}, is not managed by this contract",
                                    contract_addr
                        )))
                    }
                },
                _ => {
                    return Err(StdError::generic_err("Only CustomToken supported"));
                }
            };
            
            let (amount_0, amount_1) = query_pool_amounts(
                &deps.querier, 
                &config, 
                in_token.clone(),
                out_token.clone(),
            )?;

            // Sienna takes commission before swap
            let commission = offer.amount * config.commission;
            let swap_amount = offer.amount - commission;

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
    }
}

fn query_pool_amounts(
    querier: &QuerierWrapper,
    config: &Config,
    token_0: Contract,
    token_1: Contract,
) -> StdResult<(Uint128, Uint128)> {
    Ok((
        balance_query(querier, config.address.clone(), config.viewing_key.clone(), &token_0)?,
        balance_query(querier, config.address.clone(), config.viewing_key.clone(), &token_1)?,
    ))
}
