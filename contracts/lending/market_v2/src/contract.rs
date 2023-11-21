#[cfg(not(feature = "library"))]
use shade_protocol::c_std::entry_point;
use shade_protocol::{
    c_std::{
        to_binary, Addr, Binary, Coin as StdCoin, Decimal, Deps, DepsMut, Env, MessageInfo, Reply,
        Response, StdError, StdResult, SubMsg, Timestamp, Uint128, WasmMsg,
    },
    query_authentication::viewing_keys,
    utils::Query,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, VIEWING_KEY};

use lending_utils::token::Token;

const CTOKEN_INIT_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let ctoken_msg = lend_token::msg::InstantiateMsg {
        name: "Lent ".to_owned() + &msg.name,
        symbol: "L".to_owned() + &msg.symbol,
        decimals: msg.decimals,
        controller: env.contract.clone().into(),
        distributed_token: msg.distributed_token.as_contract_info().unwrap().into(),
        viewing_key: msg.viewing_key.clone(),
    };
    let ctoken_instantiate = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.token_id,
        msg: to_binary(&ctoken_msg)?,
        funds: vec![],
        label: format!("ctoken_contract_{}", env.contract.address),
        code_hash: msg.ctoken_code_hash.clone(),
    };

    let cfg = Config {
        // those will be overwritten in a response
        ctoken_contract: Addr::unchecked(""),
        ctoken_code_hash: msg.ctoken_code_hash,
        governance_contract: deps.api.addr_validate(&msg.gov_contract)?,
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        token_id: msg.token_id,
        market_token: msg.market_token,
        market_cap: msg.market_cap,
        rates: msg.interest_rate.validate()?,
        interest_charge_period: msg.interest_charge_period,
        last_charged: env.block.time.seconds()
            - env.block.time.seconds() % msg.interest_charge_period,
        common_token: msg.common_token,
        collateral_ratio: msg.collateral_ratio,
        price_oracle: msg.price_oracle,
        credit_agency: info.sender.clone(),
        reserve_factor: msg.reserve_factor,
        borrow_limit_ratio: msg.borrow_limit_ratio,
    };
    CONFIG.save(deps.storage, &cfg)?;
    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_submessage(SubMsg::reply_on_success(
            ctoken_instantiate,
            CTOKEN_INIT_REPLY_ID,
        )))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        CTOKEN_INIT_REPLY_ID => reply::token_instantiate_reply(deps, env, msg),
        _ => Err(ContractError::UnrecognisedReply(msg.id)),
    }
}

mod reply {
    use super::*;

    use lending_utils::parse_reply::parse_reply_instantiate_data;

    pub fn token_instantiate_reply(
        deps: DepsMut,
        _env: Env,
        msg: Reply,
    ) -> Result<Response, ContractError> {
        let id = msg.id;
        let res =
            parse_reply_instantiate_data(msg).map_err(|err| ContractError::ReplyParseFailure {
                id,
                err: err.to_string(),
            })?;

        let mut response = Response::new();

        let addr = deps.api.addr_validate(&res.contract_address)?;
        if id == CTOKEN_INIT_REPLY_ID {
            CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
                config.ctoken_contract = addr.clone();
                response = Response::new().add_attribute("ctoken", addr);
                Ok(config)
            })?;
        }

        Ok(response)
    }
}

/// Execution entry point
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;
    Ok(to_binary(&"")?)
}
