use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage, Uint128, HumanAddr,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::{
    band::{InitMsg, ReferenceData},
    utils::asset::Contract,
    secretswap::{
        PairQuery, SimulationResponse,
        PoolResponse, PairResponse,
        Token, Asset, AssetInfo,
    },
};

use cosmwasm_storage::{singleton, singleton_read, Singleton, ReadonlySingleton, };

pub static PAIR_INFO: &[u8] = b"pair_info";
pub static POOL: &[u8] = b"pool";

pub fn pair_info_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, PairResponse> {
    singleton_read(storage, PAIR_INFO)
}

pub fn pair_info_w<S: Storage>(storage: &mut S) -> Singleton<S, PairResponse> {
    singleton(storage, PAIR_INFO)
}

pub fn pool_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, PoolResponse> {
    singleton_read(storage, POOL)
}

pub fn pool_w<S: Storage>(storage: &mut S) -> Singleton<S, PoolResponse> {
    singleton(storage, POOL)
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
    return match msg {
        HandleMsg::MockPool { 
            token_a, amount_a,
            token_b, amount_b,
        } => {
            let asset_infos = vec![
                AssetInfo {
                    token: Token {
                        contract_addr: token_a.address,
                        token_code_hash: token_a.code_hash,
                        viewing_key: "SecretSwap".to_string()
                    }
                },
                AssetInfo {
                    token: Token {
                        contract_addr: token_b.address,
                        token_code_hash: token_b.code_hash,
                        viewing_key: "SecretSwap".to_string()
                    }
                },
            ];
            pool_w(&mut deps.storage).save(
                &PoolResponse {
                    assets: vec![
                        Asset {
                            amount: amount_a,
                            info: asset_infos[0].clone(),
                        },
                        Asset {
                            amount: amount_b,
                            info: asset_infos[1].clone(),
                        },
                    ],
                    total_share: Uint128(0),
                }
            )?;

            pair_info_w(&mut deps.storage).save(
                &PairResponse {
                    asset_infos,
                    contract_addr: HumanAddr("".to_string()),
                    liquidity_token: HumanAddr("".to_string()),
                    token_code_hash: "".to_string(),
                    asset0_volume: Uint128::zero(),
                    asset1_volume: Uint128::zero(),
                    factory: Contract {
                        address: HumanAddr("".to_string()),
                        code_hash: "".to_string(),
                    },
                }
            )?;
            Ok(HandleResponse::default())
        }
    };
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: PairQuery,
) -> StdResult<Binary> {
    match msg {
        PairQuery::Pool { } => {
            to_binary(&pool_r(&deps.storage).load()?)
        },
        PairQuery::Pair { } => {
            to_binary(&pair_info_r(&deps.storage).load()?)
        },
        PairQuery::Simulation { offer_asset } => {
            to_binary(&SimulationResponse {
                return_amount: Uint128::zero(),
                spread_amount: Uint128::zero(),
                commission_amount: Uint128::zero(),
            })
        },

    }
}
