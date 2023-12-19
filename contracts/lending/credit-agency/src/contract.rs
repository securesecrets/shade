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
