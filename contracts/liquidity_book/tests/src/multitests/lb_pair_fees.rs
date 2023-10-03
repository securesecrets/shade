use crate::multitests::test_helper::*;

use super::test_helper::{
    increase_allowance_helper, init_addrs, liquidity_parameters_generator, mint_token_helper,
    setup, ID_ONE,
};
use anyhow::Ok;
use cosmwasm_std::{ContractInfo, Uint128, Uint256};
use shade_multi_test::interfaces::{
    lb_factory, lb_pair, lb_token, snip20, utils::DeployedContracts,
};
use shade_protocol::{
    lb_libraries::types::LBPairInformation, liquidity_book::lb_pair::RemoveLiquidity,
    multi_test::App,
};

pub const DEPOSIT_AMOUNT: u128 = 1_000_000_000_000_000_000 as u128;

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
    let (mut app, lb_factory, deployed_contracts) = setup()?;

    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let shade = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_x = token_type_generator(&shade)?;
    let token_y = token_type_generator(&silk)?;

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
        token_x.clone(),
        token_y.clone(),
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::lb_token_query(&app, &lb_pair.lb_pair.contract)?;

    lb_token::set_viewing_key(
        &mut app,
        &addrs.batman().as_str(),
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
        &addrs.batman().as_str(),
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

    snip20::transfer_exec(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        lb_pair.lb_pair.contract.address.to_string(),
        amount_in,
    )?;

    lb_pair::swap(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        true,
        addrs.batman(),
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
            token_x: token_type_generator(&token_x)?,
            token_y: token_type_generator(&token_y)?,
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

    snip20::transfer_exec(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        lb_pair.lb_pair.contract.address.to_string(),
        amount_in,
    )?;

    lb_pair::swap(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        false,
        addrs.batman(),
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
            token_x: token_type_generator(&token_x)?,
            token_y: token_type_generator(&token_y)?,
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

    snip20::transfer_exec(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        lb_pair.lb_pair.contract.address.to_string(),
        amount_in,
    )?;

    lb_pair::swap(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        true,
        addrs.batman(),
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
            token_x: token_type_generator(&token_x)?,
            token_y: token_type_generator(&token_y)?,
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

    snip20::transfer_exec(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        lb_pair.lb_pair.contract.address.to_string(),
        amount_in,
    )?;

    lb_pair::swap(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        false,
        addrs.batman(),
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
            token_x: token_type_generator(&token_x)?,
            token_y: token_type_generator(&token_y)?,
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

// #[test]
// pub fn test_fuzz_swap_in_x_and_y() -> Result<(), anyhow::Error> {
//     let addrs = init_addrs();
//     let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

//     Ok(())
// }
