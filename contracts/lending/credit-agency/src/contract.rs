#[cfg(not(feature = "library"))]
use shade_protocol::{
    c_std::{
        from_binary, shd_entry_point, to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env,
        MessageInfo, Reply, Response, StdError, StdResult, SubMsg, SubMsgResult,
    },
    contract_interfaces::{
        credit_agency::{ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg, UserDataResponse},
        lend_market::{ExecuteMsg as MarketExecuteMsg, InstantiateMsg as MarketInstantiateMsg},
        snip20::{ExecuteMsg as Snip20ExecuteMsg, Snip20ReceiveMsg},
    },
    lending_utils::{token::Token, Authentication, ViewingKey},
    utils::{asset::Contract, InstantiateCallback, Query},
};

use crate::{
    error::ContractError,
    state::{
        find_value, insert_or_update, Config, MarketState, CONFIG, INIT_MARKET, MARKETS,
        MARKET_VIEWING_KEY,
    },
};

const INIT_MARKET_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MarketInstantiateMsg,
) -> Result<Response, ContractError> {
    let market_init = msg.to_cosmos_msg(
        // TODO needs more complexitr
        format!("market_contract_{}", env.block.time),
        msg.lend_market_id.clone(),
        msg.lend_market_code_hash.clone(),
        vec![],
    )?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_submessage(SubMsg::reply_on_success(market_init, INIT_MARKET_REPLY_ID))
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        CreateMarket(market_cfg) => execute::create_market(deps, env, info, market_cfg),
        _ => panic!("only create_market implemented"),
    }
}

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    Ok(Response::default()
        .add_attribute_plaintext("what", "market reply to credit agency")
        .add_attribute_plaintext("reply id", msg.id.to_string()))
}

mod execute {
    use super::*;

    use shade_protocol::c_std::SubMsg;
    use shade_protocol::{
        credit_agency::{MarketConfig, ReceiveMsg},
        lend_market::{
            ExecuteMsg as MarketExecuteMsg, InstantiateMsg as MarketInstantiateMsg,
            QueryMsg as MarketQueryMsg, ReceiveMsg as MarketReceiveMsg,
        },
        utils::{InstantiateCallback, Query},
    };

    use crate::state::{MarketState, ENTERED_MARKETS, INIT_MARKET};
    use lend_market::state::Config as MarketConfiguration;

    pub fn create_market(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        market_cfg: MarketInstantiateMsg,
    ) -> Result<Response, ContractError> {
        // let cfg = CONFIG.load(deps.storage)?;

        let market_init = market_cfg.to_cosmos_msg(
            // TODO needs more complexitr
            format!("market_contract_{}", env.block.time),
            market_cfg.lend_market_id.clone(),
            market_cfg.lend_market_code_hash.clone(),
            vec![],
        )?;

        Ok(Response::new()
            .add_attribute("action", "create_market")
            .add_attribute("sender", info.sender)
            // .add_message(market_init))
            .add_submessage(SubMsg::reply_on_success(market_init, INIT_MARKET_REPLY_ID)))
    }
}

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    panic!("no queries implemented");
}
