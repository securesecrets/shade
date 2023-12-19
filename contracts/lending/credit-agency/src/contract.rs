#[cfg(not(feature = "library"))]
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
    info: Messageinfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    todo!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        CreateMarket(market_cfg) => execute::create_market(deps, env, info, market_cfg),
        Liquidate {
            account,
            collateral_denom,
        } => {
            let account = deps.api.addr_validate(&account)?;

            // Assert that only one native denom was sent.
            if info.funds.is_empty() || info.funds.len() != 1 {
                return Err(ContractError::LiquidationOnlyOneDenomRequired {});
            }

            let coin = utils::coin::Coin::new(
                info.funds[0].amount.u128(),
                Token::Native(info.funds[0].denom.clone()),
            );

            execute::liquidate(deps, info.sender, account, coin, collateral_denom)
        }
        EnterMarket { account } => {
            let account = deps.api.addr_validate(&account)?;
            execute::enter_market(deps, info, account)
        }
        ExitMarket { market } => {
            let market = deps.api.addr_validate(&market)?;
            execute::exit_market(deps, info, market)
        }
        RepayWithCollateral {
            max_collateral,
            amount_to_repay,
            estimate_multiplier,
        } => execute::repay_with_collateral(
            deps,
            info.sender,
            max_collateral,
            amount_to_repay,
            estimate_multiplier,
        ),
        Receive(msg) => execute::receive_snip20_message(deps, env, info, msg),
        AdjustMarketId { new_market_id } => restricted::adjust_market_id(deps, info, new_market_id),
        AdjustTokenId { new_token_id } => restricted::adjust_token_id(deps, info, new_token_id),
        AdjustCommonToken { new_common_token } => {
            restricted::adjust_common_token(deps, info, new_common_token)
        }
    }
}
