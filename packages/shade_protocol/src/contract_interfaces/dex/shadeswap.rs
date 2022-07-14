use crate::{
    contract_interfaces::{
        mint,
        dex,
        oracles::band,
    },
    utils::{
        asset::Contract,
        price::{normalize_price, translate_price},
    },
};
use crate::c_std::{Uint128, Addr, StdResult, StdError, Deps, DepsMut};

use crate::utils::Query;
use cosmwasm_schema::{cw_serde};

/*
#[cw_serde]
pub struct Token {
    pub contract_addr: Addr,
    pub token_code_hash: String,
    pub viewing_key: String,
}

#[cw_serde]
pub struct AssetInfo {
    pub token: Token,
}

#[cw_serde]
pub struct Asset {
    pub amount: Uint128,
    pub info: AssetInfo,
}

#[cw_serde]
pub struct Simulation {
    pub offer_asset: Asset,
}
*/

#[cw_serde]
pub enum PairQuery {
    PairInfo,
}

impl Query for PairQuery {
    const BLOCK_SIZE: usize = 256;
}

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
pub struct TokenPair(pub TokenType, pub TokenType);

/*
#[cw_serde]
pub struct SimulationResponse {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
    pub commission_amount: Uint128,
}
*/

#[cw_serde]
pub struct PairInfoResponse {
    pub liquidity_token: Contract,
    pub factory: Contract,
    pub pair: TokenPair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint128,
    pub contract_version: u32,
}

/*
#[cw_serde]
pub struct PoolResponse {
    pub assets: Vec<Asset>,
    pub total_share: Uint128,
}
*/

pub fn is_pair(
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
}

/*
pub fn price(
    deps: Deps,
    pair: dex::TradingPair,
    sscrt: Contract,
    band: Contract,
) -> StdResult<Uint128> {

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

pub fn amount_per_scrt(
    deps: Deps,
    pair: dex::TradingPair,
    sscrt: Contract,
) -> StdResult<Uint128> {

    let response: SimulationResponse = PairQuery::Simulation {
        offer_asset: Asset {
            amount: Uint128::new(1_000_000), // 1 sSCRT (6 decimals)
            info: AssetInfo {
                token: Token {
                    contract_addr: sscrt.address,
                    token_code_hash: sscrt.code_hash,
                    viewing_key: "SecretSwap".to_string(),
                },
            },
        },
    }
    .query(
        &deps.querier,
        pair.contract.code_hash,
        pair.contract.address,
    )?;

    Ok(response.return_amount)
}

pub fn pool_cp(
    deps: Deps,
    pair: dex::TradingPair,
) -> StdResult<Uint128> {
    let pool: PoolResponse = PairQuery::Pool {}.query(
        &deps.querier,
        pair.contract.code_hash,
        pair.contract.address,
    )?;

    // Constant Product
    Ok(Uint128::new(pool.assets[0].amount.u128() * pool.assets[1].amount.u128()))
}
*/
