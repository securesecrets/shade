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

    // Redeem sSCRT -> SCRT
    messages.push(
        redeem_msg(
            amount,
            None,
            None,
            256,
            config.sscrt.code_hash.clone(),
            config.sscrt.address.clone(),
        )?
    );

    let mut validator = choose_validator(&deps, env.block.time)?;

    messages.push(CosmosMsg::Staking(StakingMsg::Delegate {
        validator: validator.address.clone(),
        amount: Coin {
            amount,
            denom: "uscrt".to_string(),
        }
    }));

    Ok(HandleResponse {
        messages: messages,
        log: vec![],
        data: Some( to_binary( 
            &HandleAnswer::Receive {
                status: ResponseStatus::Success,
                validator,
            } 
        )?),
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin: Option<HumanAddr>,
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
    validator: HumanAddr,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.admin && env.message.sender != config.treasury {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    if let Some(delegation) = deps.querier.query_delegation(env.contract.address, validator.clone())? {

        let mut messages: Vec<CosmosMsg> = vec![];

        messages.push(CosmosMsg::Staking(StakingMsg::Undelegate {
            validator,
            amount: delegation.amount.clone(),
        }));

        return Ok(HandleResponse {
            messages: messages,
            log: vec![],
            data: Some( to_binary( 
                &HandleAnswer::Unbond {
                    status: ResponseStatus::Success,
                    delegation,
                }
            )?),
        });
    }

    Err(StdError::GenericErr { 
        msg: "No delegation".to_string(),
        backtrace: None 
    })
}

/*
 * Claims rewards and collects completed unbondings
 * from a given validator and returns them directly to treasury
 *
 * TODO: convert to sSCRT first or rely on treasury to do so
 */
pub fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    validator: HumanAddr,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Staking(StakingMsg::Withdraw {
                validator,
                recipient: Some(config.treasury),
            })
        ],
        log: vec![],
        data: Some( to_binary(
            &HandleAnswer::Claim {
                status: ResponseStatus::Success,
            }
        )?),
    })
}

pub fn choose_validator<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    seed: u64,
) -> StdResult<Validator> {

    let mut validators = deps.querier.query_validators()?;
    let bounds = (config_r(&deps.storage).load()?).validator_bounds;

    // filter down to viable candidates
    if let Some(bounds) = bounds {
        let mut candidates = vec![];
        for validator in validators {
            if is_validator_inbounds(&validator, &bounds) {
                candidates.push(validator);
            }
        }
        validators = candidates;
    }

    if validators.len() == 0 {
        return Err(StdError::GenericErr { 
            msg: "No validators within bounds".to_string(),
            backtrace: None 
        })
    }
    // seed will likely be env.block.time
    Ok(validators[(seed % validators.len() as u64) as usize].clone())
}

pub fn is_validator_inbounds(
    validator: &Validator,
    bounds: &ValidatorBounds,
) -> bool {

    validator.commission <= bounds.max_commission && validator.commission >= bounds.min_commission
}
