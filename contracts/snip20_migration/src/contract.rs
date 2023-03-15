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
    snip20::helpers::{mint_msg, register_receive, send_msg},
    snip20_migration::{AmountMinted, ExecuteAnswer, RegisteredToken},
    utils::{
        asset::Contract,
        generic_response::ResponseStatus,
        pad_handle_result,
        pad_query_result,
        storage::plus::{ItemStorage, MapStorage},
    },
};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = Config { admin: msg.admin };
    state.save(deps.storage)?;

    match msg.tokens {
        Some(tokens) => Ok(Response::default().add_message(register_tokens(deps, tokens)?)),
        None => Ok(Response::default()),
    }
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { admin, .. } => {
            let mut config = Config::load(deps.storage)?;
            validate_admin(
                &deps.querier,
                AdminPermissions::Snip20MigrationAdmin,
                info.sender.to_string(),
                &config.admin,
            )?;
            config.admin = admin.into_valid(deps.api)?;
            config.save(deps.storage)?;
            Ok(
                Response::default().set_data(to_binary(&ExecuteAnswer::SetConfig {
                    status: ResponseStatus::Success,
                    config,
                })?),
            )
        }

        ExecuteMsg::Receive { from, amount, .. } => {
            let from_addr = deps.api.addr_validate(&from)?;
            try_burn_and_mint(deps, &env, info, from_addr, amount)
        }
        ExecuteMsg::RegisterMigrationTokens {
            burn_token,
            mint_token,
            ..
        } => {
            let tokens = RegisteredToken {
                burn_token: burn_token.into_valid(deps.api)?,
                mint_token: mint_token.into_valid(deps.api)?,
            };
            Ok(Response::default().add_message(register_tokens(deps, tokens)?))
        }
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&QueryAnswer::Config {
            config: Config::load(deps.storage)?,
        }),
        QueryMsg::Metrics { token } => to_binary(&QueryAnswer::Metrics {
            amount_minted: match AmountMinted::may_load(deps.storage, token.to_string())? {
                Some(minted_amount) => Some(minted_amount.0),
                None => None,
            },
        }),
        QueryMsg::RegistragionStatus { token } => to_binary(&QueryAnswer::RegistrationStatus {
            status: RegisteredToken::may_load(deps.storage, token.to_string())?,
        }),
    }
}

pub fn try_burn_and_mint(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    from: Addr,
    burn_amount: Uint128,
) -> StdResult<Response> {
    let registered_token = RegisteredToken::load(deps.storage, info.sender.clone().to_string())?;

    let metrics = AmountMinted::load(
        deps.storage,
        registered_token.mint_token.address.clone().to_string(),
    )?;
    AmountMinted(metrics.0 + burn_amount).save(
        deps.storage,
        registered_token.mint_token.clone().address.to_string(),
    )?;

    Ok(Response::default().add_message(mint_msg(
        from.clone(),
        burn_amount,
        None,
        None,
        &registered_token.mint_token,
    )?))
}

pub fn register_tokens(deps: DepsMut, tokens: RegisteredToken) -> StdResult<CosmosMsg> {
    tokens.save(deps.storage, tokens.clone().burn_token.address.to_string())?;
    AmountMinted(Uint128::zero())
        .save(deps.storage, tokens.clone().mint_token.address.to_string())?;
    let msg = register_receive(
        tokens.burn_token.clone().code_hash,
        None,
        &tokens.burn_token,
    )?;
    StdResult::Ok(msg)
}
