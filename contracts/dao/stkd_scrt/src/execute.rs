use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{
        to_binary,
        Addr,
        Binary,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Uint128,
    },
};

use shade_protocol::{
    dao::{
        adapter,
        stkd_scrt::{staking_derivatives, Config, ExecuteAnswer},
    },
    utils::{
        generic_response::ResponseStatus,
        wrap::{unwrap, wrap_and_send},
    },
};

use crate::storage::*;

pub fn receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _sender: Addr,
    _from: Addr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.sscrt.address {
        return Err(StdError::generic_err("Only accepts sSCRT"));
    }

    // Unwrap & stake
    Ok(Response::new()
        .add_message(unwrap(amount, config.sscrt.clone())?)
        .add_message(staking_derivatives::stake_msg(
            amount,
            &config.staking_derivatives,
        )?)
        .set_data(to_binary(&ExecuteAnswer::Receive {
            status: ResponseStatus::Success,
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

    Ok(Response::new()
        .add_message(staking_derivatives::unbond_msg(
            amount,
            &config.staking_derivatives,
        )?)
        .set_data(to_binary(&adapter::ExecuteAnswer::Unbond {
            status: ResponseStatus::Success,
            amount,
        })?))
}

/* Claims completed unbondings, wraps them,
 * and returns them to treasury
 */
pub fn claim(deps: DepsMut, env: Env, _info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    let claimable = staking_derivatives::holdings_query(
        &deps.querier,
        env.contract.address,
        VIEWING_KEY.load(deps.storage)?,
        env.block.time.seconds(),
        &config.staking_derivatives,
    )?
    .claimable_scrt;

    let mut messages = vec![];
    if !claimable.is_zero() {
        messages.push(staking_derivatives::claim_msg(&config.staking_derivatives)?);
        messages.append(&mut wrap_and_send(
            claimable,
            config.owner,
            config.sscrt,
            None,
        )?);
    }

    Ok(Response::new().add_messages(messages).set_data(to_binary(
        &adapter::ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
            amount: claimable,
        },
    )?))
}
