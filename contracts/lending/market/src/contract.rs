#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Coin as StdCoin, Decimal, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;

use utils::wyndex::SwapOperation;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, QueryTotalCreditLine, TotalDebtResponse,
    TransferableAmountResponse,
};
use crate::state::{debt, Config, CONFIG};

use utils::token::Token;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lend-market";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const CTOKEN_INIT_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let ctoken_msg = wynd_lend_token::msg::InstantiateMsg {
        name: "Lent ".to_owned() + &msg.name,
        symbol: "L".to_owned() + &msg.symbol,
        decimals: msg.decimals,
        controller: env.contract.address.to_string(),
        distributed_token: msg.distributed_token.clone(),
    };
    let ctoken_instantiate = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.token_id,
        msg: to_binary(&ctoken_msg)?,
        funds: vec![],
        label: format!("ctoken_contract_{}", env.contract.address),
    };
    debt::init(deps.storage)?;

    let cfg = Config {
        // those will be overwritten in a response
        ctoken_contract: Addr::unchecked(""),
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

    use cw_utils::parse_reply_instantiate_data;

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
        Deposit {} => {
            let received_tokens = require_single_denom(&info.funds)?;
            execute::deposit(deps, env, info.sender.into_string(), received_tokens)
        }
        Withdraw { amount } => execute::withdraw(deps, env, info, amount),
        Borrow { amount } => execute::borrow(deps, env, info, amount),
        Repay {} => {
            let repay_tokens = require_single_denom(&info.funds)?;
            execute::repay(deps, env, repay_tokens, info.sender)
        }
        RepayTo { account } => {
            let account = deps.api.addr_validate(&account)?;
            let repay_tokens = require_single_denom(&info.funds)?;
            execute::repay_to(deps, env, info.sender, repay_tokens, account)
        }
        TransferFrom {
            source,
            destination,
            amount,
            liquidation_price,
        } => {
            let source = deps.api.addr_validate(&source)?;
            let destination = deps.api.addr_validate(&destination)?;
            if liquidation_price == Decimal::zero() {
                return Err(ContractError::ZeroLiquidationPrice {});
            }
            execute::transfer_from(
                deps,
                env,
                info,
                source,
                destination,
                amount,
                liquidation_price,
            )
        }
        AdjustCommonToken { new_token } => {
            execute::adjust_common_token(deps, info.sender, new_token)
        }
        // TODO: should allow cw20?
        SwapWithdrawFrom {
            account,
            buy,
            sell_limit,
            estimate_multiplier,
        } => execute::swap_withdraw_from(
            deps,
            info.sender,
            account,
            buy,
            sell_limit,
            estimate_multiplier,
        ),
        AdjustCollateralRatio { new_ratio } => {
            restricted::adjust_collateral_ratio(deps, info, new_ratio)
        }
        AdjustReserveFactor { new_factor } => {
            restricted::adjust_reserve_factor(deps, info, new_factor)
        }
        AdjustPriceOracle { new_oracle } => restricted::adjust_price_oracle(deps, info, new_oracle),
        AdjustMarketCap { new_cap } => restricted::adjust_market_cap(deps, info, new_cap),
        AdjustInterestRates { new_interest_rates } => {
            restricted::adjust_interest_rates(deps, env, info, new_interest_rates)
        }
        Receive(msg) => execute::receive_cw20_message(deps, env, info, msg),
    }
}

/// Checks if `funds` contains only one denom and return the Coin version of it.
fn require_single_denom(funds: &[StdCoin]) -> Result<utils::coin::Coin, ContractError> {
    if funds.len() != 1 {
        return Err(ContractError::RequiresExactlyOneCoin {});
    };
    Ok(utils::coin::Coin {
        denom: Token::Native(funds[0].denom.clone()),
        amount: funds[0].amount,
    })
}

// Available credit line helpers
mod cr_utils {
    use utils::{
        amount::base_to_token,
        credit_line::{CreditLineResponse, CreditLineValues},
    };

    use crate::interest::query_ctoken_multiplier;

    use super::*;

    use cosmwasm_std::{Deps, DivideByZeroError, Fraction};

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
        let credit: CreditLineResponse = deps.querier.query_wasm_smart(
            &config.credit_agency,
            &QueryTotalCreditLine::TotalCreditLine { account },
        )?;
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
        let credit: CreditLineResponse = deps.querier.query_wasm_smart(
            &config.credit_agency,
            &QueryTotalCreditLine::TotalCreditLine {
                account: account.clone(),
            },
        )?;
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
    use cosmwasm_std::{from_binary, SubMsg};
    use cw20::Cw20ReceiveMsg;
    use utils::{
        amount::{base_to_token, token_to_base},
        coin::Coin,
        wyndex::SimulateSwapOperationsResponse,
    };
    use wyndex_oracle::msg::QueryMsg as OracleQueryMsg;

    use wyndex_oracle::state::Config as OracleConfig;

    use crate::{
        interest::{calculate_interest, epochs_passed, query_ctoken_multiplier, InterestUpdate},
        msg::{CreditAgencyExecuteMsg, ReceiveMsg},
        state::debt,
    };

    use super::*;

    /// Helper struct for return of [`charge_interest`] function
    #[derive(Debug, Clone)]
    pub struct Ratios<T> {
        pub messages: Vec<SubMsg<T>>,
        pub ctoken_ratio: Decimal,
        pub debt_ratio: Decimal,
    }

    impl<T> Ratios<T> {
        pub fn unchanged() -> Self {
            Self {
                messages: vec![],
                ctoken_ratio: Decimal::one(),
                debt_ratio: Decimal::one(),
            }
        }

        pub fn is_unchanged(&self) -> bool {
            self.messages.is_empty()
        }
    }

    /// Function that is supposed to be called before every mint/burn operation.
    /// It calculates ratio for increasing both debt and ctokens
    /// It also mints any amount of outstanding reserve as ltokens to be sent to the gov contract
    /// debt formula:
    /// b_ratio = calculate_interest() * epochs_passed * epoch_length / 31.556.736 (seconds in a year)
    /// ctokens formula:
    /// c_ratio = b_supply() * b_ratio / l_supply()
    /// Up to 2 SubMsgs are returned as a result of this function
    /// One for ctoken rebase and one for the minting of any reserve balance rather than let it sit idle.
    /// The debt multiplier is adjusted inside this function.
    pub fn charge_interest<T>(deps: DepsMut, env: Env) -> Result<Ratios<T>, ContractError> {
        use wynd_lend_token::msg::ExecuteMsg;

        let mut cfg = CONFIG.load(deps.storage)?;
        let epochs_passed = epochs_passed(&cfg, env)?;

        if epochs_passed == 0 {
            return Ok(Ratios {
                messages: vec![],
                ctoken_ratio: Decimal::one(),
                debt_ratio: Decimal::one(),
            });
        }

        cfg.last_charged += epochs_passed * cfg.interest_charge_period;
        CONFIG.save(deps.storage, &cfg)?;

        // If there is an interest update rebase btoken and ctoken and mint reserve to governance
        // contract.
        if let Some(InterestUpdate {
            reserve,
            ctoken_ratio,
            debt_ratio,
        }) = calculate_interest(deps.as_ref(), epochs_passed)?
        {
            debt::rebase(deps.storage, debt_ratio + Decimal::one())?;

            let ctoken_rebase = to_binary(&ExecuteMsg::Rebase {
                ratio: ctoken_ratio + Decimal::one(),
            })?;
            let cwrapped = SubMsg::new(WasmMsg::Execute {
                contract_addr: cfg.ctoken_contract.to_string(),
                msg: ctoken_rebase,
                funds: vec![],
            });
            let mut messages = vec![cwrapped];
            // If we have a reserve, rather than leave it sitting idle,
            // mint the reserve as ltokens and send them to the governance contract
            if reserve > Uint128::zero() {
                let mint_msg = to_binary(&wynd_lend_token::msg::ExecuteMsg::MintBase {
                    recipient: cfg.governance_contract.to_string(),
                    amount: reserve,
                })?;
                let wrapped_msg = SubMsg::new(WasmMsg::Execute {
                    contract_addr: cfg.ctoken_contract.to_string(),
                    msg: mint_msg,
                    funds: vec![],
                });

                messages.push(wrapped_msg);
            }

            Ok(Ratios {
                messages,
                ctoken_ratio: ctoken_ratio + Decimal::one(),
                debt_ratio: debt_ratio + Decimal::one(),
            })
        } else {
            Ok(Ratios::unchanged())
        }
    }

    // Register the account into Credit Agency as a depositor.
    fn enter_market<T>(cfg: &Config, account: &Addr) -> StdResult<SubMsg<T>> {
        let msg = to_binary(&CreditAgencyExecuteMsg::EnterMarket {
            account: account.to_string(),
        })?;

        Ok(SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.credit_agency.to_string(),
            msg,
            funds: vec![],
        }))
    }

    /// Handler for `ExecuteMsg::Deposit`
    /// This function checks the validity of sent funds and if they increase the deposit over the
    /// max allowed. Both native and cw20 tokens are managed.
    pub fn deposit(
        mut deps: DepsMut,
        env: Env,
        address: String,
        received_tokens: utils::coin::Coin,
    ) -> Result<Response, ContractError> {
        let address = deps.api.addr_validate(&address)?;
        let cfg = CONFIG.load(deps.storage)?;
        if received_tokens.denom != cfg.market_token {
            return Err(ContractError::InvalidDenom(cfg.market_token.to_string()));
        }

        let mut response = Response::new();

        // Check if funds sent increase total deposit over max cap in terms of base token.
        if let Some(cap) = cfg.market_cap {
            let ctoken_info = query::ctoken_info(deps.as_ref(), &cfg)?;
            let ctoken_base_supply =
                token_to_base(ctoken_info.total_supply, ctoken_info.multiplier);
            if ctoken_base_supply + received_tokens.amount > cap {
                return Err(ContractError::DepositOverCap {
                    attempted_deposit: received_tokens.amount,
                    ctoken_base_supply,
                    cap,
                });
            }
        }

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps.branch(), env)?;
        if !charge_msgs.is_unchanged() {
            response = response.add_submessages(charge_msgs.messages);
        }

        let mint_msg = to_binary(&wynd_lend_token::msg::ExecuteMsg::MintBase {
            recipient: address.to_string(),
            amount: received_tokens.amount,
        })?;
        let wrapped_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.ctoken_contract.to_string(),
            msg: mint_msg,
            funds: vec![],
        });

        response = response
            .add_attribute("action", "deposit")
            .add_attribute("sender", address.to_string())
            .add_submessage(wrapped_msg)
            .add_submessage(enter_market(&cfg, &address)?);
        Ok(response)
    }

    /// Handler for `ExecuteMsg::Withdraw`
    pub fn withdraw(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;

        if cr_utils::transferable_amount(deps.as_ref(), &cfg, &info.sender)? < amount {
            return Err(ContractError::CannotWithdraw {
                account: info.sender.to_string(),
                amount,
            });
        }

        let mut response = Response::new();

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps.branch(), env)?;
        if !charge_msgs.is_unchanged() {
            response = response.add_submessages(charge_msgs.messages);
        }

        // Burn the C tokens
        let burn_msg = to_binary(&wynd_lend_token::msg::ExecuteMsg::BurnBaseFrom {
            owner: info.sender.to_string(),
            amount,
        })?;
        let wrapped_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.ctoken_contract.to_string(),
            msg: burn_msg,
            funds: vec![],
        });

        // Send the base assets from contract to lender
        let send_msg = cfg.market_token.send_msg(&info.sender, amount)?;

        response = response
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender)
            .add_submessage(wrapped_msg)
            .add_message(send_msg);
        Ok(response)
    }

    /// Handler for `ExecuteMsg::Borrow`
    pub fn borrow(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;

        if !cr_utils::can_borrow(deps.as_ref(), &cfg, &info.sender, amount)? {
            return Err(ContractError::CannotBorrow {
                amount,
                account: info.sender.to_string(),
            });
        }

        let mut response = Response::new();

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps.branch(), env)?;
        if !charge_msgs.is_unchanged() {
            response = response.add_submessages(charge_msgs.messages);
        }

        debt::increase(deps.storage, &info.sender, amount)?;

        // Sent tokens to sender's account
        let send_msg = cfg.market_token.send_msg(&info.sender, amount)?;

        response = response
            .add_attribute("action", "borrow")
            .add_attribute("sender", info.sender.clone())
            .add_submessage(enter_market(&cfg, &info.sender)?)
            .add_message(send_msg);
        Ok(response)
    }

    /// Handler for `ExecuteMsg::Repay`
    /// Repay allows to send btokens to the contract to burn them and receive back previously
    /// deposited market tokens. If more tokens are sent to repay the debt, these are sent back to
    /// the sender.
    pub fn repay(
        mut deps: DepsMut,
        env: Env,
        repay_tokens: utils::coin::Coin,
        sender: Addr,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        if repay_tokens.denom != cfg.market_token {
            return Err(ContractError::InvalidDenom(cfg.market_token.to_string()));
        }

        // Create rebase messages for tokens based on interest and supply
        let charge_msgs = charge_interest(deps.branch(), env)?;

        let mut response = Response::new();
        if !charge_msgs.is_unchanged() {
            response = response.add_submessages(charge_msgs.messages);
        }

        let send_back = debt::decrease(deps.storage, &sender, repay_tokens.amount)?;

        response = response
            .add_attribute("action", "repay")
            .add_attribute("sender", sender.clone());

        // Return surplus of sent tokens
        if !send_back.is_zero() {
            let bank_msg = cfg.market_token.send_msg(sender, send_back)?;
            response = response.add_message(bank_msg);
        }

        Ok(response)
    }

    /// Handler for `ExecuteMsg::RepayTo`
    /// Allows to repay account's debt for for both native and cw20 tokens. Requires sender to be a
    /// Credit Agency, otherwise fails.
    pub fn repay_to(
        mut deps: DepsMut,
        env: Env,
        sender: Addr,
        repay_tokens: utils::coin::Coin,
        account: Addr,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        if cfg.credit_agency != sender {
            return Err(ContractError::RequiresCreditAgency {});
        }
        if repay_tokens.denom != cfg.market_token {
            return Err(ContractError::InvalidDenom(cfg.market_token.to_string()));
        }

        let debt = debt::of(deps.storage, &account)?;
        // if account has less debt then caller wants to pay off, liquidation fails
        if repay_tokens.amount > debt {
            return Err(ContractError::LiquidationInsufficientDebt {
                account: account.to_string(),
                debt,
            });
        }

        let mut response = Response::new();

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps.branch(), env)?;
        if !charge_msgs.is_unchanged() {
            response = response.add_submessages(charge_msgs.messages);
        }

        debt::decrease(deps.storage, &account, repay_tokens.amount)?;

        response = response
            .add_attribute("action", "repay_to")
            .add_attribute("sender", sender)
            .add_attribute("debtor", account);
        Ok(response)
    }

    /// Handler for `ExecuteMsg::TransferFrom`
    /// Requires sender to be a Credit Agency, otherwise fails
    /// Amount must be in common denom (from CA)
    pub fn transfer_from(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        source: Addr,
        destination: Addr,
        amount: Uint128,
        liquidation_price: Decimal,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        if cfg.credit_agency != info.sender {
            return Err(ContractError::RequiresCreditAgency {});
        }

        let mut response = Response::new();

        // charge interests before transferring tokens
        let charge_msgs = charge_interest(deps.branch(), env)?;
        if !charge_msgs.is_unchanged() {
            response = response.add_submessages(charge_msgs.messages);
        }

        // calculate repaid value
        let price_rate = query::price_market_local_per_common(deps.as_ref())?.rate_sell_per_buy;

        let repaid_value = cr_utils::divide(amount, price_rate * liquidation_price)
            .map_err(|_| ContractError::ZeroPrice {})?;

        // transfer claimed amount of repaid value in ctokens from account source to destination
        // using base message here, since the rebase messages from `charge_interest` are not applied yet,
        // so the multiplier is not updated yet
        let msg = to_binary(&wynd_lend_token::msg::ExecuteMsg::TransferBaseFrom {
            sender: source.to_string(),
            recipient: destination.to_string(),
            amount: repaid_value,
        })?;
        let transfer_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.ctoken_contract.to_string(),
            msg,
            funds: vec![],
        });

        response = response
            .add_submessage(enter_market(&cfg, &destination)?)
            .add_attribute("action", "transfer_from")
            .add_attribute("from", source)
            .add_attribute("to", destination)
            .add_submessage(transfer_msg);
        Ok(response)
    }

    /// Handler for `ExecuteMsg::AdjustCommonToken`
    pub fn adjust_common_token(
        deps: DepsMut,
        sender: Addr,
        new_token: Token,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;

        if sender != cfg.credit_agency {
            return Err(ContractError::Unauthorized {});
        }

        cfg.common_token = new_token;

        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new())
    }

    /// Returns the addrress associated to the pool for the two given `Token`s.
    // fn query_pool_address(
    //     deps: Deps,
    //     cfg: &Config,
    //     denom1: Token,
    //     denom2: Token,
    // ) -> Result<Addr, ContractError> {
    //     let pool_addr: Addr = deps.querier.query_wasm_smart(
    //         cfg.price_oracle.clone(),
    //         &OracleQueryMsg::PoolAddress {
    //             first_asset: denom1.into(),
    //             second_asset: denom2.into(),
    //         },
    //     )?;
    //     Ok(pool_addr)
    // }

    pub fn swap_withdraw_from(
        deps: DepsMut,
        sender: Addr,
        account: String,
        buy: Coin,
        sell_limit: Uint128,
        estimate_multiplier: Decimal,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        if cfg.credit_agency != sender {
            return Err(ContractError::RequiresCreditAgency {});
        }

        // estimate multiplier must be at least 1.0, otherwise swap won't succeed
        if estimate_multiplier < Decimal::one() {
            return Err(ContractError::InvalidEstimateMultiplier {});
        }

        let send_msg = buy.denom.send_msg(sender, buy.amount)?;

        // if swap is between same denoms, only send tokens
        if cfg.market_token == buy.denom {
            return Ok(Response::new().add_message(send_msg));
        }

        // variable to store the swap to perform.
        let mut operations: Vec<SwapOperation> = vec![];

        // If the market token is the common token we can directly swap it for the buy token.
        if cfg.market_token == cfg.common_token {
            operations.push(SwapOperation::WyndexSwap {
                offer_asset_info: cfg.common_token.clone().into(),
                ask_asset_info: buy.denom.into(),
            })
        // If buy token is the common token we can directly swap it with the market token.
        } else if cfg.common_token == buy.denom {
            operations.push(SwapOperation::WyndexSwap {
                offer_asset_info: cfg.market_token.clone().into(),
                ask_asset_info: buy.denom.into(),
            })
        // Else we need two swaps.
        } else {
            operations.extend(vec![
                SwapOperation::WyndexSwap {
                    offer_asset_info: cfg.market_token.clone().into(),
                    ask_asset_info: cfg.common_token.clone().into(),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: cfg.common_token.clone().into(),
                    ask_asset_info: buy.denom.into(),
                },
            ])
        };

        // Compute an estimate of market tokens required to complete the buy.
        let swap_response: SimulateSwapOperationsResponse = deps.querier.query_wasm_smart(
            &cfg.price_oracle,
            &OracleQueryMsg::SimulateReverseSwapOperations {
                ask_amount: buy.amount,
                operations: operations.clone(),
            },
        )?;

        // Add a margin
        let estimate = swap_response.amount * estimate_multiplier;

        if estimate > sell_limit {
            return Err(ContractError::EstimateHigherThanLimit {
                estimate,
                sell_limit,
            });
        }

        // Burn the C tokens based on the estimate.
        let multiplier = query_ctoken_multiplier(deps.as_ref(), &cfg)?;
        let burn_msg: Binary = to_binary(&wynd_lend_token::msg::ExecuteMsg::BurnFrom {
            owner: account,
            amount: base_to_token(estimate, multiplier),
        })?;
        let burn_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.ctoken_contract.to_string(),
            msg: burn_msg,
            funds: vec![],
        });

        let oracle_config: OracleConfig = deps
            .querier
            .query_wasm_smart(cfg.price_oracle, &OracleQueryMsg::Config {})?;

        let swap_msg = cfg.market_token.swap_msg(
            oracle_config.multi_hop,
            operations,
            Some(buy.amount),
            estimate,
        )?;

        Ok(Response::new()
            .add_submessage(burn_msg)
            .add_message(swap_msg)
            .add_message(send_msg))
    }

    pub fn receive_cw20_message(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Cw20ReceiveMsg,
    ) -> Result<Response, ContractError> {
        use ReceiveMsg::*;
        // TODO: make functions accept or Addr or String
        match from_binary(&msg.msg)? {
            Deposit => deposit(
                deps,
                env,
                msg.sender,
                utils::coin::Coin {
                    denom: Token::Cw20(info.sender.to_string()),
                    amount: msg.amount,
                },
            ),
            Repay => {
                let sender = deps.api.addr_validate(msg.sender.as_str())?;
                repay(
                    deps,
                    env,
                    utils::coin::Coin {
                        denom: Token::Cw20(info.sender.to_string()),
                        amount: msg.amount,
                    },
                    sender,
                )
            }
            RepayTo { account } => {
                let account = deps.api.addr_validate(account.as_str())?;
                let sender = deps.api.addr_validate(msg.sender.as_str())?;
                repay_to(
                    deps,
                    env,
                    sender,
                    utils::coin::Coin {
                        denom: Token::Cw20(info.sender.to_string()),
                        amount: msg.amount,
                    },
                    account,
                )
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;
    let res = match msg {
        Configuration {} => to_binary(&query::config(deps, env)?)?,
        TokensBalance { account } => to_binary(&query::tokens_balance(deps, env, account)?)?,
        TransferableAmount { token, account } => {
            let token = deps.api.addr_validate(&token)?;
            to_binary(&query::transferable_amount(deps, token, account)?)?
        }
        Withdrawable { account } => to_binary(&query::withdrawable(deps, env, account)?)?,
        Borrowable { account } => to_binary(&query::borrowable(deps, env, account)?)?,
        Interest {} => to_binary(&query::interest(deps)?)?,
        PriceMarketLocalPerCommon {} => to_binary(&query::price_market_local_per_common(deps)?)?,
        CreditLine { account } => {
            let account = deps.api.addr_validate(&account)?;
            to_binary(&query::credit_line(deps, env, account)?)?
        }
        Reserve {} => to_binary(&query::reserve(deps, env)?)?,
        Apy {} => to_binary(&query::apy(deps)?)?,
        TotalDebt {} => {
            let (total, multiplier) = debt::total(deps.storage)?;
            to_binary(&TotalDebtResponse { total, multiplier })?
        }
    };
    Ok(res)
}

mod query {
    use super::*;

    use cosmwasm_std::{Decimal, Deps, Uint128};
    use cw20::BalanceResponse;
    use utils::coin::Coin;
    use utils::credit_line::{CreditLineResponse, CreditLineValues};
    use utils::price::{coin_times_price_rate, PriceRate};
    use wynd_lend_token::msg::{QueryMsg as TokenQueryMsg, TokenInfoResponse};
    use wyndex::oracle::TwapResponse;
    use wyndex_oracle::msg::QueryMsg as OracleQueryMsg;

    use crate::interest::{calculate_interest, epochs_passed, utilisation};
    use crate::msg::{ApyResponse, InterestResponse, ReserveResponse, TokensBalanceResponse};
    use crate::state::{debt, SECONDS_IN_YEAR};

    fn token_balance(
        deps: Deps,
        token_contract: &Addr,
        address: String,
    ) -> StdResult<BalanceResponse> {
        deps.querier
            .query_wasm_smart(token_contract, &TokenQueryMsg::Balance { address })
    }

    fn base_balance(
        deps: Deps,
        token_contract: &Addr,
        address: String,
    ) -> StdResult<BalanceResponse> {
        deps.querier
            .query_wasm_smart(token_contract, &TokenQueryMsg::BaseBalance { address })
    }

    pub fn ctoken_balance(
        deps: Deps,
        config: &Config,
        account: impl ToString,
    ) -> Result<Coin, ContractError> {
        Ok(config
            .market_token
            .amount(token_balance(deps, &config.ctoken_contract, account.to_string())?.balance))
    }

    pub fn ctoken_base_balance(
        deps: Deps,
        config: &Config,
        account: impl ToString,
    ) -> Result<Coin, ContractError> {
        Ok(config
            .market_token
            .amount(base_balance(deps, &config.ctoken_contract, account.to_string())?.balance))
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

    /// Handler for `QueryMsg::TransferableAmount`
    pub fn transferable_amount(
        deps: Deps,
        token: Addr,
        account: String,
    ) -> Result<TransferableAmountResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if token == config.ctoken_contract {
            let transferable = cr_utils::transferable_amount(deps, &config, account)?;
            Ok(TransferableAmountResponse { transferable })
        } else {
            Err(ContractError::UnrecognisedToken(token.to_string()))
        }
    }

    /// Handler for `QueryMsg::Withdrawable`
    pub fn withdrawable(deps: Deps, env: Env, account: String) -> Result<Coin, ContractError> {
        use std::cmp::min;

        let cfg = CONFIG.load(deps.storage)?;

        let transferable = cr_utils::transferable_amount(deps, &cfg, &account)?;
        let ctoken_balance = ctoken_base_balance(deps, &cfg, &account)?;
        let allowed_to_withdraw = min(transferable, ctoken_balance.amount);
        let withdrawable = min(
            allowed_to_withdraw,
            cfg.market_token
                .query_balance(deps, env.contract.address)?
                .into(),
        );

        Ok(cfg.market_token.amount(withdrawable))
    }

    /// Handler for `QueryMsg::Borrowable`
    pub fn borrowable(deps: Deps, env: Env, account: String) -> Result<Coin, ContractError> {
        use std::cmp::min;

        let cfg = CONFIG.load(deps.storage)?;

        let borrowable = cr_utils::query_borrowable_tokens(deps, &cfg, account)?;
        let borrowable = min(
            borrowable,
            cfg.market_token
                .query_balance(deps, env.contract.address)?
                .into(),
        );

        Ok(cfg.market_token.amount(borrowable))
    }

    pub fn ctoken_info(deps: Deps, config: &Config) -> Result<TokenInfoResponse, ContractError> {
        crate::interest::ctoken_info(deps, config)
    }

    /// Handler for `QueryMsg::Interest`
    pub fn interest(deps: Deps) -> Result<InterestResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let ctoken_info = ctoken_info(deps, &config)?;

        let supplied = ctoken_info.total_supply_base();
        let (borrowed, _) = debt::total(deps.storage)?;
        let utilisation = utilisation(supplied, borrowed);

        let interest = config.rates.calculate_interest_rate(utilisation);

        Ok(InterestResponse {
            interest,
            utilisation,
            charge_period: Timestamp::from_seconds(config.interest_charge_period),
        })
    }

    /// Handler for `QueryMsg::PriceMarketLocalPerCommon`
    /// Returns the ratio of the twap of the market token over the common token.
    pub fn price_market_local_per_common(deps: Deps) -> Result<PriceRate, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        // If tokens are the same, just return 1:1.
        // TODO: we should add more test to check that return one is fine. There could be
        // situation with small unbalances inside pools. When unbalances are with a mean around 1
        // this could be fine, but in cases where this is not true, it could be a problem.
        if config.common_token == config.market_token {
            Ok(PriceRate {
                sell_denom: config.market_token.clone(),
                buy_denom: config.common_token,
                rate_sell_per_buy: Decimal::one(),
            })
        } else {
            let price_response: TwapResponse = deps.querier.query_wasm_smart(
                config.price_oracle.clone(),
                &OracleQueryMsg::Twap {
                    offer: config.market_token.clone().into(),
                    ask: config.common_token.clone().into(),
                },
            )?;
            Ok(PriceRate {
                sell_denom: config.market_token,
                buy_denom: config.common_token,
                rate_sell_per_buy: price_response.a_per_b,
            })
        }
    }

    /// Handler for `QueryMsg::CreditLine`
    /// Returns the debt and credit situation of the `account` after applying interests.
    pub fn credit_line(
        deps: Deps,
        env: Env,
        account: Addr,
    ) -> Result<CreditLineResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let mut collateral = ctoken_base_balance(deps, &config, &account)?;
        let mut debt = Coin {
            denom: config.market_token.clone(),
            amount: debt::of(deps.storage, &account)?,
        };

        // Simulate charging interest for any periods `charge_interest` wasn't called for yet
        if let Some(update) = calculate_interest(deps, epochs_passed(&config, env)?)? {
            collateral.amount += collateral.amount * update.ctoken_ratio;
            debt.amount += debt.amount * update.debt_ratio;
        }

        if collateral.amount.is_zero() && debt.amount.is_zero() {
            return Ok(CreditLineValues::zero().make_response(config.common_token));
        }

        let price_ratio = price_market_local_per_common(deps)?;
        let collateral = coin_times_price_rate(&collateral, &price_ratio)?;
        let debt = coin_times_price_rate(&debt, &price_ratio)?.amount;
        let credit_line = collateral.amount * config.collateral_ratio;
        let borrow_limit = credit_line * config.borrow_limit_ratio;
        Ok(
            CreditLineValues::new(collateral.amount, credit_line, borrow_limit, debt)
                .make_response(config.common_token),
        )
    }

    /// Handler for `QueryMsg::Reserve`
    pub fn reserve(deps: Deps, env: Env) -> Result<ReserveResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        let reserve = calculate_interest(deps, epochs_passed(&config, env)?)?
            .map(|update| update.reserve)
            .unwrap_or(Uint128::zero());

        Ok(ReserveResponse { reserve })
    }

    /// Handler for `QueryMsg::Apy`
    pub fn apy(deps: Deps) -> Result<ApyResponse, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        let charge_periods = SECONDS_IN_YEAR / (cfg.interest_charge_period as u128);

        let ctoken_info = ctoken_info(deps, &cfg)?;
        let (borrowed, _) = debt::total(deps.storage)?;
        let supplied = ctoken_info.total_supply_base();
        let utilisation = utilisation(supplied, borrowed);

        let rate = cfg.rates.calculate_interest_rate(utilisation);

        let borrower = (Decimal::one() + rate / Uint128::new(charge_periods))
            .checked_pow(charge_periods as u32)?
            - Decimal::one();
        let lender = borrower * utilisation * (Decimal::one() - cfg.reserve_factor);

        Ok(ApyResponse { borrower, lender })
    }
}

mod restricted {
    use super::*;

    use utils::interest::Interest;

    pub fn ensure_governance(cfg: &Config, info: &MessageInfo) -> Result<(), ContractError> {
        if cfg.governance_contract != info.sender {
            return Err(ContractError::Unauthorized {});
        }
        Ok(())
    }

    pub fn adjust_collateral_ratio(
        deps: DepsMut,
        info: MessageInfo,
        new_ratio: Decimal,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        ensure_governance(&cfg, &info)?;
        cfg.collateral_ratio = new_ratio;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new())
    }

    pub fn adjust_reserve_factor(
        deps: DepsMut,
        info: MessageInfo,
        new_factor: Decimal,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        ensure_governance(&cfg, &info)?;
        cfg.reserve_factor = new_factor;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new())
    }

    pub fn adjust_price_oracle(
        deps: DepsMut,
        info: MessageInfo,
        new_oracle: String,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        ensure_governance(&cfg, &info)?;
        cfg.price_oracle = new_oracle;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new())
    }

    pub fn adjust_market_cap(
        deps: DepsMut,
        info: MessageInfo,
        new_cap: Option<Uint128>,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        ensure_governance(&cfg, &info)?;
        cfg.market_cap = new_cap;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new())
    }

    pub fn adjust_interest_rates(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        new_interest_rates: Interest,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        ensure_governance(&cfg, &info)?;
        let charge_msgs = execute::charge_interest(deps.branch(), env)?;
        let mut response = Response::new();
        if !charge_msgs.is_unchanged() {
            response = response.add_submessages(charge_msgs.messages);
        }
        let interest_rates = new_interest_rates.validate()?;
        cfg.rates = interest_rates;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(response)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    CONFIG.update::<_, StdError>(deps.storage, |mut cfg| {
        if let Some(token_id) = msg.wynd_lend_token_id {
            cfg.token_id = token_id;
        }
        Ok(cfg)
    })?;

    Ok(Response::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divide_u128_by_decimal_rounding() {
        assert_eq!(
            cr_utils::divide(60u128.into(), Decimal::percent(60)).unwrap(),
            Uint128::new(100u128)
        );
    }
}
