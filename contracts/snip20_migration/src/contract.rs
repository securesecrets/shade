use crate::storage::*;
use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{
        shd_entry_point,
        to_binary,
        Addr,
        Binary,
        CosmosMsg,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Uint128,
    },
    contract_interfaces::snip20_migration::{
        Config,
        ExecuteMsg,
        InstantiateMsg,
        QueryAnswer,
        QueryMsg,
    },
    snip20::helpers::{burn_msg, mint_msg, register_receive},
    snip20_migration::{ExecuteAnswer, RegisteredToken},
    utils::generic_response::ResponseStatus,
};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(deps.storage, &Config { admin: msg.admin })?;

    match msg.tokens {
        Some(tokens) => Ok(Response::default().add_message(register_tokens(deps, env, tokens)?)),
        None => Ok(Response::default()),
    }
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { admin, .. } => {
            let mut config = CONFIG.load(deps.storage)?;
            validate_admin(
                &deps.querier,
                AdminPermissions::Snip20MigrationAdmin,
                info.sender.to_string(),
                &config.admin,
            )?;
            config.admin = admin.into_valid(deps.api)?;
            CONFIG.save(deps.storage, &config)?;
            Ok(
                Response::default().set_data(to_binary(&ExecuteAnswer::SetConfig {
                    status: ResponseStatus::Success,
                    config,
                })?),
            )
        }

        ExecuteMsg::Receive { from, amount, .. } => {
            let from_addr = deps.api.addr_validate(&from)?;
            try_burn_and_mint(deps, info, from_addr, amount)
        }
        ExecuteMsg::RegisterMigrationTokens {
            burn_token,
            mint_token,
            burnable,
            ..
        } => {
            let config = CONFIG.load(deps.storage)?;
            validate_admin(
                &deps.querier,
                AdminPermissions::Snip20MigrationAdmin,
                info.sender.to_string(),
                &config.admin,
            )?;
            let tokens = RegisteredToken {
                burn_token: burn_token.into_valid(deps.api)?,
                mint_token: mint_token.into_valid(deps.api)?,
                burnable,
            };
            Ok(Response::default().add_message(register_tokens(deps, env, tokens)?))
        }
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&QueryAnswer::Config {
            config: CONFIG.load(deps.storage)?,
        }),
        QueryMsg::Metrics { token } => to_binary(&QueryAnswer::Metrics {
            amount_minted: match AMOUNT_MINTED
                .may_load(deps.storage, deps.api.addr_validate(&token)?)?
            {
                Some(minted_amount) => {
                    let mut amount_minted = Uint128::zero();
                    // Round to nearest 10,000
                    if !minted_amount.lt(&Uint128::new(100_000_000_000)) {
                        let rounded_minted_amount =
                            minted_amount.checked_div(Uint128::new(100_000_000_000))?;
                        amount_minted =
                            rounded_minted_amount.checked_mul(Uint128::new(100_000_000_000))?;
                    }
                    Ok(amount_minted)
                }
                None => Err(StdError::generic_err("token not found")),
            }?,
        }),
        QueryMsg::RegistrationStatus { token } => to_binary(&QueryAnswer::RegistrationStatus {
            status: REGISTERD_TOKENS.load(deps.storage, deps.api.addr_validate(&token)?)?,
        }),
    }
}

pub fn try_burn_and_mint(
    deps: DepsMut,
    info: MessageInfo,
    from: Addr,
    burn_amount: Uint128,
) -> StdResult<Response> {
    let mut msgs = vec![];

    let registered_token = REGISTERD_TOKENS.load(deps.storage, info.sender.clone())?;

    match registered_token.burnable {
        Some(burnable) => {
            if burnable {
                msgs.push(burn_msg(
                    burn_amount.clone(),
                    None,
                    None,
                    &registered_token.burn_token,
                )?);
            }
        }
        None => {}
    }

    msgs.push(mint_msg(
        from.clone(),
        burn_amount,
        None,
        None,
        &registered_token.mint_token,
    )?);

    let metrics = AMOUNT_MINTED.load(deps.storage, registered_token.mint_token.address.clone())?;
    AMOUNT_MINTED.save(
        deps.storage,
        registered_token.mint_token.clone().address,
        &(metrics + burn_amount),
    )?;

    Ok(Response::default().add_messages(msgs))
}

pub fn register_tokens(deps: DepsMut, env: Env, tokens: RegisteredToken) -> StdResult<CosmosMsg> {
    REGISTERD_TOKENS.save(deps.storage, tokens.clone().burn_token.address, &tokens)?;
    AMOUNT_MINTED.save(
        deps.storage,
        tokens.clone().mint_token.address,
        &Uint128::zero(),
    )?;
    let msg = register_receive(env.contract.code_hash, None, &tokens.burn_token)?;
    StdResult::Ok(msg)
}
