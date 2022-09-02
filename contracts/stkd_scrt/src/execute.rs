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
        stkd_scrt::{staking_derivatives, Config, ExecuteAnswer, ValidatorBounds},
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
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.sscrt.address {
        return Err(StdError::generic_err("Only accepts sSCRT"));
    }

    Ok(Response::new()
        .add_message(unwrap(amount, config.sscrt.clone())?)
        // Stake
        .add_message(staking_derivatives::ExecuteMsg::Stake {}.to_cosmos_msg(
            config.staking_derivatives,
            vec![Coin {
                amount,
                denom: "uscrt".to_string(),
            }],
        )?)
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

    Ok(Response::new()
        .add_messages()
        .set_data(to_binary(&adapter::ExecuteAnswer::Update {
            status: ResponseStatus::Success,
        })?))
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
        .add_message(
            staking_derivatives::ExecuteMsg::Unbond {
                redeem_amount: amount,
            }
            .to_cosmos_msg(&config.staking_derivatives, vec![])?,
        )
        .set_data(to_binary(&adapter::ExecuteAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: unbonding,
        })?))
}

pub fn unwrap_and_stake(
    _deps: DepsMut,
    amount: Uint128,
    staking_derivatives: Contract,
    token: Contract,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![
        // unwrap
        unwrap(amount, token.clone())?,
        // Stake
        staking_derivatives::ExecuteMsg::Stake {}.to_cosmos_msg(
            config.staking_derivatives,
            vec![Coin {
                amount,
                denom: "uscrt".to_string(),
            }],
        )?,
    ])
}

/* Claims completed unbondings, wraps them,
 * and returns them to treasury
 */
pub fn claim(deps: DepsMut, _env: Env, _info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    Ok(Response::new()
        .add_message(
            staking_derivatives::ExecuteMsg::Claim {}
                .to_cosmos_msg(&config.staking_derivatives, vec![])?,
        )
        .add_messages(wrap_and_send(
            claim_amount,
            config.owner,
            config.sscrt,
            None,
        )?)
        .set_data(to_binary(&adapter::ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
            amount: claim_amount,
        })?))
}
