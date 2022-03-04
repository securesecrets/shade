use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, 
    HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage, Uint128, HumanAddr,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::utils::{InitCallback, Query};
use shade_protocol::{
    sienna::{
        PairInfo, Pair, TokenType, TokenTypeAmount, 
        PairQuery, PairInfoResponse, 
        SimulationResponse, CustomToken,
    },
    utils::asset::Contract,
};
use cosmwasm_storage::{singleton, singleton_read, Singleton, ReadonlySingleton, };

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

pub static PAIR_INFO: &[u8] = b"pair_info";

pub fn pair_info_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, PairInfo> {
    singleton_read(storage, PAIR_INFO)
}

pub fn pair_info_w<S: Storage>(storage: &mut S) -> Singleton<S, PairInfo> {
    singleton(storage, PAIR_INFO)
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {

    Ok(InitResponse::default())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    MockPool {
        token_a: Contract,
        amount_a: Uint128,
        token_b: Contract,
        amount_b: Uint128,
    },
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::MockPool { 
            token_a, amount_a,
            token_b, amount_b,
        } => {
            let pair_info = PairInfo {
                liquidity_token: Contract {
                    address: HumanAddr("".to_string()),
                    code_hash: "".to_string(),
                },
                factory: Contract {
                    address: HumanAddr("".to_string()),
                    code_hash: "".to_string(),
                },
                pair: Pair {
                    token_0: CustomToken {
                        contract_addr: token_a.address,
                        token_code_hash: token_a.code_hash,
                    },
                    token_1: CustomToken {
                        contract_addr: token_b.address,
                        token_code_hash: token_b.code_hash,
                    },
                },
                amount_0: amount_a,
                amount_1: amount_b,
                total_liquidity: Uint128(0), 
            };

            pair_info_w(&mut deps.storage).save(&pair_info)?;

            Ok(HandleResponse::default())
        }
    }

    // TODO: actual swap handle
}

/*
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    secretswap::PairQuery::Simulation,
    secretswap::PairQuery::Pair,
    secretswap::PairQuery::Pool,
}
*/

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: PairQuery,
) -> StdResult<Binary> {
    match msg {
        PairQuery::PairInfo { } => {
            to_binary(&PairInfoResponse {
                pair_info: pair_info_r(&deps.storage).load()?,
            })
        },
        PairQuery::SwapSimulation { offer } => {

            match offer.token {
                TokenType::CustomToken { custom_token } => {

                    let pair_info = pair_info_r(&deps.storage).load()?;
                    //TODO: check you dont exceed pool size
                    if custom_token == pair_info.pair.token_0 {
                        return to_binary(&SimulationResponse {
                            return_amount: Uint128((pair_info.amount_0.u128() * pair_info.amount_1.u128()) / (pair_info.amount_0.u128() + offer.amount.u128()) - pair_info.amount_1.u128()),
                            spread_amount: Uint128::zero(),
                            commission_amount: Uint128::zero(),
                        })
                    }
                    else if custom_token == pair_info.pair.token_1 {
                        return to_binary(&SimulationResponse {
                            return_amount: Uint128((pair_info.amount_0.u128() * pair_info.amount_1.u128()) / (pair_info.amount_1.u128() + offer.amount.u128()) - pair_info.amount_0.u128()),
                            spread_amount: Uint128::zero(),
                            commission_amount: Uint128::zero(),
                        })
                    }
                    return Err(StdError::generic_err("Failed to match offer token"))

                },
                _ => {
                    Err(StdError::generic_err("Only CustomToken supported"))
                },
            }
        },
    }
}
