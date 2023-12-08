use anyhow::Ok;
use serial_test::serial;
use shade_multi_test::interfaces::{
    lb_factory,
    lb_pair,
    lb_token,
    snip20,
    utils::DeployedContracts,
};
use shade_protocol::{
    c_std::{ContractInfo, StdError, Uint128, Uint256},
    lb_libraries::{math::u24::U24, types::LBPairInformation},
    liquidity_book::lb_pair::RemoveLiquidity,
    multi_test::App,
};
use std::{cmp::Ordering, ops::Add};

use crate::multitests::test_helper::*;

pub const PRECISION: u128 = 1_000_000_000_000_000_000;
pub const ACTIVE_ID: u32 = ID_ONE - 24647;

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

    let lb_token = lb_pair::query_lb_token(&app, &lb_pair.lb_pair.contract)?;

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
#[serial]
pub fn test_simple_mint() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

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

    // query balance for token_minted and calculating the residue
    let silk_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    let expected_batman_balance = amount_x
        - ((amount_x * Uint128::from(PRECISION / nb_bins_x as u128)) / Uint128::from(PRECISION))
            * Uint128::from(nb_bins_x as u128);

    assert_eq!(silk_balance, expected_batman_balance, "test_SimpleMint::1");

    let shd_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    let expected_batman_balance = amount_y
        - ((amount_y * Uint128::from(PRECISION / nb_bins_y as u128)) / Uint128::from(PRECISION))
            * Uint128::from(nb_bins_y as u128);

    assert_eq!(shd_balance, expected_batman_balance, "test_SimpleMint::2");

    let total_bins = get_total_bins(nb_bins_x, nb_bins_y) as u32;

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
        let (reserves_x, reserves_y) = lb_pair::query_bin(&app, &lb_pair.lb_pair.contract, id)?;

        match id.cmp(&ACTIVE_ID) {
            Ordering::Less => {
                assert_eq!(reserves_x, 0u128, "test_sample_mint::3");
                assert_eq!(
                    reserves_y,
                    ((amount_y * Uint128::from(PRECISION / nb_bins_y as u128))
                        / Uint128::from(PRECISION))
                    .u128(),
                    "test_sample_mint::4"
                );
            }
            Ordering::Equal => {
                assert_approx_eq_rel(
                    Uint256::from(reserves_x),
                    Uint256::from(
                        ((amount_x * Uint128::from(PRECISION / nb_bins_x as u128))
                            / Uint128::from(PRECISION))
                        .u128(),
                    ),
                    Uint256::from(1_000_000_000_000_000u128),
                    "test_sample_mint::5",
                );
                assert_approx_eq_rel(
                    Uint256::from(reserves_y),
                    Uint256::from(
                        ((amount_y * Uint128::from(PRECISION / nb_bins_y as u128))
                            / Uint128::from(PRECISION))
                        .u128(),
                    ),
                    Uint256::from(1_000_000_000_000_000u128),
                    "test_sample_mint::6",
                )
            }
            Ordering::Greater => {
                assert_eq!(reserves_y, 0u128, "test_sample_mint::7");
                assert_eq!(
                    reserves_x,
                    ((amount_x * Uint128::from(PRECISION / nb_bins_x as u128))
                        / Uint128::from(PRECISION))
                    .u128(),
                    "test_sample_mint::8"
                );
            }
        }

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        assert!(balance > Uint256::MIN, "test_sample_mint::9");
    }

    Ok(())
}

#[test]
#[serial]
pub fn test_mint_twice() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

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
        liquidity_parameters.clone(),
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

    lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.lb_pair.contract,
        liquidity_parameters,
    )?;

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, nb_bins_y);
        let (reserves_x, reserves_y) = lb_pair::query_bin(&app, &lb_pair.lb_pair.contract, id)?;

        match id.cmp(&ACTIVE_ID) {
            Ordering::Less => {
                assert_eq!(reserves_x, 0u128, "test_sample_mint::3");
                assert_eq!(
                    reserves_y,
                    2 * ((amount_y * Uint128::from(PRECISION / nb_bins_y as u128))
                        / Uint128::from(PRECISION))
                    .u128(),
                    "test_sample_mint::4"
                );
            }
            Ordering::Equal => {
                assert_approx_eq_rel(
                    Uint256::from(reserves_x),
                    Uint256::from(
                        2 * ((amount_x * Uint128::from(PRECISION / nb_bins_x as u128))
                            / Uint128::from(PRECISION))
                        .u128(),
                    ),
                    Uint256::from(1_000_000_000_000_000_u128),
                    "test_sample_mint::5",
                );
                assert_approx_eq_rel(
                    Uint256::from(reserves_y),
                    Uint256::from(
                        2 * ((amount_y * Uint128::from(PRECISION / nb_bins_y as u128))
                            / Uint128::from(PRECISION))
                        .u128(),
                    ),
                    Uint256::from(1_000_000_000_000_000_u128),
                    "test_sample_mint::6",
                )
            }
            Ordering::Greater => {
                assert_eq!(reserves_y, 0u128, "test_sample_mint::7");
                assert_eq!(
                    reserves_x,
                    2 * ((amount_x * Uint128::from(PRECISION / nb_bins_x as u128))
                        / Uint128::from(PRECISION))
                    .u128(),
                    "test_sample_mint::8"
                );
            }
        }

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        assert_eq!(
            balance,
            balances[i as usize] + balances[i as usize],
            "test_sample_mint::9"
        );
    }

    Ok(())
}

#[test]
#[serial]
pub fn test_mint_with_different_bins() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;
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

    Ok(())
}

#[test]
#[serial]
pub fn test_simple_burn() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;
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

    let shd_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance, amount_y);

    let silk_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(silk_balance, amount_x);

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

        assert_eq!(balance, Uint256::MIN);
    }

    Ok(())
}

#[test]
#[serial]
pub fn test_burn_half_twice() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;
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

    let residue_silk_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    let residue_shd_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
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

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;

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
            amounts: half_balances,
            deadline: 99999999999,
        },
    )?;

    let silk_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_approx_eq_rel(
        Uint256::from(silk_balance),
        Uint256::from(reserves_x / 2),
        Uint256::from(10_000_000_000u128),
        "test_burn__half_twice::1",
    );

    let shd_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_approx_eq_rel(
        Uint256::from(shd_balance),
        Uint256::from(reserves_y / 2),
        Uint256::from(10_000_000_000u128),
        "test_burn__half_twice::2",
    );

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

    let silk_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(
        silk_balance.u128(),
        reserves_x + residue_silk_balance.u128()
    );

    let shd_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(shd_balance.u128(), reserves_y + residue_shd_balance.u128());

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.lb_pair.contract)?;

    assert_eq!(reserves_x, 0u128);
    assert_eq!(reserves_y, 0u128);

    Ok(())
}

#[test]
#[serial]
pub fn test_query_next_non_empty_bin() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;
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

    //calculate lower id
    let lower_id = ACTIVE_ID - nb_bins_y as u32 + 1u32;
    //calculate upper id
    let upper_id = ACTIVE_ID + nb_bins_y as u32 - 1u32;

    let mut id = lb_pair::query_next_non_empty_bin(&app, &lb_pair.lb_pair.contract, false, 0)?;
    assert_eq!(lower_id, id);

    let total_bins = get_total_bins(nb_bins_x, nb_bins_y) as u32;

    for i in 0..(total_bins - 1u32) {
        id = lb_pair::query_next_non_empty_bin(&app, &lb_pair.lb_pair.contract, false, id)?;
        assert_eq!(lower_id + i + 1u32, id);
    }

    let mut id =
        lb_pair::query_next_non_empty_bin(&app, &lb_pair.lb_pair.contract, true, U24::MAX)?;
    assert_eq!(upper_id, id);

    let mut balances = vec![Uint256::zero(); 1_usize];
    let mut ids = vec![0u32; 1_usize];

    ids[0] = ACTIVE_ID;
    balances[0] = lb_token::query_balance(
        &app,
        &lb_token,
        addrs.batman(),
        addrs.batman(),
        String::from("viewing_key"),
        ACTIVE_ID.to_string(),
    )?;

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

    id = lb_pair::query_next_non_empty_bin(
        &app,
        &lb_pair.lb_pair.contract,
        false,
        ACTIVE_ID - 1u32,
    )?;

    assert_eq!(id, ACTIVE_ID + 1);

    id =
        lb_pair::query_next_non_empty_bin(&app, &lb_pair.lb_pair.contract, true, ACTIVE_ID + 1u32)?;

    assert_eq!(id, ACTIVE_ID - 1);

    Ok(())
}

#[test]
#[serial]
pub fn test_revert_mint_zero_shares() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;
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
#[serial]
pub fn test_revert_burn_empty_array() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;
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
#[serial]
pub fn test_revert_burn_more_than_balance() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;
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
#[serial]
pub fn test_revert_burn_zero() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;
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
