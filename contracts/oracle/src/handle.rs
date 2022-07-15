use crate::state::{config_r, config_w, dex_pairs_r, dex_pairs_w, index_r, index_w};
use shade_protocol::c_std::{
    to_binary,
    Api,
    Env,
    DepsMut,
    Response,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::{
    snip20::helpers::{token_info_query, TokenInfo},
};
use shade_protocol::{
    contract_interfaces::{
        dex::{dex, secretswap, sienna},
        oracles::oracle::{HandleAnswer, IndexElement},
        snip20::helpers::Snip20Asset,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};

pub fn register_pair(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pair: Contract,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;
    if info.sender != config.admin {
        return Err(StdError::generic_err("unauthorized"));
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
    } else if sienna::is_pair(deps, pair.clone())? {
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

    if let Some(tp) = trading_pair {
        if let Some(td) = token_data {
            // If symbol would override an index
            if let Some(_) = index_r(deps.storage).may_load(td.1.symbol.as_bytes())? {
                return Err(StdError::generic_err(
                    "Symbol already registered as an index",
                ));
            }

            if let Some(mut pairs) = dex_pairs_r(deps.storage).may_load(td.1.symbol.as_bytes())? {
                //TODO: Check pair already registered
                pairs.push(tp.clone());
                dex_pairs_w(deps.storage).save(td.1.symbol.as_bytes(), &pairs)?;
            } else {
                dex_pairs_w(deps.storage).save(td.1.symbol.as_bytes(), &vec![tp.clone()])?;
            }

            return Ok(Response {
                messages: vec![],
                log: vec![],
                data: Some(to_binary(&HandleAnswer::RegisterPair {
                    status: ResponseStatus::Success,
                    symbol: td.1.symbol,
                    pair: tp,
                })?),
            });
        }
        return Err(StdError::generic_err("Failed to extract token data"));
    }

    Err(StdError::generic_err("Failed to extract Trading Pair"))
}

pub fn unregister_pair(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    symbol: String,
    pair: Contract,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;
    if info.sender != config.admin {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(mut pair_list) = dex_pairs_r(deps.storage).may_load(symbol.as_bytes())? {
        if let Some(i) = pair_list
            .iter()
            .position(|p| p.contract.address == pair.address)
        {
            pair_list.remove(i);

            dex_pairs_w(deps.storage).save(symbol.as_bytes(), &pair_list)?;

            return Ok(Response {
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
fn fetch_token_paired_to_sscrt_on_sswap(
    deps: DepsMut,
    sscrt_addr: Addr,
    pair: &Contract,
) -> StdResult<(Contract, TokenInfo)> {
    // Query for snip20's in the pair
    let response: secretswap::PairResponse = secretswap::PairQuery::Pair {}.query(
        &deps.querier,
        pair.code_hash.clone(),
        pair.address.clone(),
    )?;

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

fn fetch_token_paired_to_sscrt_on_sienna(
    deps: DepsMut,
    sscrt_addr: Addr,
    pair: &Contract,
) -> StdResult<(Contract, TokenInfo)> {
    // Query for snip20's in the pair
    let response: sienna::PairInfoResponse = (sienna::PairQuery::PairInfo).query(
        &deps.querier,
        pair.code_hash.clone(),
        pair.address.clone(),
    )?;

    let mut token_contract = match response.pair_info.pair.token_0 {
        sienna::TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => Contract {
            address: contract_addr,
            code_hash: token_code_hash,
        },
        sienna::TokenType::NativeToken { denom } => {
            return Err(StdError::generic_err(
                "Sienna Native Token pairs not supported",
            ));
        }
    };

    // if thats sscrt, switch it
    if token_contract.address == sscrt_addr {
        token_contract = match response.pair_info.pair.token_1 {
            sienna::TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => Contract {
                address: contract_addr,
                code_hash: token_code_hash,
            },
            sienna::TokenType::NativeToken { denom: _ } => {
                return Err(StdError::generic_err(
                    "Sienna Native Token pairs not supported",
                ));
            }
        };
    }
    // if its not, make sure other is sscrt
    else {
        match response.pair_info.pair.token_1 {
            sienna::TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                if contract_addr != sscrt_addr {
                    // if we get here, neither the first or second tokens were sscrt
                    return Err(StdError::NotFound {
                        kind: "Not an SSCRT Pair".to_string(),
                        backtrace: None,
                    });
                }
            }
            sienna::TokenType::NativeToken { denom: _ } => {
                return Err(StdError::generic_err(
                    "Sienna Native Token pairs not supported",
                ));
            }
        }
    }

    let token_info = token_info_query(
        &deps.querier,
        1,
        token_contract.code_hash.clone(),
        token_contract.address.clone(),
    )?;

    Ok((token_contract, token_info))
}

pub fn register_index(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    symbol: String,
    basket: Vec<IndexElement>,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;
    if info.sender != config.admin {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(pairs) = dex_pairs_r(deps.storage).may_load(symbol.as_bytes())? {
        if pairs.len() > 0 {
            return Err(StdError::generic_err(
                "Symbol collides with an existing Dex pair",
            ));
        }
    }

    index_w(deps.storage).save(symbol.as_bytes(), &basket)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::RegisterIndex {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Option<Addr>,
    band: Option<Contract>,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;
    if info.sender != config.admin {
        return Err(StdError::generic_err("unauthorized"));
    }

    // Save new info
    let mut config = config_w(deps.storage);
    config.update(|mut state| {
        if let Some(admin) = admin {
            state.admin = admin;
        }
        if let Some(band) = band {
            state.band = band;
        }

        Ok(state)
    })?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?))
}
