#[cfg(not(feature = "library"))]
use shade_protocol::{
    c_std::{
        from_binary, shd_entry_point, to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env,
        MessageInfo, Reply, Response, StdError, StdResult, SubMsgResult, Uint128,
    },
    contract_interfaces::snip20::{ExecuteMsg as Snip20ExecuteMsg, Snip20ReceiveMsg},
    lending_utils::{token::Token, Authentication, ViewingKey},
    query_auth::helpers::{authenticate_permit, authenticate_vk, PermitAuthentication},
    snip20,
    utils::{asset::Contract, Query},
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg, UserDataResponse},
    state::{
        find_value, insert_or_update, Config, MarketState, CONFIG, INIT_MARKET, MARKETS,
        MARKET_VIEWING_KEY,
    },
};

use either::Either;

use std::{collections::BTreeSet, ops::Deref};

const INIT_MARKET_REPLY_ID: u64 = 0;

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let default_estimate_multiplier = if msg.default_estimate_multiplier >= Decimal::one() {
        msg.default_estimate_multiplier
    } else {
        return Err(ContractError::InvalidEstimateMultiplier {});
    };

    // let random = env.block.random.unwrap();
    // let mut rng = Prng::new(random.as_slice(), job_id.as_bytes());
    MARKET_VIEWING_KEY.save(
        deps.storage,
        &ViewingKey {
            key: msg.market_viewing_key.to_owned(),
            address: env.contract.address.to_string(),
        },
    )?;

    if msg.liquidation_threshold > Decimal::percent(5) {
        return Err(ContractError::InvalidLiquidationThreshold {});
    }

    // TODO: should we validate Tokens?
    let cfg = Config {
        gov_contract: msg.gov_contract,
        query_auth: msg.query_auth,
        lend_market_id: msg.lend_market_id,
        lend_market_code_hash: msg.lend_market_code_hash,
        market_viewing_key: msg.market_viewing_key,
        ctoken_token_id: msg.ctoken_token_id,
        ctoken_code_hash: msg.ctoken_code_hash,
        reward_token: msg.reward_token,
        common_token: msg.common_token,
        liquidation_price: msg.liquidation_price,
        liquidation_threshold: msg.liquidation_threshold,
        borrow_limit_ratio: msg.borrow_limit_ratio,
        default_estimate_multiplier,
    };
    CONFIG.save(deps.storage, &cfg)?;
    INIT_MARKET.save(deps.storage, &None)?;
    MARKETS.save(deps.storage, &vec![])?;

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
        CreateMarket(market_cfg) => execute::create_market(deps, env, info, market_cfg),
        EnterMarket { account, market } => {
            let account = deps.api.addr_validate(&account)?;
            execute::enter_market(deps, info, market, account)
        }
        ExitMarket { market } => execute::exit_market(deps, info, market),
        Receive(msg) => receive_snip20_message(deps, env, info, msg),
        AdjustMarketId { new_market_id } => todo!(), //restricted::adjust_market_id(deps, info, new_market_id),
        AdjustTokenId { new_token_id } => todo!(), //restricted::adjust_token_id(deps, info, new_token_id),
        AdjustCommonToken { new_common_token } => {
            todo!() //restricted::adjust_common_token(deps, info, new_common_token)
        }
    }
}

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    println!("AGENCY REPLY");
    match (msg.id, msg.result) {
        (INIT_MARKET_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                /*
                let result: UnbondResponse = from_binary(&x)?;
                // Unbonding stored in try_unbond function
                // Because of here you can't access the sender of the TX this was stored previously
                let pending_unbonding = PENDING_UNBONDING.may_load(deps.storage)?;

                if let Some(unbonding_processing) = pending_unbonding {
                    Ok(Response::default())
                } else {
                    Err(StdError::generic_err("No active market instantiate"))
                }
                */
                Ok(Response::default())
            }
            None => Err(StdError::generic_err("Init market null response")),
        },
        _ => Err(StdError::generic_err("Unknown reply id")),
    }
}

pub fn receive_snip20_message(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Snip20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&msg.msg.unwrap())? {
        ReceiveMsg::Liquidate {
            account,
            collateral_denom,
        } => {
            let account = deps.api.addr_validate(&account)?;
            let sender = deps.api.addr_validate(&msg.sender)?;
            let config = CONFIG.load(deps.storage)?;

            execute::liquidate(
                deps,
                sender,
                account,
                shade_protocol::lending_utils::coin::Coin {
                    denom: Token::Cw20(
                        Contract::new(&info.sender, &config.ctoken_code_hash).into(),
                    ),
                    amount: msg.amount,
                },
                collateral_denom,
            )
        }
    }
}

mod execute {
    use super::*;

    use shade_protocol::c_std::{ensure_eq, from_binary, StdError, StdResult, SubMsg, WasmMsg};
    use shade_protocol::lending_utils::{
        coin::Coin,
        credit_line::{CreditLineResponse, CreditLineValues},
        price::{coin_times_price_rate, PriceRate},
    };

    use crate::{
        msg::{MarketConfig, ReceiveMsg},
        state::{MarketState, ENTERED_MARKETS, INIT_MARKET},
    };
    use lend_market::{
        msg::{ExecuteMsg as MarketExecuteMsg, QueryMsg as MarketQueryMsg},
        state::Config as MarketConfiguration,
    };

    pub fn create_market(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        market_cfg: MarketConfig,
    ) -> Result<Response, ContractError> {
        let market_token = market_cfg.market_token;

        let cfg = CONFIG.load(deps.storage)?;

        // Only governance contract can instantiate a market.
        ensure_eq!(
            info.sender,
            cfg.gov_contract.address,
            ContractError::Unauthorized {}
        );

        // Collateral ratio must be lower then liquidation price, otherwise
        // liquidation could decrese debt less then it decreases potential credit.
        if market_cfg.collateral_ratio >= cfg.liquidation_price {
            // TODO: shouldn't we use also a margin? Collateral ration should be 90% of liquidation price.
            return Err(ContractError::MarketCfgCollateralFailure {});
        }

        let mut markets = MARKETS.load(deps.storage)?;
        if let Some(state) = find_value::<Token, MarketState>(&markets, &market_token) {
            use MarketState::*;

            let err = match state {
                Instantiating => ContractError::MarketCreating(market_token.denom()),
                Ready(_) => ContractError::MarketAlreadyExists(market_token.denom()),
            };
            return Err(err);
        }
        insert_or_update(
            &mut markets,
            market_token.clone(),
            MarketState::Instantiating,
        );
        MARKETS.save(deps.storage, &markets)?;

        INIT_MARKET.save(deps.storage, &Some(market_token.clone()))?;

        let market_msg = lend_market::msg::InstantiateMsg {
            // Fields required for the lend-token instantiation.
            name: market_cfg.name,
            symbol: market_cfg.symbol,
            decimals: market_cfg.decimals,
            distributed_token: cfg.reward_token,
            ctoken_id: cfg.ctoken_token_id,
            ctoken_code_hash: cfg.ctoken_code_hash,
            viewing_key: cfg.market_viewing_key,

            market_token: market_token.clone(),
            market_cap: market_cfg.market_cap,
            interest_rate: market_cfg.interest_rate,
            interest_charge_period: market_cfg.interest_charge_period,
            common_token: cfg.common_token,
            collateral_ratio: market_cfg.collateral_ratio,
            price_oracle: market_cfg.price_oracle,
            reserve_factor: market_cfg.reserve_factor,
            gov_contract: cfg.gov_contract.address.to_string(),
            borrow_limit_ratio: cfg.borrow_limit_ratio,
            query_auth: cfg.query_auth,
            credit_agency_code_hash: env.contract.code_hash.clone(),
        };
        let market_instantiate = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: cfg.lend_market_id,
            code_hash: cfg.lend_market_code_hash,
            msg: to_binary(&market_msg)?,
            funds: vec![],
            label: format!("market_contract_{}", market_token),
        };

        Ok(Response::new()
            .add_attribute("action", "create_market")
            .add_attribute("sender", info.sender)
            .add_submessage(SubMsg::reply_always(
                market_instantiate,
                INIT_MARKET_REPLY_ID,
            )))
    }

    pub fn enter_market(
        deps: DepsMut,
        info: MessageInfo,
        market: Contract,
        account: Addr,
    ) -> Result<Response, ContractError> {
        if market.address != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        // TODO verify a valid market contract

        let mut markets = ENTERED_MARKETS.load(deps.storage)?;
        let mut entered_markets =
            find_value::<Addr, Vec<Contract>>(&markets, &Addr::unchecked(&account))
                .cloned()
                .unwrap_or_default();
        if !entered_markets.contains(&market) {
            entered_markets.push(market.clone());
        }
        insert_or_update(&mut markets, account.clone(), entered_markets);

        Ok(Response::new()
            .add_attribute("action", "enter_market")
            .add_attribute("market", market.address)
            .add_attribute("account", account))
    }

    pub fn exit_market(
        deps: DepsMut,
        info: MessageInfo,
        market: Contract,
    ) -> Result<Response, ContractError> {
        let common_token = CONFIG.load(deps.storage)?.common_token;

        let token_viewing_key = MARKET_VIEWING_KEY.load(deps.storage)?;
        let authentication = Authentication::ViewingKey(token_viewing_key);

        let mut markets = ENTERED_MARKETS.load(deps.storage)?;
        let mut entered_markets =
            find_value::<Addr, Vec<Contract>>(&markets, &Addr::unchecked(&info.sender))
                .cloned()
                .unwrap_or_default();
        if !entered_markets.contains(&market) {
            return Err(ContractError::NotOnMarket {
                address: info.sender,
                market: market.address.clone(),
            });
        }

        let market_credit_line: CreditLineResponse = deps.querier.query_wasm_smart(
            market.code_hash.clone(),
            market.address.to_string(),
            &MarketQueryMsg::CreditLine {
                account: info.sender.clone(),
                authentication: authentication.clone(),
            },
        )?;

        if !market_credit_line.debt.amount.is_zero() {
            return Err(ContractError::DebtOnMarket {
                address: info.sender,
                market: market.address.clone(),
                debt: market_credit_line.debt,
            });
        }

        // It can be removed before everything is checked, as if anything would fail, this removal
        // would not be applied. And in `reduced_credit_line` we don't want this market to be
        // there, so removing early.
        entered_markets.retain(|x| x != &market);

        let reduced_credit_line = entered_markets
            .iter()
            .map(|market| -> Result<CreditLineValues, ContractError> {
                let price_response: CreditLineResponse = deps.querier.query_wasm_smart(
                    market.code_hash.clone(),
                    market.address.to_string(),
                    &MarketQueryMsg::CreditLine {
                        account: info.sender.clone(),
                        authentication: authentication.clone(),
                    },
                )?;
                let price_response = price_response.validate(&common_token)?;
                Ok(price_response)
            })
            .try_fold(
                CreditLineValues::zero(),
                |total, credit_line| match credit_line {
                    Ok(cl) => Ok(total + cl),
                    Err(err) => Err(err),
                },
            )?;

        if reduced_credit_line.credit_line < reduced_credit_line.debt {
            return Err(ContractError::NotEnoughCollat {
                debt: reduced_credit_line.debt,
                credit_line: reduced_credit_line.credit_line,
                collateral: reduced_credit_line.collateral,
            });
        }

        insert_or_update(&mut markets, info.sender.clone(), entered_markets);
        ENTERED_MARKETS.save(deps.storage, &markets)?;

        Ok(Response::new()
            .add_attribute("action", "exit_market")
            .add_attribute("market", market.address)
            .add_attribute("account", info.sender))
    }

    fn create_repay_to_submessage(
        coin: shade_protocol::lending_utils::coin::Coin,
        debt_market: Contract,
        account: Addr,
    ) -> StdResult<SubMsg> {
        match coin.denom {
            Token::Cw20(contract_info) => {
                let repay_to_msg: Binary = to_binary(&lend_market::msg::ReceiveMsg::RepayTo {
                    account: account.to_string(),
                })?;

                let msg = to_binary(&Snip20ExecuteMsg::Send {
                    recipient: debt_market.address.to_string(),
                    recipient_code_hash: debt_market.code_hash.into(),
                    amount: coin.amount,
                    msg: Some(repay_to_msg),
                    memo: None,
                    padding: None,
                })
                .unwrap();

                Ok(SubMsg::new(WasmMsg::Execute {
                    contract_addr: contract_info.address.to_string(),
                    code_hash: contract_info.code_hash,
                    msg,
                    funds: vec![],
                }))
            }
        }
    }

    /// Liquidate implements the liquidation logic for both native and cw20 tokens.
    pub fn liquidate(
        deps: DepsMut,
        sender: Addr,
        // Account to liquidate.
        account: Addr,
        coins: shade_protocol::lending_utils::coin::Coin,
        collateral_denom: Token,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        let token_viewing_key = MARKET_VIEWING_KEY.load(deps.storage)?;
        let authentication = Authentication::ViewingKey(token_viewing_key);

        // assert that given account actually has more debt then credit
        let total_credit_line =
            query::total_credit_line(deps.as_ref(), account.to_string(), authentication)?;
        let total_credit_line = total_credit_line.validate(&cfg.common_token)?;
        // apply the liquidation threshold, so that user wouldn't become liquidated right away
        if total_credit_line.debt
            <= total_credit_line.credit_line
                + (total_credit_line.credit_line * cfg.liquidation_threshold)
        {
            return Err(ContractError::LiquidationNotAllowed {});
        }

        // Count debt and repay it. This requires that market returns error if repaying more then balance.
        let debt_market = query::market(deps.as_ref(), &coins.denom)?.market;

        let repay_to_msg =
            create_repay_to_submessage(coins.clone(), debt_market.clone(), account.clone())?;

        // find price rate of collateral denom
        let price_response: PriceRate = deps.querier.query_wasm_smart(
            debt_market.code_hash,
            debt_market.address.to_string(),
            &MarketQueryMsg::PriceMarketLocalPerCommon {},
        )?;

        // find market with wanted collateral_denom
        let collateral_market = query::market(deps.as_ref(), &collateral_denom)?.market;

        // transfer claimed amount as reward
        let msg = to_binary(&lend_market::msg::ExecuteMsg::TransferFrom {
            source: account.clone(),
            destination: sender.clone(),
            // transfer repaid amount represented as amount of common tokens, which is
            // calculated into collateral_denom's amount later in the market
            amount: coin_times_price_rate(&coins, &price_response)?.amount,
            liquidation_price: cfg.liquidation_price,
        })?;
        let transfer_from_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: collateral_market.address.to_string(),
            code_hash: collateral_market.code_hash.clone(),
            msg,
            funds: vec![],
        });

        Ok(Response::new()
            .add_attribute("action", "liquidate")
            .add_attribute("liquidator", sender)
            .add_attribute("account", account)
            .add_attribute("collateral_denom", collateral_denom.denom())
            .add_submessage(repay_to_msg)
            .add_submessage(transfer_from_msg))
    }
}

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;

    let res = match msg {
        Configuration {} => to_binary(&CONFIG.load(deps.storage)?)?,
        Market { market_token } => to_binary(&query::market(deps, &market_token)?)?,
        ListMarkets { limit } => to_binary(&query::list_markets(deps, limit)?)?,
        TotalCreditLine {
            account,
            authentication,
        } => to_binary(&query::total_credit_line(deps, account, authentication)?)?,
        ListEnteredMarkets { account } => to_binary(&query::entered_markets(deps, account)?)?,
        IsOnMarket { account, market } => to_binary(&query::is_on_market(deps, account, market)?)?,
        Liquidation { account } => to_binary(&query::liquidation(deps, account)?)?,
        UserData {
            account,
            authentication,
            tokens_balance,
            withdrawable,
            borrowable,
            credit_line,
        } => to_binary(&query::user_data(
            deps,
            account,
            authentication,
            tokens_balance,
            withdrawable,
            borrowable,
            credit_line,
        )?)?,
    };

    Ok(res)
}

mod query {
    use shade_protocol::c_std::StdResult;

    use lend_market::msg::{QueryMsg as MarketQueryMsg, TokensBalanceResponse};
    use shade_protocol::lending_utils::{
        coin::Coin,
        credit_line::{CreditLineResponse, CreditLineValues},
        Authentication,
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
        let markets = MARKETS.load(deps.storage)?;
        let state = find_value::<Token, MarketState>(&markets, market_token)
            .ok_or_else(|| ContractError::NoMarket(market_token.denom()))?;

        let addr = state
            .clone()
            .to_contract()
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
        limit: Option<u32>,
    ) -> Result<ListMarketsResponse, ContractError> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

        let markets: StdResult<Vec<_>> = MARKETS
            .load(deps.storage)?
            .into_iter()
            .map(|(market_token, market_state)| {
                let result = market_state.to_contract().map(|contract| MarketResponse {
                    market_token,
                    market: contract,
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
        authentication: Authentication,
    ) -> Result<CreditLineResponse, ContractError> {
        let common_token = CONFIG.load(deps.storage)?.common_token;
        let markets = ENTERED_MARKETS.load(deps.storage)?;
        let entered_markets =
            find_value::<Addr, Vec<Contract>>(&markets, &Addr::unchecked(&account))
                .cloned()
                .unwrap_or_default();

        let total_credit_line: CreditLineValues = entered_markets
            .into_iter()
            .map(|market| {
                let price_response: CreditLineResponse = deps.querier.query_wasm_smart(
                    market.code_hash,
                    market.address.to_string(),
                    &MarketQueryMsg::CreditLine {
                        account: deps.api.addr_validate(&account)?,
                        authentication: authentication.clone(),
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
    ) -> Result<ListEnteredMarketsResponse, ContractError> {
        let account = Addr::unchecked(account);
        let markets = ENTERED_MARKETS.load(deps.storage)?;
        let entered_markets =
            find_value::<Addr, Vec<Contract>>(&markets, &Addr::unchecked(&account))
                .cloned()
                .unwrap_or_default();

        Ok(ListEnteredMarketsResponse {
            markets: entered_markets,
        })
    }

    pub fn is_on_market(
        deps: Deps,
        account: String,
        market: Contract,
    ) -> Result<IsOnMarketResponse, ContractError> {
        let account = Addr::unchecked(account);
        let markets = ENTERED_MARKETS.load(deps.storage)?;
        let entered_markets =
            find_value::<Addr, Vec<Contract>>(&markets, &Addr::unchecked(&account))
                .cloned()
                .unwrap_or_default();

        Ok(IsOnMarketResponse {
            participating: entered_markets.contains(&market),
        })
    }

    pub fn liquidation(deps: Deps, account: String) -> Result<LiquidationResponse, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        let account_addr = deps.api.addr_validate(&account)?;

        let market_viewing_key = MARKET_VIEWING_KEY.load(deps.storage)?;
        let authentication = Authentication::ViewingKey(market_viewing_key);

        // check whether the given account actually has more debt then credit
        let total_credit_line: CreditLineResponse =
            total_credit_line(deps, account.clone(), authentication.clone())?;

        // add liquidation threshold to the credit line
        let additional_amount = total_credit_line.credit_line.amount * cfg.liquidation_threshold;
        let mut tcl_with_cushion = total_credit_line.credit_line;
        tcl_with_cushion.amount += additional_amount;

        let can_liquidate = total_credit_line.debt > tcl_with_cushion;

        let markets = ENTERED_MARKETS.load(deps.storage)?;
        let entered_markets =
            find_value::<Addr, Vec<Contract>>(&markets, &Addr::unchecked(&account))
                .cloned()
                .unwrap_or_default();

        let market_data: Result<Vec<_>, _> = entered_markets
            .into_iter()
            .map(|market| -> Result<(Contract, Coin, Coin), ContractError> {
                let token_balances: TokensBalanceResponse = deps.querier.query_wasm_smart(
                    market.code_hash.clone(),
                    market.address.to_string(),
                    &MarketQueryMsg::TokensBalance {
                        account: deps.api.addr_validate(&account)?,
                        authentication: authentication.clone(),
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

    // Query message that summs up all the other queries in a one call
    pub fn user_data(
        deps: Deps,
        account: String,
        authentication: Authentication,
        token_balance: bool,
        withdrawable: bool,
        borrowable: bool,
        credit_line: bool,
    ) -> StdResult<UserDataResponse> {
        let common_token = CONFIG.load(deps.storage)?.common_token;
        let markets = ENTERED_MARKETS.load(deps.storage)?;
        let entered_markets =
            find_value::<Addr, Vec<Contract>>(&markets, &Addr::unchecked(&account))
                .cloned()
                .unwrap_or_default();

        let token_balance = if token_balance {
            entered_markets
                .iter()
                .map(|market| {
                    let market = market.clone();
                    let balance_response: lend_market::msg::TokensBalanceResponse =
                        deps.querier.query_wasm_smart(
                            market.code_hash.clone(),
                            market.address.to_string(),
                            &MarketQueryMsg::TokensBalance {
                                account: deps.api.addr_validate(&account)?,
                                authentication: authentication.clone(),
                            },
                        )?;
                    Ok((market.clone(), balance_response))
                })
                .collect::<StdResult<Vec<(Contract, lend_market::msg::TokensBalanceResponse)>>>()?
        } else {
            vec![]
        };

        let withdrawable = if withdrawable {
            entered_markets
                .iter()
                .map(|market| {
                    let balance_response: Coin = deps.querier.query_wasm_smart(
                        market.code_hash.clone(),
                        market.address.to_string(),
                        &MarketQueryMsg::Withdrawable {
                            account: deps.api.addr_validate(&account)?,
                            authentication: authentication.clone(),
                        },
                    )?;
                    Ok((market.clone(), balance_response))
                })
                .collect::<StdResult<Vec<(Contract, Coin)>>>()?
        } else {
            vec![]
        };

        let borrowable = if borrowable {
            entered_markets
                .iter()
                .map(|market| {
                    let balance_response: Coin = deps.querier.query_wasm_smart(
                        market.code_hash.clone(),
                        market.address.to_string(),
                        &MarketQueryMsg::Borrowable {
                            account: deps.api.addr_validate(&account)?,
                            authentication: authentication.clone(),
                        },
                    )?;
                    Ok((market.clone(), balance_response))
                })
                .collect::<StdResult<Vec<(Contract, Coin)>>>()?
        } else {
            vec![]
        };

        let credit_line = if credit_line {
            entered_markets
                .into_iter()
                .map(|market| {
                    let price_response: CreditLineResponse = deps.querier.query_wasm_smart(
                        market.code_hash.clone(),
                        market.address.to_string(),
                        &MarketQueryMsg::CreditLine {
                            account: deps.api.addr_validate(&account)?,
                            authentication: authentication.clone(),
                        },
                    )?;
                    Ok((market.clone(), price_response))
                })
                .collect::<StdResult<Vec<(Contract, CreditLineResponse)>>>()?
        } else {
            vec![]
        };

        Ok(UserDataResponse {
            token_balance,
            withdrawable,
            borrowable,
            credit_line,
        })
    }
}
