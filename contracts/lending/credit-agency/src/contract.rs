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
    let default_estimate_multiplier = if msg.default_estimate_multiplier >= Decimal::one() {
        msg.default_estimate_multiplier
    } else {
        return Err(ContractError::InvalidEstimateMultiplier {});
    };

    // TODO: should we validate Tokens?
    let cfg = Config {
        gov_contract: msg.gov_contract,
        lend_market_id: msg.lending_market_id,
        lend_token_id: msg.lending_token_id,
        reward_token: msg.reward_token,
        common_token: msg.common_token,
        liquidation_price: msg.liquidation_price,
        borrow_limit_ratio: msg.borrow_limit_ratio,
        default_estimate_multiplier,
    };
    CONFIG.save(deps.storage, &cfg)?;
    NEXT_REPLY_ID.save(deps.storage, &0)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
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

mod execute {
    use super::*;

    use shade_protocol::c_std::{ensure_eq, from_binary, StdError, StdResult, SubMsg, WasmMsg};
    use utils::{
        coin::Coin,
        credit_line::{CreditLineResponse, CreditLineValues},
        price::{coin_times_price_rate, PriceRate},
    };

    use crate::{
        msg::{MarketConfig, ReceiveMsg},
        state::{MarketState, ENTERED_MARKETS, MARKETS, REPLY_IDS},
    };
    use lending_market::{
        msg::{ExecuteMsg as MarketExecuteMsg, QueryMsg as MarketQueryMsg},
        state::Config as MarketConfiguration,
    };

    // pub fn create_market(
    //     deps: DepsMut,
    //     env: Env,
    //     info: MessageInfo,
    //     market_cfg: MarketConfig,
    // ) -> Result<Response, ContractError> {
    //     let market_token = market_cfg.market_token;

    //     let cfg = CONFIG.load(deps.storage)?;

    //     // Only governance contract can instantiate a market.
    //     ensure_eq!(
    //         info.sender,
    //         cfg.gov_contract,
    //         ContractError::Unauthorized {}
    //     );

    //     // Collateral ratio must be lower then liquidation price, otherwise
    //     // liquidation could decrese debt less then it decreases potential credit.
    //     if market_cfg.collateral_ratio >= cfg.liquidation_price {
    //         // TODO: shouldn't we use also a margin? Collateral ration should be 90% of liquidation price.
    //         return Err(ContractError::MarketCfgCollateralFailure {});
    //     }

    //     if let Some(state) = MARKETS.may_load(deps.storage, &market_token)? {
    //         use MarketState::*;

    //         let err = match state {
    //             Instantiating => ContractError::MarketCreating(market_token.denom()),
    //             Ready(_) => ContractError::MarketAlreadyExists(market_token.denom()),
    //         };
    //         return Err(err);
    //     }
    //     MARKETS.save(deps.storage, &market_token, &MarketState::Instantiating)?;

    //     let reply_id =
    //         NEXT_REPLY_ID.update(deps.storage, |id| -> Result<_, StdError> { Ok(id + 1) })?;
    //     REPLY_IDS.save(deps.storage, reply_id, &market_token)?;

    //     let market_msg = wynd_lend_market::msg::InstantiateMsg {
    //         // Fields required for the wynd_lend-token instantiation.
    //         name: market_cfg.name,
    //         symbol: market_cfg.symbol,
    //         decimals: market_cfg.decimals,
    //         distributed_token: cfg.reward_token,
    //         token_id: cfg.wynd_lend_token_id,

    //         market_token: market_token.clone(),
    //         market_cap: market_cfg.market_cap,
    //         interest_rate: market_cfg.interest_rate,
    //         interest_charge_period: market_cfg.interest_charge_period,
    //         common_token: cfg.common_token,
    //         collateral_ratio: market_cfg.collateral_ratio,
    //         price_oracle: market_cfg.price_oracle,
    //         reserve_factor: market_cfg.reserve_factor,
    //         gov_contract: cfg.gov_contract.to_string(),
    //         borrow_limit_ratio: cfg.borrow_limit_ratio,
    //     };
    //     let market_instantiate = WasmMsg::Instantiate {
    //         admin: Some(env.contract.address.to_string()),
    //         code_id: cfg.lending_market_id,
    //         msg: to_binary(&market_msg)?,
    //         funds: vec![],
    //         label: format!("market_contract_{}", market_token),
    //     };

    //     Ok(Response::new()
    //         .add_attribute("action", "create_market")
    //         .add_attribute("sender", info.sender)
    //         .add_submessage(SubMsg::reply_on_success(market_instantiate, reply_id)))
    // }

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
        todo!();
        // let state = MARKETS
        //     .may_load(deps.storage, market_token)?
        //     .ok_or_else(|| ContractError::NoMarket(market_token.denom()))?;

        // let addr = state
        //     .to_addr()
        //     .ok_or_else(|| ContractError::MarketCreating(market_token.denom()))?;

        // Ok(MarketResponse {
        //     market_token: market_token.to_owned(),
        //     market: addr,
        // })
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

        todo!();
        // let markets: StdResult<Vec<_>> = MARKETS
        //     .range(deps.storage, start, None, Order::Ascending)
        //     .map(|m| {
        //         let (market_token, market) = m?;

        //         let result = market.to_addr().map(|addr| MarketResponse {
        //             market_token,
        //             market: addr,
        //         });

        //         Ok(result)
        //     })
        //     .filter_map(|m| m.transpose())
        //     .take(limit)
        //     .collect();

        // Ok(ListMarketsResponse { markets: markets? })
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

        todo!()
        // let total_credit_line: CreditLineValues = markets
        //     .into_iter()
        //     .map(|market| {
        //         let price_response: CreditLineResponse = deps.querier.query_wasm_smart(
        //             market,
        //             &MarketQueryMsg::CreditLine {
        //                 account: deps.api.addr_validate(&account)?,
        //             },
        //         )?;
        //         let price_response = price_response.validate(&common_token.clone())?;
        //         Ok(price_response)
        //     })
        //     .collect::<Result<Vec<CreditLineValues>, ContractError>>()?
        //     .iter()
        //     .sum();
        // Ok(total_credit_line.make_response(common_token))
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

        todo!();
        // let market_data: Result<Vec<_>, _> = markets
        //     .into_iter()
        //     .map(|market| -> Result<(Addr, Coin, Coin), ContractError> {
        //         let token_balances: TokensBalanceResponse = deps.querier.query_wasm_smart(
        //             &market,
        //             &MarketQueryMsg::TokensBalance {
        //                 account: deps.api.addr_validate(&account)?,
        //             },
        //         )?;

        //         Ok((market, token_balances.collateral, token_balances.debt))
        //     })
        //     .collect();
        // let market_data = market_data?;

        // let collateral: Vec<_> = market_data
        //     .iter()
        //     .filter(|(_, collateral, _)| !collateral.amount.is_zero())
        //     .cloned()
        //     .map(|(market, collateral, _)| (market, collateral))
        //     .collect();
        // let debt: Vec<_> = market_data
        //     .into_iter()
        //     .filter(|(_, _, debt)| !debt.amount.is_zero())
        //     .map(|(market, _, debt)| (market, debt))
        //     .collect();

        // Ok(LiquidationResponse {
        //     can_liquidate,
        //     debt,
        //     collateral,
        // })
    }
}
