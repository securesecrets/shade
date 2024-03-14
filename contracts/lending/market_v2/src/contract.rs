#[cfg(not(feature = "library"))]
use shade_protocol::c_std::shd_entry_point;
use shade_protocol::{
    c_std::{
        from_binary, to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Reply,
        Response, StdError, StdResult, SubMsg, Timestamp, Uint128, WasmMsg,
    },
    contract_interfaces::{
        oracles::{band::ReferenceData, oracle::QueryMsg::Price},
        query_auth::helpers::{authenticate_permit, authenticate_vk, PermitAuthentication},
        snip20::Snip20ReceiveMsg,
    },
    lend_market::{ExecuteMsg, InstantiateMsg, QueryMsg},
    lending_utils::{token::Token, Authentication, ViewingKey},
    utils::{asset::Contract, InstantiateCallback, Query},
};

use crate::{
    error::ContractError,
    msg::{AuthPermit, ReceiveMsg, TotalDebtResponse},
    state::{debt, Config, CONFIG, VIEWING_KEY},
};

const CTOKEN_INIT_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /*
    let ctoken_init = lend_token::msg::InstantiateMsg {
        name: "Lent ".to_owned() + &msg.name,
        symbol: "L".to_owned() + &msg.symbol,
        decimals: msg.decimals,
        controller: env.contract.clone().into(),
        distributed_token: msg.distributed_token.as_contract_info().unwrap().into(),
        viewing_key: msg.viewing_key.clone(),
        query_auth: msg.query_auth.clone().into(),
    }
    .to_cosmos_msg(
        format!("ctoken_contract_{}", env.contract.address),
        msg.ctoken_id,
        msg.ctoken_code_hash.clone(),
        vec![],
    )?;
    */

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
    // .add_message(ctoken_init))
    // .add_submessage(SubMsg::reply_on_success(ctoken_init, CTOKEN_INIT_REPLY_ID)))
}

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    Ok(Response::default()
        .add_attribute_plaintext("what", "ctoken reply to market")
        .add_attribute_plaintext("reply id", msg.id.to_string()))
}

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    panic!("no queries implemented");
}
