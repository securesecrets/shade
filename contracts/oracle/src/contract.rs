use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, 
    Env, Extern, HandleResponse, InitResponse, 
    Querier, StdResult, StdError, Storage, Uint128,
};
use shade_protocol::{
    oracle::{
        InitMsg, HandleMsg,
        QueryMsg, OracleConfig,
    },
    band::ReferenceData,
};
use crate::{
    state::{
        config_w, config_r,
        hard_coded_r, hard_coded_w,
    },
    query, handle,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = OracleConfig {
        owner: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        band: msg.band,
    };

    config_w(&mut deps.storage).save(&state)?;
    hard_coded_w(&mut deps.storage).save("SHD".as_bytes(), &ReferenceData {
                //11.47 * 10^18
                rate: Uint128(1147 * 10u128.pow(16)),
                last_updated_base: 0,
                last_updated_quote: 0
            })?;
    hard_coded_w(&mut deps.storage).save("SILK".as_bytes(), &ReferenceData {
                //1$
                rate: Uint128(1 * 10u128.pow(18)),
                last_updated_base: 0,
                last_updated_quote: 0
            })?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {

    match msg {
        HandleMsg::UpdateConfig {
            owner,
            band,
        } => handle::try_update_config(deps, env, owner, band),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query::config(deps)?),
        QueryMsg::GetPrice { symbol } => to_binary(&query::get_price(deps, symbol)?),
        QueryMsg::GetPrices { symbols } => to_binary(&query::get_prices(deps, symbols)?),
    }
}
