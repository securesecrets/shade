use crate::state::{config_r, config_w, index_w, sswap_pairs_r, sswap_pairs_w};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage,
};
use secret_toolkit::{
    snip20::{token_info_query, TokenInfo},
    utils::Query,
};
use shade_protocol::{
    asset::Contract,
    generic_response::ResponseStatus,
    oracle::{HandleAnswer, IndexElement, SswapPair},
    secretswap::{PairQuery, PairResponse},
    snip20::Snip20Asset,
};

pub fn register_sswap_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pair: Contract,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let (token_contract, token_info) =
        fetch_token_paired_to_sscrt_on_sswap(deps, config.sscrt.address, &pair)?;

    sswap_pairs_w(&mut deps.storage).save(
        token_info.symbol.as_bytes(),
        &SswapPair {
            pair,
            asset: Snip20Asset {
                contract: token_contract,
                token_info: token_info.clone(),
                token_config: None,
            },
        },
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterSswapPair {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn unregister_sswap_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pair: Contract,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let (_, token_info) = fetch_token_paired_to_sscrt_on_sswap(deps, config.sscrt.address, &pair)?;

    sswap_pairs_w(&mut deps.storage).remove(token_info.symbol.as_bytes());

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UnregisterSswapPair {
            status: ResponseStatus::Success,
        })?),
    })
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
    let response: PairResponse =
        PairQuery::Pair {}.query(&deps.querier, pair.code_hash.clone(), pair.address.clone())?;

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

    match sswap_pairs_r(&deps.storage).may_load(symbol.as_bytes())? {
        None => {}
        Some(_) => {
            return Err(StdError::GenericErr {
                msg: "symbol collides with an existing SecretSwap Pair".to_string(),
                backtrace: None,
            });
        }
    }

    //Dont need this, can just use may_load
    /*
    indices_w(&mut deps.storage).update(|mut symbols| {
        symbols.push(symbol.clone());
        Ok(symbols)
    })?;
    */

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
