use crate::multitests::test_helper::*;

use super::test_helper::{
    increase_allowance_helper, init_addrs, liquidity_parameters_generator, mint_token_helper,
    setup, ID_ONE,
};
use anyhow::Ok;
use cosmwasm_std::{ContractInfo, StdError, Uint128};
use shade_multi_test::interfaces::{
    lb_factory, lb_pair, lb_token, snip20, utils::DeployedContracts,
};
use shade_protocol::{
    lb_libraries::{types::LBPairInformation},
    multi_test::App,
};

pub const DEPOSIT_AMOUNT: u128 = 1_000_000_000_000_000_000_u128;

pub const ACTIVE_ID: u32 = ID_ONE;

pub fn lb_pair_setup() -> Result<
    (
        App,
        ContractInfo,
        DeployedContracts,
        LBPairInformation,
        ContractInfo,
    ),
    anyhow::Error,
> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts) = setup(None)?;

    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let shade = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_x = token_type_snip20_generator(&shade)?;
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
    )?;
    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x,
        token_y,
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::lb_token_query(&app, &lb_pair.lb_pair.contract)?;

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
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let tokens_to_mint = vec![(SHADE, amount_x), (SILK, amount_y)];

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
        lb_pair.lb_pair.contract.address.to_string(),
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
        &lb_pair.lb_pair.contract,
        liquidity_parameters,
    )?;

    Ok((
        app,
        lb_factory.into(),
        deployed_contracts,
        lb_pair,
        lb_token,
    ))
}

#[test]
pub fn test_fuzz_swap_in_x() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    let amount_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_in, amount_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.lb_pair.contract, amount_out, true)?;
    assert_eq!(amount_out_left, Uint128::zero());

    let tokens_to_mint = vec![(SHADE, amount_in)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;

    let shd_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance, amount_in);

    let token_x = &extract_contract_info(&deployed_contracts, SHADE)?;

    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_in,
    )?;

    let shd_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance, Uint128::zero());

    let silk_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, amount_out);

    Ok(())
}

#[test]
pub fn test_fuzz_swap_in_y() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    let amount_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_in, amount_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.lb_pair.contract, amount_out, false)?;
    assert_eq!(amount_out_left, Uint128::MIN);

    let tokens_to_mint = vec![(SILK, amount_in)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;

    let token_y = &extract_contract_info(&deployed_contracts, SILK)?;

    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_y,
        amount_in,
    )?;

    let shd_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance, amount_out);

    let silk_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, Uint128::zero());

    Ok(())
}

#[test]
pub fn test_fuzz_swap_out_for_y() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    let amount_in = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_out, amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_in, true)?;

    assert!(amount_out > Uint128::zero());
    assert_eq!(amount_in_left, Uint128::zero());

    let tokens_to_mint = vec![(SHADE, amount_in)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;

    let token_x = &extract_contract_info(&deployed_contracts, SHADE)?;

    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_in,
    )?;

    let shd_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance, Uint128::zero());

    let silk_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, amount_out);

    Ok(())
}

#[test]
pub fn test_fuzz_swap_out_for_x() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    let amount_in = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_out, amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_in, false)?;

    assert!(amount_out > Uint128::zero());
    assert_eq!(amount_in_left, Uint128::zero());

    let tokens_to_mint = vec![(SILK, amount_in)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;

    let token_y = &extract_contract_info(&deployed_contracts, SILK)?;

    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_y,
        amount_in,
    )?;

    let silk_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, Uint128::zero());

    let shade_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shade_balance, amount_out);

    Ok(())
}

#[test]

pub fn test_revert_swap_insufficient_amount_in() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    let token_x = &extract_contract_info(&deployed_contracts, SHADE)?;

    let result = lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_x,
        Uint128::zero(),
    );

    assert_eq!(
        result,
        Err(StdError::GenericErr {
            msg: "Insufficient amount in!".to_string()
        })
    );

    let token_y = &extract_contract_info(&deployed_contracts, SILK)?;

    let result = lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_y,
        Uint128::zero(),
    );

    assert_eq!(
        result,
        Err(StdError::GenericErr {
            msg: "Insufficient amount in!".to_string()
        })
    );

    Ok(())
}

#[test]
pub fn test_revert_swap_insufficient_amount_out() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    // Simulate transferring 1 token to the LB pair contract
    let token_amount = Uint128::from(1u128);

    let tokens_to_mint = vec![(SHADE, token_amount)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;

    let token_x = &extract_contract_info(&deployed_contracts, SHADE)?;

    let result = lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_x,
        token_amount,
    );

    // Check for the expected error
    assert_eq!(
        result,
        Err(StdError::GenericErr {
            msg: "Insufficient amount out!".to_string()
        })
    );
    let tokens_to_mint = vec![(SILK, token_amount)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;

    let token_y = &extract_contract_info(&deployed_contracts, SILK)?;

    let result = lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_y,
        token_amount,
    );

    // Check for the expected error
    assert_eq!(
        result,
        Err(StdError::GenericErr {
            msg: "Insufficient amount out!".to_string()
        })
    );
    Ok(())
}

#[test]
pub fn test_revert_swap_out_of_liquidity() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    // Simulate transferring 2e18 tokens to the LB pair contract
    let token_amount = Uint128::from(2 * DEPOSIT_AMOUNT);
    let tokens_to_mint = vec![(SHADE, token_amount)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;

    let token_x = &extract_contract_info(&deployed_contracts, SHADE)?;

    let result = lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_x,
        token_amount,
    );
    // Check for the expected error
    assert_eq!(
        result,
        Err(StdError::GenericErr {
            msg: "Not enough liquidity!".to_string()
        })
    );

    let tokens_to_mint = vec![(SILK, token_amount)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    let token_y = &extract_contract_info(&deployed_contracts, SILK)?;

    let result = lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_y,
        token_amount,
    );

    // Check for the expected error
    assert_eq!(
        result,
        Err(StdError::GenericErr {
            msg: "Not enough liquidity!".to_string()
        })
    );
    Ok(())
}
