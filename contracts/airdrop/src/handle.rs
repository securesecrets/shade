use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage, CosmosMsg, HumanAddr, Uint128, from_binary, Empty};
use shade_protocol::asset::Contract;
use crate::state::{config_r, config_w, sn_delegators_r, sn_delegators_w};
use shade_protocol::airdrop::{HandleAnswer, ValidatorWeight, StoredDelegator, Delegation};
use shade_protocol::generic_response::ResponseStatus;
use secret_toolkit::snip20::{token_info_query, mint_msg};

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin: Option<HumanAddr>,
    airdrop_snip20: Option<Contract>,
    sn_validator_weights: Option<Vec<ValidatorWeight>>,
    sn_banned_validators: Option<Vec<HumanAddr>>,
    sn_whale_cap: Option<Uint128>,
    start_date: Option<u64>,
    end_date: Option<u64>,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(admin) = admin {
            state.owner = admin;
        }
        if let Some(airdrop_snip20) = airdrop_snip20 {
            state.airdrop_snip20 = airdrop_snip20;
        }
        if let Some(sn_validator_weights) = sn_validator_weights {
            state.sn_validator_weights = sn_validator_weights;
        }
        if let Some(sn_banned_validators) = sn_banned_validators {
            state.sn_banned_validators = sn_banned_validators
        }
        if let Some(sn_whale_cap) = sn_whale_cap {
            state.sn_whale_cap = Some(sn_whale_cap);
        }
        if let Some(start_date) = start_date {
            state.start_date = start_date;
        }
        if let Some(end_date) = end_date {
            state.end_date = Some(end_date);
        }

        Ok(state)
    });

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_redeem<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    // Check if airdrop started
    if env.block.time < config.start_date {
        return Err(StdError::Unauthorized { backtrace: None })
    }
    if let Some(end_date) = config.end_date {
        if env.block.time > end_date {
            return Err(StdError::Unauthorized { backtrace: None })
        }
    }

    // Calculate airdrop if eligible
    let mint_amount = airdrop_from_address(&deps, env.message.sender.clone())?;

    // Redeem and then cancel
    let messages =  vec![mint_msg(env.message.sender.clone(), mint_amount,
                                  None, 1,
                                  config.airdrop_snip20.code_hash,
                                  config.airdrop_snip20.address)?];

    // We can ignore if delegator is eligible since this was already checked in the calculator
    sn_delegators_w(&mut deps.storage).update(env.message.sender.to_string().as_bytes(),
                                              |state| {
                                                  let mut delegator = state.unwrap();
                                                  delegator.redeemed = true;
                                                  Ok(delegator)
                                              });

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Redeem {
            status: ResponseStatus::Success } )? )
    })
}

pub fn airdrop_from_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<Uint128> {
    let config = config_r(&deps.storage).load()?;

    let delegator = match sn_delegators_r(&deps.storage).load(
        address.to_string().as_bytes()) {
        Ok(delegator) => delegator,
        Err(_) => return Err(StdError::NotFound { kind: "Address".to_string(), backtrace: None })
    };

    if delegator.redeemed {
        return Err(StdError::Unauthorized { backtrace: None })
    }

    Ok(calculate_airdrop(delegator.delegations, config.sn_validator_weights,
                         config.sn_banned_validators, config.sn_whale_cap))
}

pub fn calculate_airdrop (
    delegations: Vec<Delegation>,
    validator_weights: Vec<ValidatorWeight>,
    banned_validators: Vec<HumanAddr>,
    whale_cap: Option<Uint128>,
) -> Uint128 {
    let mut total:u128 = 0;
    // NOTE: PLACEHOLDER EQUATION
    for delegation in &delegations {
        // Check that delegation is not on banned validator
        // and also delegation is not exceeding whale cap
        if !banned_validators.contains(&delegation.validator_address) &&
            delegation.amount < whale_cap {
            let mut weight:u128 = 100;
            // Check if delegating to validators with special weights
            for validator in &validator_weights {
                if delegation.validator_address == validator.validator_address {
                    weight = validator.weight.u128();
                    break;
                }
            }
            total += (delegation.amount.u128() * weight) / 100;
        }
    }

    Uint128(total)
}