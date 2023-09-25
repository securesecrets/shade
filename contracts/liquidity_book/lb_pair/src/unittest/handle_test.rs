use core::result::Result::Ok;
use std::str::FromStr;

use cosmwasm_std::{Uint128, Uint256};
use shade_multi_test::interfaces::{lb_pair, lb_token, snip20};
use shade_protocol::contract_interfaces::liquidity_book::lb_token::QueryAnswer;

use super::test_helper::{init_addrs, remove_liquidity_parameters_helper};
use crate::unittest::test_helper::{
    assert_approx_eq_rel, init_lb_pair, liquidity_parameters_helper,
    lp_tokens_tempate_for_100_sscrts, mint_increase_allowance_helper,
};

/***********************************************
 * Workflow of Init *
 ***********************************************
 *
 *  1. lb-pair ---initializes---> lb-token
 *
 *  2. lb-pair ---register_receive + set_viewing_key---> Snip-20  (x2)
 *
 ***********************************************/
#[test]
fn test_init() -> Result<(), anyhow::Error> {
    // init snip-20 x2

    let (app, lb_pair_contract_info, deployed_contracts) = init_lb_pair()?;

    let lb_token_info = lb_pair::lb_token_query(&app, &lb_pair_contract_info.clone().into())?;

    let contract_info_lb_token = lb_token::contract_info_query(&app, &lb_token_info)?;

    match contract_info_lb_token {
        QueryAnswer::TokenContractInfo { curators, .. } => {
            assert_eq!(curators[0], lb_pair_contract_info.address)
        }
        _ => (),
    }

    let result = snip20::balance_query(
        &app,
        lb_pair_contract_info.address.as_str(),
        &deployed_contracts,
        "SHD",
        "viewing_key".to_string(),
    )?;

    assert_eq!(result, Uint128::zero());

    Ok(())
}

/***********************************************
 * Workflow of Init *
 ***********************************************
 *
 *  1. lb-pair ---transfer's itself---> Snip-20
 *
 *  2. lb-pair ---mint tokens---> lb-token
 *
 ***********************************************/
#[test]
fn test_add_liquidity() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_pair_contract_info, deployed_contracts) = init_lb_pair()?;
    let lb_token_info = lb_pair::lb_token_query(&app, &lb_pair_contract_info.clone().into())?;

    //add minter -> mint tokens -> set_vk -> check balance
    mint_increase_allowance_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        &lb_pair_contract_info,
    )?;

    //add_liquidity
    let liquidity_parameters = liquidity_parameters_helper(
        &deployed_contracts,
        Uint128::from(100 * 1000_000u128),
        Uint128::from(100 * 1000_000u128),
    )?;
    lb_pair::add_liquidity(
        &mut app,
        addrs.user1().as_str(),
        &lb_pair_contract_info.clone().into(),
        liquidity_parameters.clone(),
    )?;

    // query lb-pair SHD balance for token_minted
    let balance = snip20::balance_query(
        &mut app,
        lb_pair_contract_info.address.as_str(),
        &deployed_contracts,
        "SHD",
        "viewing_key".to_owned(),
    )?;
    assert_eq!(balance, Uint128::from(99999996u128));

    // query balance SCRT for token_minted
    let balance = snip20::balance_query(
        &mut app,
        lb_pair_contract_info.address.as_str(),
        &deployed_contracts,
        "SSCRT",
        "viewing_key".to_owned(),
    )?;
    assert_eq!(balance, Uint128::from(99999996u128));

    //Checking share/tokens minted after add_liq
    //p.s: values are calculated and are assumed to be correct
    let log_shares_array = lp_tokens_tempate_for_100_sscrts()?;

    let mut i = 0;
    for id in log_shares_array {
        let liquidity = lb_token::id_balance_query(&app, &lb_token_info, id.0.to_string())?;

        match liquidity {
            shade_protocol::liquidity_book::lb_token::QueryAnswer::IdTotalBalance { amount } => {
                assert_eq!(&amount, id.1)
            }
            _ => (),
        }

        let bin_reserves = lb_pair::bin_query(&app, &lb_pair_contract_info.clone().into(), id.0)?;

        assert_eq!(
            bin_reserves.0,
            Uint128::from(
                (Uint128::from(100 * 1000000u128).u128()
                    * liquidity_parameters.distribution_x[i] as u128)
            )
            .multiply_ratio(1u128, 1e18 as u128)
            .u128()
        );

        i += 1;
    }
    Ok(())
}

/***********************************************
 * Workflow of Init *
 ***********************************************
 *
 *  1. lb-pair ---transfer's itself---> Snip-20
 *
 *  2. lb-pair ---mint tokens---> lb-token
 *
 ***********************************************/
#[test]
fn test_remove_liquidity() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_pair_contract_info, deployed_contracts) = init_lb_pair()?;
    let lb_token_info = lb_pair::lb_token_query(&app, &lb_pair_contract_info.clone().into())?;

    //add minter -> mint tokens -> set_vk -> check balance
    mint_increase_allowance_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        &lb_pair_contract_info,
    )?;

    // query lb-pair balance
    let balance = snip20::balance_query(
        &mut app,
        lb_pair_contract_info.address.as_str(),
        &deployed_contracts,
        "SHD",
        "viewing_key".to_owned(),
    )?;
    assert_eq!(balance, Uint128::from(0u128));

    //add_liquidity
    let liquidity_parameters = liquidity_parameters_helper(
        &deployed_contracts,
        Uint128::from(100 * 1000000u128),
        Uint128::from(100 * 1000000u128),
    )?;
    lb_pair::add_liquidity(
        &mut app,
        addrs.user1().as_str(),
        &lb_pair_contract_info.clone().into(),
        liquidity_parameters.clone(),
    )?;

    // query lb-pair balance and user balance
    let balance = snip20::balance_query(
        &mut app,
        lb_pair_contract_info.address.as_str(),
        &deployed_contracts,
        "SHD",
        "viewing_key".to_owned(),
    )?;
    assert_eq!(balance, Uint128::from(99999996u128));

    let balance = snip20::balance_query(
        &mut app,
        addrs.user1().as_str(),
        &deployed_contracts,
        "SHD",
        "viewing_key".to_owned(),
    )?;
    assert_eq!(balance, Uint128::from(900_000_004u128));

    //removes parital liquidity.
    let percentage_removed = 50;
    let (remove_liquidity_parameters, remove_liq_log) =
        remove_liquidity_parameters_helper(&deployed_contracts, percentage_removed)?;
    //removing 50% of the liquidity.
    lb_pair::remove_liquidity(
        &mut app,
        addrs.user1().as_str(),
        &lb_pair_contract_info.clone().into(),
        remove_liquidity_parameters.clone(),
    )?;

    // query lb-pair balance and user balance
    let balance = snip20::balance_query(
        &mut app,
        lb_pair_contract_info.address.as_str(),
        &deployed_contracts,
        "SHD",
        "viewing_key".to_owned(),
    )?;
    assert_approx_eq_rel(balance.u128(), 50_000_000u128, 100u128);

    let balance = snip20::balance_query(
        &mut app,
        addrs.user1().as_str(),
        &deployed_contracts,
        "SHD",
        "viewing_key".to_owned(),
    )?;
    assert_approx_eq_rel(balance.u128(), 950_000_000u128, 100u128);

    //check user balance and lb-pair balance for SHD
    // start: user-balance = 1000*10^6 ,lb-pair-balance = 0
    // add_liquidity: user-balance = 9000*10^6 ,lb-pair-balance = 100*10^6

    let mut i = 0;
    for (id, amt) in remove_liq_log {
        let liquidity = lb_token::id_balance_query(&app, &lb_token_info, id.to_string())?;

        match liquidity {
            shade_protocol::liquidity_book::lb_token::QueryAnswer::IdTotalBalance { amount } => {
                assert_eq!(&amount, amt)
            }
            _ => (),
        }

        //get bin_reserves
        let bin_reserves = lb_pair::bin_query(&app, &lb_pair_contract_info.clone().into(), id)?;

        assert_eq!(
            bin_reserves.0,
            Uint128::from(
                Uint128::from((100u8 - percentage_removed) as u128 * 1000000u128).u128()
                    * liquidity_parameters.distribution_x[i] as u128
            )
            .multiply_ratio(1u128, 1e18 as u128)
            .u128()
        );

        i += 1;
    }

    //remove all:
    let (remove_liquidity_parameters, remove_liq_log) =
        remove_liquidity_parameters_helper(&deployed_contracts, percentage_removed)?;
    //removing 50% of the liquidity.
    lb_pair::remove_liquidity(
        &mut app,
        addrs.user1().as_str(),
        &lb_pair_contract_info.clone().into(),
        remove_liquidity_parameters.clone(),
    )?;

    // query lb-pair balance and user balance
    let balance = snip20::balance_query(
        &mut app,
        lb_pair_contract_info.address.as_str(),
        &deployed_contracts,
        "SHD",
        "viewing_key".to_owned(),
    )?;
    assert_approx_eq_rel(0u128, balance.u128(), 100u128);

    let balance = snip20::balance_query(
        &mut app,
        addrs.user1().as_str(),
        &deployed_contracts,
        "SHD",
        "viewing_key".to_owned(),
    )?;
    assert_approx_eq_rel(balance.u128(), 1000_000_000u128, 100u128);

    Ok(())
}

#[test]
fn test_swap_liquidity() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_pair_contract_info, deployed_contracts) = init_lb_pair()?;
    let lb_token_info = lb_pair::lb_token_query(&app, &lb_pair_contract_info.clone().into())?;

    //add minter -> mint tokens -> set_vk -> check balance
    mint_increase_allowance_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        &lb_pair_contract_info,
    )?;

    // query lb-pair balance
    let balance = snip20::balance_query(
        &mut app,
        lb_pair_contract_info.address.as_str(),
        &deployed_contracts,
        "SHD",
        "viewing_key".to_owned(),
    )?;
    assert_eq!(balance, Uint128::from(0u128));

    //add_liquidity 1:1 for SSCRT - SHD
    let liquidity_parameters = liquidity_parameters_helper(
        &deployed_contracts,
        Uint128::from(100 * 1000000u128),
        Uint128::from(100 * 1000000u128),
    )?;
    lb_pair::add_liquidity(
        &mut app,
        addrs.user1().as_str(),
        &lb_pair_contract_info.clone().into(),
        liquidity_parameters.clone(),
    )?;

    let (amount_in, amount_out_left, fee) = lb_pair::swap_in_query(
        &app,
        &lb_pair_contract_info.clone().into(),
        Uint128::from(10_000_000u128),
        true,
    )?;

    println!(
        "amount_in {:?}, amount_out_left {:?}, fee {:?}",
        amount_in, amount_out_left, fee
    );

    let (amount_out, amount_in_left, fee) = lb_pair::swap_out_query(
        &app,
        &lb_pair_contract_info.clone().into(),
        Uint128::from(10_000_000u128),
        true,
    )?;

    println!(
        "amount_out {:?}, amount_in_left {:?}, fee {:?}",
        amount_out, amount_in_left, fee
    );

    //make swap and check balances

    Ok(())
}
