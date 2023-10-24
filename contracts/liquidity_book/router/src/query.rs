use std::str::FromStr;

use cosmwasm_std::{
    to_binary,
    Binary,
    Deps,
    QuerierWrapper,
    QueryRequest,
    StdError,
    StdResult,
    Uint128,
    Uint256,
    WasmQuery,
};
use shade_protocol::{liquidity_book::lb_pair::SwapResult, Contract};
use shadeswap_shared::{
    core::TokenAmount,
    msg::amm_pair::{QueryMsg as AMMPairQueryMsg, QueryMsgResponse as AMMPairQueryReponse},
    router::{Hop, QueryMsgResponse},
};

pub fn pair_contract_config(
    querier: &QuerierWrapper,
    pair_contract_address: Contract,
) -> StdResult<AMMPairQueryReponse> {
    let result: AMMPairQueryReponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_contract_address.address.to_string(),
        code_hash: pair_contract_address.code_hash.clone(),
        msg: to_binary(&AMMPairQueryMsg::GetPairInfo {})?,
    }))?;

    return Ok(result);
}

pub fn swap_simulation(
    deps: Deps,
    path: Vec<Hop>,
    offer: TokenAmount,
    exclude_fee: Option<bool>,
) -> StdResult<Binary> {
    let mut sum_total_fee_amount: Uint128 = Uint128::zero();
    let mut sum_lp_fee_amount: Uint128 = Uint128::zero();
    let mut sum_shade_dao_fee_amount: Uint128 = Uint128::zero();
    let mut next_in = offer.clone();
    let querier = &deps.querier;

    for hop in path {
        let contract = Contract {
            address: deps.api.addr_validate(&hop.addr)?,
            code_hash: hop.code_hash,
        };
        let contract_info: AMMPairQueryReponse =
            querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract.address.to_string(),
                code_hash: contract.code_hash.clone(),
                msg: to_binary(&AMMPairQueryMsg::GetPairInfo {})?,
            }))?;

        match contract_info {
            AMMPairQueryReponse::GetPairInfo {
                liquidity_token: _,
                factory: _,
                pair,
                amount_0: _,
                amount_1: _,
                total_liquidity: _,
                contract_version: _,
                fee_info: _,
                stable_info: _,
            } => {
                let result: AMMPairQueryReponse =
                    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                        contract_addr: contract.address.to_string(),
                        code_hash: contract.code_hash.clone(),
                        msg: to_binary(&AMMPairQueryMsg::SwapSimulation {
                            offer: next_in.clone(),
                            exclude_fee,
                        })?,
                    }))?;
                match result {
                    AMMPairQueryReponse::SwapSimulation {
                        total_fee_amount,
                        lp_fee_amount,
                        shade_dao_fee_amount,
                        result,
                        price: _,
                    } => {
                        if pair.1 == next_in.token {
                            next_in = TokenAmount {
                                token: pair.0,
                                amount: result.return_amount,
                            };
                        } else {
                            next_in = TokenAmount {
                                token: pair.1,
                                amount: result.return_amount,
                            };
                        }
                        sum_total_fee_amount =
                            total_fee_amount.checked_add(sum_total_fee_amount)?;
                        sum_lp_fee_amount = lp_fee_amount.checked_add(sum_lp_fee_amount)?;
                        sum_shade_dao_fee_amount =
                            shade_dao_fee_amount.checked_add(sum_shade_dao_fee_amount)?;
                    }
                    _ => return Err(StdError::generic_err("Failed to complete hop.")),
                };
            }
            _ => return Err(StdError::generic_err("Failed to complete hop.")),
        }
    }

    to_binary(&QueryMsgResponse::SwapSimulation {
        total_fee_amount: sum_total_fee_amount,
        lp_fee_amount: sum_lp_fee_amount,
        shade_dao_fee_amount: sum_shade_dao_fee_amount,
        result: SwapResult {
            return_amount: next_in.amount,
        },
        price: (Uint256::from_str(&next_in.amount.to_string())?
            / Uint256::from_str(&offer.amount.to_string())?)
        .to_string(),
    })
}
