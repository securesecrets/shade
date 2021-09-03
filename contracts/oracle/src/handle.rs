use cosmwasm_std::{
    to_binary, Api,
    Env, Extern, HandleResponse,
    Querier, StdResult, StdError, Storage,
    HumanAddr,
};
use secret_toolkit::{
    utils::Query,
    snip20::{
        token_info_query, 
    },
};
use shade_protocol::{
    oracle::{
        HandleAnswer,
        SswapPair
    },
    asset::Contract,
    generic_response::ResponseStatus,
    snip20::{
        Snip20Asset,
        token_config_query,
    },
    secretswap::{
        PairQuery,
        PairResponse,
    }
};
use crate::state::{
    config_w, config_r,
    sswap_pairs_w,
};

pub fn register_sswap_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pair: Contract,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    //Query for snip20's in the pair
    let response: PairResponse = PairQuery::Pair {}.query(
        &deps.querier,
        pair.code_hash.clone(),
        pair.address.clone(),
    )?;

    let mut token_contract = Contract {
        address: response.asset_infos[0].token.contract_addr.clone(),
        code_hash: response.asset_infos[0].token.token_code_hash.clone(),
    };
    // if thats sscrt, switch it
    if token_contract.address == config.sscrt.address {
        token_contract = Contract {
            address: response.asset_infos[1].token.contract_addr.clone(),
            code_hash: response.asset_infos[1].token.token_code_hash.clone(),
        }
    }
    // if neither is sscrt
    else if response.asset_infos[1].token.contract_addr != config.sscrt.address {
        return Err(StdError::NotFound { kind: "Not an SSCRT Pair".to_string(), backtrace: None });
    }

    let token_info = token_info_query(&deps.querier, 1,
                      token_contract.code_hash.clone(),
                      token_contract.address.clone())?;
    let token_config = token_config_query(&deps.querier, token_contract.clone())?;

    sswap_pairs_w(&mut deps.storage).save(token_info.symbol.as_bytes(), &SswapPair {
        pair,
        asset: Snip20Asset {
            contract: token_contract,
            token_info: token_info.clone(),
            token_config,
        }
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::RegisterSswapPair {
            status: ResponseStatus::Success } )? )
    })

}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
    band: Option<Contract>,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(owner) = owner {
            state.owner = owner;
        }
        if let Some(band) = band {
            state.band = band;
        }

        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateConfig{
            status: ResponseStatus::Success } )? )
    })
}
