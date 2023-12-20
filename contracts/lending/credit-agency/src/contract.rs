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

use either::Either;

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
        Configuration {} => to_binary(&CONFIG.load(deps.storage)?)?,
        Market { market_token } => to_binary(&query::market(deps, &market_token)?)?,
        ListMarkets { start_after, limit } => {
            to_binary(&query::list_markets(deps, start_after, limit)?)?
        }
        TotalCreditLine { account } => to_binary(&query::total_credit_line(deps, account)?)?,
        ListEnteredMarkets {
            account,
            start_after,
            limit,
        } => to_binary(&query::entered_markets(deps, account, start_after, limit)?)?,
        IsOnMarket { account, market } => to_binary(&query::is_on_market(deps, account, market)?)?,
        Liquidation { account } => to_binary(&query::liquidation(deps, account)?)?,
    };

    Ok(res)
}

mod query {
    use shade_protocol::{
        c_std::{Order, StdResult},
        secret_storage_plus::Bound,
    };

    use lend_market::msg::{QueryMsg as MarketQueryMsg, TokensBalanceResponse};
    use lending_utils::{
        coin::Coin,
        credit_line::{CreditLineResponse, CreditLineValues},
    };

    use crate::{
        msg::{
            IsOnMarketResponse, LiquidationResponse, ListEnteredMarketsResponse,
            ListMarketsResponse, MarketResponse,
        },
        state::{ENTERED_MARKETS, MARKETS},
    };

    use super::*;

    /// Returns the address of the market associated to the given `market_token`. Returns an error
    /// if the market does not exists or is being created.
    pub fn market(deps: Deps, market_token: &Token) -> Result<MarketResponse, ContractError> {
        let state = MARKETS
            .may_load(deps.storage, market_token)?
            .ok_or_else(|| ContractError::NoMarket(market_token.denom()))?;

        let addr = state
            .to_addr()
            .ok_or_else(|| ContractError::MarketCreating(market_token.denom()))?;

        Ok(MarketResponse {
            market_token: market_token.to_owned(),
            market: addr,
        })
    }

    // settings for pagination
    const MAX_LIMIT: u32 = 30;
    const DEFAULT_LIMIT: u32 = 10;

    pub fn list_markets(
        deps: Deps,
        start_after: Option<Token>,
        limit: Option<u32>,
    ) -> Result<ListMarketsResponse, ContractError> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.as_ref().map(Bound::exclusive);

        let markets: StdResult<Vec<_>> = MARKETS
            .range(deps.storage, start, None, Order::Ascending)
            .map(|m| {
                let (market_token, market) = m?;

                let result = market.to_addr().map(|addr| MarketResponse {
                    market_token,
                    market: addr,
                });

                Ok(result)
            })
            .filter_map(|m| m.transpose())
            .take(limit)
            .collect();

        Ok(ListMarketsResponse { markets: markets? })
    }

    /// Handler for `QueryMsg::TotalCreditLine`
    /// Computes the sum of `CreditLineValues` for all markets the `address` is participating to.
    pub fn total_credit_line(
        deps: Deps,
        account: String,
    ) -> Result<CreditLineResponse, ContractError> {
        let common_token = CONFIG.load(deps.storage)?.common_token;
        let markets = ENTERED_MARKETS
            .may_load(deps.storage, &Addr::unchecked(&account))?
            .unwrap_or_default();

        let total_credit_line: CreditLineValues = markets
            .into_iter()
            .map(|market| {
                let price_response: CreditLineResponse = deps.querier.query_wasm_smart(
                    market,
                    &MarketQueryMsg::CreditLine {
                        account: deps.api.addr_validate(&account)?,
                    },
                )?;
                let price_response = price_response.validate(&common_token.clone())?;
                Ok(price_response)
            })
            .collect::<Result<Vec<CreditLineValues>, ContractError>>()?
            .iter()
            .sum();
        Ok(total_credit_line.make_response(common_token))
    }

    pub fn entered_markets(
        deps: Deps,
        account: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<ListEnteredMarketsResponse, ContractError> {
        let account = Addr::unchecked(account);
        let markets = ENTERED_MARKETS
            .may_load(deps.storage, &account)?
            .unwrap_or_default()
            .into_iter();

        let markets = if let Some(start_after) = &start_after {
            Either::Left(
                markets
                    .skip_while(move |market| market != start_after)
                    .skip(1),
            )
        } else {
            Either::Right(markets)
        };

        let markets = markets.take(limit.unwrap_or(u32::MAX) as usize).collect();

        Ok(ListEnteredMarketsResponse { markets })
    }

    pub fn is_on_market(
        deps: Deps,
        account: String,
        market: String,
    ) -> Result<IsOnMarketResponse, ContractError> {
        let account = Addr::unchecked(account);
        let market = Addr::unchecked(market);
        let markets = ENTERED_MARKETS
            .may_load(deps.storage, &account)?
            .unwrap_or_default();

        Ok(IsOnMarketResponse {
            participating: markets.contains(&market),
        })
    }

    pub fn liquidation(deps: Deps, account: String) -> Result<LiquidationResponse, ContractError> {
        let account_addr = deps.api.addr_validate(&account)?;

        // check whether the given account actually has more debt then credit
        let total_credit_line: CreditLineResponse = total_credit_line(deps, account.clone())?;
        let can_liquidate = total_credit_line.debt > total_credit_line.credit_line;

        let markets = ENTERED_MARKETS
            .may_load(deps.storage, &account_addr)?
            .unwrap_or_default();

        let market_data: Result<Vec<_>, _> = markets
            .into_iter()
            .map(|market| -> Result<(Addr, Coin, Coin), ContractError> {
                let token_balances: TokensBalanceResponse = deps.querier.query_wasm_smart(
                    &market,
                    &MarketQueryMsg::TokensBalance {
                        account: deps.api.addr_validate(&account)?,
                    },
                )?;

                Ok((market, token_balances.collateral, token_balances.debt))
            })
            .collect();
        let market_data = market_data?;

        let collateral: Vec<_> = market_data
            .iter()
            .filter(|(_, collateral, _)| !collateral.amount.is_zero())
            .cloned()
            .map(|(market, collateral, _)| (market, collateral))
            .collect();
        let debt: Vec<_> = market_data
            .into_iter()
            .filter(|(_, _, debt)| !debt.amount.is_zero())
            .map(|(market, _, debt)| (market, debt))
            .collect();

        Ok(LiquidationResponse {
            can_liquidate,
            debt,
            collateral,
        })
    }
}
