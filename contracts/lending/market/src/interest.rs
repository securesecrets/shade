use crate::state::debt;
use cosmwasm_std::{Decimal, Deps, Env, Fraction, Uint128};
use lend_token::msg::TokenInfoResponse;
use lending_utils::amount::token_to_base;

use crate::{
    state::{Config, CONFIG, SECONDS_IN_YEAR, VIEWING_KEY},
    ContractError,
};

/// Values that should be updated when interest is charged for all pending charge periods
pub struct InterestUpdate {
    /// The new RESERVE value
    pub reserve: Uint128,
    /// The ratio to rebase CTokens by
    pub ctoken_ratio: Decimal,
    /// The ratio to rebase debt by
    pub debt_ratio: Decimal,
}

/// Returns how many charging periods happened between now and last charge.
pub fn epochs_passed(cfg: &Config, env: Env) -> Result<u64, ContractError> {
    Ok((env.block.time.seconds() - cfg.last_charged) / cfg.interest_charge_period)
}

/// Calculates new values after applying all pending interest charges
pub fn calculate_interest(
    deps: Deps,
    epochs_passed: u64,
) -> Result<Option<InterestUpdate>, ContractError> {
    // Adapted from the compound interest formula: https://en.wikipedia.org/wiki/Compound_interest
    fn compounded_interest_rate(
        interest_rate: Decimal,
        charge_period: u64,
        epochs_passed: u64,
    ) -> Result<Decimal, ContractError> {
        // The interest rate per charge period
        let scaled_interest_rate = Decimal::from_ratio(
            Uint128::from(charge_period) * interest_rate.numerator(),
            Uint128::from(SECONDS_IN_YEAR) * interest_rate.denominator(),
        );
        Ok(
            (Decimal::one() + scaled_interest_rate).checked_pow(epochs_passed as u32)?
                - Decimal::one(),
        )
    }

    if epochs_passed == 0 {
        return Ok(None);
    }

    let cfg = CONFIG.load(deps.storage)?;

    let ctoken_info = ctoken_info(deps, &cfg)?;

    let supplied = token_to_base(ctoken_info.total_supply, ctoken_info.multiplier);
    let (borrowed, _) = debt::total(deps.storage)?;

    // safety - if there are no ctokens, don't charge interest (would panic later)
    if supplied == Uint128::zero() {
        return Ok(None);
    }

    let interest = cfg
        .rates
        .calculate_interest_rate(utilisation(supplied, borrowed));
    let debt_ratio = compounded_interest_rate(interest, cfg.interest_charge_period, epochs_passed)?;

    // Add to reserve only portion of money charged here
    let charged_interest = debt_ratio * borrowed;
    let reserve = cfg.reserve_factor * charged_interest;

    // remember to add old reserve balance into supplied tokens
    let base_asset_balance = supplied - borrowed;

    let c_supply = borrowed + base_asset_balance - reserve;

    // lMul = b_supply() * ratio / c_supply
    let ctoken_ratio: Decimal = Decimal::from_ratio(borrowed * debt_ratio, c_supply);

    Ok(Some(InterestUpdate {
        reserve,
        ctoken_ratio,
        debt_ratio,
    }))
}

/// Figure out the current utilisation given the amount supplied and borrowed (in base tokens)
pub fn utilisation(supplied: Uint128, borrowed: Uint128) -> Decimal {
    if supplied.is_zero() {
        Decimal::zero()
    } else {
        Decimal::from_ratio(borrowed, supplied)
    }
}

pub fn ctoken_info(deps: Deps, config: &Config) -> Result<TokenInfoResponse, ContractError> {
    let ctoken_contract = &config.ctoken_contract;
    Ok(deps
        .querier
        .query_wasm_smart(ctoken_contract, &lend_token::msg::QueryMsg::TokenInfo {})?)
}

pub fn query_ctoken_multiplier(deps: Deps, cfg: &Config) -> Result<Decimal, ContractError> {
    let resp: lend_token::msg::MultiplierResponse = deps.querier.query_wasm_smart(
        cfg.ctoken_contract.clone(),
        &lend_token::QueryMsg::Multiplier {},
    )?;
    Ok(resp.multiplier)
}
