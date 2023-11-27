use anyhow::Ok;
use serial_test::serial;
use shade_multi_test::{
    interfaces::{lb_factory, lb_pair, snip20},
    multi::{admin::init_admin_auth, lb_pair::LbPair, lb_staking::LbStaking, lb_token::LbToken},
};
use shade_protocol::{
    c_std::{ContractInfo, StdError},
    lb_libraries::{
        constants::BASIS_POINT_MAX,
        math::{
            encoded_sample::{MASK_UINT12, MASK_UINT20},
            u24::U24,
        },
    },
    liquidity_book::{lb_factory::PresetResponse, lb_pair::RewardsDistributionAlgorithm},
    swap::core::TokenType,
    utils::MultiTestable,
};

use crate::multitests::test_helper::*;

#[test]
#[serial]
pub fn test_setup() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, _deployed_contracts) = setup(None, None)?;
    //query fee recipient
    let fee_recipient = lb_factory::query_fee_recipient(&mut app, &lb_factory.clone().into())?;

    assert_eq!(fee_recipient.as_str(), addrs.joker().as_str());

    //query getMinBinStep
    let min_bin_step = lb_factory::query_min_bin_step(&mut app, &lb_factory.clone().into())?;
    assert_eq!(min_bin_step, 1u8); // fixed in contract

    Ok(())
}

#[test]
#[serial]
pub fn test_set_lb_pair_implementation() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, _deployed_contracts) = setup(None, None)?;
    let lb_pair_stored_code = app.store_code(LbPair::default().contract());

    lb_factory::set_lb_pair_implementation(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        lb_pair_stored_code.code_id,
        lb_pair_stored_code.code_hash,
    )?;
    let lb_pair_code_info = lb_factory::query_lb_pair_implementation(&mut app, &lb_factory.into())?;
    assert_eq!(lb_pair_stored_code.code_id, lb_pair_code_info.id);

    Ok(())
}

#[test]
#[serial]
pub fn test_revert_set_lb_pair_implementation() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, _deployed_contracts) = setup(None, None)?;
    let lb_pair_stored_code = app.store_code(LbPair::default().contract());

    lb_factory::set_lb_pair_implementation(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        lb_pair_stored_code.code_id,
        lb_pair_stored_code.code_hash.clone(),
    )?;
    let err = lb_factory::set_lb_pair_implementation(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.into(),
        lb_pair_stored_code.code_id,
        lb_pair_stored_code.code_hash,
    );

    assert_eq!(
        err.unwrap_err(),
        StdError::generic_err(format!(
            "LB implementation is already set to code ID {}!",
            lb_pair_stored_code.code_id
        ))
    );

    Ok(())
}

#[test]
#[serial]
pub fn test_set_lb_token_implementation() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, _deployed_contracts) = setup(None, None)?;
    let lb_token_stored_code = app.store_code(LbToken::default().contract());
    lb_factory::set_lb_token_implementation(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        lb_token_stored_code.code_id,
        lb_token_stored_code.code_hash,
    )?;
    let lb_token_code_info =
        lb_factory::query_lb_token_implementation(&mut app, &lb_factory.into())?;
    assert_eq!(lb_token_stored_code.code_id, lb_token_code_info.id);
    Ok(())
}

#[test]
#[serial]
pub fn test_create_lb_pair() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts) = setup(None, None)?;

    // 3. Create an LBPair.

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

    // 4. Check if the number of LBPairs is 1.
    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.clone(),
        token_y.clone(),
    )?;

    assert_eq!(all_pairs.len(), 1);
    // 5. Get the LBPair information using `factory.getLBPairInformation`.
    let lb_pair_info = lb_factory::query_lb_pair_information(
        &mut app,
        &lb_factory.into(),
        token_x.clone(),
        token_y.clone(),
        DEFAULT_BIN_STEP,
    )?;

    assert_eq!(lb_pair_info, all_pairs[0]);
    assert_eq!(lb_pair_info.bin_step, DEFAULT_BIN_STEP);
    assert!(lb_pair_info.created_by_owner);
    assert!(!lb_pair_info.ignored_for_routing);

    // 6. Validate that the returned information matches the expected values (binStep, LBPair address, createdByOwner, ignoredForRouting).
    //SORTED token
    if token_x.unique_key() < token_y.unique_key() {
        assert_eq!(lb_pair_info.info.token_x, token_x);
        assert_eq!(lb_pair_info.info.token_y, token_y);
    } else {
        assert_eq!(lb_pair_info.info.token_x, token_y);
        assert_eq!(lb_pair_info.info.token_y, token_x);
    }

    let (
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    ) = lb_pair::query_static_fee_params(&app, &lb_pair_info.info.contract)?;
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

    let (volatility_accumulator, volatility_reference, id_reference, time_of_last_update) =
        lb_pair::query_variable_fee_params(&app, &lb_pair_info.info.contract)?;

    assert_eq!(volatility_reference, 0);
    assert_eq!(volatility_accumulator, 0);
    assert_eq!(time_of_last_update, 0);
    assert_eq!(id_reference, ID_ONE);

    Ok(())
}

#[test]
#[serial]
pub fn test_create_lb_pair_factory_unlocked() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts) = setup(None, None)?;

    let shd = extract_contract_info(&deployed_contracts, SHADE)?;
    let sscrt = extract_contract_info(&deployed_contracts, SSCRT)?;
    let token_x = token_type_snip20_generator(&shd)?;
    let token_y = token_type_snip20_generator(&sscrt)?;
    // try creating as 'batman' and get an error
    let res = lb_factory::create_lb_pair(
        &mut app,
        addrs.batman().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    );
    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err(format!(
            "Preset {} is locked for users! {} is not the owner!",
            DEFAULT_BIN_STEP,
            addrs.batman().as_str()
        ))
    );
    // set open preset for bin_id
    lb_factory::set_preset_open_state(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        true,
    )?;
    // create lb_pair
    lb_factory::create_lb_pair(
        &mut app,
        addrs.batman().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    )?;
    // query and check if created by owner == false
    let lb_pair_info = lb_factory::query_lb_pair_information(
        &mut app,
        &lb_factory.clone().into(),
        token_x.clone(),
        token_y.clone(),
        DEFAULT_BIN_STEP,
    )?;
    assert!(!lb_pair_info.created_by_owner);

    // close preset
    // set open preset for bin_id
    lb_factory::set_preset_open_state(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        false,
    )?;
    // get an error on creating
    let res = lb_factory::create_lb_pair(
        &mut app,
        addrs.scare_crow().as_str(),
        &lb_factory.into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x,
        token_y,
        "viewing_key".to_string(),
        "entropy".to_string(),
    );
    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err(format!(
            "Preset {} is locked for users! {} is not the owner!",
            DEFAULT_BIN_STEP,
            addrs.scare_crow().as_str()
        ))
    );
    Ok(())
}

#[test]
#[serial]
fn test_revert_create_lb_pair() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts) = setup(None, None)?;

    let shd = extract_contract_info(&deployed_contracts, SHADE)?;
    let sscrt = extract_contract_info(&deployed_contracts, SSCRT)?;
    let token_x = token_type_snip20_generator(&shd)?;
    let token_y = token_type_snip20_generator(&sscrt)?;
    //Batmam tried to create a lb_pair
    let res = lb_factory::create_lb_pair(
        &mut app,
        addrs.batman().as_str(),
        &lb_factory.into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    );
    //Check failing error
    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err(format!(
            "Preset {} is locked for users! {} is not the owner!",
            DEFAULT_BIN_STEP,
            addrs.batman().as_str()
        ))
    );

    let admin_contract = init_admin_auth(&mut app, &addrs.admin());

    let new_lb_factory = lb_factory::init(
        &mut app,
        addrs.admin().as_str(),
        addrs.admin(),
        admin_contract.into(),
        addrs.admin(),
    )?;

    //can't create a pair if the preset is not set
    let res = lb_factory::create_lb_pair(
        &mut app,
        addrs.batman().as_str(),
        &new_lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x,
        token_y,
        "viewing_key".to_string(),
        "entropy".to_string(),
    );
    //Check failing error
    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err(format!("Bin step {} has no preset!", DEFAULT_BIN_STEP,))
    );

    lb_factory::set_pair_preset(
        &mut app,
        addrs.admin().as_str(),
        &new_lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        DEFAULT_BASE_FACTOR,
        DEFAULT_FILTER_PERIOD,
        DEFAULT_DECAY_PERIOD,
        DEFAULT_REDUCTION_FACTOR,
        DEFAULT_VARIABLE_FEE_CONTROL,
        DEFAULT_PROTOCOL_SHARE,
        DEFAULT_MAX_VOLATILITY_ACCUMULATOR,
        DEFAULT_OPEN_STATE,
        DEFAULT_TOTAL_REWARD_BINS,
        Some(RewardsDistributionAlgorithm::TimeBasedRewards),
        1,
        100,
        None,
    )?;

    //can't create a pair if quote asset is not whitelisted
    let sbtc = extract_contract_info(&deployed_contracts, SBTC)?;
    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let token_x = token_type_snip20_generator(&sbtc)?;
    let token_y = token_type_snip20_generator(&silk)?;
    let res = lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &new_lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    );
    //Check failing error
    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err(format!(
            "Quote Asset {} is not whitelisted!",
            token_y.unique_key(),
        ))
    );

    lb_factory::add_quote_asset(
        &mut app,
        addrs.admin().as_str(),
        &new_lb_factory.clone().into(),
        TokenType::CustomToken {
            contract_addr: silk.address,
            token_code_hash: silk.code_hash,
        },
    )?;
    lb_factory::add_quote_asset(
        &mut app,
        addrs.admin().as_str(),
        &new_lb_factory.clone().into(),
        TokenType::CustomToken {
            contract_addr: sbtc.address.clone(),
            token_code_hash: sbtc.code_hash,
        },
    )?;
    //can't create a pair if quote asset is the same as the base asset
    let res = lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &new_lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x.clone(),
        token_x.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    );
    //Check failing error
    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err(format!(
            "Tokens are identical! Both addresses are {}!",
            sbtc.address.as_str()
        ))
    );
    //can't create a pair if the implementation is not set
    let res = lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &new_lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    );
    //Check failing error
    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err("The LBPair implementation has not been set yet!")
    );

    //can't create a pair the same pair twice
    let lb_token_stored_code = app.store_code(LbToken::default().contract());
    let lb_pair_stored_code = app.store_code(LbPair::default().contract());
    let staking_contract = app.store_code(LbStaking::default().contract());

    lb_factory::set_lb_pair_implementation(
        &mut app,
        addrs.admin().as_str(),
        &new_lb_factory.clone().into(),
        lb_pair_stored_code.code_id,
        lb_pair_stored_code.code_hash,
    )?;

    lb_factory::set_lb_token_implementation(
        &mut app,
        addrs.admin().as_str(),
        &new_lb_factory.clone().into(),
        lb_token_stored_code.code_id,
        lb_token_stored_code.code_hash,
    )?;

    lb_factory::set_staking_contract_implementation(
        &mut app,
        addrs.admin().as_str(),
        &new_lb_factory.clone().into(),
        staking_contract.code_id,
        staking_contract.code_hash,
    )?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &new_lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    )?;

    let res = lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &new_lb_factory.into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err(format!(
            "LBPair ({}, {}, bin_step: {}) already exists!",
            token_x.unique_key(),
            token_y.unique_key(),
            DEFAULT_BIN_STEP
        ))
    );

    Ok(())
}

#[test]
#[serial]
fn test_fuzz_set_preset() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, _deployed_contracts) = setup(None, None)?;
    let mut bin_step: u16 = generate_random(0, u16::MAX);
    let base_factor: u16 = generate_random(0, u16::MAX);
    let mut filter_period: u16 = generate_random(0, u16::MAX);
    let mut decay_period: u16 = generate_random(0, u16::MAX);
    let mut reduction_factor: u16 = generate_random(0, u16::MAX);
    let mut variable_fee_control: u32 = generate_random(0, U24::MAX);
    let mut protocol_share: u16 = generate_random(0, u16::MAX);
    let mut max_volatility_accumulator: u32 = generate_random(0, U24::MAX);
    let is_open: bool = generate_random(0, 1) == 0;

    let min_bin_step = lb_factory::query_min_bin_step(&mut app, &lb_factory.clone().into())?;

    // Bounds checking for each parameter
    bin_step = bound(bin_step, min_bin_step as u16, u16::MAX);
    filter_period = bound(filter_period, 0, MASK_UINT12.as_u16() - 1);
    decay_period = bound(decay_period, filter_period + 1, MASK_UINT12.as_u16());
    reduction_factor = bound(reduction_factor, 0u16, BASIS_POINT_MAX);
    variable_fee_control = bound(variable_fee_control, 0u32, BASIS_POINT_MAX as u32);
    protocol_share = bound(protocol_share, 0, 2_500);
    max_volatility_accumulator = bound(max_volatility_accumulator, 0, MASK_UINT20.as_u32());

    lb_factory::set_pair_preset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        bin_step,
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
        is_open,
        DEFAULT_TOTAL_REWARD_BINS,
        Some(RewardsDistributionAlgorithm::TimeBasedRewards),
        1,
        100,
        None,
    )?;

    // Additional assertions and verifications
    let all_bin_steps = lb_factory::query_all_bin_steps(&mut app, &lb_factory.clone().into())?;

    if bin_step != DEFAULT_BIN_STEP {
        assert_eq!(all_bin_steps.len(), 2);
        assert_eq!(all_bin_steps[0], DEFAULT_BIN_STEP);
        assert_eq!(all_bin_steps[1], bin_step);
    } else {
        assert_eq!(all_bin_steps.len(), 1);
        assert_eq!(all_bin_steps[0], bin_step);
    }

    let PresetResponse {
        base_factor: base_factor_view,
        filter_period: filter_period_view,
        decay_period: decay_period_view,
        reduction_factor: reduction_factor_view,
        variable_fee_control: variable_fee_control_view,
        protocol_share: protocol_share_view,
        max_volatility_accumulator: max_volatility_accumulator_view,
        is_open: is_open_view,
    } = lb_factory::query_preset(&mut app, &lb_factory.into(), bin_step)?;

    assert_eq!(base_factor, base_factor_view);
    assert_eq!(filter_period, filter_period_view);
    assert_eq!(decay_period, decay_period_view);
    assert_eq!(reduction_factor, reduction_factor_view);
    assert_eq!(variable_fee_control, variable_fee_control_view);
    assert_eq!(protocol_share, protocol_share_view);
    assert_eq!(max_volatility_accumulator, max_volatility_accumulator_view);
    assert_eq!(is_open, is_open_view);

    Ok(())
}

#[test]
#[serial]
fn test_remove_preset() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, _deployed_contracts) = setup(None, None)?;

    // Set presets
    lb_factory::set_pair_preset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP + 1,
        DEFAULT_BASE_FACTOR,
        DEFAULT_FILTER_PERIOD,
        DEFAULT_DECAY_PERIOD,
        DEFAULT_REDUCTION_FACTOR,
        DEFAULT_VARIABLE_FEE_CONTROL,
        DEFAULT_PROTOCOL_SHARE,
        DEFAULT_MAX_VOLATILITY_ACCUMULATOR,
        DEFAULT_OPEN_STATE,
        DEFAULT_TOTAL_REWARD_BINS,
        Some(RewardsDistributionAlgorithm::TimeBasedRewards),
        1,
        100,
        None,
    )?;

    lb_factory::set_pair_preset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP - 1,
        DEFAULT_BASE_FACTOR,
        DEFAULT_FILTER_PERIOD,
        DEFAULT_DECAY_PERIOD,
        DEFAULT_REDUCTION_FACTOR,
        DEFAULT_VARIABLE_FEE_CONTROL,
        DEFAULT_PROTOCOL_SHARE,
        DEFAULT_MAX_VOLATILITY_ACCUMULATOR,
        DEFAULT_OPEN_STATE,
        DEFAULT_TOTAL_REWARD_BINS,
        Some(RewardsDistributionAlgorithm::TimeBasedRewards),
        1,
        100,
        None,
    )?;

    let all_bin_steps = lb_factory::query_all_bin_steps(&mut app, &lb_factory.clone().into())?;
    assert_eq!(all_bin_steps.len(), 3, "test_remove_preset::1");

    // Remove preset
    lb_factory::remove_preset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
    )?;

    let all_bin_steps = lb_factory::query_all_bin_steps(&mut app, &lb_factory.clone().into())?;
    assert_eq!(all_bin_steps.len(), 2, "test_remove_preset::2");

    // Expect getPreset to fail for removed bin_step
    let res = lb_factory::query_preset(&mut app, &lb_factory.clone().into(), DEFAULT_BIN_STEP);
    assert!(
        res.is_err(),
        "Should revert because bin_step no longer exists"
    );

    // Expect failure if not owner
    let res = lb_factory::remove_preset(
        &mut app,
        addrs.batman().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
    );
    assert!(res.is_err(), "Should revert because not owner");

    // Expect failure if bin_step does not exist
    let res = lb_factory::remove_preset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.into(),
        DEFAULT_BIN_STEP,
    );
    assert!(
        res.is_err(),
        "Should revert because bin_step no longer exists"
    );

    Ok(())
}

#[test]
#[serial]
pub fn test_set_fees_parameters_on_pair() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts) = setup(None, None)?;

    let sscrt = extract_contract_info(&deployed_contracts, SSCRT)?;
    let shd = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_x = token_type_snip20_generator(&sscrt)?;
    let token_y = token_type_snip20_generator(&shd)?;
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
    // let liquidity_parameters = liquidity_parameters_generator(
    //     &deployed_contracts,
    //     ID_ONE,
    //     Uint128::from(100u128 * 10 ^ 18),
    //     Uint128::from(100u128 * 10 ^ 18),
    //     10,
    //     10,
    // )?;

    // 4. Check if the number of LBPairs is 1.
    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.clone(),
        token_y.clone(),
    )?;

    // lb_pair::add_liquidity(
    //     &mut app,
    //     &lb_factory.address.as_str(),
    //     &all_pairs[0].lb_pair.contract,
    //     liquidity_parameters,
    // )?;

    let (
        old_volatility_accumulator,
        old_volatility_reference,
        old_id_reference,
        old_time_of_last_update,
    ) = lb_pair::query_variable_fee_params(&app, &all_pairs[0].info.contract)?;

    // Set the fees parameters on pair
    lb_factory::set_fees_parameters_on_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        token_x.clone(),
        token_y.clone(),
        DEFAULT_BIN_STEP,
        DEFAULT_BASE_FACTOR * 2,
        DEFAULT_FILTER_PERIOD * 2,
        DEFAULT_DECAY_PERIOD * 2,
        DEFAULT_REDUCTION_FACTOR * 2,
        DEFAULT_VARIABLE_FEE_CONTROL * 2,
        DEFAULT_PROTOCOL_SHARE * 2,
        DEFAULT_MAX_VOLATILITY_ACCUMULATOR * 2,
    )?;

    // Validate that the fees parameters were correctly set
    let (
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    ) = lb_pair::query_static_fee_params(&app, &all_pairs[0].info.contract)?;

    assert_eq!(base_factor, DEFAULT_BASE_FACTOR * 2);
    assert_eq!(filter_period, DEFAULT_FILTER_PERIOD * 2);
    assert_eq!(decay_period, DEFAULT_DECAY_PERIOD * 2);
    assert_eq!(reduction_factor, DEFAULT_REDUCTION_FACTOR * 2);
    assert_eq!(variable_fee_control, DEFAULT_VARIABLE_FEE_CONTROL * 2);
    assert_eq!(protocol_share, DEFAULT_PROTOCOL_SHARE * 2);
    assert_eq!(
        max_volatility_accumulator,
        DEFAULT_MAX_VOLATILITY_ACCUMULATOR * 2
    );

    let (volatility_accumulator, volatility_reference, id_reference, time_of_last_update) =
        lb_pair::query_variable_fee_params(&app, &all_pairs[0].info.contract)?;

    assert_eq!(volatility_accumulator, old_volatility_accumulator);
    assert_eq!(volatility_reference, old_volatility_reference);
    assert_eq!(id_reference, old_id_reference);
    assert_eq!(time_of_last_update, old_time_of_last_update);

    // Simulate invalid operations (not the owner, pair not exist)

    // Set the fees parameters on pair
    let res = lb_factory::set_fees_parameters_on_pair(
        &mut app,
        addrs.batman().as_str(),
        &lb_factory.into(),
        token_x,
        token_y,
        DEFAULT_BIN_STEP,
        DEFAULT_BASE_FACTOR * 2,
        DEFAULT_FILTER_PERIOD * 2,
        DEFAULT_DECAY_PERIOD * 2,
        DEFAULT_REDUCTION_FACTOR * 2,
        DEFAULT_VARIABLE_FEE_CONTROL * 2,
        DEFAULT_PROTOCOL_SHARE * 2,
        DEFAULT_MAX_VOLATILITY_ACCUMULATOR * 2,
    );

    assert!(res.is_err());

    Ok(())
}

#[test]
#[serial]
pub fn test_set_fee_recipient() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, _deployed_contracts) = setup(None, None)?;
    lb_factory::set_fee_recipient(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        addrs.batman(),
    )?;

    // Assert fee recipient is set correctly
    let fee_recipient = lb_factory::query_fee_recipient(&mut app, &lb_factory.clone().into())?;
    assert_eq!(
        fee_recipient,
        addrs.batman().as_str(),
        "test_set_fee_recipient::1"
    );

    // Try setting fee recipient when not the owner, should revert
    let err = lb_factory::set_fee_recipient(
        &mut app,
        addrs.scare_crow().as_str(),
        &lb_factory.clone().into(),
        addrs.scare_crow(),
    );
    assert!(err.is_err());

    // Try setting fee recipient to the same recipient, should revert
    let err = lb_factory::set_fee_recipient(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.into(),
        addrs.batman(),
    );
    assert!(err.is_err(),);

    Ok(())
}

#[test]
#[serial]
pub fn test_fuzz_open_presets() -> Result<(), anyhow::Error> {
    let addrs = init_addrs(); // Initialize addresses
    let (mut app, lb_factory, _deployed_contracts) = setup(None, None)?; // Setup

    let min_bin_step = lb_factory::query_min_bin_step(&mut app, &lb_factory.clone().into())?;
    let max_bin_step = u16::MAX;

    let bin_step: u16 = generate_random(min_bin_step as u16, max_bin_step);

    // Presets are not open to the public by default
    if bin_step == DEFAULT_BIN_STEP {
        let PresetResponse { is_open, .. } =
            lb_factory::query_preset(&mut app, &lb_factory.clone().into(), bin_step)?;
        assert!(!is_open, "test_fuzz_open_presets::1");
    } else {
        let err = lb_factory::query_preset(&mut app, &lb_factory.clone().into(), bin_step);
        assert_eq!(
            err.unwrap_err(),
            StdError::generic_err(
                StdError::GenericErr {
                    msg: format!(
                        "Querier contract error: Bin step {} has no preset!",
                        bin_step
                    )
                }
                .to_string()
            )
        );
    }

    lb_factory::set_pair_preset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        bin_step,
        DEFAULT_BASE_FACTOR,
        DEFAULT_FILTER_PERIOD,
        DEFAULT_DECAY_PERIOD,
        DEFAULT_REDUCTION_FACTOR,
        DEFAULT_VARIABLE_FEE_CONTROL,
        DEFAULT_PROTOCOL_SHARE,
        DEFAULT_MAX_VOLATILITY_ACCUMULATOR,
        true,
        DEFAULT_TOTAL_REWARD_BINS,
        Some(RewardsDistributionAlgorithm::TimeBasedRewards),
        1,
        100,
        None,
    )?;
    let PresetResponse { is_open, .. } =
        lb_factory::query_preset(&mut app, &lb_factory.clone().into(), bin_step)?;
    assert!(is_open, "test_fuzz_open_presets::2");

    // Can't set to the same state
    let err = lb_factory::set_preset_open_state(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        bin_step,
        true,
    );
    assert_eq!(
        err.unwrap_err(),
        StdError::generic_err("Preset open state is already in the same state!")
    );

    // Can be closed
    lb_factory::set_preset_open_state(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        bin_step,
        false,
    )?;

    let PresetResponse { is_open, .. } =
        lb_factory::query_preset(&mut app, &lb_factory.clone().into(), bin_step)?;
    assert!(!is_open, "test_fuzz_open_presets::3");

    // Can't open if not the owner
    let err = lb_factory::set_preset_open_state(
        &mut app,
        addrs.batman().as_str(),
        &lb_factory.clone().into(),
        bin_step,
        true,
    );
    assert!(err.is_err(),);

    // Can't set to the same state
    let err = lb_factory::set_preset_open_state(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.into(),
        bin_step,
        false,
    );
    assert!(err.is_err(),);

    Ok(())
}

#[test]
#[serial]
pub fn test_add_quote_asset() -> Result<(), anyhow::Error> {
    let addrs = init_addrs(); // Initialize addresses
    let (mut app, lb_factory, mut deployed_contracts) = setup(None, None)?; // Setup

    let num_quote_assets_before =
        lb_factory::query_number_of_quote_assets(&mut app, &lb_factory.clone().into())?;
    println!("Before: {num_quote_assets_before}");

    let num_quote_assets =
        lb_factory::query_number_of_quote_assets(&mut app, &lb_factory.clone().into())?;
    println!("check: {num_quote_assets}");

    let _num_quote_assets =
        lb_factory::query_number_of_quote_assets(&mut app, &lb_factory.clone().into())?;

    snip20::init(
        &mut app,
        addrs.admin().as_str(),
        &mut deployed_contracts,
        "SOMOS",
        "SOMOS",
        8,
        Some(shade_protocol::snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: None,
            enable_redeem: None,
            enable_mint: Some(true),
            enable_burn: None,
            enable_transfer: Some(true),
        }),
    )
    .unwrap();
    let sosmo = extract_contract_info(&deployed_contracts, "SOMOS")?;
    let new_token = token_type_snip20_generator(&sosmo)?;
    // Check if the new token is a quote asset
    let is_quote_asset =
        lb_factory::query_is_quote_asset(&mut app, &lb_factory.clone().into(), new_token.clone())?;
    assert!(!is_quote_asset, "test_add_quote_asset::1");

    let _num_quote_assets =
        lb_factory::query_number_of_quote_assets(&mut app, &lb_factory.clone().into())?;

    // Add the new token as a quote asset
    lb_factory::add_quote_asset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        new_token.clone(),
    )?;

    let _num_quote_assets =
        lb_factory::query_number_of_quote_assets(&mut app, &lb_factory.clone().into())?;

    // Validate that the new token is now a quote asset
    let is_quote_asset =
        lb_factory::query_is_quote_asset(&mut app, &lb_factory.clone().into(), new_token.clone())?;
    assert!(is_quote_asset, "test_add_quote_asset::2");

    let _num_quote_assets =
        lb_factory::query_number_of_quote_assets(&mut app, &lb_factory.clone().into())?;

    let num_quote_assets_after =
        lb_factory::query_number_of_quote_assets(&mut app, &lb_factory.clone().into())?;

    assert_eq!(
        num_quote_assets_after,
        num_quote_assets_before + 1,
        "test_add_quote_asset::3"
    );
    // assert_eq!(num_quote_assets_after, 6, "test_add_quote_asset::4");
    // assert_eq!(num_quote_assets_before, 5, "test_add_quote_asset::5");

    let last_quote_asset = lb_factory::query_quote_asset_at_index(
        &mut app,
        &lb_factory.clone().into(),
        num_quote_assets_before,
    )?;
    assert_eq!(last_quote_asset, new_token, "test_add_quote_asset::6");

    // Try to add the same asset when not the owner
    let err = lb_factory::add_quote_asset(
        &mut app,
        addrs.batman().as_str(),
        &lb_factory.clone().into(),
        new_token.clone(),
    );
    assert!(err.is_err());

    // Try to add the same asset again, should revert
    let err: Result<(), StdError> = lb_factory::add_quote_asset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.into(),
        new_token.clone(),
    );
    assert_eq!(
        err.unwrap_err(),
        StdError::generic_err(format!(
            "Quote Asset {} is already whitelisted!",
            new_token.unique_key()
        ))
    );

    Ok(())
}

#[test]
#[serial]
pub fn test_remove_quote_asset() -> Result<(), anyhow::Error> {
    let addrs = init_addrs(); // Initialize addresses
    let (mut app, lb_factory, mut deployed_contracts) = setup(None, None)?; // Setup

    //SSCRT and SHD already added as quote asset
    let num_quote_assets_before =
        lb_factory::query_number_of_quote_assets(&mut app, &lb_factory.clone().into())?;
    println!("{num_quote_assets_before}");

    snip20::init(
        &mut app,
        addrs.admin().as_str(),
        &mut deployed_contracts,
        USDC,
        USDC,
        8,
        Some(shade_protocol::snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: None,
            enable_redeem: None,
            enable_mint: Some(true),
            enable_burn: None,
            enable_transfer: Some(true),
        }),
    )
    .unwrap();

    let _num_quote_assets =
        lb_factory::query_number_of_quote_assets(&mut app, &lb_factory.clone().into())?;

    let usdc_info: ContractInfo = extract_contract_info(&deployed_contracts, USDC)?;
    let usdc_token_type = token_type_snip20_generator(&usdc_info)?;
    let usdc = usdc_token_type;

    let _num_quote_assets =
        lb_factory::query_number_of_quote_assets(&mut app, &lb_factory.clone().into())?;

    // Add the new token as a quote asset
    lb_factory::add_quote_asset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        usdc.clone(),
    )?;

    let _num_quote_assets =
        lb_factory::query_number_of_quote_assets(&mut app, &lb_factory.clone().into())?;

    // Check if usdc is a quote asset
    let is_quote_asset =
        lb_factory::query_is_quote_asset(&mut app, &lb_factory.clone().into(), usdc.clone())?;
    assert!(is_quote_asset, "test_remove_quote_asset::1");

    // Remove usdc as a quote asset
    lb_factory::remove_quote_asset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        usdc.clone(),
    )?;

    // Validate that usdc is no longer a quote asset
    let is_quote_asset =
        lb_factory::query_is_quote_asset(&mut app, &lb_factory.clone().into(), usdc.clone())?;
    assert!(!is_quote_asset, "test_remove_quote_asset::2");

    let num_quote_assets_after =
        lb_factory::query_number_of_quote_assets(&mut app, &lb_factory.clone().into())?;
    assert_eq!(
        num_quote_assets_after, num_quote_assets_before,
        "test_remove_quote_asset::3"
    );

    // Try to remove usdc when not the owner
    let err = lb_factory::remove_quote_asset(
        &mut app,
        addrs.batman().as_str(),
        &lb_factory.clone().into(),
        usdc.clone(),
    );
    assert!(err.is_err());

    // Try to remove usdc again, should revert
    let err =
        lb_factory::remove_quote_asset(&mut app, addrs.admin().as_str(), &lb_factory.into(), usdc);
    assert!(err.is_err());

    Ok(())
}

#[test]
#[serial]
pub fn test_force_decay() -> Result<(), anyhow::Error> {
    let addrs = init_addrs(); // Initialize addresses
    let (mut app, lb_factory, deployed_contracts) = setup(None, None)?; // Setup

    let sscrt_info = extract_contract_info(&deployed_contracts, SSCRT)?;
    let sscrt = token_type_snip20_generator(&sscrt_info)?;

    let shd_info = extract_contract_info(&deployed_contracts, SHADE)?;
    let shd = token_type_snip20_generator(&shd_info)?;

    // Create a new LBPair with usdt and usdc
    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        sscrt.clone(),
        shd.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    )?;

    let all_pairs =
        lb_factory::query_all_lb_pairs(&mut app, &lb_factory.clone().into(), sscrt, shd)?;

    let lb_pair = all_pairs[0].clone().info;

    // Force decay on the created LBPair
    lb_factory::force_decay(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        lb_pair.clone(),
    )?;

    // Try to force decay when not the owner
    let err = lb_factory::force_decay(
        &mut app,
        addrs.batman().as_str(),
        &lb_factory.into(),
        lb_pair,
    );
    assert!(err.is_err());

    Ok(())
}

#[test]
#[serial]
pub fn test_get_all_lb_pair() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts) = setup(None, None)?;

    // 3. Create an LBPair.

    let shd = extract_contract_info(&deployed_contracts, SHADE)?;
    let sscrt = extract_contract_info(&deployed_contracts, SSCRT)?;
    let token_x = token_type_snip20_generator(&shd)?;
    let token_y = token_type_snip20_generator(&sscrt)?;

    lb_factory::set_pair_preset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        5,
        DEFAULT_BASE_FACTOR,
        DEFAULT_FILTER_PERIOD,
        DEFAULT_DECAY_PERIOD,
        DEFAULT_REDUCTION_FACTOR,
        DEFAULT_VARIABLE_FEE_CONTROL,
        DEFAULT_PROTOCOL_SHARE,
        DEFAULT_MAX_VOLATILITY_ACCUMULATOR,
        DEFAULT_OPEN_STATE,
        DEFAULT_TOTAL_REWARD_BINS,
        Some(RewardsDistributionAlgorithm::TimeBasedRewards),
        1,
        100,
        None,
    )?;

    lb_factory::set_pair_preset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        20,
        DEFAULT_BASE_FACTOR,
        DEFAULT_FILTER_PERIOD,
        DEFAULT_DECAY_PERIOD,
        DEFAULT_REDUCTION_FACTOR,
        DEFAULT_VARIABLE_FEE_CONTROL,
        DEFAULT_PROTOCOL_SHARE,
        DEFAULT_MAX_VOLATILITY_ACCUMULATOR,
        DEFAULT_OPEN_STATE,
        DEFAULT_TOTAL_REWARD_BINS,
        Some(RewardsDistributionAlgorithm::TimeBasedRewards),
        1,
        100,
        None,
    )?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        5,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    )?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        20,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    )?;

    let all_pairs = lb_factory::query_all_lb_pairs(&mut app, &lb_factory.into(), token_x, token_y)?;

    assert_eq!(all_pairs.len(), 2);

    Ok(())
}
