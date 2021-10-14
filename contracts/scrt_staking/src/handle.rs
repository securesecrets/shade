use cosmwasm_std::{
    debug_print, to_binary, Api, Binary,
    Env, Extern, Storage, HandleResponse,
    StdResult, StdError,
    CosmosMsg, Uint128,
    Delegation, Coin, StakingMsg,
    Validator, Querier, HumanAddr,
};
use secret_toolkit::{
    snip20,
    snip20::{
        token_info_query,
        register_receive_msg, 
        set_viewing_key_msg,
        redeem_msg, deposit_msg,
    },
};

use shade_protocol::{
    scrt_staking::{
        HandleAnswer,
        ValidatorBounds,
    },
    asset::Contract,
    generic_response::ResponseStatus,
};

use std::cmp;

use crate::state::{
    config_w, config_r, 
    self_address_r,
    viewing_key_r,
};

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    _from: HumanAddr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    debug_print!("Received {}", amount);

    //TODO: verify sscrt else (fail/send to treasury)

    // Redeem all sscrt for scrt
    // Fail if incorrect denom
    // Stake all current scrt - unbondings

    let mut messages: Vec<CosmosMsg> = vec![];

    let config = config_r(&deps.storage).load()?;

    if config.sscrt.address != env.message.sender {
        return Err(StdError::GenericErr { 
            msg: "Only accepts sSCRT".to_string(), 
            backtrace: None 
        });
    }

    let sscrt_balance = snip20::QueryMsg::Balance { 
        address: self_address_r(&deps.storage).load()?, 
        key: viewing_key_r(&deps.storage).load()?,
    }.query(
        &deps.querier,
        1,
        config.sscrt.code_hash.clone(),
        config.sscrt.address.clone(),
    )?;
    
    // Redeem sSCRT -> SCRT
    messages.push(
        redeem_msg(
            sscrt_balance,
            None,
            None,
            256,
            config.sscrt.code_hash.clone(),
            config.sscrt.address.clone(),
        )?
    );

    let scrt_balance = (deps.querier.query_balance(env.contract.address.clone(), &"uscrt".to_string())?).amount;

    let validator = choose_validator(&deps, env.block.time)?;

    messages.push(CosmosMsg::Staking(StakingMsg::Delegate {
        validator: validator.address,
        amount: Coin {
            denom: "uscrt".to_string(),
            amount: scrt_balance,
        },
    }));

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( 
            &HandleAnswer::Receive {
                status: ResponseStatus::Success,
            } 
        )?),
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
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
        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success } )? )
    })
}

pub fn unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {

    let mut messages: Vec<CosmosMsg> = vec![];
    let mut delegations = deps.querier.query_all_delegations(env.contract.address)?;

    // Sorting largest delegation first, undelegating from largest first
    delegations.sort_by(|a, b| b.amount.amount.cmp(&a.amount.amount));

    let mut remaining_amount = amount;

    for delegation in delegations {
        let mut unstaking_amount = remaining_amount;

        if delegation.amount.amount < remaining_amount {
            unstaking_amount = delegation.amount.amount;
        }

        messages.push(CosmosMsg::Staking(StakingMsg::Undelegate {
            validator: HumanAddr(delegation.validator.to_string()),
            amount: Coin {
                denom: "uscrt".to_string(),
                amount: unstaking_amount,
            },
        }));

        remaining_amount = (remaining_amount - unstaking_amount)?;
        if remaining_amount <= Uint128(0) {
            break;
        }
    }

    Ok(HandleResponse {
        messages: messages,
        log: vec![],
        data: Some( to_binary( 
            &HandleAnswer::Receive {
                status: ResponseStatus::Success,
            }
        )?),
    })
}

/*
 * Claims rewards and collects completed unbondings
 * from a given validator and returns them directly to treasury
 *
 * TODO: convert to sSCRT first or rely on treasury to do so
 */
pub fn collect<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    validator: HumanAddr,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Staking(StakingMsg::Withdraw {
                validator,
                recipient: Some(config.treasury.address),
            })
        ],
        log: vec![],
        data: Some( to_binary( 
            &HandleAnswer::Receive {
                status: ResponseStatus::Success,
            }
        )?),
    })
}

pub fn choose_validator<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    seed: u64,
) -> StdResult<Validator> {

    let validators = deps.querier.query_validators()?;
    let bounds = (config_r(&deps.storage).load()?).validator_bounds;
    let mut candidates = vec![];

    for validator in validators {
        if is_validator_inbounds(&validator, &bounds) {
            candidates.push(validator);
        }
    }

    // seed will likely be env.block.time
    Ok(candidates[(seed % candidates.len() as u64) as usize].clone())
}

pub fn is_validator_inbounds(
    validator: &Validator,
    bounds: &ValidatorBounds,
) -> bool {

    validator.commission <= bounds.max_commission && validator.commission >= bounds.min_commission
}

