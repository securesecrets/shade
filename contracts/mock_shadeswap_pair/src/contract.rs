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
        shadeswap::{
            PairInfoResponse,
            PairQuery,
            QueryMsgResponse,
            SwapTokens,
            TokenPairSerde,
            TokenType,
        },
    },
    math_compat::{Decimal, Uint128},
    secret_toolkit::snip20::{balance_query, register_receive_msg, send_msg, set_viewing_key_msg},
    utils::asset::Contract,
};

use shade_protocol::storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

pub static PAIR_INFO: &[u8] = b"pair_info";
pub static FEE_RATE: &[u8] = b"fee_rate";
pub static MOCK: &[u8] = b"mock";
pub static SELF_ADDR: &[u8] = b"self_addr";
pub static WHITELIST: &[u8] = b"whitelist";

pub fn pair_info_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, PairInfoResponse> {
    singleton_read(storage, PAIR_INFO)
}

pub fn pair_info_w<S: Storage>(storage: &mut S) -> Singleton<S, PairInfoResponse> {
    singleton(storage, PAIR_INFO)
}

pub fn fee_rate_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Decimal> {
    singleton_read(storage, FEE_RATE)
}

pub fn fee_rate_w<S: Storage>(storage: &mut S) -> Singleton<S, Decimal> {
    singleton(storage, FEE_RATE)
}

pub fn mock_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, bool> {
    singleton_read(storage, MOCK)
}

pub fn mock_w<S: Storage>(storage: &mut S) -> Singleton<S, bool> {
    singleton(storage, MOCK)
}

pub fn self_addr_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, HumanAddr> {
    singleton_read(storage, SELF_ADDR)
}

pub fn self_addr_w<S: Storage>(storage: &mut S) -> Singleton<S, HumanAddr> {
    singleton(storage, SELF_ADDR)
}

pub fn whitelist_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, HumanAddr> {
    singleton_read(storage, WHITELIST)
}

pub fn whitelist_w<S: Storage>(storage: &mut S) -> Singleton<S, HumanAddr> {
    singleton(storage, WHITELIST)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub token_0: Contract,
    pub token_1: Contract,
    pub fee_rate: Decimal,
    pub whitelist: HumanAddr,
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    pair_info_w(&mut deps.storage).save(&PairInfoResponse {
        liquidity_token: Contract {
            address: HumanAddr("".to_string()),
            code_hash: "".to_string(),
        },
        factory: Contract {
            address: HumanAddr("".to_string()),
            code_hash: "".to_string(),
        },
        pair: TokenPairSerde {
            token_0: TokenType::CustomToken {
                contract_addr: msg.token_0.address.clone(),
                token_code_hash: msg.token_0.code_hash.clone(),
            },
            token_1: TokenType::CustomToken {
                contract_addr: msg.token_1.address.clone(),
                token_code_hash: msg.token_1.code_hash.clone(),
            },
        },
        amount_0: Uint128::zero(),
        amount_1: Uint128::zero(),
        total_liquidity: Uint128::zero(),
        contract_version: Uint128::zero(),
    })?;
    let messages = vec![
        set_viewing_key_msg(
            "SecretSwap".to_string(),
            None,
            1,
            msg.token_0.code_hash.clone(),
            msg.token_0.address.clone(),
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

    fee_rate_w(&mut deps.storage).save(&msg.fee_rate)?;
    mock_w(&mut deps.storage).save(&false)?;
    self_addr_w(&mut deps.storage).save(&env.contract.address.clone())?;
    whitelist_w(&mut deps.storage).save(&msg.whitelist)?;

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
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        msg: Option<Binary>,
    },
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
            pair_info_w(&mut deps.storage).save(&PairInfoResponse {
                liquidity_token: Contract {
                    address: HumanAddr("".to_string()),
                    code_hash: "".to_string(),
                },
                factory: Contract {
                    address: HumanAddr("".to_string()),
                    code_hash: "".to_string(),
                },
                pair: TokenPairSerde {
                    token_0: TokenType::CustomToken {
                        contract_addr: token_a.address.clone(),
                        token_code_hash: token_a.code_hash.clone(),
                    },
                    token_1: TokenType::CustomToken {
                        contract_addr: token_b.address.clone(),
                        token_code_hash: token_b.code_hash.clone(),
                    },
                },
                amount_0: amount_a,
                amount_1: amount_b,
                total_liquidity: Uint128::zero(),
                contract_version: Uint128::zero(),
            })?;

            mock_w(&mut deps.storage).save(&true)?;

            Ok(HandleResponse::default())
        }
        HandleMsg::Receive {
            sender,
            msg,
            amount,
            ..
        } => {
            let mut messages = vec![];
            if let Some(message) = msg {
                match from_binary(&message)? {
                    SwapTokens {
                        expected_return, ..
                    } => {
                        let pair = get_pair_res(&deps).unwrap();
                        let mut token_0_addr = HumanAddr("".to_string());
                        let mut token_0_code_hash = "".to_string();
                        match pair.pair.token_0 {
                            TokenType::CustomToken {
                                contract_addr,
                                token_code_hash,
                            } => {
                                token_0_addr = contract_addr;
                                token_0_code_hash = token_code_hash;
                            }
                            _ => {}
                        }
                        let mut token_1_addr = HumanAddr("".to_string());
                        let mut token_1_code_hash = "".to_string();
                        match pair.pair.token_1 {
                            TokenType::CustomToken {
                                contract_addr,
                                token_code_hash,
                            } => {
                                token_1_addr = contract_addr;
                                token_1_code_hash = token_code_hash;
                            }
                            _ => {}
                        }

                        let mut contract_addr = HumanAddr("".to_string());
                        let mut contract_code_hash = "".to_string();
                        let mut return_amount = Uint128::zero();
                        if token_0_addr == env.message.sender {
                            contract_addr = token_1_addr.clone();
                            contract_code_hash = token_1_code_hash.clone();
                            let return_amount_pre_fee = dex::pool_take_amount(
                                amount,
                                pair.amount_0.checked_sub(amount)?,
                                pair.amount_1,
                            );
                            let fee = return_amount_pre_fee * fee_rate_r(&deps.storage).load()?;
                            if env.message.sender.clone() == whitelist_r(&deps.storage).load()? {
                                return_amount = return_amount_pre_fee;
                            } else {
                                return_amount = return_amount_pre_fee.checked_sub(fee)?;
                            }
                        } else if token_1_addr == env.message.sender {
                            contract_addr = token_0_addr.clone();
                            contract_code_hash = token_0_code_hash.clone();
                            let return_amount_pre_fee =
                                dex::pool_take_amount(amount, pair.amount_1, pair.amount_0);
                            let fee = return_amount_pre_fee * fee_rate_r(&deps.storage).load()?;
                            if env.message.sender.clone() == whitelist_r(&deps.storage).load()? {
                                return_amount = return_amount_pre_fee;
                            } else {
                                return_amount = return_amount_pre_fee.checked_sub(fee)?;
                            }
                        }
                        if return_amount < expected_return.unwrap() {
                            return Err(StdError::unauthorized());
                        }
                        messages.push(send_msg(
                            sender,
                            c_std::Uint128::from(return_amount.u128()),
                            None,
                            None,
                            None,
                            1,
                            contract_code_hash.clone(),
                            contract_addr,
                        )?);
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

pub fn get_pair_res<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<PairInfoResponse> {
    let mut pair = pair_info_r(&deps.storage).load()?;
    let mut token_0_addr = HumanAddr("".to_string());
    let mut token_0_code_hash = "".to_string();
    match pair.pair.token_0.clone() {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => {
            token_0_addr = contract_addr;
            token_0_code_hash = token_code_hash;
        }
        _ => {}
    }
    let mut token_1_addr = HumanAddr("".to_string());
    let mut token_1_code_hash = "".to_string();
    match pair.pair.token_1.clone() {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => {
            token_1_addr = contract_addr;
            token_1_code_hash = token_code_hash;
        }
        _ => {}
    }

    let token_0_bal = Uint128::from(
        balance_query(
            &deps.querier,
            self_addr_r(&deps.storage).load()?,
            "SecretSwap".to_string(),
            1,
            token_0_code_hash.clone(),
            token_0_addr.clone(),
        )?
        .amount
        .u128(),
    );
    let token_1_bal = Uint128::from(
        balance_query(
            &deps.querier,
            self_addr_r(&deps.storage).load()?,
            "SecretSwap".to_string(),
            1,
            token_1_code_hash.clone(),
            token_1_addr.clone(),
        )?
        .amount
        .u128(),
    );
    pair.amount_0 = token_0_bal.clone();
    pair.amount_1 = token_1_bal.clone();
    Ok(pair)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: PairQuery,
) -> StdResult<Binary> {
    match msg {
        PairQuery::PairInfo {} => to_binary(&pair_info_r(&deps.storage).load()?),
        PairQuery::GetEstimatedPrice { offer, exclude_fee } => {
            let mut pair = get_pair_res(deps)?;
            if mock_r(&deps.storage).load()? {
                pair = pair_info_r(&deps.storage).load()?;
            }
            let mut return_amount = Uint128::zero();
            if pair.pair.token_0 == offer.token {
                /*
                let take_amount = dex::pool_take_amount(
                        offer_asset.amount,
                        pair.amount_0,
                        pair.amount_1,
                    );

                return Err(StdError::generic_err(
                        format!("INPUT 0 pools input: {}, give: {}, take: {}",
                                offer_asset.amount,
                                pair.amount_0,
                                pair.amount_1
                        )
                ));
                */
                let return_amount_pre_fee =
                    dex::pool_take_amount(offer.amount, pair.amount_0, pair.amount_1);
                let fee = return_amount_pre_fee * fee_rate_r(&deps.storage).load()?;
                if Some(true) == exclude_fee {
                    return_amount = return_amount_pre_fee;
                } else {
                    return_amount = return_amount_pre_fee.checked_sub(fee)?;
                }
            } else if pair.pair.token_1 == offer.token {
                let return_amount_pre_fee =
                    dex::pool_take_amount(offer.amount, pair.amount_1, pair.amount_0);
                let fee = return_amount_pre_fee * fee_rate_r(&deps.storage).load()?;
                if Some(true) == exclude_fee {
                    return_amount = return_amount_pre_fee;
                } else {
                    return_amount = return_amount_pre_fee.checked_sub(fee)?;
                }
            }
            if return_amount == Uint128::zero() {
                return Err(StdError::generic_err("Not a token on this pair"));
            }
            let resp = QueryMsgResponse::EstimatedPrice {
                estimated_price: return_amount,
            };
            return to_binary(&resp);
        }
    }
}
