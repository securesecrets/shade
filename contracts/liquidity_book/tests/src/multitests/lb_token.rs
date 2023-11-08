use std::ops::{Add, Mul};

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
use shade_multi_test::interfaces::{lb_factory, lb_pair, lb_token, utils::DeployedContracts};
use shade_protocol::{
    lb_libraries::{
        constants::SCALE_OFFSET,
        math::uint256_to_u256::ConvertU256,
        types::LBPairInformation,
    },
    liquidity_book::lb_pair::RemoveLiquidity,
    multi_test::App,
};

pub const PRECISION: u128 = 1_000_000_000_000_000_000_u128;

pub const ACTIVE_ID: u32 = ID_ONE - 24647;

pub fn init_setup() -> Result<
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
    let (mut app, lb_factory, deployed_contracts) = setup(None, None)?;

    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let shade = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_x = token_type_snip20_generator(&silk)?;
    let token_y = token_type_snip20_generator(&shade)?;

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
    Ok((
        app,
        lb_factory.into(),
        deployed_contracts,
        lb_pair,
        lb_token,
    ))
}

#[test]
pub fn test_simple_mint() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = init_setup()?;

    let amount_x = Uint128::from(600 * 100_000_000_u128); //10^8
    let amount_y = Uint128::from(100 * 100_000_000_u128);
    let nb_bins_x = 6;
    let nb_bins_y = 6;

    let token_x = extract_contract_info(&deployed_contracts, SILK)?;
    let token_y = extract_contract_info(&deployed_contracts, SHADE)?;

    let tokens_to_mint = vec![(SILK, amount_x), (SHADE, amount_y)];

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

    let total_bins = get_total_bins(nb_bins_x, nb_bins_y) as u32;

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
        let (reserves_x, reserves_y) = lb_pair::query_bin(&app, &lb_pair.lb_pair.contract, id)?;
        let price = lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, id)?;

        let expected_balance_x = U256::from(reserves_x);
        let expected_balance_y = U256::from(reserves_y);

        let total: U256 = expected_balance_x * U256::from_str_prefixed(price.to_string().as_str())?
            + (expected_balance_y << 128);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        assert_eq!(total.u256_to_uint256(), balance);
        assert!(balance > Uint256::MIN, "test_sample_mint::9");
    }

    Ok(())
}

#[test]
pub fn test_mint_twice() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = init_setup()?;

    let amount_x = Uint128::from(600 * 100_000_000_u128); //10^8
    let amount_y = Uint128::from(100 * 100_000_000_u128);
    let nb_bins_x = 6;
    let nb_bins_y = 6;

    let token_x = extract_contract_info(&deployed_contracts, SILK)?;
    let token_y = extract_contract_info(&deployed_contracts, SHADE)?;

    let tokens_to_mint = vec![
        (SILK, amount_x.mul(Uint128::from(2u128))),
        (SHADE, amount_y.mul(Uint128::from(2u128))),
    ];

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
    let total_bins = get_total_bins(nb_bins_x, nb_bins_y) as u32;

    lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        liquidity_parameters.clone(),
    )?;

    let mut total: Vec<U256> = vec![U256::ZERO; total_bins as usize];
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
        let (reserves_x, reserves_y) = lb_pair::query_bin(&app, &lb_pair.lb_pair.contract, id)?;
        let price = lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, id)?;

        let expected_balance_x = U256::from(reserves_x);
        let expected_balance_y = U256::from(reserves_y);

        total[i as usize] = expected_balance_x
            * U256::from_str_prefixed(price.to_string().as_str())?
            + (expected_balance_y << 128);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        assert_eq!(total[i as usize].u256_to_uint256(), balance);
        assert!(balance > Uint256::MIN, "test_sample_mint::9");
    }

    lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        liquidity_parameters,
    )?;

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        assert_eq!(
            total[i as usize]
                .u256_to_uint256()
                .mul(Uint256::from(2u128)),
            balance
        );
        assert!(balance > Uint256::MIN, "test_sample_mint::9");
    }

    Ok(())
}

#[test]
pub fn test_mint_with_different_bins() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = init_setup()?;
    let amount_x = Uint128::from(600 * 100_000_000_u128); //10^8
    let amount_y = Uint128::from(100 * 100_000_000_u128);
    let nb_bins_x = 6;
    let nb_bins_y = 6;

    let token_x = extract_contract_info(&deployed_contracts, SILK)?;
    let token_y = extract_contract_info(&deployed_contracts, SHADE)?;

    let tokens_to_mint = vec![(SILK, amount_x + amount_x), (SHADE, amount_y + amount_y)];

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

    // Adding liquidity with nb_bins_x and nb_bins_y
    let liquidity_parameters = liquidity_parameters_generator(
        &deployed_contracts,
        ACTIVE_ID,
        token_x.clone(),
        token_y.clone(),
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

    let total_bins = get_total_bins(nb_bins_x, nb_bins_y) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
        balances[i as usize] = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
    }

    // Adding liquidity with nb_bins_x and 0 for nb_bins_y
    let liquidity_parameters = liquidity_parameters_generator(
        &deployed_contracts,
        ACTIVE_ID,
        token_x.clone(),
        token_y.clone(),
        amount_x,
        amount_y,
        nb_bins_x,
        0,
    )?;

    lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        liquidity_parameters,
    )?;

    let liquidity_parameters = liquidity_parameters_generator(
        &deployed_contracts,
        ACTIVE_ID,
        token_x,
        token_y,
        amount_x,
        amount_y,
        0,
        nb_bins_y,
    )?;

    lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        liquidity_parameters,
    )?;

    // Verify
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        if id == ACTIVE_ID {
            assert_eq!(
                balance,
                balances[i as usize] + balances[i as usize],
                "test_MintWithDifferentBins::1",
            );
        } else {
            assert_eq!(
                balance,
                balances[i as usize] + balances[i as usize],
                "test_MintWithDifferentBins::2"
            );
        }
    }

    let total_bins = get_total_bins(nb_bins_x, nb_bins_y) as u32;

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
        let (reserves_x, reserves_y) = lb_pair::query_bin(&app, &lb_pair.lb_pair.contract, id)?;
        let price = lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, id)?;

        let expected_balance_x = U256::from(reserves_x);
        let expected_balance_y = U256::from(reserves_y);

        let total: U256 = expected_balance_x * U256::from_str_prefixed(price.to_string().as_str())?
            + (expected_balance_y << 128);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        assert_eq!(total.u256_to_uint256(), balance);
        assert!(balance > Uint256::MIN, "test_sample_mint::9");
    }

    Ok(())
}

#[test]
pub fn test_simple_burn() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = init_setup()?;
    let amount_x = Uint128::from(600 * 100_000_000_u128); //10^8
    let amount_y = Uint128::from(100 * 100_000_000_u128);
    let nb_bins_x = 6;
    let nb_bins_y = 6;

    let token_x = extract_contract_info(&deployed_contracts, SILK)?;
    let token_y = extract_contract_info(&deployed_contracts, SHADE)?;

    let tokens_to_mint = vec![(SILK, amount_x), (SHADE, amount_y)];

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

    // Adding liquidity with nb_bins_x and nb_bins_y
    let liquidity_parameters = liquidity_parameters_generator(
        &deployed_contracts,
        ACTIVE_ID,
        token_x.clone(),
        token_y.clone(),
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

    let total_bins = get_total_bins(nb_bins_x, nb_bins_y) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
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

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(reserves_x, 0u128);
    assert_eq!(reserves_y, 0u128);

    let total_bins = get_total_bins(nb_bins_x, nb_bins_y) as u32;

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        assert_eq!(balance, Uint256::zero());
    }

    let total_bins = get_total_bins(nb_bins_x, nb_bins_y) as u32;

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
        let (reserves_x, reserves_y) = lb_pair::query_bin(&app, &lb_pair.lb_pair.contract, id)?;
        let price = lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, id)?;

        let expected_balance_x = U256::from(reserves_x);
        let expected_balance_y = U256::from(reserves_y);

        let total: U256 = expected_balance_x * U256::from_str_prefixed(price.to_string().as_str())?
            + (expected_balance_y << 128);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        assert_eq!(total.u256_to_uint256(), balance);
    }

    Ok(())
}

#[test]
pub fn test_burn_half_twice() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = init_setup()?;
    let amount_x = Uint128::from(600 * 100_000_000_u128); //10^8
    let amount_y = Uint128::from(100 * 100_000_000_u128);
    let nb_bins_x = 6;
    let nb_bins_y = 6;

    let token_x = extract_contract_info(&deployed_contracts, SILK)?;
    let token_y = extract_contract_info(&deployed_contracts, SHADE)?;

    let tokens_to_mint = vec![(SILK, amount_x), (SHADE, amount_y)];

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

    // Adding liquidity with nb_bins_x and nb_bins_y
    let liquidity_parameters = liquidity_parameters_generator(
        &deployed_contracts,
        ACTIVE_ID,
        token_x.clone(),
        token_y.clone(),
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

    let total_bins = get_total_bins(nb_bins_x, nb_bins_y) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut half_balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
        ids[i as usize] = id;
        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
        half_balances[i as usize] = balance / Uint256::from(2u128);
        balances[i as usize] = balance - half_balances[i as usize];
    }

    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(1u128),
            amount_y_min: Uint128::from(1u128),
            ids: ids.clone(),
            amounts: half_balances.clone(),
            deadline: 99999999999,
        },
    )?;

    let mut total: Vec<U256> = vec![U256::ZERO; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        assert_eq!(balance, balances[i as usize]);
    }

    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(1u128),
            amount_y_min: Uint128::from(1u128),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    )?;

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
        let (reserves_x, reserves_y) = lb_pair::query_bin(&app, &lb_pair.lb_pair.contract, id)?;
        let price = lb_pair::query_price_from_id(&app, &lb_pair.lb_pair.contract, id)?;

        let expected_balance_x = U256::from(reserves_x);
        let expected_balance_y = U256::from(reserves_y);

        total[i as usize] = expected_balance_x
            * U256::from_str_prefixed(price.to_string().as_str())?
            + (expected_balance_y << SCALE_OFFSET);

        assert_eq!(total[i as usize].u256_to_uint256(), Uint256::zero());
    }

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(reserves_x, 0u128);
    assert_eq!(reserves_y, 0u128);

    Ok(())
}

#[test]
pub fn test_revert_mint_zero_tokens() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = init_setup()?;
    let amount_x = Uint128::from(0u128); //10^8
    let amount_y = Uint128::from(0u128);
    let nb_bins_x = 6;
    let nb_bins_y = 6;

    let token_x = extract_contract_info(&deployed_contracts, SILK)?;
    let token_y = extract_contract_info(&deployed_contracts, SHADE)?;

    let tokens_to_mint = vec![(SILK, amount_x), (SHADE, amount_y)];

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

    // Adding liquidity with nb_bins_x and nb_bins_y
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

    let res = lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        liquidity_parameters,
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err(format!(
            "Zero amount for bin id: {:?}",
            ACTIVE_ID - nb_bins_y as u32 + 1
        ))
    );

    Ok(())
}

#[test]
pub fn test_revert_burn_empty_array() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = init_setup()?;
    let amount_x = Uint128::from(600 * 100_000_000_u128); //10^8
    let amount_y = Uint128::from(100 * 100_000_000_u128);
    let nb_bins_x = 6;
    let nb_bins_y = 6;

    let token_x = extract_contract_info(&deployed_contracts, SILK)?;
    let token_y = extract_contract_info(&deployed_contracts, SHADE)?;

    let tokens_to_mint = vec![(SILK, amount_x), (SHADE, amount_y)];

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

    // Adding liquidity with nb_bins_x and nb_bins_y
    let liquidity_parameters = liquidity_parameters_generator(
        &deployed_contracts,
        ACTIVE_ID,
        token_x.clone(),
        token_y.clone(),
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

    //uneven
    let mut ids = vec![];
    let mut balances = vec![Uint256::zero()];

    let res = lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(0u128),
            amount_y_min: Uint128::from(0u128),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err("Invalid input!".to_string())
    );

    //uneven input
    ids = vec![0u32];
    balances = vec![];

    let res = lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(0u128),
            amount_y_min: Uint128::from(0u128),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err("Invalid input!".to_string())
    );

    //both zero
    ids = vec![];
    balances = vec![];
    let res = lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(0u128),
            amount_y_min: Uint128::from(0u128),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err("Invalid input!".to_string())
    );

    // non-zero values
    ids = vec![ACTIVE_ID];
    let balances = vec![Uint256::one(), Uint256::one()];

    let res = lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(0u128),
            amount_y_min: Uint128::from(0u128),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err("Invalid input!".to_string())
    );

    Ok(())
}

#[test]
pub fn test_revert_burn_more_than_balance() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = init_setup()?;
    let amount_x = Uint128::from(600 * 100_000_000_u128); //10^8
    let amount_y = Uint128::from(100 * 100_000_000_u128);
    let nb_bins_x = 1;
    let nb_bins_y = 0;

    let token_x = extract_contract_info(&deployed_contracts, SILK)?;
    let token_y = extract_contract_info(&deployed_contracts, SHADE)?;

    let tokens_to_mint = vec![(SILK, amount_x), (SHADE, amount_y)];

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

    // Adding liquidity with nb_bins_x and nb_bins_y
    let liquidity_parameters = liquidity_parameters_generator(
        &deployed_contracts,
        ACTIVE_ID,
        token_x.clone(),
        token_y.clone(),
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

    //uneven

    let balance = lb_token::query_balance(
        &app,
        &lb_token,
        addrs.batman(),
        addrs.batman(),
        String::from("viewing_key"),
        ACTIVE_ID.to_string(),
    )?;

    let ids = vec![ACTIVE_ID];
    let balances = vec![balance.add(Uint256::one())];

    let res = lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(0u128),
            amount_y_min: Uint128::from(0u128),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err("Generic error: insufficient funds".to_string())
    );

    Ok(())
}

#[test]
pub fn test_revert_burn_zero() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = init_setup()?;
    let amount_x = Uint128::from(600 * 100_000_000_u128); //10^8
    let amount_y = Uint128::from(100 * 100_000_000_u128);
    let nb_bins_x = 1;
    let nb_bins_y = 0;

    let token_x = extract_contract_info(&deployed_contracts, SILK)?;
    let token_y = extract_contract_info(&deployed_contracts, SHADE)?;

    let tokens_to_mint = vec![(SILK, amount_x), (SHADE, amount_y)];

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

    // Adding liquidity with nb_bins_x and nb_bins_y
    let liquidity_parameters = liquidity_parameters_generator(
        &deployed_contracts,
        ACTIVE_ID,
        token_x.clone(),
        token_y.clone(),
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

    //uneven

    let ids = vec![ACTIVE_ID];
    let balances = vec![Uint256::zero()];

    let res = lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        RemoveLiquidity {
            token_x: token_type_snip20_generator(&token_x)?,
            token_y: token_type_snip20_generator(&token_y)?,
            bin_step: lb_pair.bin_step,
            amount_x_min: Uint128::from(0u128),
            amount_y_min: Uint128::from(0u128),
            ids,
            amounts: balances,
            deadline: 99999999999,
        },
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err(format!("Zero Shares for bin id: {:?}", ACTIVE_ID))
    );

    Ok(())
}
