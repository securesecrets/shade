use crate::{
    c_std::{Addr, Deps, StdResult, Uint128},
    contract_interfaces::{dex::dex, oracles::band},
    utils::{
        asset::Contract,
        price::{normalize_price, translate_price},
        Query,
    },
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

#[cw_serde]
pub enum TokenType {
    CustomToken {
        contract_addr: Addr,
        token_code_hash: String,
    },
    NativeToken {
        denom: String,
    },
}

#[cw_serde]
pub struct Pair {
    pub token_0: TokenType,
    pub token_1: TokenType,
}

/*
#[cw_serde]
pub struct AssetInfo {
    pub token: Token,
}
*/

#[cw_serde]
pub struct TokenTypeAmount {
    pub amount: Uint128,
    pub token: TokenType,
}

#[cw_serde]
pub struct Swap {
    pub send: SwapOffer,
}

#[cw_serde]
pub struct SwapOffer {
    pub recipient: Addr,
    pub amount: Uint128,
    pub msg: Binary,
}

#[cw_serde]
pub enum ReceiverCallbackMsg {
    Swap {
        expected_return: Option<Uint128>,
        to: Option<Addr>,
    },
}

#[cw_serde]
pub struct CallbackMsg {
    pub swap: CallbackSwap,
}

#[cw_serde]
pub struct CallbackSwap {
    pub expected_return: Uint128,
}

#[cw_serde]
pub struct SwapSimulation {
    pub offer: TokenTypeAmount,
}

#[cw_serde]
pub enum PairQuery {
    /*
    Pool {},
    */
    PairInfo,
    SwapSimulation { offer: TokenTypeAmount },
}

impl Query for PairQuery {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct SimulationResponse {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
    pub commission_amount: Uint128,
}

#[cw_serde]
pub struct PairInfo {
    pub liquidity_token: Contract,
    pub factory: Contract,
    pub pair: Pair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint128,
    pub contract_version: u32,
}

#[cw_serde]
pub struct PairInfoResponse {
    pub pair_info: PairInfo,
}

/*pub fn is_pair(
    deps: DepsMut,
    pair: Contract,
) -> StdResult<bool> {
    Ok(
        match (PairQuery::PairInfo).query::<PairInfoResponse>(
            &deps.querier,
            &pair
        ) {
            Ok(_) => true,
            Err(_) => false,
        },
    )
}*/

pub fn price(
    deps: &Deps,
    pair: dex::TradingPair,
    sscrt: Contract,
    band: Contract,
) -> StdResult<Uint128> {
    // TODO: This should be passed in to avoid multipl BAND SCRT queries in one query
    let scrt_result = band::reference_data(deps, "SCRT".to_string(), "USD".to_string(), band)?;

    // SCRT-USD / SCRT-symbol
    Ok(translate_price(
        scrt_result.rate,
        normalize_price(
            amount_per_scrt(deps, pair.clone(), sscrt)?,
            pair.asset.token_info.decimals,
        ),
    ))
}

pub fn amount_per_scrt(deps: &Deps, pair: dex::TradingPair, sscrt: Contract) -> StdResult<Uint128> {
    let response: SimulationResponse = PairQuery::SwapSimulation {
        offer: TokenTypeAmount {
            amount: Uint128::new(1_000_000), // 1 sSCRT (6 decimals)
            token: TokenType::CustomToken {
                contract_addr: sscrt.address,
                token_code_hash: sscrt.code_hash,
            },
        },
    }
    .query(&deps.querier, &pair.contract)?;

    Ok(response.return_amount)
}

pub fn pool_cp(deps: &Deps, pair: dex::TradingPair) -> StdResult<Uint128> {
    let pair_info: PairInfoResponse = PairQuery::PairInfo.query(&deps.querier, &pair.contract)?;

    // Constant Product
    Ok(Uint128::new(
        pair_info.pair_info.amount_0.u128() * pair_info.pair_info.amount_1.u128(),
    ))
}
