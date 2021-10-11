use cosmwasm_std::{
    debug_print, to_binary, Api, Binary,
    Env, Extern, HandleResponse,
    Querier, StdError, StdResult, Storage, 
    CosmosMsg, HumanAddr, Uint128,
    Delegation, Coin, StakingMsg,
};
use secret_toolkit::{
    snip20::{
        token_info_query,
        register_receive_msg, 
        set_viewing_key_msg,
    },
};

use shade_protocol::{
    staking_pool::HandleAnswer,
    asset::Contract,
    generic_response::ResponseStatus,
};

use crate::state::{
    config_w, config_r, 
    delegations_w, delegations_r,
    unbondings_w, unbondings_r,
};

use rand::random;

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    _from: HumanAddr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    debug_print!("Received {}", amount);

    // Redeem sscrt for scrt
    // Fail if incorrect denom

    // Stake all current scrt - unbondings

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

    let bonds = deps.querier.query_all_delegations(env.contract.address)?;

    let unbonding = Unbonding {
        amount,
        start: env.block.time,
    }

}

pub fn collect<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {

    let unbondings = unbondings_r(&deps.storage).load();
    for unbonding in unbondings {
        //Determine complete unbondings
        //Send completed unbondings to user
        //remove unbonding
    }

}

pub fn claim_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {

    let delegations = deps.querier.query_all_delegations(env.contract.address.clone())?;
    let mut messages = vec![];

    for delegation in delegations {

        messages.push(
            CosmosMsg::Staking(StakingMsg::Withdraw {
                validator: delegation.validator,
            })
        );
    }

    let balance = deps.querier.query_balance(env.contract.address.clone(), &"uscrt".to_string())?;

    messages.push(
        CosmosMsg::Staking(StakingMsg::Delegate {
            validator: choose_validator(),
            amount: Coin {
                denom: "uscrt".to_string(),
                amount: Uint128(balance),
            },
        })
    )

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

pub fn choose_validator<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<Validator> {

    let validators = deps.querier.query_all_validators()?;
    let bounds = (config_r(&deps.storage).load()?).validator_bounds;
    let candidates = vec![];

    for validator in validators {
        if is_validator_inbounds(&validator, &bounds) {
            candidates.push(validator);
        }
    }

    let choice = random();

    candidates[choice % candidates.length()]
}

pub fn is_validator_inbounds(
    validator: Validator,
    bounds: ValidatorBounds,
) -> bool {

    validator.commission <= bounds.max_commission && validator.commission >= bounds.min_commission
}
