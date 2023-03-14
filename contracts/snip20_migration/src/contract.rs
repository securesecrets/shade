use shade_protocol::{
    c_std::{
        shd_entry_point,
        to_binary,
        Binary,
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
    snip20_migration::{AmountMinted, RegisteredToken},
    utils::{asset::Contract, pad_handle_result, pad_query_result, storage::plus::ItemStorage},
};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let status = Config { admin: msg.admin };
    state.save(deps.storage)?;

    let mut response = Response::default();

    match msg.tokens {
        Some(tokens) => {
            register_tokens(deps, &mut response, tokens)?;
        }
        None => {}
    }

    Ok(Response::default())
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::SetConfig {} => Ok(Response::default()),
        ExecuteMsg::Recieve {} => Ok(Response::default()),
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

pub fn register_tokens(
    deps: DepsMut,
    response: &mut Response,
    tokens: RegisteredToken,
) -> StdResult<Option<bool>> {
    tokens.save(deps.storage, tokens.mint_token.clone().address)?;
    AmountMinted(Uint128::zero()).save(deps.storage)?;
    response.add_message(register_receive(
        token.butn_token.clone().code_hash,
        None,
        token.burn_token.address,
    ));
    Some(true);
}
