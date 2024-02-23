use crate::multitests::test_helper::*;

use super::test_helper::{
    increase_allowance_helper,
    init_addrs,
    liquidity_parameters_generator,
    mint_token_helper,
    setup,
    ID_ONE,
};
use anyhow::Ok;
use shade_multi_test::interfaces::{lb_factory, lb_pair, lb_token};
use shade_protocol::{
    c_std::{ContractInfo, Timestamp, Uint128},
    liquidity_book::lb_pair::RewardsDistributionAlgorithm,
};

pub const DEPOSIT_AMOUNT: u128 = 1_000_000_000_000_000_000;
pub const ACTIVE_ID: u32 = ID_ONE;

#[test]
pub fn test_calculate_volume_based_rewards() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _, _) =
        setup(None, Some(RewardsDistributionAlgorithm::VolumeBasedRewards))?;

    let btc = extract_contract_info(&deployed_contracts, SBTC)?;
    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let token_x = token_type_snip20_generator(&btc)?;
    let token_y = token_type_snip20_generator(&silk)?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ACTIVE_ID + 5994,
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

    let deposit_ratio = DEPOSIT_AMOUNT;

    let amount_x = Uint128::from(((deposit_ratio) * 10000_0000) / 40000);
    let amount_y = Uint128::from((deposit_ratio) * 1000_000);

    let nb_bins_x = 10;
    let nb_bins_y = 10;

    let token_x = extract_contract_info(&deployed_contracts, SBTC)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let tokens_to_mint = vec![(SBTC, amount_x), (SILK, amount_y)];

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
        ACTIVE_ID + 5994,
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

    //generate random number
    // let amount_y_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));
    let amount_y_out = amount_y.multiply_ratio(5u128, 10u128).u128();
    // get swap_in for y
    let (amount_x_in, _amount_y_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.info.contract, amount_y_out.into(), true)?;

    // mint the tokens
    let tokens_to_mint = vec![(SBTC, amount_x_in)];
    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    // make a swap with amount_x_in
    let token_x: &ContractInfo = &extract_contract_info(&deployed_contracts, SBTC)?;
    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_x_in,
    )?;

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 600);

    app.set_time(timestamp);

    //generate random number
    // let amount_y_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));
    let amount_x_out = amount_x.multiply_ratio(5u128, 10u128).u128(); // get swap_in for y
    let (amount_y_in, _amount_x_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.info.contract, amount_x_out.into(), false)?;

    // mint the tokens
    let tokens_to_mint = vec![(SILK, amount_y_in)];
    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    // make a swap with amount_x_in
    let token_x: &ContractInfo = &extract_contract_info(&deployed_contracts, SILK)?;
    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_y_in,
    )?;

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?;

    let _distribution = lb_pair::query_rewards_distribution(&app, &lb_pair.info.contract, None)?;
    // println!("Distribution {:?}", _distribution);
    Ok(())
}

#[test]
pub fn test_fuzz_calculate_volume_based_rewards() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _, _) =
        setup(None, Some(RewardsDistributionAlgorithm::VolumeBasedRewards))?;

    let btc = extract_contract_info(&deployed_contracts, SBTC)?;
    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let token_x = token_type_snip20_generator(&btc)?;
    let token_y = token_type_snip20_generator(&silk)?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ACTIVE_ID + 5994,
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

    let deposit_ratio = generate_random(1u128, DEPOSIT_AMOUNT);

    let amount_x = Uint128::from(((deposit_ratio) * 10000_0000) / 40000); // 25_000_000 satoshi
    let amount_y = Uint128::from((deposit_ratio) * 1000_000); // 10_000 silk

    let nb_bins_x = 10;
    let nb_bins_y = 10;

    let token_x = extract_contract_info(&deployed_contracts, SBTC)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let tokens_to_mint = vec![(SBTC, amount_x), (SILK, amount_y)];

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
        ACTIVE_ID + 5994,
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

    //generate random number
    // let amount_y_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));
    let amount_y_out = Uint128::from(generate_random(1u128, amount_y.u128() - 1));
    // get swap_in for y
    let (amount_x_in, _amount_y_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.info.contract, amount_y_out, true)?;

    // mint the tokens
    let tokens_to_mint = vec![(SBTC, amount_x_in)];
    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    // make a swap with amount_x_in
    let token_x: &ContractInfo = &extract_contract_info(&deployed_contracts, SBTC)?;
    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_x_in,
    )?;

    //generate random number
    // let amount_y_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));
    let amount_x_out = Uint128::from(generate_random(1u128, amount_x.u128() - 1)); // get swap_in for y
    let (amount_y_in, _amount_x_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.info.contract, amount_x_out, false)?;

    // mint the tokens
    let tokens_to_mint = vec![(SILK, amount_y_in)];
    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    // make a swap with amount_x_in
    let token_x: &ContractInfo = &extract_contract_info(&deployed_contracts, SILK)?;
    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_y_in,
    )?;

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?;

    let _distribution = lb_pair::query_rewards_distribution(&app, &lb_pair.info.contract, None)?;
    Ok(())
}

#[test]
pub fn test_calculate_time_based_rewards() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _, _) = setup(None, None)?;

    let sscrt = extract_contract_info(&deployed_contracts, SSCRT)?;
    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let token_x = token_type_snip20_generator(&sscrt)?;
    let token_y = token_type_snip20_generator(&silk)?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ACTIVE_ID,
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

    let amount_x = Uint128::from(DEPOSIT_AMOUNT); // 25_000_000 satoshi
    let amount_y = Uint128::from(DEPOSIT_AMOUNT); // 10_000 silk

    let nb_bins_x = 10;
    let nb_bins_y = 10;

    let token_x = extract_contract_info(&deployed_contracts, SSCRT)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let tokens_to_mint = vec![(SSCRT, amount_x), (SILK, amount_y)];

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

    let (bin_reserves_x, bin_reserves_y, _) =
        lb_pair::query_bin_reserves(&app, &lb_pair.info.contract, ACTIVE_ID)?;

    println!(
        "bin_reserves_x: {:?}, bin_reserves_y {:?}",
        bin_reserves_x, bin_reserves_y
    );

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 3);
    app.set_time(timestamp);

    //making a swap for token y hence the bin id moves to the right
    let (amount_x_in, _amount_y_out_left, _fee) = lb_pair::query_swap_in(
        &app,
        &lb_pair.info.contract,
        Uint128::from(bin_reserves_y + 1),
        true,
    )?;

    // mint the tokens
    let tokens_to_mint = vec![(SSCRT, amount_x_in)];
    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    // make a swap with amount_x_in
    let token_x: &ContractInfo = &extract_contract_info(&deployed_contracts, SSCRT)?;
    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_x_in,
    )?;

    let active_id = lb_pair::query_active_id(&app, &lb_pair.info.contract)?;

    assert_eq!(active_id, ACTIVE_ID - 1);

    //making a swap for token y hence the bin id moves to the right
    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 7);
    app.set_time(timestamp);

    let (amount_x_in, _amount_y_out_left, _fee) = lb_pair::query_swap_in(
        &app,
        &lb_pair.info.contract,
        Uint128::from(bin_reserves_y * 5),
        true,
    )?;

    // mint the tokens
    let tokens_to_mint = vec![(SSCRT, amount_x_in)];
    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    // make a swap with amount_x_in
    let token_x: &ContractInfo = &extract_contract_info(&deployed_contracts, SSCRT)?;
    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_x_in,
    )?;

    let active_id = lb_pair::query_active_id(&app, &lb_pair.info.contract)?;

    assert_eq!(active_id, ACTIVE_ID - 1 - 5);

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 43);
    app.set_time(timestamp);

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?;

    let _distribution = lb_pair::query_rewards_distribution(&app, &lb_pair.info.contract, None)?;

    println!("distribution {:?}", _distribution);

    Ok(())
}

#[test]
pub fn test_fuzz_calculate_time_based_rewards() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _, _) = setup(None, None)?;

    let sscrt = extract_contract_info(&deployed_contracts, SSCRT)?;
    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let token_x = token_type_snip20_generator(&sscrt)?;
    let token_y = token_type_snip20_generator(&silk)?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ACTIVE_ID,
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

    let amount_x = Uint128::from(DEPOSIT_AMOUNT); // 25_000_000 satoshi
    let amount_y = Uint128::from(DEPOSIT_AMOUNT); // 10_000 silk

    let nb_bins_x = 10;
    let nb_bins_y = 10;

    let token_x = extract_contract_info(&deployed_contracts, SSCRT)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let tokens_to_mint = vec![(SSCRT, amount_x), (SILK, amount_y)];

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

    let (_, bin_reserves_y, _) =
        lb_pair::query_bin_reserves(&app, &lb_pair.info.contract, ACTIVE_ID)?;

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 3);
    app.set_time(timestamp);

    //making a swap for token y hence the bin id moves to the right
    let (amount_x_in, _amount_y_out_left, _fee) = lb_pair::query_swap_in(
        &app,
        &lb_pair.info.contract,
        Uint128::from(bin_reserves_y + 1),
        true,
    )?;

    // mint the tokens
    let tokens_to_mint = vec![(SSCRT, amount_x_in)];
    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    // make a swap with amount_x_in
    let token_x: &ContractInfo = &extract_contract_info(&deployed_contracts, SSCRT)?;
    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_x_in,
    )?;

    let active_id = lb_pair::query_active_id(&app, &lb_pair.info.contract)?;
    assert_eq!(active_id, ACTIVE_ID - 1);

    //making a swap for token y hence the bin id moves to the right
    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 7);
    app.set_time(timestamp);

    let (amount_x_in, _amount_y_out_left, _fee) = lb_pair::query_swap_in(
        &app,
        &lb_pair.info.contract,
        Uint128::from(bin_reserves_y * 5),
        true,
    )?;

    // mint the tokens
    let tokens_to_mint = vec![(SSCRT, amount_x_in)];
    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    // make a swap with amount_x_in
    let token_x: &ContractInfo = &extract_contract_info(&deployed_contracts, SSCRT)?;
    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_x_in,
    )?;

    let active_id = lb_pair::query_active_id(&app, &lb_pair.info.contract)?;
    assert_eq!(active_id, ACTIVE_ID - 1 - 5);

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 43);
    app.set_time(timestamp);

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?;

    let _distribution = lb_pair::query_rewards_distribution(&app, &lb_pair.info.contract, None)?;
    // println!("_distribution {:?}", _distribution);
    Ok(())
}

#[test]
pub fn test_reset_rewards_config() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _, _) = setup(None, None)?;

    let sscrt = extract_contract_info(&deployed_contracts, SSCRT)?;
    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let token_x = token_type_snip20_generator(&sscrt)?;
    let token_y = token_type_snip20_generator(&silk)?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ACTIVE_ID,
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

    let amount_x = Uint128::from(DEPOSIT_AMOUNT); // 25_000_000 satoshi
    let amount_y = Uint128::from(DEPOSIT_AMOUNT); // 10_000 silk

    let nb_bins_x = 10;
    let nb_bins_y = 10;

    let token_x = extract_contract_info(&deployed_contracts, SSCRT)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let tokens_to_mint = vec![(SSCRT, amount_x), (SILK, amount_y)];

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

    lb_pair::reset_rewards_epoch(
        &mut app,
        addrs.admin().as_str(),
        &lb_pair.info.contract,
        Some(RewardsDistributionAlgorithm::VolumeBasedRewards),
        None,
    )?;

    let (_, bin_reserves_y, _) =
        lb_pair::query_bin_reserves(&app, &lb_pair.info.contract, ACTIVE_ID)?;

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 3);
    app.set_time(timestamp);

    //making a swap for token y hence the bin id moves to the right
    let (amount_x_in, _amount_y_out_left, _fee) = lb_pair::query_swap_in(
        &app,
        &lb_pair.info.contract,
        Uint128::from(bin_reserves_y + 1),
        true,
    )?;

    // mint the tokens
    let tokens_to_mint = vec![(SSCRT, amount_x_in)];
    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    // make a swap with amount_x_in
    let token_x: &ContractInfo = &extract_contract_info(&deployed_contracts, SSCRT)?;
    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_x_in,
    )?;

    let active_id = lb_pair::query_active_id(&app, &lb_pair.info.contract)?;

    assert_eq!(active_id, ACTIVE_ID - 1);

    roll_time(&mut app, Some(100));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?;

    let _distribution = lb_pair::query_rewards_distribution(&app, &lb_pair.info.contract, None)?;
    //Eventhough the distribution was changes mid epoch the effects of change will occur after the epoch.

    assert!(
        _distribution
            .weightages
            .iter()
            .all(|&x| x == _distribution.weightages[0])
    );

    //making a swap for token y hence the bin id moves to the right
    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 7);
    app.set_time(timestamp);

    let (amount_x_in, _amount_y_out_left, _fee) = lb_pair::query_swap_in(
        &app,
        &lb_pair.info.contract,
        Uint128::from(bin_reserves_y * 5),
        true,
    )?;

    // mint the tokens
    let tokens_to_mint = vec![(SSCRT, amount_x_in)];
    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    // make a swap with amount_x_in
    let token_x: &ContractInfo = &extract_contract_info(&deployed_contracts, SSCRT)?;
    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_x_in,
    )?;

    let active_id = lb_pair::query_active_id(&app, &lb_pair.info.contract)?;

    assert_eq!(active_id, ACTIVE_ID - 1 - 5);

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 43);
    app.set_time(timestamp);

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?;

    let _distribution = lb_pair::query_rewards_distribution(&app, &lb_pair.info.contract, None)?;
    //Eventhough the distribution was changes mid epoch the effects of change will occur after the epoch.

    assert!(
        _distribution
            .weightages
            .iter()
            .any(|&x| x != _distribution.weightages[0])
    );

    // println!("_distribution {:?}", _distribution);

    Ok(())
}
