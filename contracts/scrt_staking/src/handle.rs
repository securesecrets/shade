use shade_protocol::c_std::{
    debug_print,
    to_binary,
    Api,
    BalanceResponse,
    BankQuery,
    Binary,
    Coin,
    CosmosMsg,
    Env,
    Extern,
    Response,
    Addr,
    Querier,
    StakingMsg,
    StdError,
    StdResult,
    Storage,
    Uint128,
    Validator,
};

use shade_protocol::snip20::helpers::{deposit_msg, redeem_msg};

use shade_protocol::{
    contract_interfaces::dao::{
        adapter,
        scrt_staking::{Config, HandleAnswer, ValidatorBounds},
        treasury::Flag,
    },
    utils::{
        asset::{scrt_balance, Contract},
        generic_response::ResponseStatus,
        wrap::{unwrap, wrap_and_send},
    },
};

use crate::{
    query,
    state::{config_r, config_w, self_address_r, unbonding_r, unbonding_w},
};

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: Addr,
    _from: Addr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<Response> {
    debug_print!("Received {}", amount);

    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.sscrt.address {
        return Err(StdError::generic_err("Only accepts sSCRT"));
    }

    let validator = choose_validator(&deps, env.block.time)?;

    Ok(Response {
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
    config: Config,
) -> StdResult<Response> {
    let cur_config = config_r(&deps.storage).load()?;

    if cur_config.admins.contains(&env.message.sender) {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    config_w(&mut deps.storage).save(&config)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

/* Claim rewards and restake, hold enough for pending unbondings
 * Send reserves unbonded funds to treasury
 */
pub fn update<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: Addr,
) -> StdResult<Response> {
    let mut messages = vec![];

    let config = config_r(&deps.storage).load()?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    let scrt_balance = scrt_balance(deps, self_address_r(&deps.storage).load()?)?;

    // Claim Rewards
    let rewards = query::rewards(&deps)?;
    if rewards >= Uint128::zero() {
        messages.append(&mut withdraw_rewards(deps)?);
    }

    let mut stake_amount = rewards + scrt_balance;
    let unbonding = unbonding_r(&deps.storage).load()?;

    // Don't restake funds that unbonded
    if unbonding < stake_amount {
        stake_amount = (stake_amount - unbonding)?;
    } else {
        stake_amount = Uint128::zero();
    }

    if stake_amount > Uint128::zero() {
        let validator = choose_validator(&deps, env.block.time)?;
        messages.push(CosmosMsg::Staking(StakingMsg::Delegate {
            validator: validator.address.clone(),
            amount: Coin {
                amount: stake_amount,
                denom: "uscrt".to_string(),
            },
        }));
    }

    Ok(Response {
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
    asset: Addr,
    amount: Uint128,
) -> StdResult<Response> {
    /* Unbonding to the scrt staking contract
     * Once scrt is on balance sheet, treasury can claim
     * and this contract will take all scrt->sscrt and send
     */

    let config = config_r(&deps.storage).load()?;

    //TODO: needs treasury & manager as admin, maybe just manager?
    /*
    if env.message.sender != config.admin && env.message.sender != config.treasury {
        return Err(StdError::Unauthorized { backtrace: None });
    }
    */
    if !config.admins.contains(&env.message.sender) || config.owner != env.message.sender {
        return Err(StdError::unauthorized());
    }

    if asset != config.sscrt.address {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    let self_address = self_address_r(&deps.storage).load()?;
    let delegations = query::delegations(&deps)?;

    let delegated = Uint128::new(
        delegations
            .iter()
            .map(|d| d.amount.amount.u128())
            .sum::<u128>(),
    );
    let scrt_balance = scrt_balance(&deps, self_address)?;
    let rewards = query::rewards(deps)?;


    let mut messages = vec![];
    let mut undelegated = vec![];

    let mut unbonding = unbonding_r(&deps.storage).load()? + amount;
    let total = scrt_balance + rewards + delegated;
    let mut reserves = scrt_balance + rewards;

    if total < unbonding {
        return Err(StdError::generic_err(
            format!("Total Unbond amount {} greater than delegated {}; rew {}, bal {}",
                    unbonding + amount, delegated, rewards, scrt_balance)
        ));
    }

    // Send full unbonding
    if unbonding < reserves {
        messages.append(&mut wrap_and_send(unbonding, 
                                           config.owner, 
                                           config.sscrt, 
                                           None)?);
        reserves = (reserves - unbonding)?;
        unbonding = Uint128::zero();
    }
    // Send all reserves
    else {
        messages.append(&mut wrap_and_send(reserves, 
                                           config.owner, 
                                           config.sscrt, 
                                           None)?);
        reserves = Uint128::zero();
        unbonding = (unbonding - reserves)?;
    }

    while unbonding > Uint128::zero() {

        // Unbond from largest validator first
        let max_delegation = delegations.iter().max_by_key(|d| {
            if undelegated.contains(&d.validator) {
                Uint128::zero()
            } else {
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
                    || delegation.amount.amount.clone() == Uint128::zero()
                {
                    break;
                }

                // This delegation isn't enough to fully unbond
                if delegation.amount.amount.clone() < unbonding {
                    messages.push(
                        CosmosMsg::Staking(
                            StakingMsg::Undelegate {
                                validator: delegation.validator.clone(),
                                amount: delegation.amount.clone(),
                            }
                        )
                    );
                    unbonding = (unbonding - delegation.amount.amount.clone())?;
                }
                else {
                    messages.push(
                        CosmosMsg::Staking(
                            StakingMsg::Undelegate {
                                validator: delegation.validator.clone(),
                                amount: Coin {
                                    denom: delegation.amount.denom.clone(),
                                    amount: unbonding,
                                }
                            }
                        )
                    );
                    unbonding = Uint128::zero();
                }

                undelegated.push(delegation.validator.clone());
            }
        }
    }

    unbonding_w(&mut deps.storage).save(&unbonding)?;

    Ok(Response {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: unbonding,
        })?),
    })
}

pub fn withdraw_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
) -> StdResult<Vec<CosmosMsg>> {
    let mut messages = vec![];
    let address = self_address_r(&deps.storage).load()?;

    for delegation in deps.querier.query_all_delegations(address.clone())? {
        messages.push(CosmosMsg::Staking(StakingMsg::Withdraw {
            validator: delegation.validator,
            recipient: Some(address.clone()),
        }));
    }

    Ok(messages)
}

pub fn unwrap_and_stake<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    amount: Uint128,
    validator: Validator,
    token: Contract,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![
        // unwrap
        unwrap(amount, token.clone())?,
        // Stake
        CosmosMsg::Staking(StakingMsg::Delegate {
            validator: validator.address.clone(),
            amount: Coin {
                amount,
                denom: "uscrt".to_string(),
            },
        }),
    ])
}

/* Claims completed unbondings, wraps them,
 * and returns them to treasury
 */
pub fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: Addr,
) -> StdResult<Response> {
    let config = config_r(&deps.storage).load()?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    if !config.admins.contains(&env.message.sender) || !(config.owner == env.message.sender) {
        return Err(StdError::unauthorized());
    }

    let mut messages = vec![];

    let unbond_amount = unbonding_r(&deps.storage).load()?;
    let mut claim_amount = Uint128::zero();

    let scrt_balance = scrt_balance(deps, self_address_r(&deps.storage).load()?)?;

    if scrt_balance >= unbond_amount {
        claim_amount = unbond_amount;
    } else {
        // Claim Rewards
        let rewards = query::rewards(&deps)?;

        if rewards >= Uint128::zero() {
            messages.append(&mut withdraw_rewards(deps)?);
        }

        if rewards + scrt_balance >= unbond_amount {
            claim_amount = unbond_amount;
        } else {
            claim_amount = rewards + scrt_balance;
        }
    }

    messages.append(&mut wrap_and_send(
        claim_amount,
        config.owner,
        config.sscrt,
        None,
    )?);

    unbonding_w(&mut deps.storage).update(|u| Ok((u - claim_amount)?))?;

    Ok(Response {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount: claim_amount,
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
