use cosmwasm_std::{
    debug_print, to_binary, Api, Binary,
    Env, Extern, Storage, HandleResponse,
    StdResult, StdError,
    CosmosMsg, Uint128,
    Coin, StakingMsg,
    Validator, Querier, HumanAddr,
    BankQuery, BalanceResponse,
};

use secret_toolkit::{
    snip20::{
        redeem_msg, deposit_msg,
        send_msg,
    },
};

use shade_protocol::{
    treasury::Flag,
    scrt_staking::{
        HandleAnswer,
        ValidatorBounds,
    },
    generic_response::ResponseStatus,
};

use crate::{
    query,
    state::{
        config_w, config_r, 
        self_address_r,
    },
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

    let config = config_r(&deps.storage).load()?;

    if config.sscrt.address != env.message.sender {
        return Err(StdError::GenericErr { 
            msg: "Only accepts sSCRT".to_string(), 
            backtrace: None 
        });
    }

    let validator = choose_validator(&deps, env.block.time)?;

    Ok(HandleResponse {
        messages: vec![
            redeem_msg(
                amount,
                None,
                None,
                256,
                config.sscrt.code_hash.clone(),
                config.sscrt.address.clone(),
            )?,
            CosmosMsg::Staking(StakingMsg::Delegate {
                validator: validator.address.clone(),
                amount: Coin {
                    amount,
                    denom: "uscrt".to_string(),
                }
            }),
        ],
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

    /* Unbonding to the scrt staking contract
     * Once scrt is on balance sheet, treasury can claim
     * and this contract will take all scrt->sscrt and send
     */

    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.admin && env.message.sender != config.treasury {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    for delegation in deps.querier.query_all_delegations(self_address_r(&deps.storage).load()?)? {
        if delegation.validator == validator {
            return Ok(HandleResponse {
                messages: vec![
                    CosmosMsg::Staking(StakingMsg::Undelegate {
                        validator,
                        amount: delegation.amount.clone(),
                    }),
                ],
                log: vec![],
                data: Some( to_binary( 
                    &HandleAnswer::Unbond {
                        status: ResponseStatus::Success,
                        delegation,
                    }
                )?),
            });
        }
    }

    /*
    if let Some(delegation) = deps.querier.query_delegation(env.contract.address, validator.clone())? {

        return Ok(HandleResponse {
            messages: vec![
                CosmosMsg::Staking(StakingMsg::Undelegate {
                    validator,
                    amount: delegation.amount.clone(),
                }),
            ],
            log: vec![],
            data: Some( to_binary( 
                &HandleAnswer::Unbond {
                    status: ResponseStatus::Success,
                    delegation,
                }
            )?),
        });
    }
    */

    Err(StdError::GenericErr { 
        msg: "No delegation to given validator".to_string(),
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
    _env: Env,
    validator: HumanAddr,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    //TODO: query scrt balance and deposit into sscrt

    let mut messages = vec![];
    let address = self_address_r(&deps.storage).load()?;

    // Get total scrt balance, to get recently claimed rewards + lingering unbonded scrt
    let scrt_balance: BalanceResponse = deps.querier.query(&BankQuery::Balance {
        address: address.clone(),
        denom: "uscrt".to_string(),
    }.into())?;

    let amount = query::rewards(&deps)? + scrt_balance.amount.amount;

    messages.push(CosmosMsg::Staking(StakingMsg::Withdraw {
        validator,
        recipient: Some(address.clone()),
    }));

    messages.push(deposit_msg(
        amount,
        None,
        256,
        config.sscrt.code_hash.clone(),
        config.sscrt.address.clone(),
    )?);

    /* NOTE: This will likely trigger the receive callback which
     *       would result in re-delegating a portion of the funds.
     *       This case will need to be tested and mitigated by either
     *       - accounting for it when rebalancing
     *       - add a "unallocated" flag with funds to force treasury not to 
     *         allocate them, to then be allocated at rebalancing
     */
    messages.push(send_msg(
        config.treasury,
        amount,
        Some(to_binary(&Flag { flag: "unallocated".to_string()})?),
        None,
        1,
        config.sscrt.code_hash.clone(),
        config.sscrt.address.clone(),
    )?);

    Ok(HandleResponse {
        messages,
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
