use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::{
    c_std::{
        to_binary,
        Addr,
        Api,
        Binary,
        Deps,
        DepsMut,
        Env,
        Querier,
        Response,
        StdError,
        StdResult,
        Storage,
        Uint128,
    },
    contract_interfaces::{
        dex::{
            dex,
            secretswap::{
                Asset,
                AssetInfo,
                PairQuery,
                PairResponse,
                PoolResponse,
                SimulationResponse,
                Token,
            },
        },
        oracles::band::{InstantiateMsg, ReferenceData},
    },
    utils::asset::Contract,
};

use shade_protocol::storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

pub static PAIR_INFO: &[u8] = b"pair_info";
pub static POOL: &[u8] = b"pool";

pub fn pair_info_r(storage: &dyn Storage) -> ReadonlySingleton<PairResponse> {
    singleton_read(storage, PAIR_INFO)
}

pub fn pair_info_w(storage: &mut dyn Storage) -> Singleton<PairResponse> {
    singleton(storage, PAIR_INFO)
}

pub fn pool_r(storage: &dyn Storage) -> ReadonlySingleton<PoolResponse> {
    singleton_read(storage, POOL)
}

pub fn pool_w(storage: &mut dyn Storage) -> Singleton<PoolResponse> {
    singleton(storage, POOL)
}

pub fn init(_deps: DepsMut, _env: Env, _msg: InstantiateMsg) -> StdResult<Response> {
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

pub fn handle(deps: DepsMut, _env: Env, msg: ExecuteMsg) -> StdResult<Response> {
    return match msg {
        ExecuteMsg::MockPool {
            token_a,
            amount_a,
            token_b,
            amount_b,
        } => {
            let asset_infos = vec![
                AssetInfo {
                    token: Token {
                        contract_addr: token_a.address,
                        token_code_hash: token_a.code_hash,
                        viewing_key: "SecretSwap".to_string(),
                    },
                },
                AssetInfo {
                    token: Token {
                        contract_addr: token_b.address,
                        token_code_hash: token_b.code_hash,
                        viewing_key: "SecretSwap".to_string(),
                    },
                },
            ];
            pool_w(deps.storage).save(&PoolResponse {
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
                total_share: Uint128::zero(),
            })?;

            pair_info_w(deps.storage).save(&PairResponse {
                asset_infos,
                contract_addr: Addr::unchecked("".to_string()),
                liquidity_token: Addr::unchecked("".to_string()),
                token_code_hash: "".to_string(),
                asset0_volume: Uint128::zero(),
                asset1_volume: Uint128::zero(),
                factory: Contract {
                    address: Addr::unchecked("".to_string()),
                    code_hash: "".to_string(),
                },
            })?;
            Ok(Response::default())
        }
    };
}

pub fn query(deps: Deps, msg: PairQuery) -> StdResult<Binary> {
    match msg {
        PairQuery::Pool {} => to_binary(&pool_r(deps.storage).load()?),
        PairQuery::Pair {} => to_binary(&pair_info_r(deps.storage).load()?),
        PairQuery::Simulation { offer_asset } => {
            let pool = pool_r(deps.storage).load()?;

            if pool.assets[0].info == offer_asset.info {
                /*
                let take_amount = dex::pool_take_amount(
                        offer_asset.amount,
                        pool.assets[0].amount,
                        pool.assets[1].amount,
                    );

                return Err(StdError::generic_err(
                        format!("INPUT 0 pools input: {}, give: {}, take: {}",
                                offer_asset.amount,
                                pool.assets[0].amount,
                                pool.assets[1].amount
                        )
                ));
                */
                let resp = SimulationResponse {
                    return_amount: dex::pool_take_amount(
                        offer_asset.amount,
                        pool.assets[0].amount,
                        pool.assets[1].amount,
                    ),
                    spread_amount: Uint128::zero(),
                    commission_amount: Uint128::zero(),
                };
                return to_binary(&resp);
            } else if pool.assets[1].info == offer_asset.info {
                return to_binary(&SimulationResponse {
                    return_amount: dex::pool_take_amount(
                        offer_asset.amount,
                        pool.assets[1].amount,
                        pool.assets[0].amount,
                    ),
                    spread_amount: Uint128::zero(),
                    commission_amount: Uint128::zero(),
                });
            }

            return Err(StdError::generic_err("Not a token on this pair"));
        }
    }
}
