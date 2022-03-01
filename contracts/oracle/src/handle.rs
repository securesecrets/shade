use crate::state::{config_r, config_w, index_w, index_r, dex_pairs_r, dex_pairs_w };
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage,
};
use secret_toolkit::{
    snip20::{token_info_query, TokenInfo},
    utils::Query,
};
use shade_protocol::utils::asset::Contract;
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::{
    oracle::{HandleAnswer, IndexElement},
    snip20::Snip20Asset,
    secretswap,
    sienna,
    dex,
};

pub fn register_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pair: Contract,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    let mut trading_pair: Option<dex::TradingPair> = None;
    let mut token_data: Option<(Contract, TokenInfo)> = None;


    if secretswap::is_pair(deps, pair.clone())? {

         let td = fetch_token_paired_to_sscrt_on_sswap(deps, config.sscrt.address, &pair.clone())?;
        token_data = Some(td.clone());

        trading_pair = Some(dex::TradingPair {
            contract: pair.clone(),
            asset: Snip20Asset {
                contract: td.clone().0,
                token_info: td.clone().1,
                token_config: None,
            },
            dex: dex::Dex::SecretSwap,
        });

    }
    else if sienna::is_pair(deps, pair.clone())? {
        let td = fetch_token_paired_to_sscrt_on_sienna(deps, config.sscrt.address, &pair)?;
        token_data = Some(td.clone());

        trading_pair = Some(dex::TradingPair {
            contract: pair.clone(),
            asset: Snip20Asset {
                contract: td.clone().0,
                token_info: td.1,
                token_config: None,
            },
            dex: dex::Dex::SiennaSwap,
        });
    }


    match trading_pair {
        Some(tp) => {

            match token_data {

                Some(td) => {

                    // If symbol would override an index
                    if let Some(_) = index_r(&deps.storage).may_load(td.1.symbol.as_bytes())? {
                        return Err(StdError::generic_err("Symbol already registered as an index"));
                    }

                    if let Some(mut pairs) = dex_pairs_r(&deps.storage).may_load(td.1.symbol.as_bytes())? {
                        pairs.push(tp.clone());
                        dex_pairs_w(&mut deps.storage).save(
                            td.1.symbol.as_bytes(), 
                            &pairs
                        )?;
                    }
                    else {
                        dex_pairs_w(&mut deps.storage).save(
                            td.1.symbol.as_bytes(), 
                            &vec![tp.clone()]
                        )?;
                    }

                    return Ok(HandleResponse {
                        messages: vec![],
                        log: vec![],
                        data: Some(to_binary(&HandleAnswer::RegisterPair {
                            status: ResponseStatus::Success,
                            symbol: td.1.symbol,
                            pair: tp,
                        })?),
                    });
                }
                None => {
                    Err(StdError::generic_err("Failed to get token data"))
                }
            }
        }
        None => {
            Err(StdError::generic_err("Pair not recognized"))
        }
    }


}

pub fn unregister_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    symbol: String,
    pair: Contract,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // TODO: Find token symbol, pull dex pairs, remove contract addr
    /*
    if secretswap::is_pair(deps, pair.clone())? {
        let (_, token_info) = fetch_token_paired_to_sscrt_on_sswap(deps, config.sscrt.address, &pair.clone())?;
        sswap_pairs_w(&mut deps.storage).remove(token_info.symbol.as_bytes());
    }
    else if sienna::is_pair(deps, pair.clone())? {
        let (_, token_info) = fetch_token_paired_to_sscrt_on_sienna(deps, config.sscrt.address, &pair.clone())?;
        sienna_pairs_w(&mut deps.storage).remove(token_info.symbol.as_bytes());
    }
    */

    if let Some(mut pair_list) = dex_pairs_r(&deps.storage).may_load(symbol.as_bytes())? {

        if let Some(index) = pair_list.iter().position(|p| p.contract.address == pair.address) {
            pair_list.remove(index);
            
            dex_pairs_w(&mut deps.storage).save(
                    symbol.as_bytes(), &pair_list)?;

            return Ok(HandleResponse {
                messages: vec![],
                log: vec![],
                data: Some(to_binary(&HandleAnswer::UnregisterPair {
                    status: ResponseStatus::Success,
                })?),
            });
        }
    }

    Err(StdError::generic_err("Pair not found"))

}

///
/// Will fetch token Contract along with TokenInfo for {symbol} in pair argument.
/// Pair argument must represent Secret Swap contract for {symbol}/sSCRT or sSCRT/{symbol}.
///
fn fetch_token_paired_to_sscrt_on_sswap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    sscrt_addr: HumanAddr,
    pair: &Contract,
) -> StdResult<(Contract, TokenInfo)> {

    // Query for snip20's in the pair
    let response: secretswap::PairResponse =
        secretswap::PairQuery::Pair {}.query(&deps.querier, pair.code_hash.clone(), pair.address.clone())?;

    let mut token_contract = Contract {
        address: response.asset_infos[0].token.contract_addr.clone(),
        code_hash: response.asset_infos[0].token.token_code_hash.clone(),
    };

    // if thats sscrt, switch it
    if token_contract.address == sscrt_addr {
        token_contract = Contract {
            address: response.asset_infos[1].token.contract_addr.clone(),
            code_hash: response.asset_infos[1].token.token_code_hash.clone(),
        }
    }
    // if neither is sscrt
    else if response.asset_infos[1].token.contract_addr != sscrt_addr {
        return Err(StdError::NotFound {
            kind: "Not an sSCRT Pair".to_string(),
            backtrace: None,
        });
    }

    let token_info = token_info_query(
        &deps.querier,
        1,
        token_contract.code_hash.clone(),
        token_contract.address.clone(),
    )?;

    Ok((token_contract, token_info))
}

fn fetch_token_paired_to_sscrt_on_sienna<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    sscrt_addr: HumanAddr,
    pair: &Contract,
) -> StdResult<(Contract, TokenInfo)> {
    // Query for snip20's in the pair
    let response: secretswap::PairResponse =
        secretswap::PairQuery::Pair {}.query(&deps.querier, pair.code_hash.clone(), pair.address.clone())?;

    let mut token_contract = Contract {
        address: response.asset_infos[0].token.contract_addr.clone(),
        code_hash: response.asset_infos[0].token.token_code_hash.clone(),
    };

    // if thats sscrt, switch it
    if token_contract.address == sscrt_addr {
        token_contract = Contract {
            address: response.asset_infos[1].token.contract_addr.clone(),
            code_hash: response.asset_infos[1].token.token_code_hash.clone(),
        }
    }
    // if neither is sscrt
    else if response.asset_infos[1].token.contract_addr != sscrt_addr {
        return Err(StdError::NotFound {
            kind: "Not an SSCRT Pair".to_string(),
            backtrace: None,
        });
    }

    let token_info = token_info_query(
        &deps.querier,
        1,
        token_contract.code_hash.clone(),
        token_contract.address.clone(),
    )?;

    Ok((token_contract, token_info))
}

pub fn register_index<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    symbol: String,
    basket: Vec<IndexElement>,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    if let Some(pairs) = dex_pairs_r(&deps.storage).may_load(symbol.as_bytes())? {
        if pairs.len() > 0 {
            return Err(StdError::generic_err("Symbol collides with an existing Dex pair"));
        }
        
    }

    index_w(&mut deps.storage).save(symbol.as_bytes(), &basket)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterIndex {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin: Option<HumanAddr>,
    band: Option<Contract>,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(admin) = admin {
            state.admin = admin;
        }
        if let Some(band) = band {
            state.band = band;
        }

        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}
