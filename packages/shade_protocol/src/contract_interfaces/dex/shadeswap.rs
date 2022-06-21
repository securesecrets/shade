use cosmwasm_math_compat::{Decimal, Uint128};
use crate::{
    contract_interfaces::{dex::dex::{TokenType}, oracles::band},
    utils::{
        asset::Contract,
        price::{normalize_price, translate_price},
    },
};
use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Binary};

use schemars::JsonSchema;
use secret_toolkit::{utils::{Query, HandleCallback}, serialization::Base64};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    AddLiquidityToAMMContract {
        deposit: TokenPairAmount,
        slippage: Option<Decimal>,
    },
    SwapTokens {
        /// The token type to swap from.
        offer: TokenAmount,
        expected_return: Option<Uint128>,
        to: Option<HumanAddr>,
        router_link: Option<Contract>,
        callback_signature: Option<Binary>
    },
    // SNIP20 receiver interface
    Receive {
        from: HumanAddr,
        msg: Option<Binary>,
        amount: Uint128,
    },
    // Sent by the LP token contract so that we can record its address.
    OnLpTokenInitAddr,
    AddWhiteListAddress {
        address: HumanAddr,
    },
    RemoveWhitelistAddresses {
        addresses: Vec<HumanAddr>
    },
    SetAMMPairAdmin {
        admin: HumanAddr
    },
    SetStakingContract { contract: Contract },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetPairInfo,
    GetTradeHistory { pagination: Pagination },
    GetWhiteListAddress,
    GetTradeCount,
    GetAdmin,
    GetStakingContract,
    GetClaimReward{time: u128, staker: HumanAddr},
    GetEstimatedPrice { offer: TokenAmount}
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
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
        addresses: Vec<HumanAddr>,
    },
    GetTradeCount {
        count: u64,
    },
    GetAdminAddress {
        address: HumanAddr
    },
    GetClaimReward {
        amount: Uint128,
    },
    StakingContractInfo{
        staking_contract: Contract
    },
    EstimatedPrice {
        estimated_price: Uint128
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenAmount {
    pub token: TokenType,
    pub amount: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenPairAmount {
    pub pair:     TokenPair,
    pub amount_0: Uint128,
    pub amount_1: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenPair(pub TokenType, pub TokenType);

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pagination {
    pub start: u64,
    pub limit: u8,
}