use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::{
    c_std::{
        self,
        from_binary,
        to_binary,
        Api,
        Binary,
        Env,
        Extern,
        HandleResponse,
        HumanAddr,
        InitResponse,
        Querier,
        StdError,
        StdResult,
        Storage,
    },
    contract_interfaces::dex::{
        dex,
        secretswap::{
            Asset,
            AssetInfo,
            CallbackSwap,
            PairQuery,
            PairResponse,
            PoolResponse,
            SimulationResponse,
            Token,
        },
    },
    math_compat::Uint128,
    secret_toolkit::snip20::{balance_query, register_receive_msg, send_msg, set_viewing_key_msg},
    utils::asset::Contract,
};

use shade_protocol::storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

pub static PAIR_INFO: &[u8] = b"pair_info";
pub static POOL: &[u8] = b"pool";
pub static REAL_BAL: &[u8] = b"real_bal";

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
pub fn real_bal_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, bool> {
    singleton_read(storage, REAL_BAL)
}

pub fn real_bal_w<S: Storage>(storage: &mut S) -> Singleton<S, bool> {
    singleton(storage, REAL_BAL)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    token_0: Contract,
    token_1: Contract,
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let asset_infos = vec![
        AssetInfo {
            token: Token {
                contract_addr: msg.token_0.address.clone(),
                token_code_hash: msg.token_0.code_hash.clone(),
                viewing_key: "SecretSwap".to_string(),
            },
        },
        AssetInfo {
            token: Token {
                contract_addr: msg.token_1.address.clone(),
                token_code_hash: msg.token_1.code_hash.clone(),
                viewing_key: "SecretSwap".to_string(),
            },
        },
    ];
    pair_info_w(&mut deps.storage).save(&PairResponse {
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
    })?;
    let messages = vec![
        set_viewing_key_msg(
            "SecretSwap".to_string(),
            None,
            1,
            msg.token_1.code_hash.clone(),
            msg.token_1.address.clone(),
        )?,
        register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            1,
            msg.token_0.code_hash.clone(),
            msg.token_0.address.clone(),
        )?,
        set_viewing_key_msg(
            "SecretSwap".to_string(),
            None,
            1,
            msg.token_1.code_hash.clone(),
            msg.token_1.address.clone(),
        )?,
        register_receive_msg(
            env.contract_code_hash,
            None,
            1,
            msg.token_1.code_hash,
            msg.token_1.address,
        )?,
    ];

    Ok(InitResponse {
        messages,
        log: vec![],
    })
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
    Recieve {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        msg: Option<Binary>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Snip20Handle {
    Lp {},
    CallbackMsg { swap: CallbackSwap },
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    return match msg {
        HandleMsg::MockPool {
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
            pool_w(&mut deps.storage).save(&PoolResponse {
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

            pair_info_w(&mut deps.storage).save(&PairResponse {
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
            })?;
            Ok(HandleResponse::default())
        }
        HandleMsg::Recieve {
            sender,
            from,
            msg,
            amount,
        } => {
            let mut messages = vec![];
            if let Some(message) = msg {
                match from_binary(&message)? {
                    Snip20Handle::Lp {} => {
                        check_bal(deps, env)?;
                    }
                    Snip20Handle::CallbackMsg {
                        swap: CallbackSwap { expected_return },
                    } => {
                        let pool = pool_r(&deps.storage).load()?;

                        let mut contract = pool.assets[0].info.token.clone();
                        let mut return_amount = Uint128::zero();
                        let mut token_0_bal = Uint128::zero();
                        let mut token_1_bal = Uint128::zero();
                        if pool.assets[0].info.token.contract_addr == env.message.sender {
                            contract = pool.assets[1].info.token.clone();
                            return_amount = dex::pool_take_amount(
                                amount,
                                pool.assets[0].amount,
                                pool.assets[1].amount,
                            );
                            token_0_bal = pool.assets[0].amount.clone().checked_add(amount)?;
                            token_1_bal = pool.assets[1]
                                .amount
                                .clone()
                                .checked_sub(return_amount.clone())?;
                        } else if pool.assets[1].info.token.contract_addr == env.message.sender {
                            contract = pool.assets[0].info.token.clone();
                            return_amount = dex::pool_take_amount(
                                amount,
                                pool.assets[1].amount,
                                pool.assets[0].amount,
                            );
                            token_0_bal = pool.assets[0]
                                .amount
                                .clone()
                                .checked_sub(return_amount.clone())?;
                            token_1_bal =
                                pool.assets[1].amount.clone().checked_add(amount.clone())?;
                        }
                        if return_amount < expected_return {
                            return Err(StdError::unauthorized());
                        }
                        messages.push(send_msg(
                            sender,
                            c_std::Uint128::from(return_amount.u128()),
                            None,
                            None,
                            None,
                            1,
                            contract.token_code_hash.clone(),
                            contract.contract_addr,
                        )?);
                        pool_w(&mut deps.storage).save(&PoolResponse {
                            assets: vec![
                                Asset {
                                    amount: token_0_bal,
                                    info: pool.assets[0].info.clone(),
                                },
                                Asset {
                                    amount: token_1_bal,
                                    info: pool.assets[1].info.clone(),
                                },
                            ],
                            total_share: Uint128::zero(),
                        })?;
                    }
                }
            }
            Ok(HandleResponse {
                messages,
                log: vec![],
                data: None,
            })
        }
    };
}

pub fn check_bal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<bool> {
    let pair = pair_info_r(&deps.storage).load()?;
    let token_0_bal = Uint128::from(
        balance_query(
            &deps.querier,
            env.contract.address.clone(),
            pair.asset_infos[0].token.viewing_key.clone(),
            1,
            pair.asset_infos[0].token.token_code_hash.clone(),
            pair.asset_infos[0].token.contract_addr.clone(),
        )?
        .amount
        .u128(),
    );
    let token_1_bal = Uint128::from(
        balance_query(
            &deps.querier,
            env.contract.address,
            pair.asset_infos[1].token.viewing_key.clone(),
            1,
            pair.asset_infos[1].token.token_code_hash.clone(),
            pair.asset_infos[1].token.contract_addr.clone(),
        )?
        .amount
        .u128(),
    );
    pool_w(&mut deps.storage).save(&PoolResponse {
        assets: vec![
            Asset {
                amount: token_0_bal,
                info: pair.asset_infos[0].clone(),
            },
            Asset {
                amount: token_1_bal,
                info: pair.asset_infos[1].clone(),
            },
        ],
        total_share: Uint128::zero(),
    })?;
    Ok(true)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: PairQuery,
) -> StdResult<Binary> {
    match msg {
        PairQuery::Pool {} => to_binary(&pool_r(&deps.storage).load()?),
        PairQuery::Pair {} => to_binary(&pair_info_r(&deps.storage).load()?),
        PairQuery::Simulation { offer_asset } => {
            let pool = pool_r(&deps.storage).load()?;

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
