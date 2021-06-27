use crate::asset::{Asset, AssetInfo, PairInfo};
use crate::factory::QueryMsg as FactoryQueryMsg;

use cosmwasm_std::{
    from_binary, to_binary, AllBalanceResponse, Api, BalanceResponse, BankQuery, Binary, Coin,
    Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage, Uint128, WasmQuery,
};

pub fn query_pair_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    factory_contract: &HumanAddr,
    factory_contract_callback_code_hash: String,
    asset_infos: &[AssetInfo; 2],
) -> StdResult<PairInfo> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: factory_contract.clone(),
        callback_code_hash: factory_contract_callback_code_hash,
        msg: to_binary(&FactoryQueryMsg::Pair {
            asset_infos: asset_infos.clone(),
        })?,
    }))
}