#[cfg(not(feature = "library"))]
use shade_protocol::c_std::entry_point;
use shade_protocol::{
    c_std::{
        to_binary, Addr, Binary, Coin as StdCoin, Decimal, Deps, DepsMut, Env, MessageInfo, Reply,
        Response, StdError, StdResult, SubMsg, Timestamp, Uint128, WasmMsg,
    },
    query_authentication::viewing_keys,
    utils::{asset::Contract, Query},
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Config, CONFIG, VIEWING_KEY},
};

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
        credit_agency: Contract::new(&info.sender.clone(), &msg.credit_agency_code_hash).into(),
        reserve_factor: msg.reserve_factor,
        borrow_limit_ratio: msg.borrow_limit_ratio,
        oracle: msg.oracle.into()
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
    match msg {
        Withdraw { amount } => execute::withdraw(deps, env, info, amount),
    }
}

// Available credit line helpers
mod cr_lending_utils {
    use super::*;

    use shade_protocol::c_std::{Deps, DivideByZeroError, Fraction};

    use lending_utils::{
        amount::base_to_token,
        credit_line::{CreditLineResponse, CreditLineValues},
    };

    use crate::{interest::query_ctoken_multiplier, msg::QueryTotalCreditLine};

    pub fn divide(top: Uint128, bottom: Decimal) -> Result<Uint128, DivideByZeroError> {
        (top * bottom.denominator()).checked_div(bottom.numerator())
    }

    fn available_local_tokens(
        deps: Deps,
        common_tokens: Uint128,
    ) -> Result<Uint128, ContractError> {
        // Price is defined as common/local
        // (see price_market_local_per_common function from this file)
        divide(
            common_tokens,
            query::price_market_local_per_common(deps)?.rate_sell_per_buy,
        )
        .map_err(|_| ContractError::ZeroPrice {})
    }

    /// Returns the amount of local tokens that can be borrowed
    pub fn query_borrowable_tokens(
        deps: Deps,
        config: &Config,
        account: String,
    ) -> Result<Uint128, ContractError> {
        let credit: CreditLineResponse = QueryTotalCreditLine::TotalCreditLine { account }
            .query(&deps.querier, &config.credit_agency)?;
        let credit = credit.validate(&config.common_token.clone())?;

        query_borrowable_tokens_with_creditvalues(deps, &credit)
    }

    /// Returns how many market token is it possible to borrow given a `CreditLineValues`.
    pub fn query_borrowable_tokens_with_creditvalues(
        deps: Deps,
        credit: &CreditLineValues,
    ) -> Result<Uint128, ContractError> {
        // Available credit for that account amongst all markets
        let available_common = credit.borrow_limit.saturating_sub(credit.debt);
        let available_local = available_local_tokens(deps, available_common)?;
        Ok(available_local)
    }

    /// Helper that determines if an address can borrow the specified amount.
    pub fn can_borrow(
        deps: Deps,
        config: &Config,
        account: impl Into<String>,
        amount: Uint128,
    ) -> Result<bool, ContractError> {
        let available = query_borrowable_tokens(deps, config, account.into())?;
        Ok(amount <= available)
    }

    /// Helper returning amount of tokens available to transfer/withdraw
    pub fn transferable_amount(
        deps: Deps,
        config: &Config,
        account: impl Into<String>,
    ) -> Result<Uint128, ContractError> {
        let account = account.into();
        let credit: CreditLineResponse = QueryTotalCreditLine::TotalCreditLine {
            account: account.clone(),
        }
        .query(&deps.querier, &config.credit_agency)?;
        let credit = credit.validate(&config.common_token.clone())?;

        let available = query_borrowable_tokens_with_creditvalues(deps, &credit)?;
        let mut can_transfer = divide(available, config.collateral_ratio)
            .map_err(|_| ContractError::ZeroCollateralRatio {})?;
        if credit.debt.u128() == 0 {
            let multiplier = query_ctoken_multiplier(deps, config)?;
            can_transfer = std::cmp::max(
                base_to_token(can_transfer, multiplier),
                query::ctoken_balance(deps, config, &account)?.amount,
            );
        }
        Ok(can_transfer)
    }
}

mod execute {
    use lending_utils::{
        amount::{base_to_token, token_to_base},
        coin::Coin,
    };
    use shade_protocol::{
        c_std::{from_binary, SubMsg},
        contract_interfaces::snip20::Snip20ReceiveMsg,
    };

    use crate::interest::{
        calculate_interest, epochs_passed, query_ctoken_multiplier, InterestUpdate,
    };

    use super::*;

    /// Handler for `ExecuteMsg::Withdraw`
    pub fn withdraw(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        Ok(Response::new())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;
    Ok(to_binary(&"")?)
}

mod query {
    use super::*;

    use shade_protocol::{
        c_std::{ContractInfo, Decimal, Deps, Uint128},
        utils::asset::Contract,
        contract_interfaces::oracles,
    };

    use lend_token::msg::{BalanceResponse, QueryMsg as TokenQueryMsg, TokenInfoResponse};
    use lending_utils::{
        coin::Coin,
        credit_line::{CreditLineResponse, CreditLineValues},
        price::{coin_times_price_rate, PriceRate},
    };

    use crate::{
        interest::{calculate_interest, epochs_passed, utilisation},
        msg::{ApyResponse, InterestResponse, ReserveResponse, TokensBalanceResponse},
        state::{debt, SECONDS_IN_YEAR},
    };

    fn token_balance(
        deps: Deps,
        token_contract: &ContractInfo,
        address: String,
    ) -> StdResult<BalanceResponse> {
        TokenQueryMsg::Balance { address }.query(&deps.querier, token_contract)
    }

    fn base_balance(
        deps: Deps,
        token_contract: &ContractInfo,
        address: String,
    ) -> StdResult<BalanceResponse> {
        TokenQueryMsg::BaseBalance { address }.query(&deps.querier, token_contract)
    }

    pub fn ctoken_balance(
        deps: Deps,
        config: &Config,
        account: impl ToString,
    ) -> Result<Coin, ContractError> {
        Ok(config.market_token.amount(
            token_balance(
                deps,
                &Contract::new(&config.ctoken_contract, &config.ctoken_code_hash).into(),
                account.to_string(),
            )?
            .balance,
        ))
    }

    pub fn ctoken_base_balance(
        deps: Deps,
        config: &Config,
        account: impl ToString,
    ) -> Result<Coin, ContractError> {
        Ok(config.market_token.amount(
            base_balance(
                deps,
                &Contract::new(&config.ctoken_contract, &config.ctoken_code_hash).into(),
                account.to_string(),
            )?
            .balance,
        ))
    }

    /// Handler for `QueryMsg::Config`
    pub fn config(deps: Deps, env: Env) -> Result<Config, ContractError> {
        let mut config = CONFIG.load(deps.storage)?;

        let unhandled_charge_period = epochs_passed(&config, env)?;
        config.last_charged += unhandled_charge_period * config.interest_charge_period;

        Ok(config)
    }

    /// Handler for `QueryMsg::TokensBalance`
    pub fn tokens_balance(
        deps: Deps,
        env: Env,
        account: String,
    ) -> Result<TokensBalanceResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        let mut collateral = ctoken_base_balance(deps, &config, account.clone())?;
        let mut debt = Coin {
            denom: config.market_token.clone(),
            amount: debt::of(deps.storage, &deps.api.addr_validate(&account)?)?,
        };

        if let Some(update) = calculate_interest(deps, epochs_passed(&config, env)?)? {
            collateral.amount += collateral.amount * update.ctoken_ratio;
            debt.amount += debt.amount * update.debt_ratio;
        }

        Ok(TokensBalanceResponse { collateral, debt })
    }

    // /// Handler for `QueryMsg::TransferableAmount`
    // pub fn transferable_amount(
    //     deps: Deps,
    //     token: ContractInfo,
    //     account: String,
    // ) -> Result<TransferableAmountResponse, ContractError> {
    //     let config = CONFIG.load(deps.storage)?;
    //     if token == config.ctoken_contract {
    //         let transferable = cr_lending_utils::transferable_amount(deps, &config, account)?;
    //         Ok(TransferableAmountResponse { transferable })
    //     } else {
    //         Err(ContractError::UnrecognisedToken(token.to_string()))
    //     }
    // }

    // /// Handler for `QueryMsg::Withdrawable`
    // pub fn withdrawable(deps: Deps, env: Env, account: String, viewing_key: String) -> Result<Coin, ContractError> {
    //     use std::cmp::min;

    //     let cfg = CONFIG.load(deps.storage)?;

    //     let transferable = cr_lending_utils::transferable_amount(deps, &cfg, &account)?;
    //     let ctoken_balance = ctoken_base_balance(deps, &cfg, &account)?;
    //     let allowed_to_withdraw = min(transferable, ctoken_balance.amount);
    //     let withdrawable = min(
    //         allowed_to_withdraw,
    //         cfg.market_token
    //             .query_balance(deps, env.contract.address, viewing_key)?
    //             .into(),
    //     );

    //     Ok(cfg.market_token.amount(withdrawable))
    // }

    // /// Handler for `QueryMsg::Borrowable`
    // pub fn borrowable(deps: Deps, env: Env, account: String, viewing_key: String) -> Result<Coin, ContractError> {
    //     use std::cmp::min;

    //     let cfg = CONFIG.load(deps.storage)?;

    //     let borrowable = cr_lending_utils::query_borrowable_tokens(deps, &cfg, account)?;
    //     let borrowable = min(
    //         borrowable,
    //         cfg.market_token
    //             .query_balance(deps, env.contract.address.to_string(), viewing_key)?
    //             .into(),
    //     );

    //     Ok(cfg.market_token.amount(borrowable))
    // }

    // pub fn ctoken_info(deps: Deps, config: &Config) -> Result<TokenInfoResponse, ContractError> {
    //     crate::interest::ctoken_info(deps, config)
    // }

    // /// Handler for `QueryMsg::Interest`
    // pub fn interest(deps: Deps) -> Result<InterestResponse, ContractError> {
    //     let config = CONFIG.load(deps.storage)?;
    //     let ctoken_info = ctoken_info(deps, &config)?;

    //     let supplied = ctoken_info.total_supply_base();
    //     let (borrowed, _) = debt::total(deps.storage)?;
    //     let utilisation = utilisation(supplied, borrowed);

    //     let interest = config.rates.calculate_interest_rate(utilisation);

    //     Ok(InterestResponse {
    //         interest,
    //         utilisation,
    //         charge_period: Timestamp::from_seconds(config.interest_charge_period),
    //     })
    // }

    /// Handler for `QueryMsg::PriceMarketLocalPerCommon`
    /// Returns the ratio of the twap of the market token over the common token.
    pub fn price_market_local_per_common(deps: Deps) -> Result<PriceRate, ContractError> {
        todo!();

        let config = CONFIG.load(deps.storage)?;
        // If tokens are the same, just return 1:1.
        if config.common_token == config.market_token {
            Ok(PriceRate {
                sell_denom: config.market_token.clone(),
                buy_denom: config.common_token,
                rate_sell_per_buy: Decimal::one(),
            })
        } else {
            todo!();
            // let price_response: TwapResponse = OracleQueryMsg::Twap {
            //     offer: config.market_token.clone().into(),
            //     ask: config.common_token.clone().into(),
            // }
            // .query(&deps.querier, config.price_oracle.clone())?;
            Ok(PriceRate {
                sell_denom: config.market_token,
                buy_denom: config.common_token,
                rate_sell_per_buy: Decimal::one() /*price_response.a_per_b,*/
            })
        }
    }

    // /// Handler for `QueryMsg::CreditLine`
    // /// Returns the debt and credit situation of the `account` after applying interests.
    // pub fn credit_line(
    //     deps: Deps,
    //     env: Env,
    //     account: Addr,
    // ) -> Result<CreditLineResponse, ContractError> {
    //     let config = CONFIG.load(deps.storage)?;
    //     let mut collateral = ctoken_base_balance(deps, &config, &account)?;
    //     let mut debt = Coin {
    //         denom: config.market_token.clone(),
    //         amount: debt::of(deps.storage, &account)?,
    //     };

    //     // Simulate charging interest for any periods `charge_interest` wasn't called for yet
    //     if let Some(update) = calculate_interest(deps, epochs_passed(&config, env)?)? {
    //         collateral.amount += collateral.amount * update.ctoken_ratio;
    //         debt.amount += debt.amount * update.debt_ratio;
    //     }

    //     if collateral.amount.is_zero() && debt.amount.is_zero() {
    //         return Ok(CreditLineValues::zero().make_response(config.common_token));
    //     }

    //     let price_ratio = price_market_local_per_common(deps)?;
    //     let collateral = coin_times_price_rate(&collateral, &price_ratio)?;
    //     let debt = coin_times_price_rate(&debt, &price_ratio)?.amount;
    //     let credit_line = collateral.amount * config.collateral_ratio;
    //     let borrow_limit = credit_line * config.borrow_limit_ratio;
    //     Ok(
    //         CreditLineValues::new(collateral.amount, credit_line, borrow_limit, debt)
    //             .make_response(config.common_token),
    //     )
    // }

    // /// Handler for `QueryMsg::Reserve`
    // pub fn reserve(deps: Deps, env: Env) -> Result<ReserveResponse, ContractError> {
    //     let config = CONFIG.load(deps.storage)?;

    //     let reserve = calculate_interest(deps, epochs_passed(&config, env)?)?
    //         .map(|update| update.reserve)
    //         .unwrap_or(Uint128::zero());

    //     Ok(ReserveResponse { reserve })
    // }

    // /// Handler for `QueryMsg::Apy`
    // pub fn apy(deps: Deps) -> Result<ApyResponse, ContractError> {
    //     let cfg = CONFIG.load(deps.storage)?;
    //     let charge_periods = SECONDS_IN_YEAR / (cfg.interest_charge_period as u128);

    //     let ctoken_info = ctoken_info(deps, &cfg)?;
    //     let (borrowed, _) = debt::total(deps.storage)?;
    //     let supplied = ctoken_info.total_supply_base();
    //     let utilisation = utilisation(supplied, borrowed);

    //     let rate = cfg.rates.calculate_interest_rate(utilisation);

    //     let borrower = (Decimal::one() + rate / Uint128::new(charge_periods))
    //         .checked_pow(charge_periods as u32)?
    //         - Decimal::one();
    //     let lender = borrower * utilisation * (Decimal::one() - cfg.reserve_factor);

    //     Ok(ApyResponse { borrower, lender })
    // }
}
