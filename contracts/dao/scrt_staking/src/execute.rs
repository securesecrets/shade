use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{
        to_binary,
        Addr,
        Binary,
        Coin,
        CosmosMsg,
        Deps,
        DepsMut,
        DistributionMsg,
        Env,
        MessageInfo,
        Response,
        StakingMsg,
        StdError,
        StdResult,
        Uint128,
        Validator,
    },
};

use shade_protocol::snip20::helpers::redeem_msg;

use shade_protocol::{
    dao::{
        adapter,
        scrt_staking::{Config, ExecuteAnswer, ValidatorBounds},
    },
    utils::{
        asset::{scrt_balance, Contract},
        generic_response::ResponseStatus,
        wrap::{unwrap, wrap_and_send},
    },
};

use crate::{
    query,
    storage::{CONFIG, SELF_ADDRESS, UNBONDING},
};

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    _from: Addr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<Response> {
    //panic!("scrt staking Received {}", amount);

    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.sscrt.address {
        return Err(StdError::generic_err("Only accepts sSCRT"));
    }

    let validator = choose_validator(deps, env.block.time.seconds())?;

    Ok(Response::new()
        .add_messages(vec![
            redeem_msg(amount, None, None, &config.sscrt)?,
            CosmosMsg::Staking(StakingMsg::Delegate {
                validator: validator.address.clone(),
                amount: Coin {
                    amount,
                    denom: "uscrt".to_string(),
                },
            }),
        ])
        .set_data(to_binary(&ExecuteAnswer::Receive {
            status: ResponseStatus::Success,
            validator,
        })?))
}

pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    config: Config,
) -> StdResult<Response> {
    let cur_config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::ScrtStakingAdmin,
        &info.sender,
        &cur_config.admin_auth,
    )?;

    // Save new info
    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    )
}

/* Claim rewards and restake, hold enough for pending unbondings
 * Send reserves unbonded funds to treasury
 */
pub fn update(deps: DepsMut, env: Env, _info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let mut messages = vec![];
    //let asset = deps.api.addr_validate(asset.as_str())?;

    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    let scrt_balance = scrt_balance(deps.as_ref(), SELF_ADDRESS.load(deps.storage)?)?;

    // Claim Rewards
    let rewards = query::rewards(deps.as_ref())?;
    if !rewards.is_zero() {
        messages.append(&mut withdraw_rewards(deps.as_ref())?);
    }

    let mut stake_amount = rewards + scrt_balance;
    let unbonding = UNBONDING.load(deps.storage)?;

    // Don't restake funds that unbonded
    if unbonding < stake_amount {
        stake_amount = stake_amount - unbonding;
    } else {
        stake_amount = Uint128::zero();
    }

    if stake_amount > Uint128::zero() {
        let validator = choose_validator(deps, env.block.time.seconds())?;
        messages.push(CosmosMsg::Staking(StakingMsg::Delegate {
            validator: validator.address.clone(),
            amount: Coin {
                amount: stake_amount,
                denom: "uscrt".to_string(),
            },
        }));
    }

    Ok(Response::new().add_messages(messages).set_data(to_binary(
        &adapter::ExecuteAnswer::Update {
            status: ResponseStatus::Success,
        },
    )?))
}

pub fn unbond(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    asset: Addr,
    amount: Uint128,
) -> StdResult<Response> {
    /* Unbonding to the scrt staking contract
     * Once scrt is on balance sheet, treasury can claim
     * and this contract will take all scrt->sscrt and send
     */

    //let asset = deps.api.addr_validate(asset.as_str())?;
    let config = CONFIG.load(deps.storage)?;

    if validate_admin(
        &deps.querier,
        AdminPermissions::ScrtStakingAdmin,
        &info.sender,
        &config.admin_auth,
    )
    .is_err()
        && config.owner != info.sender
    {
        return Err(StdError::generic_err("Unauthorized"));
    }

    if asset != config.sscrt.address {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    let self_address = SELF_ADDRESS.load(deps.storage)?;
    let delegations = query::delegations(deps.as_ref())?;

    let delegated = Uint128::new(
        delegations
            .iter()
            .map(|d| d.amount.amount.u128())
            .sum::<u128>(),
    );
    let scrt_balance = scrt_balance(deps.as_ref(), self_address)?;
    let rewards = query::rewards(deps.as_ref())?;

    let mut messages = vec![];

    if !rewards.is_zero() {
        messages.append(&mut withdraw_rewards(deps.as_ref())?);
    }

    let mut undelegated = vec![];

    let prev_unbonding = UNBONDING.load(deps.storage)?;

    let mut total = delegated + scrt_balance + rewards + delegated;
    total -= prev_unbonding;

    if total - prev_unbonding < amount {
        return Err(StdError::generic_err(format!(
            "Total Unbond amount {} greater than delegated {}; rew {}, bal {}",
            amount, delegated, rewards, scrt_balance
        )));
    }

    let mut unbonding = amount;
    let mut total_unbonding = amount + prev_unbonding;
    let reserves = scrt_balance + rewards;

    if total_unbonding.is_zero() {
        return Ok(
            Response::new().set_data(to_binary(&adapter::ExecuteAnswer::Unbond {
                status: ResponseStatus::Success,
                amount: unbonding,
            })?),
        );
    }

    // Send full unbonding
    if total_unbonding <= reserves {
        messages.append(&mut wrap_and_send(
            unbonding,
            config.owner,
            config.sscrt,
            None,
        )?);
        total_unbonding = Uint128::zero();
    }
    // Send all reserves
    else if !reserves.is_zero() {
        messages.append(&mut wrap_and_send(
            reserves,
            config.owner,
            config.sscrt,
            None,
        )?);
        total_unbonding -= reserves;
    }

    UNBONDING.save(deps.storage, &total_unbonding)?;

    while !total_unbonding.is_zero() {
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
                if delegation.amount.amount.clone() < unbonding
                    && !delegation.amount.amount.clone().is_zero()
                {
                    messages.push(CosmosMsg::Staking(StakingMsg::Undelegate {
                        validator: delegation.validator.clone(),
                        amount: delegation.amount.clone(),
                    }));
                    unbonding = unbonding - delegation.amount.amount.clone();
                    undelegated.push(delegation.validator.clone());
                } else if !delegation.amount.amount.clone().is_zero() {
                    messages.push(CosmosMsg::Staking(StakingMsg::Undelegate {
                        validator: delegation.validator.clone(),
                        amount: Coin {
                            denom: delegation.amount.denom.clone(),
                            amount: unbonding,
                        },
                    }));
                    unbonding = Uint128::zero();
                    undelegated.push(delegation.validator.clone());
                }
            }
        }
    }

    Ok(Response::new().add_messages(messages).set_data(to_binary(
        &adapter::ExecuteAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: unbonding,
        },
    )?))
}

pub fn withdraw_rewards(deps: Deps) -> StdResult<Vec<CosmosMsg>> {
    let mut messages = vec![];
    let address = SELF_ADDRESS.load(deps.storage)?;

    for delegation in deps.querier.query_all_delegations(address.clone())? {
        messages.push(CosmosMsg::Distribution(
            DistributionMsg::WithdrawDelegatorReward {
                validator: delegation.validator,
            },
        ));
    }

    Ok(messages)
}

pub fn unwrap_and_stake(
    _deps: DepsMut,
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
pub fn claim(deps: DepsMut, _env: Env, _info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    //let asset = deps.api.addr_validate(asset.as_str())?;
    if asset != config.sscrt.address {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    let mut messages = vec![];

    let unbond_amount = UNBONDING.load(deps.storage)?;
    let claim_amount;

    let scrt_balance = scrt_balance(deps.as_ref(), SELF_ADDRESS.load(deps.storage)?)?;

    if scrt_balance >= unbond_amount {
        claim_amount = unbond_amount;
    } else {
        // Claim Rewards
        let rewards = query::rewards(deps.as_ref())?;

        if !rewards.is_zero() {
            //assert!(false, "withdraw rewards");
            messages.append(&mut withdraw_rewards(deps.as_ref())?);
        }

        if rewards + scrt_balance >= unbond_amount {
            claim_amount = unbond_amount;
        } else {
            claim_amount = rewards + scrt_balance;
        }
    }

    if !claim_amount.is_zero() {
        messages.append(&mut wrap_and_send(
            claim_amount,
            config.owner,
            config.sscrt,
            None,
        )?);

        //assert!(false, "u - claim_amount: {} - {}", unbond_amount, claim_amount);
        let u = UNBONDING.load(deps.storage)?;
        UNBONDING.save(deps.storage, &(u - claim_amount))?;
    }

    Ok(Response::new().add_messages(messages).set_data(to_binary(
        &adapter::ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
            amount: claim_amount,
        },
    )?))
}

pub fn choose_validator(deps: DepsMut, seed: u64) -> StdResult<Validator> {
    let mut validators = deps.querier.query_all_validators()?;

    // filter down to viable candidates
    if let Some(bounds) = (CONFIG.load(deps.storage)?).validator_bounds {
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

    // seed will likely be env.block.time.seconds()
    Ok(validators[(seed % validators.len() as u64) as usize].clone())
}

pub fn is_validator_inbounds(validator: &Validator, bounds: &ValidatorBounds) -> bool {
    validator.commission <= bounds.max_commission && validator.commission >= bounds.min_commission
}
