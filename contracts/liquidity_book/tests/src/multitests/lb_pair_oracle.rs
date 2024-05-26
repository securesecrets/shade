use crate::multitests::test_helper::*;
use anyhow::Ok;
use cosmwasm_std::{Timestamp, Uint128};
use lb_libraries::oracle_helper::MAX_SAMPLE_LIFETIME;
use serial_test::serial;
use shade_multi_test::interfaces::{
    lb_factory, lb_pair, lb_token, snip20, utils::DeployedContracts,
};
use shade_protocol::{
    c_std::ContractInfo, liquidity_book::lb_pair::LBPairInformation, multi_test::App,
};
use std::ops::Sub;

pub const DEPOSIT_AMOUNT: u128 = 100_000_000u128;
pub const ACTIVE_ID: u32 = ID_ONE;

pub fn lb_pair_setup(
) -> Result<(App, ContractInfo, DeployedContracts, LBPairInformation), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _, _) = setup(None, None)?;

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

    let lb_token = lb_pair::query_lb_token(&app, &lb_pair.info.contract)?;

    lb_token::set_viewing_key(
        &mut app,
        addrs.batman().as_str(),
        &lb_token,
        "viewing_key".to_owned(),
    )?;

    let amount_x = Uint128::from(DEPOSIT_AMOUNT);
    let amount_y = Uint128::from(DEPOSIT_AMOUNT);
    let nb_bins_x = 50;
    let nb_bins_y = 50;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SSCRT)?;

    let tokens_to_mint = vec![(SHADE, amount_x), (SSCRT, amount_y)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;

    increase_allowance_helper(
        &mut app,
        &deployed_contracts,
        addrs.batman().into_string(),
        lb_pair.info.contract.address.to_string(),
        tokens_to_mint,
    )?;

    //Adding liquidity
    let liquidity_parameters = liquidity_parameters_generator(
        &deployed_contracts,
        ACTIVE_ID,
        token_x,
        token_y,
        amount_x,
        amount_y,
        nb_bins_x,
        nb_bins_y,
    )?;

    lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        liquidity_parameters,
    )?;

    Ok((app, lb_factory.into(), deployed_contracts, lb_pair))
}

#[test]
#[serial]
pub fn test_query_oracle_parameters() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let (sample_lifetime, size, last_updated, first_timestamp) =
        lb_pair::query_oracle_parameters(&app, &lb_pair.info.contract)?;

    assert_eq!(size, u16::MAX);
    assert_eq!(last_updated, app.block_info().time.seconds());
    assert_eq!(first_timestamp, app.block_info().time.seconds());
    assert_eq!(sample_lifetime, MAX_SAMPLE_LIFETIME);

    Ok(())
}

#[test]
#[serial]
pub fn test_query_oracle_sample_at_init() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair) = lb_pair_setup()?;

    let (
        cumulative_id,
        cumulative_volatility,
        cumulative_bin_crossed,
        cumulative_volume_x,
        cumulative_volume_y,
        cumulative_fee_x,
        cumulative_fee_y,
        oracle_id,
        cumulative_txns,
        lifetime,
        created_at,
    ) = lb_pair::query_oracle_sample_at(&app, &lb_pair.info.contract, 1)?;

    assert_eq!(cumulative_id, 0);
    assert_eq!(cumulative_volatility, 0);
    assert_eq!(cumulative_bin_crossed, 0);
    assert_eq!(cumulative_volume_x, 0);
    assert_eq!(cumulative_volume_y, 0);
    assert_eq!(cumulative_fee_x, 0);
    assert_eq!(cumulative_fee_y, 0);
    assert_eq!(oracle_id, 1);
    assert_eq!(cumulative_txns, 0);
    assert_eq!(lifetime, 0);
    assert_eq!(created_at, app.block_info().time.seconds());

    Ok(())
}

#[test]
#[serial]
pub fn test_query_oracle_sample_at_one_swap() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair) = lb_pair_setup()?;

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 5);
    app.set_time(timestamp);

    let swap_amount = DEPOSIT_AMOUNT / 50;

    //Make a swap
    let amount_out = Uint128::from(swap_amount);

    let (amount_in, amount_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.info.contract, amount_out, false)?;
    assert_eq!(amount_out_left, Uint128::zero());

    let tokens_to_mint = vec![(SSCRT, amount_in)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;

    let token_y = &extract_contract_info(&deployed_contracts, SSCRT)?;

    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_y,
        amount_in,
    )?;

    let shd_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance, amount_out);

    let silk_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SSCRT,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, Uint128::zero());

    //Check the sample 1 then
    let (
        _cumulative_id,
        cumulative_volatility,
        cumulative_bin_crossed,
        cumulative_volume_x,
        cumulative_volume_y,
        cumulative_fee_x,
        cumulative_fee_y,
        oracle_id,
        cumulative_txns,
        lifetime,
        created_at,
    ) = lb_pair::query_oracle_sample_at(&app, &lb_pair.info.contract, 1)?;

    //     assert_eq!(cumulative_id as u32, ACTIVE_ID); // only one bin
    assert_eq!(cumulative_volatility, 0); // no movment in bins
    assert_eq!(cumulative_bin_crossed, 0);
    assert_eq!(cumulative_volume_x, swap_amount);
    assert_eq!(cumulative_volume_y, amount_in.u128());
    assert_eq!(cumulative_fee_x, 0);
    assert_eq!(cumulative_fee_y, amount_in.u128() - swap_amount);
    assert_eq!(oracle_id, 1);
    assert_eq!(cumulative_txns, 1);
    assert_eq!(lifetime, 5);
    assert_eq!(created_at, app.block_info().time.seconds().sub(5));

    Ok(())
}

#[test]
#[serial]
pub fn test_fuzz_query_oracle_sample_at_one_swap() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair) = lb_pair_setup()?;

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 1);
    app.set_time(timestamp);

    let swap_amount = generate_random(1u128, DEPOSIT_AMOUNT);

    //Make a swap
    let amount_out = Uint128::from(swap_amount);

    let (amount_in, amount_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.info.contract, amount_out, false)?;
    assert_eq!(amount_out_left, Uint128::zero());

    let tokens_to_mint = vec![(SSCRT, amount_in)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;

    let token_y = &extract_contract_info(&deployed_contracts, SSCRT)?;

    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_y,
        amount_in,
    )?;

    //Check the sample 1 then
    let (
        _cumulative_id,
        _cumulative_volatility,
        _cumulative_bin_crossed,
        cumulative_volume_x,
        cumulative_volume_y,
        cumulative_fee_x,
        cumulative_fee_y,
        oracle_id,
        cumulative_txns,
        _lifetime,
        _created_at,
    ) = lb_pair::query_oracle_sample_at(&app, &lb_pair.info.contract, 1)?;

    assert_eq!(cumulative_volume_x, swap_amount);
    assert_eq!(cumulative_volume_y, amount_in.u128());
    assert_eq!(cumulative_fee_x, 0);
    assert!(cumulative_fee_y > 0);
    assert_eq!(oracle_id, 1);
    assert_eq!(cumulative_txns, 1);

    Ok(())
}

#[test]
#[serial]
pub fn test_fuzz_update_oracle_id() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair) = lb_pair_setup()?;

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 1);
    app.set_time(timestamp);

    let swap_amount = generate_random(1u128, DEPOSIT_AMOUNT / 100);

    //Make a swap
    let amount_out: Uint128 = Uint128::from(swap_amount);

    let (amount_in, amount_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.info.contract, amount_out, false)?;
    assert_eq!(amount_out_left, Uint128::zero());

    let tokens_to_mint = vec![(SSCRT, amount_in)];

    let mut cumm_swap_amount = swap_amount;
    let mut cumm_amount_in = amount_in;

    for i in 1..100 {
        mint_token_helper(
            &mut app,
            &deployed_contracts,
            &addrs,
            addrs.batman().into_string(),
            tokens_to_mint.clone(),
        )?;

        let token_y = &extract_contract_info(&deployed_contracts, SSCRT)?;

        lb_pair::swap_snip_20(
            &mut app,
            addrs.batman().as_str(),
            &lb_pair.info.contract,
            Some(addrs.batman().to_string()),
            token_y,
            amount_in,
        )?;

        //Check the sample 1 then
        let (
            _cumulative_id,
            _cumulative_volatility,
            _cumulative_bin_crossed,
            _cumulative_volume_x,
            _cumulative_volume_y,
            cumulative_fee_x,
            cumulative_fee_y,
            oracle_id,
            cumulative_txns,
            _lifetime,
            _created_at,
        ) = lb_pair::query_oracle_sample_at(&app, &lb_pair.info.contract, i)?;

        //   assert_eq!(cumulative_id as u32, ACTIVE_ID); // only one bin
        //   assert!(cumulative_bin_crossed > 0);
        //   assert_eq!(cumulative_volume_x, cumm_swap_amount);
        //   assert_eq!(cumulative_volume_y, cumm_amount_in.u128());
        assert_eq!(cumulative_fee_x, 0);
        assert!(cumulative_fee_y > 0);
        assert_eq!(oracle_id, i);
        assert_eq!(cumulative_txns, 1);

        let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 121);
        app.set_time(timestamp);
        cumm_swap_amount += swap_amount;
        cumm_amount_in += amount_in;
    }
    Ok(())
}

#[test]
#[serial]
pub fn test_fuzz_update_cumm_txns() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair) = lb_pair_setup()?;

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 1);
    app.set_time(timestamp);

    let swap_amount = generate_random(1u128, DEPOSIT_AMOUNT / 100);

    //Make a swap
    let amount_out: Uint128 = Uint128::from(swap_amount);

    let (amount_in, amount_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.info.contract, amount_out, false)?;
    assert_eq!(amount_out_left, Uint128::zero());

    let tokens_to_mint = vec![(SSCRT, amount_in)];

    let mut cumm_amount_in = amount_in;

    for i in 1..100 {
        mint_token_helper(
            &mut app,
            &deployed_contracts,
            &addrs,
            addrs.batman().into_string(),
            tokens_to_mint.clone(),
        )?;

        let token_y = &extract_contract_info(&deployed_contracts, SSCRT)?;

        lb_pair::swap_snip_20(
            &mut app,
            addrs.batman().as_str(),
            &lb_pair.info.contract,
            Some(addrs.batman().to_string()),
            token_y,
            amount_in,
        )?;

        //Check the sample 1 then
        let (
            _cumulative_id,
            _cumulative_volatility,
            _cumulative_bin_crossed,
            _cumulative_volume_x,
            _cumulative_volume_y,
            _cumulative_fee_x,
            _cumulative_fee_y,
            oracle_id,
            cumulative_txns,
            _lifetime,
            _created_at,
        ) = lb_pair::query_oracle_sample_at(&app, &lb_pair.info.contract, 1)?;

        assert_eq!(oracle_id, 1);
        assert_eq!(cumulative_txns, i);

        cumm_amount_in += amount_in;
        let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 1);
        app.set_time(timestamp);
    }
    Ok(())
}

#[test]
#[serial]
pub fn test_fuzz_query_oracle_sample_after() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair) = lb_pair_setup()?;

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 1);
    app.set_time(timestamp);

    let swap_amount = generate_random(1u128, DEPOSIT_AMOUNT / 70000);

    //Make a swap
    let amount_out: Uint128 = Uint128::from(swap_amount);

    let (amount_in, amount_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.info.contract, amount_out, false)?;
    assert_eq!(amount_out_left, Uint128::zero());

    let tokens_to_mint = vec![(SSCRT, amount_in)];

    let mut cumm_amount_in = amount_in;

    for _ in 0..1 {
        mint_token_helper(
            &mut app,
            &deployed_contracts,
            &addrs,
            addrs.batman().into_string(),
            tokens_to_mint.clone(),
        )?;

        let token_y = &extract_contract_info(&deployed_contracts, SSCRT)?;

        lb_pair::swap_snip_20(
            &mut app,
            addrs.batman().as_str(),
            &lb_pair.info.contract,
            Some(addrs.batman().to_string()),
            token_y,
            amount_in,
        )?;

        cumm_amount_in += amount_in;
        let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 121);
        app.set_time(timestamp);
    }

    let responses = lb_pair::query_oracle_sample_after(&app, &lb_pair.info.contract, 1)?;
    let mut oracle_id = 1;
    for res in responses {
        assert_eq!(res.cumulative_txns, 1);
        assert_eq!(res.oracle_id, oracle_id);
        oracle_id += 1;
    }
    for _ in 0..100 {
        mint_token_helper(
            &mut app,
            &deployed_contracts,
            &addrs,
            addrs.batman().into_string(),
            tokens_to_mint.clone(),
        )?;

        let token_y = &extract_contract_info(&deployed_contracts, SSCRT)?;

        lb_pair::swap_snip_20(
            &mut app,
            addrs.batman().as_str(),
            &lb_pair.info.contract,
            Some(addrs.batman().to_string()),
            token_y,
            amount_in,
        )?;

        cumm_amount_in += amount_in;
        let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 121);
        app.set_time(timestamp);
    }

    let responses = lb_pair::query_oracle_sample_after(&app, &lb_pair.info.contract, oracle_id)?;

    for res in responses {
        assert_eq!(res.cumulative_txns, 1);
        assert_eq!(res.oracle_id, oracle_id);
        oracle_id += 1;
    }
    Ok(())
}
