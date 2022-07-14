use shade_protocol::c_std::{
    to_binary,
    Api,
    Binary,
    Env,
    DepsMut,
    Response,
    Addr,
    Response,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use shade_protocol::c_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::{
    contract_interfaces::dex::{
        dex::pool_take_amount,
        sienna::{
            Pair,
            PairInfo,
            PairInfoResponse,
            PairQuery,
            SimulationResponse,
            TokenType,
            TokenTypeAmount,
        },
    },
    utils::asset::Contract,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {}

impl InstantianteCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

pub static PAIR_INFO: &[u8] = b"pair_info";

pub fn pair_info_r(storage: &dyn Storage) -> ReadonlySingleton<PairInfo> {
    singleton_read(storage, PAIR_INFO)
}

pub fn pair_info_w(storage: &mut dyn Storage) -> Singleton<PairInfo> {
    singleton(storage, PAIR_INFO)
}

pub fn init(
    _deps: DepsMut,
    _env: Env,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    MockPool {
        token_a: Contract,
        amount_a: Uint128,
        token_b: Contract,
        amount_b: Uint128,
    },
}

pub fn handle(
    deps: DepsMut,
    _env: Env,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::MockPool {
            token_a,
            amount_a,
            token_b,
            amount_b,
        } => {
            let pair_info = PairInfo {
                liquidity_token: Contract {
                    address: Addr::unchecked("".to_string()),
                    code_hash: "".to_string(),
                },
                factory: Contract {
                    address: Addr::unchecked("".to_string()),
                    code_hash: "".to_string(),
                },
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
                total_liquidity: Uint128::zero(),
                contract_version: 0,
            };

            pair_info_w(deps.storage).save(&pair_info)?;

            Ok(Response::default())
        }
    }

    // TODO: actual swap handle
}

pub fn query(
    deps: Deps,
    msg: PairQuery,
) -> StdResult<Binary> {
    match msg {
        PairQuery::PairInfo => to_binary(&PairInfoResponse {
            pair_info: pair_info_r(&deps.storage).load()?,
        }),
        PairQuery::SwapSimulation { offer } => {
            //TODO: check swap doesnt exceed pool size

            let mut in_token = match offer.token {
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

            let pair_info = pair_info_r(&deps.storage).load()?;

            match pair_info.pair.token_0 {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } => {
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
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } => {
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
