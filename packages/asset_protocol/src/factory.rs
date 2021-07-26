use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::asset::AssetInfo;

use cosmwasm_std::{
    from_binary, to_binary, AllBalanceResponse, Api, BalanceResponse, BankQuery, Binary, Coin,
    Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage, Uint128, WasmQuery,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Pair {
        asset_infos: [AssetInfo; 2],
    },
    Pairs {
        start_after: Option<[AssetInfo; 2]>,
        limit: Option<u32>,
    },
}

