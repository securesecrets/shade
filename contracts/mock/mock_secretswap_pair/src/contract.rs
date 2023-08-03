use cosmwasm_schema::cw_serde;
use shade_protocol::{
    c_std::{
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
        shd_entry_point,
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
        oracles::band::InstantiateMsg,
    },
    utils::asset::Contract,
};

use crate::storage::{POOL, PAIR_INFO};

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

#[shd_entry_point]
pub fn execute(deps: DepsMut, _env: Env, msg: ExecuteMsg) -> StdResult<Response> {
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
            POOL.save(deps.storage, &PoolResponse {
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

            PAIR_INFO.save(deps.storage, &PairResponse {
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

#[shd_entry_point]
pub fn query(deps: Deps, msg: PairQuery) -> StdResult<Binary> {
    match msg {
        PairQuery::Pool {} => to_binary(&POOL.load(deps.storage)?),
        PairQuery::Pair {} => to_binary(&PAIR_INFO.load(deps.storage)?),
        PairQuery::Simulation { offer_asset } => {
            let pool = POOL.load(deps.storage)?;

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
