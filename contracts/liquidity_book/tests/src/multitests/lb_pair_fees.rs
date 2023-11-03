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
use cosmwasm_std::{ContractInfo, StdError, Uint128, Uint256};
use ethnum::U256;
use shade_multi_test::interfaces::{
    lb_factory,
    lb_pair,
    lb_token,
    snip20,
    utils::DeployedContracts,
};
use shade_protocol::{
    lb_libraries::{
        math::{encoded_sample::MASK_UINT20, u24::U24},
        types::LBPairInformation,
    },
    liquidity_book::lb_pair::RemoveLiquidity,
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
        "entropy".to_string(),
    )?;
    let all_pairs =
        lb_factory::query_all_lb_pairs(&mut app, &lb_factory.clone().into(), token_x, token_y)?;
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
    let nb_bins_x = 10;
    let nb_bins_y = 10;

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

    snip20::set_viewing_key_exec(
        &mut app,
        addrs.scare_crow().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    snip20::set_viewing_key_exec(
        &mut app,
        addrs.scare_crow().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    snip20::set_viewing_key_exec(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_string(),
    )?;

    snip20::set_viewing_key_exec(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_string(),
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
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;
    let amount_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_in, amount_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.lb_pair.contract, amount_out, true)?;
    assert_eq!(amount_out_left, Uint128::MIN);

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

    //REMOVE LIQUIDITY

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let total_bins = get_total_bins(10, 10) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, 10);
        ids[i as usize] = id;
        balances[i as usize] = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
    }

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;
    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(reserves_x),
            amount_y_min: Uint128::from(reserves_y),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    )?;

    let (protocol_fee_x, _) = lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(
        balance_x.u128(),
        DEPOSIT_AMOUNT + amount_in.u128() - protocol_fee_x
    );

    assert_eq!(balance_y.u128(), reserves_y + amount_out.u128());

    Ok(())
}

#[test]
pub fn test_fuzz_swap_in_y() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

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

    let silk_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, Uint128::zero());
    let shd_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance, amount_out);

    //REMOVE LIQUIDITY

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let total_bins = get_total_bins(10, 10) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, 10);
        ids[i as usize] = id;
        balances[i as usize] = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
    }

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;
    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(reserves_x),
            amount_y_min: Uint128::from(reserves_y),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    )?;

    let (_, protocol_fee_y) = lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_x.u128(), reserves_x + amount_out.u128());

    assert_eq!(
        balance_y.u128(),
        DEPOSIT_AMOUNT + amount_in.u128() - protocol_fee_y
    );

    Ok(())
}

#[test]
pub fn test_fuzz_swap_out_for_x() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

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

    //REMOVE LIQUIDITY

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let total_bins = get_total_bins(10, 10) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, 10);
        ids[i as usize] = id;
        balances[i as usize] = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
    }

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;
    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(reserves_x),
            amount_y_min: Uint128::from(reserves_y),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    )?;

    let (protocol_fee_x, _) = lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(
        balance_x.u128(),
        DEPOSIT_AMOUNT + amount_in.u128() - protocol_fee_x
    );

    assert_eq!(balance_y.u128(), reserves_y + amount_out.u128());
    Ok(())
}

#[test]
pub fn test_fuzz_swap_out_for_y() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

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

    //REMOVE LIQUIDITY

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let total_bins = get_total_bins(10, 10) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, 10);
        ids[i as usize] = id;
        balances[i as usize] = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
    }

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;
    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(reserves_x),
            amount_y_min: Uint128::from(reserves_y),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    )?;

    let (_, protocol_fee_y) = lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_x.u128(), reserves_x + amount_out.u128());

    assert_eq!(
        balance_y.u128(),
        DEPOSIT_AMOUNT + amount_in.u128() - protocol_fee_y
    );

    Ok(())
}

#[test]
pub fn test_fuzz_swap_in_x_and_y() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    //generate random number
    let amount_y_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));
    // get swap_in for y
    let (amount_x_in, amount_y_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.lb_pair.contract, amount_y_out, true)?;
    // amount_y_out_left == zero since the amount_x_out must be less than total deposit
    assert_eq!(amount_y_out_left, Uint128::zero());
    // mint the tokens
    let tokens_to_mint = vec![(SHADE, amount_x_in)];
    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    // make a swap with amount_x_in
    let token_x = &extract_contract_info(&deployed_contracts, SHADE)?;
    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_x_in,
    )?;
    // check the balance of silk if it's equal to the amount_y_out
    let silk_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, amount_y_out);
    let shd_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance, Uint128::zero());

    //generate random number
    let amount_x_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));
    // get swap_in for y
    let (amount_y_in, amount_x_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.lb_pair.contract, amount_x_out, false)?;
    // amount_y_out_left == zero since the amount_x_out must be less than total deposit
    assert_eq!(amount_x_out_left, Uint128::zero());
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

    let token_y = &extract_contract_info(&deployed_contracts, SILK)?;

    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_y,
        amount_y_in,
    )?;

    // check the balance of silk if it's equal to the amount_y_out
    let silk_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, amount_y_out);
    let shd_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance, amount_x_out);

    //REMOVE LIQUIDITY
    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let total_bins = get_total_bins(10, 10) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, 10);
        ids[i as usize] = id;
        balances[i as usize] = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
    }

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;

    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(reserves_x),
            amount_y_min: Uint128::from(reserves_y),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let lb_pair_balance_x = snip20::balance_query(
        &mut app,
        lb_pair.lb_pair.contract.address.as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let lb_pair_balance_y = snip20::balance_query(
        &mut app,
        lb_pair.lb_pair.contract.address.as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(lb_pair_balance_x.u128(), protocol_fee_x);
    assert_eq!(lb_pair_balance_y.u128(), protocol_fee_y);

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_x.u128(), reserves_x + amount_x_out.u128());
    assert_eq!(balance_y.u128(), reserves_y + amount_y_out.u128());

    assert_eq!(
        reserves_x,
        DEPOSIT_AMOUNT + amount_x_in.u128() - amount_x_out.u128() - protocol_fee_x
    );
    assert_eq!(
        reserves_y,
        DEPOSIT_AMOUNT + amount_y_in.u128() - amount_y_out.u128() - protocol_fee_y
    );

    Ok(())
}

#[test]
pub fn test_fuzz_swap_in_y_and_x() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    //generate random number
    let amount_x_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));
    // get swap_in for y
    let (amount_y_in, amount_x_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.lb_pair.contract, amount_x_out, false)?;
    // amount_y_out_left == zero since the amount_x_out must be less than total deposit
    assert_eq!(amount_x_out_left, Uint128::zero());
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
    let token_y = &extract_contract_info(&deployed_contracts, SILK)?;

    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_y,
        amount_y_in,
    )?;

    // check the balance of silk if it's equal to the amount_y_out
    let silk_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, Uint128::zero());
    let shd_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance, amount_x_out);

    //generate random number
    let amount_y_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));
    // get swap_in for y
    let (amount_x_in, amount_y_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.lb_pair.contract, amount_y_out, true)?;
    // amount_y_out_left == zero since the amount_x_out must be less than total deposit
    assert_eq!(amount_y_out_left, Uint128::zero());
    // mint the tokens
    let tokens_to_mint = vec![(SHADE, amount_x_in)];
    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    // make a swap with amount_x_in

    let token_x = &extract_contract_info(&deployed_contracts, SHADE)?;
    lb_pair::swap_snip_20(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_x_in,
    )?;

    // check the balance of silk if it's equal to the amount_y_out
    let silk_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, amount_y_out);
    let shd_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance, amount_x_out);

    //REMOVE LIQUIDITY
    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let total_bins = get_total_bins(10, 10) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, 10);
        ids[i as usize] = id;
        balances[i as usize] = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
    }

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;

    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(reserves_x),
            amount_y_min: Uint128::from(reserves_y),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let lb_pair_balance_x = snip20::balance_query(
        &mut app,
        lb_pair.lb_pair.contract.address.as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let lb_pair_balance_y = snip20::balance_query(
        &mut app,
        lb_pair.lb_pair.contract.address.as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(lb_pair_balance_x.u128(), protocol_fee_x);
    assert_eq!(lb_pair_balance_y.u128(), protocol_fee_y);

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_x.u128(), reserves_x + amount_x_out.u128());
    assert_eq!(balance_y.u128(), reserves_y + amount_y_out.u128());

    assert_eq!(
        reserves_x,
        DEPOSIT_AMOUNT + amount_x_in.u128() - amount_x_out.u128() - protocol_fee_x
    );
    assert_eq!(
        reserves_y,
        DEPOSIT_AMOUNT + amount_y_in.u128() - amount_y_out.u128() - protocol_fee_y
    );

    Ok(())
}

#[test]
pub fn test_fuzz_swap_out_x_and_y() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    let amount_x_in = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_y_out, amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_x_in, true)?;

    assert!(amount_y_out > Uint128::zero());
    assert_eq!(amount_in_left, Uint128::zero());

    let tokens_to_mint = vec![(SHADE, amount_x_in)];

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
        amount_x_in,
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
    assert_eq!(silk_balance, amount_y_out);

    let amount_y_in = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_x_out, amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_y_in, false)?;

    assert!(amount_x_out > Uint128::zero());
    assert_eq!(amount_in_left, Uint128::zero());

    let tokens_to_mint = vec![(SILK, amount_y_in)];

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
        amount_y_in,
    )?;

    let silk_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, amount_y_out);

    let shade_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shade_balance, amount_x_out);

    //REMOVE LIQUIDITY

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let total_bins = get_total_bins(10, 10) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, 10);
        ids[i as usize] = id;
        balances[i as usize] = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
    }

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;
    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(reserves_x),
            amount_y_min: Uint128::from(reserves_y),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let lb_pair_balance_x = snip20::balance_query(
        &mut app,
        lb_pair.lb_pair.contract.address.as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let lb_pair_balance_y = snip20::balance_query(
        &mut app,
        lb_pair.lb_pair.contract.address.as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(lb_pair_balance_x.u128(), protocol_fee_x);
    assert_eq!(lb_pair_balance_y.u128(), protocol_fee_y);
    let balance_x = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(balance_x.u128(), reserves_x + amount_x_out.u128());
    assert_eq!(balance_y.u128(), reserves_y + amount_y_out.u128());

    assert_eq!(
        reserves_x,
        DEPOSIT_AMOUNT + amount_x_in.u128() - amount_x_out.u128() - protocol_fee_x
    );
    assert_eq!(
        reserves_y,
        DEPOSIT_AMOUNT + amount_y_in.u128() - amount_y_out.u128() - protocol_fee_y
    );
    Ok(())
}

#[test]
pub fn test_fuzz_swap_out_y_and_x() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    let amount_y_in = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_x_out, amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_y_in, false)?;

    assert!(amount_x_out > Uint128::zero());
    assert_eq!(amount_in_left, Uint128::zero());

    let tokens_to_mint = vec![(SILK, amount_y_in)];

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
        amount_y_in,
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
    assert_eq!(shade_balance, amount_x_out);

    let amount_x_in = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_y_out, amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_x_in, true)?;

    assert!(amount_y_out > Uint128::zero());
    assert_eq!(amount_in_left, Uint128::zero());

    let tokens_to_mint = vec![(SHADE, amount_x_in)];

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
        amount_x_in,
    )?;

    let shd_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance, amount_x_out);

    let silk_balance = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, amount_y_out);

    //REMOVE LIQUIDITY

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let total_bins = get_total_bins(10, 10) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, 10);
        ids[i as usize] = id;
        balances[i as usize] = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
    }

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;
    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(reserves_x),
            amount_y_min: Uint128::from(reserves_y),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let lb_pair_balance_x = snip20::balance_query(
        &mut app,
        lb_pair.lb_pair.contract.address.as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let lb_pair_balance_y = snip20::balance_query(
        &mut app,
        lb_pair.lb_pair.contract.address.as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(lb_pair_balance_x.u128(), protocol_fee_x);
    assert_eq!(lb_pair_balance_y.u128(), protocol_fee_y);
    let balance_x = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(balance_x.u128(), reserves_x + amount_x_out.u128());
    assert_eq!(balance_y.u128(), reserves_y + amount_y_out.u128());

    assert_eq!(
        reserves_x,
        DEPOSIT_AMOUNT + amount_x_in.u128() - amount_x_out.u128() - protocol_fee_x
    );
    assert_eq!(
        reserves_y,
        DEPOSIT_AMOUNT + amount_y_in.u128() - amount_y_out.u128() - protocol_fee_y
    );
    Ok(())
}

#[test]
pub fn test_fee_x_2_lp() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    let amount = Uint128::from(DEPOSIT_AMOUNT);
    //add_liquidity second time:
    let tokens_to_mint = vec![(SHADE, amount), (SILK, amount)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    //Adding liquidity
    let liquidity_parameters = liquidity_parameters_generator(
        &deployed_contracts,
        ACTIVE_ID,
        token_x,
        token_y,
        amount,
        amount,
        10,
        10,
    )?;

    lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        liquidity_parameters,
    )?;

    let amount_in = Uint128::from(DEPOSIT_AMOUNT);

    let (amount_out, _amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_in, true)?;

    let tokens_to_mint = vec![(SHADE, amount_in)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.scare_crow().into_string(),
        tokens_to_mint.clone(),
    )?;
    let token_x = &extract_contract_info(&deployed_contracts, SHADE)?;

    lb_pair::swap_snip_20(
        &mut app,
        addrs.scare_crow().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.scare_crow().to_string()),
        token_x,
        amount_in,
    )?;

    //REMOVE LIQUIDITY

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let total_bins = get_total_bins(10, 10) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, 10);
        ids[i as usize] = id;
        balances[i as usize] = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?
        .checked_div(Uint256::from(2u128))?;
    }

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;
    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(reserves_x / 2 - 1),
            amount_y_min: Uint128::from(reserves_y / 2 - 1),
            ids: ids.clone(),
            amounts: balances.clone(),
            deadline: 99999999999,
        },
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_approx_eq_abs(
        Uint256::from(balance_x.u128()),
        Uint256::from(DEPOSIT_AMOUNT + (amount_in.u128() - protocol_fee_x) / 2),
        Uint256::from(2u128),
        "test_fee_x_2_lp::1",
    );
    assert_approx_eq_abs(
        Uint256::from(balance_y.u128()),
        Uint256::from(DEPOSIT_AMOUNT - (amount_out.u128() + protocol_fee_y) / 2),
        Uint256::from(2u128),
        "test_fee_x_2_lp::2",
    );

    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(reserves_x / 2 - 1),
            amount_y_min: Uint128::from(reserves_y / 2 - 1),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_approx_eq_abs(
        Uint256::from(balance_x.u128()),
        Uint256::from(DEPOSIT_AMOUNT * 2 + (amount_in.u128() - protocol_fee_x)),
        Uint256::from(2u128),
        "test_fee_x_2_lp::3",
    );
    assert_approx_eq_abs(
        Uint256::from(balance_y.u128()),
        Uint256::from(DEPOSIT_AMOUNT * 2 - (amount_out.u128() + protocol_fee_y)),
        Uint256::from(2u128),
        "test_fee_x_2_lp::4",
    );

    Ok(())
}

#[test]
pub fn test_fee_y_2_lp() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    let amount = Uint128::from(DEPOSIT_AMOUNT);
    //add_liquidity second time:
    let tokens_to_mint = vec![(SHADE, amount), (SILK, amount)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;
    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    //Adding liquidity
    let liquidity_parameters = liquidity_parameters_generator(
        &deployed_contracts,
        ACTIVE_ID,
        token_x,
        token_y,
        amount,
        amount,
        10,
        10,
    )?;

    lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        liquidity_parameters,
    )?;

    let amount_in = Uint128::from(DEPOSIT_AMOUNT);

    let (amount_out, _amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_in, false)?;

    let tokens_to_mint = vec![(SILK, amount_in)];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.scare_crow().into_string(),
        tokens_to_mint.clone(),
    )?;

    let token_y = &extract_contract_info(&deployed_contracts, SILK)?;

    lb_pair::swap_snip_20(
        &mut app,
        addrs.scare_crow().as_str(),
        &lb_pair.lb_pair.contract,
        Some(addrs.scare_crow().into_string()),
        token_y,
        amount_in,
    )?;

    //REMOVE LIQUIDITY

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let total_bins = get_total_bins(10, 10) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, 10);
        ids[i as usize] = id;
        balances[i as usize] = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?
        .checked_div(Uint256::from(2u128))?;
    }

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;
    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(reserves_x / 2 - 1),
            amount_y_min: Uint128::from(reserves_y / 2 - 1),
            ids: ids.clone(),
            amounts: balances.clone(),
            deadline: 99999999999,
        },
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_approx_eq_abs(
        Uint256::from(balance_x.u128()),
        Uint256::from(DEPOSIT_AMOUNT - (amount_out.u128() + protocol_fee_x) / 2),
        Uint256::from(2u128),
        "test_fee_y_2_lp::1",
    );
    assert_approx_eq_abs(
        Uint256::from(balance_y.u128()),
        Uint256::from(DEPOSIT_AMOUNT + (amount_in.u128() - protocol_fee_y) / 2),
        Uint256::from(2u128),
        "test_fee_y_2_lp::2",
    );

    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(reserves_x / 2 - 1),
            amount_y_min: Uint128::from(reserves_y / 2 - 1),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_approx_eq_abs(
        Uint256::from(balance_x.u128()),
        Uint256::from(DEPOSIT_AMOUNT * 2 - (amount_out.u128() + protocol_fee_x)),
        Uint256::from(2u128),
        "test_fee_y_2_lp::3",
    );
    assert_approx_eq_abs(
        Uint256::from(balance_y.u128()),
        Uint256::from(DEPOSIT_AMOUNT * 2 + (amount_in.u128() - protocol_fee_y)),
        Uint256::from(2u128),
        "test_fee_y_2_lp::4",
    );

    Ok(())
}

#[test]
pub fn test_fees_2lp_flash_loan() {}

#[test]
pub fn test_collect_protocol_fees_x_tokens() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    let amount_y_in = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_x_out, amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_y_in, true)?;

    assert!(amount_x_out > Uint128::zero());
    assert_eq!(amount_in_left, Uint128::zero());

    let tokens_to_mint = vec![(SHADE, amount_y_in)];

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
        amount_y_in,
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(protocol_fee_y, 0);

    lb_pair::collect_protocol_fees(&mut app, addrs.admin().as_str(), &lb_pair.lb_pair.contract)?;

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_x.u128(), protocol_fee_x - 1);

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_y.u128(), 0);

    Ok(())
}

#[test]
pub fn test_collect_protocol_fees_y_tokens() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    let amount_x_in = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_y_out, amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_x_in, false)?;

    assert!(amount_y_out > Uint128::zero());
    assert_eq!(amount_in_left, Uint128::zero());

    let tokens_to_mint = vec![(SILK, amount_x_in)];

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
        amount_x_in,
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(protocol_fee_x, 0);

    lb_pair::collect_protocol_fees(&mut app, addrs.admin().as_str(), &lb_pair.lb_pair.contract)?;

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_x.u128(), 0);

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_y.u128(), protocol_fee_y - 1);

    Ok(())
}

#[test]
pub fn test_collect_protocol_fees_both_tokens() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    let amount_x_in = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_y_out, amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_x_in, true)?;

    assert!(amount_y_out > Uint128::zero());
    assert_eq!(amount_in_left, Uint128::zero());

    let tokens_to_mint = vec![(SHADE, amount_x_in)];

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
        amount_x_in,
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
    assert_eq!(silk_balance, amount_y_out);

    let amount_y_in = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_x_out, amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_y_in, false)?;

    assert!(amount_x_out > Uint128::zero());
    assert_eq!(amount_in_left, Uint128::zero());

    let tokens_to_mint = vec![(SILK, amount_y_in)];

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
        amount_y_in,
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    assert!(protocol_fee_x > 0);
    assert!(protocol_fee_y > 0);

    lb_pair::collect_protocol_fees(&mut app, addrs.admin().as_str(), &lb_pair.lb_pair.contract)?;

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_x.u128(), protocol_fee_x - 1);

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_y.u128(), protocol_fee_y - 1);

    Ok(())
}

#[test]
pub fn test_collect_protocol_fees_after_swap() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    let amount_x_in = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_y_out, amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_x_in, true)?;

    assert!(amount_y_out > Uint128::zero());
    assert_eq!(amount_in_left, Uint128::zero());

    let tokens_to_mint = vec![(SHADE, amount_x_in)];

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
        amount_x_in,
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

    let prev_protocol_fee_x = protocol_fee_x;

    assert!(protocol_fee_x > 0);
    assert_eq!(protocol_fee_y, 0);

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;

    lb_pair::collect_protocol_fees(&mut app, addrs.admin().as_str(), &lb_pair.lb_pair.contract)?;

    let (reserves_x_after, reserves_y_after) =
        lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(reserves_x_after, reserves_x);
    assert_eq!(reserves_y_after, reserves_y);

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_x.u128(), protocol_fee_x - 1);

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_y.u128(), 0);

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;
    assert_eq!(protocol_fee_x, 1);
    assert_eq!(protocol_fee_y, 0);

    let amount_y_in = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_x_out, amount_in_left, _fee) =
        lb_pair::query_swap_out(&app, &lb_pair.lb_pair.contract, amount_y_in, false)?;

    assert!(amount_x_out > Uint128::zero());
    assert_eq!(amount_in_left, Uint128::zero());

    let tokens_to_mint = vec![(SILK, amount_y_in)];

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
        amount_y_in,
    )?;

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;
    let prev_protocol_fee_y = protocol_fee_y;
    assert_eq!(protocol_fee_x, 1);
    assert!(protocol_fee_y > 0);

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;

    lb_pair::collect_protocol_fees(&mut app, addrs.admin().as_str(), &lb_pair.lb_pair.contract)?;

    let (reserves_x_after, reserves_y_after) =
        lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(reserves_x_after, reserves_x);
    assert_eq!(reserves_y_after, reserves_y);

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_x.u128(), prev_protocol_fee_x - 1);

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_y.u128(), protocol_fee_y - 1);

    let (protocol_fee_x, protocol_fee_y) =
        lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;
    assert_eq!(protocol_fee_x, 1);
    assert_eq!(protocol_fee_y, 1);

    let res =
        lb_pair::collect_protocol_fees(&mut app, addrs.admin().as_str(), &lb_pair.lb_pair.contract);

    assert_eq!(
        res.unwrap_err(),
        (StdError::generic_err("Not enough funds".to_string()))
    );

    let balance_x = snip20::balance_query(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_x.u128(), prev_protocol_fee_x - 1);

    let balance_y = snip20::balance_query(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance_y.u128(), prev_protocol_fee_y - 1);

    Ok(())
}

#[test]
pub fn test_revert_total_fee_exceeded() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();

    let bin_step = Uint128::from(generate_random(1u16, u16::MAX));
    let (mut app, lb_factory, deployed_contracts) = setup(Some(bin_step.u128() as u16))?;
    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let shade = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_x = token_type_snip20_generator(&shade)?;
    let token_y = token_type_snip20_generator(&silk)?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        bin_step.u128() as u16,
        ACTIVE_ID,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    )?;
    let all_pairs =
        lb_factory::query_all_lb_pairs(&mut app, &lb_factory.clone().into(), token_x, token_y)?;
    let lb_pair = all_pairs[0].clone();
    let base_factor = Uint128::from(generate_random(1u16, u16::MAX));
    let variable_fee_control = Uint128::from(generate_random(1u32, U24::MAX));
    let max_volatility_accumulator = Uint128::from(generate_random(1u32, MASK_UINT20.as_u32()));

    let base_fee = base_factor * bin_step * Uint128::from(10_000_000_000u128);
    let var_fee = ((bin_step * max_volatility_accumulator).pow(2) * variable_fee_control
        + Uint128::from(99u128))
        / Uint128::from(100u128);

    if base_fee + var_fee > Uint128::from(10u128).pow(17) {
        let res = lb_pair::set_static_fee_parameters(
            &mut app,
            lb_factory.address.as_str(),
            &lb_pair.lb_pair.contract,
            base_factor.u128() as u16,
            1,
            1,
            1,
            variable_fee_control.u128() as u32,
            1,
            max_volatility_accumulator.u128() as u32,
        );

        assert_eq!(
            res.unwrap_err(),
            (StdError::generic_err("Max total fee exceeded!".to_string()))
        );
    } else {
        test_revert_total_fee_exceeded()?;
    }

    Ok(())
}

// #[test]
// pub fn test_fuzz_user_fee_swap_in_x() -> Result<(), anyhow::Error> {
//     let addrs = init_addrs();
//     let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;
//     let amount_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

//     let (amount_in, amount_out_left, _fee) =
//         lb_pair::query_swap_in(&app, &lb_pair.lb_pair.contract, amount_out, true)?;
//     assert_eq!(amount_out_left, Uint128::zero());

//     let tokens_to_mint = vec![(SHADE, amount_in)];

//     mint_token_helper(
//         &mut app,
//         &deployed_contracts,
//         &addrs,
//         addrs.joker().into_string(),
//         tokens_to_mint.clone(),
//     )?;

//     let token_x = &extract_contract_info(&deployed_contracts, SHADE)?;

//     lb_pair::swap_snip_20(
//         &mut app,
//         addrs.joker().as_str(),
//         &lb_pair.lb_pair.contract,
//         Some(addrs.joker().to_string()),
//         token_x,
//         amount_in,
//     )?;

//     let shd_balance = snip20::balance_query(
//         &mut app,
//         addrs.joker().as_str(),
//         &deployed_contracts,
//         SHADE,
//         "viewing_key".to_owned(),
//     )?;
//     assert_eq!(shd_balance, Uint128::zero());

//     let silk_balance = snip20::balance_query(
//         &mut app,
//         addrs.joker().as_str(),
//         &deployed_contracts,
//         SILK,
//         "viewing_key".to_owned(),
//     )?;
//     assert_eq!(silk_balance, amount_out);

//     //REMOVE LIQUIDITY

//     let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
//     let token_y = extract_contract_info(&deployed_contracts, SILK)?;

//     let total_bins = get_total_bins(10, 10) as u32;
//     let mut balances = vec![Uint256::zero(); total_bins as usize];
//     let mut ids = vec![0u32; total_bins as usize];

//     for i in 0..total_bins {
//         let id = get_id(ACTIVE_ID, i, 10);
//         ids[i as usize] = id;
//         balances[i as usize] = lb_token::query_balance(
//             &app,
//             &lb_token,
//             addrs.batman(),
//             addrs.batman(),
//             String::from("viewing_key"),
//             id.to_string(),
//         )?;
//     }

//     let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;
//     lb_pair::remove_liquidity(
//         &mut app,
//         addrs.batman().as_str(),
//         &lb_pair.lb_pair.contract,
//         RemoveLiquidity {
//             token_x: token_type_snip20_generator(&token_x)?,
//             token_y: token_type_snip20_generator(&token_y)?,
//             bin_step: lb_pair.bin_step,
//             amount_x_min: Uint128::from(reserves_x),
//             amount_y_min: Uint128::from(reserves_y),
//             ids: ids.clone(),
//             amounts: balances.clone(),
//             deadline: 99999999999,
//         },
//     )?;

//     let (protocol_fee_x, _) = lb_pair::query_protocol_fees(&app, &lb_pair.lb_pair.contract)?;

//     let balance_x = snip20::balance_query(
//         &mut app,
//         addrs.batman().as_str(),
//         &deployed_contracts,
//         SHADE,
//         "viewing_key".to_owned(),
//     )?;

//     let balance_y = snip20::balance_query(
//         &mut app,
//         addrs.batman().as_str(),
//         &deployed_contracts,
//         SILK,
//         "viewing_key".to_owned(),
//     )?;

//     assert_eq!(
//         balance_x.u128(),
//         DEPOSIT_AMOUNT + amount_in.u128() - protocol_fee_x
//     );

//     assert_eq!(balance_y.u128(), reserves_y);

//     let amount_x = Uint128::from(DEPOSIT_AMOUNT);
//     let amount_y = Uint128::from(DEPOSIT_AMOUNT);
//     let nb_bins_x = 10;
//     let nb_bins_y = 10;

//     let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
//     let token_y = extract_contract_info(&deployed_contracts, SILK)?;

//     let tokens_to_mint = vec![(SHADE, amount_x), (SILK, amount_y)];

//     mint_token_helper(
//         &mut app,
//         &deployed_contracts,
//         &addrs,
//         addrs.scare_crow().into_string(),
//         tokens_to_mint.clone(),
//     )?;

//     increase_allowance_helper(
//         &mut app,
//         &deployed_contracts,
//         addrs.scare_crow().into_string(),
//         lb_pair.lb_pair.contract.address.to_string(),
//         tokens_to_mint,
//     )?;

//     //Adding liquidity
//     let liquidity_parameters = liquidity_parameters_generator(
//         &deployed_contracts,
//         ACTIVE_ID,
//         token_x.clone(),
//         token_y.clone(),
//         amount_x,
//         amount_y,
//         nb_bins_x,
//         nb_bins_y,
//     )?;

//     lb_pair::add_liquidity(
//         &mut app,
//         addrs.scare_crow().as_str(),
//         &lb_pair.lb_pair.contract,
//         liquidity_parameters,
//     )?;

//     let total_bins = get_total_bins(10, 10) as u32;
//     let mut balances = vec![Uint256::zero(); total_bins as usize];
//     let mut ids = vec![0u32; total_bins as usize];

//     lb_token::set_viewing_key(
//         &mut app,
//         addrs.scare_crow().as_str(),
//         &lb_token,
//         "viewing_key".to_owned(),
//     )?;
//     for i in 0..total_bins {
//         let id = get_id(ACTIVE_ID, i, 10);
//         ids[i as usize] = id;
//         balances[i as usize] = lb_token::query_balance(
//             &app,
//             &lb_token,
//             addrs.scare_crow(),
//             addrs.scare_crow(),
//             String::from("viewing_key"),
//             id.to_string(),
//         )?;
//     }

//     let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;
//     lb_pair::remove_liquidity(
//         &mut app,
//         addrs.scare_crow().as_str(),
//         &lb_pair.lb_pair.contract,
//         RemoveLiquidity {
//             token_x: token_type_snip20_generator(&token_x)?,
//             token_y: token_type_snip20_generator(&token_y)?,
//             bin_step: lb_pair.bin_step,
//             amount_x_min: Uint128::from(reserves_x),
//             amount_y_min: Uint128::from(reserves_y),
//             ids,
//             amounts: balances,
//             deadline: 99999999999,
//         },
//     )?;

//     // let balance_x = snip20::balance_query(
//     //     &mut app,
//     //     addrs.scare_crow().as_str(),
//     //     &deployed_contracts,
//     //     SHADE,
//     //     "viewing_key".to_owned(),
//     // )?;

//     // let balance_y = snip20::balance_query(
//     //     &mut app,
//     //     addrs.scare_crow().as_str(),
//     //     &deployed_contracts,
//     //     SILK,
//     //     "viewing_key".to_owned(),
//     // )?;

//     // assert_eq!(balance_x.u128(), DEPOSIT_AMOUNT);

//     // assert_eq!(balance_y.u128(), DEPOSIT_AMOUNT);

//     Ok(())
// }
