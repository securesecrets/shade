use shade_protocol::{
    c_std::{
        shd_entry_point,
        to_binary,
        Addr,
        Binary,
        Deps,
        DepsMut,
        Env,
        Response,
        StdError,
        StdResult,
        Uint128,
    },
    contract_interfaces::dex::{
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
    cosmwasm_schema::cw_serde,
    utils::{
        asset::Contract, ExecuteCallback, InstantiateCallback,
        storage::plus::{Item, ItemStorage},
    },
};

#[cw_serde]
pub struct InstantiateMsg {}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct PairInfo {
    pub pair: Pair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
}

impl ItemStorage for PairInfo {
    const ITEM: Item<'static, Self> = Item::new("item-pair_info");
}

#[shd_entry_point]
pub fn instantiate(
    _deps: DepsMut, 
    _env: Env, 
    _msg: InstantiateMsg
) -> StdResult<Response> {
    Ok(Response::default())
}

#[cw_serde]
pub enum ExecuteMsg {
    MockPool {
        token_a: Contract,
        amount_a: Uint128,
        token_b: Contract,
        amount_b: Uint128,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[shd_entry_point]
pub fn handle(deps: DepsMut, _env: Env, msg: ExecuteMsg) -> StdResult<Response> {
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
    }

    // TODO: actual swap handle
}

#[shd_entry_point]
pub fn query(deps: Deps, msg: PairQuery) -> StdResult<Binary> {
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

            match pair_info.pair.token_0 {
                TokenType::CustomToken { contract_addr, .. } => {
                    if in_token.address == contract_addr {
                        return to_binary(&SimulationResponse {
                            return_amount: pool_take_amount(
                                offer.amount,
                                pair_info.amount_0,
                                pair_info.amount_1,
                            ),
                            spread_amount: Uint128::zero(),
                            commission_amount: Uint128::zero(),
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
                                offer.amount,
                                pair_info.amount_1,
                                pair_info.amount_0,
                            ),
                            spread_amount: Uint128::zero(),
                            commission_amount: Uint128::zero(),
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
