use crate::{
    c_std::{Addr, Binary, Uint128},
    utils::{
        asset::Contract,
        Query,
    },
};
use cosmwasm_schema::cw_serde;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
pub struct ContractLink {
    pub address: Addr,
    pub code_hash: String,
}

#[cw_serde]
pub enum PairQuery {
    GetPairInfo {},
    GetEstimatedPrice { offer: TokenAmount },
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
pub struct TokenPair {
    pub token_0: TokenType,
    pub token_1: TokenType,
}

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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsgResponse {
    GetPairInfo {
        liquidity_token: Contract,
        factory: Contract,
        pair: TokenPair,
        amount_0: Uint128,
        amount_1: Uint128,
        total_liquidity: Uint128,
        contract_version: u32,
    },
    GetTradeHistory {
        data: Vec<TradeHistory>,
    },
    GetWhiteListAddress {
        addresses: Vec<Addr>,
    },
    GetTradeCount {
        count: u64,
    },
    GetAdminAddress {
        address: Addr,
    },
    GetClaimReward {
        amount: Uint128,
    },
    StakingContractInfo {
        staking_contract: Contract,
    },
    EstimatedPrice {
        estimated_price: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenAmount {
    pub token: TokenType,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct SwapTokens {
    pub expected_return: Option<Uint128>,
    pub to: Option<Addr>,
    pub router_link: Option<ContractLink>,
    pub callback_signature: Option<Binary>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct TradeHistory {
    pub price: Uint128,
    pub amount: Uint128,
    pub timestamp: u64,
    pub direction: String,
    pub total_fee_amount: Uint128,
    pub lp_fee_amount: Uint128,
    pub shade_dao_fee_amount: Uint128,
}

/*
#[cw_serde]
pub struct PoolResponse {
    pub assets: Vec<Asset>,
    pub total_share: Uint128,
}
*/

/*pub fn is_pair(deps: DepsMut, pair: Contract) -> StdResult<bool> {
    Ok(
        match (PairQuery::PairInfo).query::<PairInfoResponse>(&deps.querier, &pair) {
            Ok(_) => true,
            Err(_) => false,
        },
    )
}*/

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
