#[cfg(not(feature = "library"))]
use shade_protocol::c_std::shd_entry_point;
use shade_protocol::{
    c_std::{
        to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
        StdResult, Uint128,
    },
    contract_interfaces::snip20::Snip20ReceiveMsg,
    query_auth::helpers::{authenticate_permit, authenticate_vk, PermitAuthentication},
    snip20,
    utils::Query,
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Config, CONFIG, NEXT_REPLY_ID},
};
use lending_utils::token::Token;

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    todo!();
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
        CreateMarket(market_cfg) => todo!(), // execute::create_market(deps, env, info, market_cfg),
        EnterMarket { account } => {
            let account = deps.api.addr_validate(&account)?;
            todo!() //execute::enter_market(deps, info, account)
        }
        ExitMarket { market } => {
            let market = deps.api.addr_validate(&market)?;
            todo!() //execute::exit_market(deps, info, market)
        }
        RepayWithCollateral {
            max_collateral,
            amount_to_repay,
            estimate_multiplier,
        } => todo!(), // execute::repay_with_collateral(
        //     deps,
        //     info.sender,
        //     max_collateral,
        //     amount_to_repay,
        //     estimate_multiplier,
        // ),
        Receive(msg) => todo!(), //execute::receive_snip20_message(deps, env, info, msg),
        AdjustMarketId { new_market_id } => todo!(), //restricted::adjust_market_id(deps, info, new_market_id),
        AdjustTokenId { new_token_id } => todo!(), //restricted::adjust_token_id(deps, info, new_token_id),
        AdjustCommonToken { new_common_token } => {
            todo!() //restricted::adjust_common_token(deps, info, new_common_token)
        }
    }
}

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;

    let res = match msg {
        Configuration {} => to_binary(todo!() /*&CONFIG.load(deps.storage)?*/)?,
        Market { market_token } => {
            to_binary(todo!() /*&query::market(deps, &market_token)?*/)?
        }
        ListMarkets { start_after, limit } => {
            to_binary(
                todo!(), /*&query::list_markets(deps, start_after, limit)?*/
            )?
        }
        TotalCreditLine { account } => {
            to_binary(todo!() /*&query::total_credit_line(deps, account)?*/)?
        }
        ListEnteredMarkets {
            account,
            start_after,
            limit,
        } => to_binary(
            todo!(), /*&query::entered_markets(deps, account, start_after, limit)?*/
        )?,
        IsOnMarket { account, market } => to_binary(
            todo!(), /*&query::is_on_market(deps, account, market)?*/
        )?,
        Liquidation { account } => to_binary(todo!() /*&query::liquidation(deps, account)?*/)?,
    };

    Ok(res)
}
