use cosmwasm_std::{to_binary, ContractInfo, QuerierWrapper, QueryRequest, StdResult, WasmQuery};
use interfaces::ILBPair::{self, TokensResponse};

pub fn pair_contract_config(
    querier: &QuerierWrapper,
    pair_contract_address: ContractInfo,
) -> StdResult<TokensResponse> {
    let result: ILBPair::TokensResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_contract_address.address.to_string(),
        code_hash: pair_contract_address.code_hash.clone(),
        msg: to_binary(&ILBPair::QueryMsg::GetTokens {})?,
    }))?;

    return Ok(result);
}
