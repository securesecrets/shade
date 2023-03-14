use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{
        shd_entry_point,
        to_binary,
        Binary,
        CosmosMsg,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdResult,
        Uint128,
    },
    contract_interfaces::snip20_migration::{Config, ExecuteMsg, InstantiateMsg, QueryMsg},
    snip20::helpers::register_receive,
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
            config.admin = admin;
            config.save(deps.storage)?;
            Ok(
                Response::default().set_data(to_binary(&ExecuteAnswer::SetConfig {
                    status: ResponseStatus::Success,
                    config,
                })?),
            )
        }

        ExecuteMsg::Receive { .. } => Ok(Response::default()),
        ExecuteMsg::RegisterMigrationTokens { .. } => Ok(Response::default()),
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&{}),
        QueryMsg::Metrics { token } => to_binary(&{}),
        QueryMsg::RegistragionStatus { token } => to_binary(&{}),
    }
}

pub fn register_tokens(deps: DepsMut, tokens: RegisteredToken) -> StdResult<CosmosMsg> {
    tokens.save(deps.storage, tokens.clone().mint_token.address.to_string())?;
    AmountMinted(Uint128::zero())
        .save(deps.storage, tokens.clone().mint_token.address.to_string())?;
    let msg = register_receive(
        tokens.burn_token.clone().code_hash,
        None,
        &tokens.burn_token,
    )?;
    StdResult::Ok(msg)
}
