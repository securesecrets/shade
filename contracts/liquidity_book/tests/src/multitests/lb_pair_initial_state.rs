use std::str::FromStr;

use crate::multitests::test_helper::{
    extract_contract_info,
    generate_random,
    token_type_snip20_generator,
    DEFAULT_BASE_FACTOR,
    DEFAULT_BIN_STEP,
    DEFAULT_DECAY_PERIOD,
    DEFAULT_FILTER_PERIOD,
    DEFAULT_MAX_VOLATILITY_ACCUMULATOR,
    DEFAULT_PROTOCOL_SHARE,
    DEFAULT_REDUCTION_FACTOR,
    DEFAULT_VARIABLE_FEE_CONTROL,
    SHADE,
    SSCRT,
};

use super::test_helper::{assert_approx_eq_abs, assert_approx_eq_rel, init_addrs, setup, ID_ONE};
use anyhow::Ok;
use cosmwasm_std::{ContractInfo, Uint128, Uint256};
use shade_multi_test::interfaces::{lb_factory, lb_pair, utils::DeployedContracts};
use shade_protocol::{
    lb_libraries::{math::u24::U24, oracle_helper::MAX_SAMPLE_LIFETIME, types::LBPairInformation},
    multi_test::App,
};

pub fn lb_pair_setup()
-> Result<(App, ContractInfo, DeployedContracts, LBPairInformation), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts) = setup(None)?;

    let shd = extract_contract_info(&deployed_contracts, SHADE)?;
    let sscrt = extract_contract_info(&deployed_contracts, SSCRT)?;

    let token_x = token_type_snip20_generator(&shd)?;
    let token_y = token_type_snip20_generator(&sscrt)?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    )?;
    let all_pairs =
        lb_factory::query_all_lb_pairs(&mut app, &lb_factory.clone().into(), token_x, token_y)?;
    let lb_pair = all_pairs[0].clone();
    Ok((app, lb_factory.into(), deployed_contracts, lb_pair))
}

#[test]
pub fn test_query_factory() -> Result<(), anyhow::Error> {
    let (app, lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let factory_addr = lb_pair::query_factory(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(lb_factory.address, factory_addr);

    Ok(())
}

#[test]
pub fn test_query_token_x() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, deployed_contracts, lb_pair) = lb_pair_setup()?;

    let shd = token_type_snip20_generator(&extract_contract_info(&deployed_contracts, SHADE)?)?;

    let token_x = lb_pair::query_token_x(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(token_x.unique_key(), shd.unique_key());

    Ok(())
}

#[test]
pub fn test_query_token_y() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, deployed_contracts, lb_pair) = lb_pair_setup()?;

    let sscrt = token_type_snip20_generator(&extract_contract_info(&deployed_contracts, SSCRT)?)?;

    let token_y = lb_pair::query_token_y(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(token_y.unique_key(), sscrt.unique_key());

    Ok(())
}

#[test]
pub fn test_query_bin_step() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let bin_step = lb_pair::query_bin_step(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(bin_step, DEFAULT_BIN_STEP);

    Ok(())
}

#[test]
pub fn test_query_bin_reserves() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(reserves_x, 0u128);
    assert_eq!(reserves_y, 0u128);

    Ok(())
}

#[test]
pub fn test_query_active_id() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let active_id = lb_pair::query_active_id(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(active_id, ID_ONE);

    Ok(())
}

#[test]
pub fn test_fuzz_query_bin() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let id = generate_random(0, U24::MAX);

    let (reserves_x, reserves_y) = lb_pair::query_bin(&app, &lb_pair.lb_pair.contract, id)?;

    assert_eq!(reserves_x, 0u128);
    assert_eq!(reserves_y, 0u128);
    Ok(())
}

#[test]
pub fn test_query_next_non_empty_bin() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let id = lb_pair::query_next_non_empty_bin(&app, &lb_pair.lb_pair.contract, false, 0)?;
    assert_eq!(id, 0u32);

    let id = lb_pair::query_next_non_empty_bin(&app, &lb_pair.lb_pair.contract, true, 0)?;
    assert_eq!(id, U24::MAX);

    let id = lb_pair::query_next_non_empty_bin(&app, &lb_pair.lb_pair.contract, false, U24::MAX)?;
    assert_eq!(id, 0u32);

    let id = lb_pair::query_next_non_empty_bin(&app, &lb_pair.lb_pair.contract, true, U24::MAX)?;
    assert_eq!(id, U24::MAX);
    Ok(())
}

#[test]
pub fn test_query_protocol_fees() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let (protocol_fees_x, protocol_fees_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(protocol_fees_x, 0u128);
    assert_eq!(protocol_fees_y, 0u128);
    Ok(())
}

#[test]
pub fn test_query_static_fee_parameters() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let (
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    ) = lb_pair::query_static_fee_params(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(base_factor, DEFAULT_BASE_FACTOR);
    assert_eq!(filter_period, DEFAULT_FILTER_PERIOD);
    assert_eq!(decay_period, DEFAULT_DECAY_PERIOD);
    assert_eq!(reduction_factor, DEFAULT_REDUCTION_FACTOR);
    assert_eq!(variable_fee_control, DEFAULT_VARIABLE_FEE_CONTROL);
    assert_eq!(protocol_share, DEFAULT_PROTOCOL_SHARE);
    assert_eq!(
        max_volatility_accumulator,
        DEFAULT_MAX_VOLATILITY_ACCUMULATOR
    );

    Ok(())
}

#[test]
pub fn test_query_variable_fee_parameters() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let (volatility_accumulator, volatility_reference, id_reference, time_of_last_update) =
        lb_pair::query_variable_fee_params(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(volatility_reference, 0);
    assert_eq!(volatility_accumulator, 0);
    assert_eq!(time_of_last_update, 0);
    assert_eq!(id_reference, ID_ONE);

    Ok(())
}

#[test]
pub fn test_query_oracle_parameters() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let (sample_lifetime, size, active_size, last_updated, first_timestamp) =
        lb_pair::query_oracle_parameters(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(size, 0);
    assert_eq!(active_size, 0);
    assert_eq!(last_updated, 0);
    assert_eq!(first_timestamp, 0);
    assert_eq!(sample_lifetime, MAX_SAMPLE_LIFETIME);

    Ok(())
}

#[test]
pub fn test_query_oracle_sample_at() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let (cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
        lb_pair::query_oracle_sample_at(&app, &lb_pair.lb_pair.contract, 1)?;

    assert_eq!(cumulative_id, 0);
    assert_eq!(cumulative_volatility, 0);
    assert_eq!(cumulative_bin_crossed, 0);

    Ok(())
}

#[test]
pub fn test_query_price_from_id() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;
    let delta = Uint256::from(DEFAULT_BIN_STEP).checked_mul(Uint256::from((5 * 10) ^ 13_u128))?;

    assert_approx_eq_rel(
        lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, 1_000 + ID_ONE)?,
        Uint256::from_str("924521306405372907020063908180274956666")?,
        delta,
        "test_query_id_from_price::1",
    );
    assert_approx_eq_rel(
        lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, ID_ONE - 1_000)?,
        Uint256::from_str("125245452360126660303600960578690115355")?,
        delta,
        "test_query_id_from_price::2",
    );
    assert_approx_eq_rel(
        lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, 10_000 + ID_ONE)?,
        Uint256::from_str("7457860201113570250644758522304565438757805")?,
        delta,
        "test_query_id_from_price::3",
    );
    assert_approx_eq_rel(
        lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, ID_ONE - 10_000)?,
        Uint256::from_str("15526181252368702469753297095319515")?,
        delta,
        "test_query_id_from_price::4",
    );

    assert!(
        lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, ID_ONE + 80_000)?
            < Uint256::from_str(
                "18133092123953330812316154041959812232388892985347108730495479426840526848"
            )?,
        "test_query_id_from_price::5",
    );

    assert!(
        lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, ID_ONE + 80_000)?
            > Uint256::from_str(
                "18096880266539986845478224721407196147811144510344442837666495029900738560"
            )?,
        "test_query_id_from_price::6",
    );

    assert_approx_eq_rel(
        lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, ID_ONE - 80_000)?,
        Uint256::from_str("6392")?,
        Uint256::from(10 ^ 8_u128),
        "test_query_id_from_price::7",
    );

    assert_approx_eq_rel(
        lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, ID_ONE + 12_345)?,
        Uint256::from_str("77718771515321296819382407317364352468140333")?,
        delta,
        "test_query_id_from_price::8",
    );

    assert_approx_eq_rel(
        lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, ID_ONE - 12_345)?,
        Uint256::from_str("1489885737765286392982993705955521")?,
        delta,
        "test_query_id_from_price::9",
    );

    Ok(())
}

#[test]
pub fn test_query_id_from_price() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;
    let delta = Uint256::from(1u128);

    assert_approx_eq_abs(
        Uint256::from(lb_pair::query_id_from_price(
            &app,
            &lb_pair.lb_pair.contract,
            Uint256::from_str("924521306405372907020063908180274956666")?,
        )?),
        Uint256::from(1_000 + ID_ONE),
        delta,
        "test_query_id_from_price::1",
    );

    assert_approx_eq_abs(
        Uint256::from(lb_pair::query_id_from_price(
            &app,
            &lb_pair.lb_pair.contract,
            Uint256::from_str("125245452360126660303600960578690115355")?,
        )?),
        Uint256::from(ID_ONE - 1_000),
        delta,
        "test_query_id_from_price::2",
    );

    assert_approx_eq_abs(
        Uint256::from(lb_pair::query_id_from_price(
            &app,
            &lb_pair.lb_pair.contract,
            Uint256::from_str("7457860201113570250644758522304565438757805")?,
        )?),
        Uint256::from(10_000 + ID_ONE),
        delta,
        "test_query_id_from_price::3",
    );

    assert_approx_eq_abs(
        Uint256::from(lb_pair::query_id_from_price(
            &app,
            &lb_pair.lb_pair.contract,
            Uint256::from_str("15526181252368702469753297095319515")?,
        )?),
        Uint256::from(ID_ONE - 10_000),
        delta,
        "test_query_id_from_price::4",
    );

    assert_approx_eq_abs(
        Uint256::from(lb_pair::query_id_from_price(
            &app,
            &lb_pair.lb_pair.contract,
            Uint256::from_str(
                "18114977146806524168130684952726477124021312024291123319263609183005067158",
            )?,
        )?),
        Uint256::from(ID_ONE + 80_000),
        delta,
        "test_query_id_from_price::5",
    );

    assert_approx_eq_abs(
        Uint256::from(lb_pair::query_id_from_price(
            &app,
            &lb_pair.lb_pair.contract,
            Uint256::from_str("6392")?,
        )?),
        Uint256::from(ID_ONE - 80_000),
        delta,
        "test_query_id_from_price::6",
    );

    assert_approx_eq_abs(
        Uint256::from(lb_pair::query_id_from_price(
            &app,
            &lb_pair.lb_pair.contract,
            Uint256::from_str("77718771515321296819382407317364352468140333")?,
        )?),
        Uint256::from(ID_ONE + 12_345),
        delta,
        "test_query_id_from_price::7",
    );

    assert_approx_eq_abs(
        Uint256::from(lb_pair::query_id_from_price(
            &app,
            &lb_pair.lb_pair.contract,
            Uint256::from_str("1489885737765286392982993705955521")?,
        )?),
        Uint256::from(ID_ONE - 12_345),
        delta,
        "test_query_id_from_price::8",
    );

    Ok(())
}

#[test]
fn test_fuzz_query_swap_out() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let amount_out: Uint128 = Uint128::from(generate_random(0, u128::MAX));
    let swap_for_y: bool = generate_random(0, 1) == 1;

    let (amount_in, amount_out_left, fee) =
        lb_pair::query_swap_in(&app, &lb_pair.lb_pair.contract, amount_out, swap_for_y)?;

    assert_eq!(amount_in.u128(), 0);
    assert_eq!(amount_out_left, amount_out);
    assert_eq!(fee.u128(), 0);

    Ok(())
}

#[test]
fn test_fuzz_query_swap_in() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let amount_in = Uint128::from(generate_random(0, u128::MAX));
    let swap_for_y = generate_random(0, 1) == 1;
    let (amount_out, amount_in_left, fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_in, swap_for_y)?;

    assert_eq!(amount_out.u128(), 0);
    assert_eq!(amount_in_left, amount_in);
    assert_eq!(fee.u128(), 0);

    Ok(())
}
