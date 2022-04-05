use cosmwasm_std::{
    debug_print, to_binary, Api, BalanceResponse, BankQuery, Binary, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HumanAddr, Querier, StakingMsg, StdError, StdResult, Storage, Uint128,
    Validator,
};

use secret_toolkit::snip20::{deposit_msg, redeem_msg, send_msg};

use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::{
    scrt_staking::{HandleAnswer, ValidatorBounds, Config},
    treasury::Flag,
    adapter,
};

use crate::{
    query,
    state::{
        config_r, config_w,
        self_address_r,
        unbonding_w, unbonding_r,
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

    if env.message.sender != config.sscrt.address  {
        return Err(StdError::generic_err("Only accepts sSCRT"));
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
                },
            }),
        ],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
            validator,
        })?),
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
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

/* Claim rewards and restake, hold enough for pending unbondings
 * Send available unbonded funds to treasury
 */
pub fn update<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {

    let mut messages = vec![];

    let config = config_r(&deps.storage).load()?;
    let unbonding = unbonding_r(&deps.storage).load()?;

    let scrt_balance = scrt_balance(deps)?;

    // Claim Rewards
    let rewards = query::rewards(&deps)?;
    if rewards >= Uint128::zero() {
        messages.append(&mut withdraw_rewards(deps)?);
    }

    let total = rewards + scrt_balance;

    let mut restake_amount = Uint128::zero();
    let mut unbond_amount = Uint128::zero();

    if total > unbonding {
        restake_amount = (total - unbonding)?;
        unbond_amount = unbonding;
    }
    else {
        unbond_amount = (unbonding - total)?;
    }

    unbonding_w(&mut deps.storage).update(
        |u| Ok((u - unbond_amount)?)
    )?;

    messages.append(&mut wrap_and_send(deps, unbond_amount, &config)?);

    if restake_amount > Uint128::zero() {
        let validator = choose_validator(&deps, env.block.time)?;

        messages.append(
            &mut vec![
                // wrap
                redeem_msg(
                    restake_amount,
                    None,
                    None,
                    256,
                    config.sscrt.code_hash.clone(),
                    config.sscrt.address.clone(),
                )?,
                // Stake
                CosmosMsg::Staking(StakingMsg::Delegate {
                    validator: validator.address.clone(),
                    amount: Coin {
                        amount: restake_amount,
                        denom: "uscrt".to_string(),
                    },
                }),
            ]
        );
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?),
    })

}

pub fn unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    /* Unbonding to the scrt staking contract
     * Once scrt is on balance sheet, treasury can claim
     * and this contract will take all scrt->sscrt and send
     */

    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.admin && env.message.sender != config.treasury {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    unbonding_w(&mut deps.storage).update(|u| Ok(u + amount))?;

    let mut messages = vec![];

    let delegations = deps.querier
        .query_all_delegations(self_address_r(&deps.storage).load()?)?;

    let mut unbond_amount = amount;
    let mut undelegated = vec![];

    while unbond_amount > Uint128::zero() {

        // Unbond from largest validator first
        // TODO: Continue to next highest until full amount requested
        let max_delegation = delegations.iter().max_by_key(|d| {
            if undelegated.contains(&d.validator) {
                Uint128::zero()
            }
            else {
                d.amount.amount
            }
        });

        // No more delegated funds to unbond
        match max_delegation {
            None => {
                break;
            }
            Some(delegation) => {

                if undelegated.contains(&delegation.validator)
                    || delegation.amount.amount.clone() == Uint128::zero() {
                    break;
                }

                // This delegation isn't enough to fully unbond
                if delegation.amount.amount.clone() < unbond_amount {
                    messages.push(
                        CosmosMsg::Staking(
                            StakingMsg::Undelegate {
                                validator: delegation.validator.clone(),
                                amount: delegation.amount.clone(),
                            }
                        )
                    );
                    unbond_amount = (unbond_amount - delegation.amount.amount.clone())?;
                }
                // Can fully unbond
                else {
                    messages.push(
                        CosmosMsg::Staking(
                            StakingMsg::Undelegate {
                                validator: delegation.validator.clone(),
                                amount: Coin {
                                    denom: delegation.amount.denom.clone(),
                                    amount: unbond_amount,
                                }
                            }
                        )
                    );
                    unbond_amount = Uint128::zero();
                }

                undelegated.push(delegation.validator.clone());
            }
        }
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Unbond {
            status: ResponseStatus::Success,
            delegations: undelegated,
        })?),
    })
}

pub fn scrt_balance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
) -> StdResult<Uint128> {

    let resp: BalanceResponse = deps.querier.query(
        &BankQuery::Balance {
            address: self_address_r(&deps.storage).load()?,
            denom: "uscrt".to_string(),
        }
        .into(),
    )?;

    Ok(resp.amount.amount)
}

pub fn withdraw_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
) -> StdResult<Vec<CosmosMsg>> {

    let mut messages = vec![];
    let address = self_address_r(&deps.storage).load()?;

    for delegation in deps.querier.query_all_delegations(address.clone())? {
        messages.push(
            CosmosMsg::Staking(
                StakingMsg::Withdraw {
                    validator: delegation.validator,
                    recipient: Some(address.clone()),
                }
            )
        );
    }

    Ok(messages)
}

pub fn wrap_and_send<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    amount: Uint128,
    config: &Config,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(
        vec![
            deposit_msg(
                amount,
                None,
                256,
                config.sscrt.code_hash.clone(),
                config.sscrt.address.clone(),
            )?,
            send_msg(
                config.treasury.clone(),
                amount,
                Some(to_binary(
                    &Flag {
                        flag: "unallocated".to_string(),
                    }
                )?),
                None,
                None,
                1,
                config.sscrt.code_hash.clone(),
                config.sscrt.address.clone(),
            )?
        ]
    )
}

/*
 * Claims completed unbondings, wraps them, 
 * and returns them to treasury
 */
pub fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    let mut messages = vec![];
    let address = self_address_r(&deps.storage).load()?;

    let unbond_amount = unbonding_r(&deps.storage).load()?;
    let mut claim_amount = Uint128::zero();

    let scrt_balance = scrt_balance(deps)?;

    if scrt_balance >= unbond_amount {
        let claim_amount = unbond_amount;
    }

    // need to claim some rewards first
    else {
        let rewards = query::rewards(&deps)?;

        if rewards >= Uint128::zero() {
            messages.append(&mut withdraw_rewards(deps)?);
        }

        if rewards + scrt_balance >= unbond_amount {
            claim_amount = unbond_amount;
        }
        else {
            claim_amount = rewards + scrt_balance;
        }
    }

    unbonding_w(&mut deps.storage).update(|u| Ok((u - claim_amount)?))?;

    messages.append(&mut wrap_and_send(deps, claim_amount, &config)?);

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Claim {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn choose_validator<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    seed: u64,
) -> StdResult<Validator> {

    let mut validators = deps.querier.query_validators()?;

    // filter down to viable candidates
    if let Some(bounds) = (config_r(&deps.storage).load()?).validator_bounds {

        let mut candidates = vec![];

        for validator in validators {

            if is_validator_inbounds(&validator, &bounds) {
                candidates.push(validator);
            }
        }

        validators = candidates;
    }

    if validators.is_empty() {
        return Err(StdError::generic_err("No validators within bounds"));
    }

    // seed will likely be env.block.time
    Ok(validators[(seed % validators.len() as u64) as usize].clone())
}

pub fn is_validator_inbounds(validator: &Validator, bounds: &ValidatorBounds) -> bool {
    validator.commission <= bounds.max_commission && validator.commission >= bounds.min_commission
}
