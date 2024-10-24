use anyhow::Ok;
use cosmwasm_std::{BlockInfo, Timestamp};
use serial_test::serial;
use shade_multi_test::interfaces::{lb_factory, lb_pair, lb_token, utils::DeployedContracts};
use shade_protocol::{
    c_std::{ContractInfo, Uint128, Uint256},
    liquidity_book::lb_pair::{LbPairInformation, RemoveLiquidity},
    multi_test::App,
    utils::cycle::parse_utc_datetime,
};
use std::{cmp::Ordering, ops::Add};

use crate::multitests::test_helper::*;

pub const PRECISION: u128 = 1_000_000_000_000_000_000;
pub const MARGIN_OF_ERROR: u128 = 1_000_000_000_000_000; //0.1%
pub const ACTIVE_ID: u32 = ID_ONE;
pub const NB_BINS_X: u32 = 10;
pub const NB_BINS_Y: u32 = 10;
pub const DEPOSIT_AMOUNT: u128 = 1_000_000_000_000_000_000_u128;

pub fn lb_pair_setup() -> Result<
    (
        App,
        ContractInfo,
        DeployedContracts,
        LbPairInformation,
        ContractInfo,
    ),
    anyhow::Error,
> {
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _, _) = setup(None, None)?;

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

    let lb_token = lb_pair::query_lb_token(&app, &lb_pair.info.contract)?;

    lb_token::set_viewing_key(
        &mut app,
        addrs.batman().as_str(),
        &lb_token,
        "viewing_key".to_owned(),
    )?;
    let amount_x = Uint128::from(DEPOSIT_AMOUNT); //10^8
    let amount_y = Uint128::from(DEPOSIT_AMOUNT);

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
        NB_BINS_X,
        NB_BINS_Y,
    )?;

    lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
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

fn mint_and_add_liquidity(
    app: &mut App,
    deployed_contracts: &DeployedContracts,
    addrs: &Addrs,
    lb_pair: &LbPairInformation,
    nb_bins_x: Option<u32>,
    nb_bins_y: Option<u32>,
    deposit_amount_x: u128, // New argument for deposit amount
    deposit_amount_y: u128, // New argument for deposit amount
) -> Result<(), anyhow::Error> {
    let amount_x = Uint128::from(deposit_amount_x);
    let amount_y = Uint128::from(deposit_amount_y);

    let token_x = extract_contract_info(&deployed_contracts, SILK)?;
    let token_y = extract_contract_info(&deployed_contracts, SHADE)?;

    let tokens_to_mint = vec![(SILK, amount_x), (SHADE, amount_y)];

    mint_token_helper(
        app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;

    increase_allowance_helper(
        app,
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
        nb_bins_x.unwrap_or(NB_BINS_X),
        nb_bins_y.unwrap_or(NB_BINS_Y),
    )?;

    lb_pair::add_liquidity(
        app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        liquidity_parameters,
    )?;
    Ok(())
}

#[test]
#[serial]
pub fn test_query_bin_reserves() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (app, _lb_factory, _deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    let total_bins = get_total_bins(NB_BINS_X, NB_BINS_Y) as u32;
    let amount_x = Uint128::from(DEPOSIT_AMOUNT); //10^8
    let amount_y = Uint128::from(DEPOSIT_AMOUNT);

    let mut ids = vec![];
    let mut reserves: Vec<(u128, u128)> = vec![];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, NB_BINS_Y);
        ids.push(id);
        let (reserves_x, reserves_y, _bin_id) =
            lb_pair::query_bin_reserves(&app, &lb_pair.info.contract, id)?;
        reserves.push((reserves_x, reserves_y));
        match id.cmp(&ACTIVE_ID) {
            Ordering::Less => {
                assert_eq!(reserves_x, 0u128, "test_sample_mint::3");
                assert_eq!(
                    reserves_y,
                    ((amount_y * Uint128::from(PRECISION / NB_BINS_Y as u128))
                        / Uint128::from(PRECISION))
                    .u128(),
                    "test_sample_mint::4"
                );
            }
            Ordering::Equal => {
                assert_approx_eq_rel(
                    Uint256::from(reserves_x),
                    Uint256::from(
                        ((amount_x * Uint128::from(PRECISION / NB_BINS_X as u128))
                            / Uint128::from(PRECISION))
                        .u128(),
                    ),
                    Uint256::from(MARGIN_OF_ERROR),
                    "test_sample_mint::5",
                );
                assert_approx_eq_rel(
                    Uint256::from(reserves_y),
                    Uint256::from(
                        ((amount_y * Uint128::from(PRECISION / NB_BINS_Y as u128))
                            / Uint128::from(PRECISION))
                        .u128(),
                    ),
                    Uint256::from(MARGIN_OF_ERROR),
                    "test_sample_mint::6",
                )
            }
            Ordering::Greater => {
                assert_eq!(reserves_y, 0u128, "test_sample_mint::7");
                assert_eq!(
                    reserves_x,
                    ((amount_x * Uint128::from(PRECISION / NB_BINS_X as u128))
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
pub fn test_query_bins_reserves() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (app, _lb_factory, _deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    let total_bins = get_total_bins(NB_BINS_X, NB_BINS_Y) as u32;
    let amount_x = Uint128::from(DEPOSIT_AMOUNT); //10^8
    let amount_y = Uint128::from(DEPOSIT_AMOUNT);

    let mut ids = vec![];
    let mut reserves: Vec<(u128, u128)> = vec![];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, NB_BINS_Y);
        ids.push(id);
    }

    let multiple_reserves =
        lb_pair::query_bins_reserves(&app, &lb_pair.info.contract, ids.clone())?;

    for (res, id) in multiple_reserves.iter().zip(ids.clone()) {
        let (reserves_x, reserves_y) = (res.bin_reserve_x, res.bin_reserve_y);

        reserves.push((reserves_x, reserves_y));

        match id.cmp(&ACTIVE_ID) {
            Ordering::Less => {
                assert_eq!(reserves_x, 0u128, "test_sample_mint::3");
                assert_eq!(
                    reserves_y,
                    ((amount_y * Uint128::from(PRECISION / NB_BINS_Y as u128))
                        / Uint128::from(PRECISION))
                    .u128(),
                    "test_sample_mint::4"
                );
            }
            Ordering::Equal => {
                assert_approx_eq_rel(
                    Uint256::from(reserves_x),
                    Uint256::from(
                        ((amount_x * Uint128::from(PRECISION / NB_BINS_X as u128))
                            / Uint128::from(PRECISION))
                        .u128(),
                    ),
                    Uint256::from(MARGIN_OF_ERROR),
                    "test_sample_mint::5",
                );
                assert_approx_eq_rel(
                    Uint256::from(reserves_y),
                    Uint256::from(
                        ((amount_y * Uint128::from(PRECISION / NB_BINS_Y as u128))
                            / Uint128::from(PRECISION))
                        .u128(),
                    ),
                    Uint256::from(MARGIN_OF_ERROR),
                    "test_sample_mint::6",
                )
            }
            Ordering::Greater => {
                assert_eq!(reserves_y, 0u128, "test_sample_mint::7");
                assert_eq!(
                    reserves_x,
                    ((amount_x * Uint128::from(PRECISION / NB_BINS_X as u128))
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
pub fn test_query_all_bins_reserves() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (app, _lb_factory, _deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    let total_bins = get_total_bins(NB_BINS_X, NB_BINS_Y) as u32;
    let amount_x = Uint128::from(DEPOSIT_AMOUNT); //10^8
    let amount_y = Uint128::from(DEPOSIT_AMOUNT);

    let mut ids = vec![];
    let mut reserves: Vec<(u128, u128)> = vec![];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, NB_BINS_Y);
        ids.push(id);
    }

    let (multiple_reserves, last_id, _) =
        lb_pair::query_all_bins_reserves(&app, &lb_pair.info.contract, None, None, Some(5))?;

    //checking pagination
    assert_eq!(last_id, ids[4]);
    assert_eq!(multiple_reserves.len(), 5);

    let (multiple_reserves, last_id, _) = lb_pair::query_all_bins_reserves(
        &app,
        &lb_pair.info.contract,
        Some(last_id),
        None,
        Some(5),
    )?;

    //checking pagination
    assert_eq!(last_id, ids[9]);
    assert_eq!(multiple_reserves.len(), 5);

    let (multiple_reserves, last_id, _) = lb_pair::query_all_bins_reserves(
        &app,
        &lb_pair.info.contract,
        Some(last_id),
        None,
        Some(5),
    )?;

    //checking pagination
    assert_eq!(last_id, ids[14]);
    assert_eq!(multiple_reserves.len(), 5);

    let (multiple_reserves, last_id, _) = lb_pair::query_all_bins_reserves(
        &app,
        &lb_pair.info.contract,
        Some(last_id),
        None,
        Some(5),
    )?;

    //checking pagination
    assert_eq!(last_id, 0);
    assert_eq!(multiple_reserves.len(), 4);

    let (multiple_reserves, last_id, _) =
        lb_pair::query_all_bins_reserves(&app, &lb_pair.info.contract, None, None, Some(50))?;
    assert_eq!(last_id, 0);

    for res in multiple_reserves.iter() {
        let (reserves_x, reserves_y) = (res.bin_reserve_x, res.bin_reserve_y);
        let id = res.bin_id;

        ids.push(id);
        reserves.push((reserves_x, reserves_y));

        match id.cmp(&ACTIVE_ID) {
            Ordering::Less => {
                assert_eq!(reserves_x, 0u128, "test_sample_mint::3");
                assert_eq!(
                    reserves_y,
                    ((amount_y * Uint128::from(PRECISION / NB_BINS_Y as u128))
                        / Uint128::from(PRECISION))
                    .u128(),
                    "test_sample_mint::4"
                );
            }
            Ordering::Equal => {
                assert_approx_eq_rel(
                    Uint256::from(reserves_x),
                    Uint256::from(
                        ((amount_x * Uint128::from(PRECISION / NB_BINS_X as u128))
                            / Uint128::from(PRECISION))
                        .u128(),
                    ),
                    Uint256::from(MARGIN_OF_ERROR),
                    "test_sample_mint::5",
                );
                assert_approx_eq_rel(
                    Uint256::from(reserves_y),
                    Uint256::from(
                        ((amount_y * Uint128::from(PRECISION / NB_BINS_Y as u128))
                            / Uint128::from(PRECISION))
                        .u128(),
                    ),
                    Uint256::from(MARGIN_OF_ERROR),
                    "test_sample_mint::6",
                )
            }
            Ordering::Greater => {
                assert_eq!(reserves_y, 0u128, "test_sample_mint::7");
                assert_eq!(
                    reserves_x,
                    ((amount_x * Uint128::from(PRECISION / NB_BINS_X as u128))
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
pub fn test_query_all_bins_updated_add_liquidity() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair, _lb_tokenn) = lb_pair_setup()?;

    // add liquidity already made check the ids

    let heights = lb_pair::query_all_bins_updated(&app, &lb_pair.info.contract, None, None)?;

    assert_eq!(heights.len(), 1);
    assert_eq!(heights[0], app.block_info().height);

    Ok(())
}

#[test]
#[serial]
pub fn test_query_total_supply() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, _deployed_contracts, lb_pair, _lb_tokenn) = lb_pair_setup()?;

    let supply = lb_pair::query_total_supply(&app, &lb_pair.info.contract, ACTIVE_ID)?;

    assert!(supply > Uint256::zero());

    Ok(())
}

#[test]
#[serial]
pub fn test_query_tokens() -> Result<(), anyhow::Error> {
    let (app, _lb_factory, deployed_contracts, lb_pair, _lb_tokenn) = lb_pair_setup()?;

    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let shade = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_x = token_type_snip20_generator(&silk)?;
    let token_y = token_type_snip20_generator(&shade)?;

    let (t_x, t_y) = lb_pair::query_tokens(&app, &lb_pair.info.contract)?;

    assert_eq!(t_x, token_x);
    assert_eq!(t_y, token_y);

    Ok(())
}

#[test]
#[serial]
pub fn test_query_all_bins_updated_swap() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    // add liquidity already made check the ids

    let heights = lb_pair::query_all_bins_updated(&app, &lb_pair.info.contract, None, None)?;

    assert_eq!(heights.len(), 1);
    assert_eq!(heights[0], app.block_info().height);
    let prev_height = app.block_info().height;

    app.set_block(BlockInfo {
        height: app.block_info().height.add(1),
        time: Timestamp::from_seconds(
            parse_utc_datetime(&"1995-11-13T00:00:00.00Z".to_string())
                .unwrap()
                .timestamp() as u64,
        ),
        chain_id: "chain_id".to_string(),
        random: None,
    });

    //make a swap and check the ids

    let amount_out = Uint128::from(generate_random(1u128, DEPOSIT_AMOUNT - 1));

    let (amount_in, amount_out_left, _fee) =
        lb_pair::query_swap_in(&app, &lb_pair.info.contract, amount_out, true)?;
    assert_eq!(amount_out_left, Uint128::zero());

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
        &lb_pair.info.contract,
        Some(addrs.batman().to_string()),
        token_x,
        amount_in,
    )?;

    let mut heights = lb_pair::query_all_bins_updated(&app, &lb_pair.info.contract, None, None)?;
    heights.sort();

    assert_eq!(heights.len(), 2);
    assert_eq!(heights[1], prev_height.add(1));

    Ok(())
}

#[test]
#[serial]
pub fn test_query_all_bins_updated_remove_liquidity() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    let nb_bins_x = NB_BINS_X;
    let nb_bins_y = NB_BINS_Y;

    let token_x = extract_contract_info(&deployed_contracts, SILK)?;
    let token_y = extract_contract_info(&deployed_contracts, SHADE)?;

    let heights = lb_pair::query_all_bins_updated(&app, &lb_pair.info.contract, None, None)?;

    assert_eq!(heights.len(), 1);
    assert_eq!(heights[0], app.block_info().height);
    let prev_height = app.block_info().height;

    app.set_block(BlockInfo {
        height: app.block_info().height.add(1),
        time: Timestamp::from_seconds(
            parse_utc_datetime(&"1995-11-13T00:00:00.00Z".to_string())
                .unwrap()
                .timestamp() as u64,
        ),
        chain_id: "chain_id".to_string(),
        random: None,
    });

    let total_bins = get_total_bins(nb_bins_x as u32, nb_bins_y as u32) as u32;
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

    let (reserves_x, reserves_y) = lb_pair::query_reserves(&app, &lb_pair.info.contract)?;

    lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
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
    let mut heights = lb_pair::query_all_bins_updated(&app, &lb_pair.info.contract, None, None)?;
    heights.sort();

    assert_eq!(heights.len(), 2);
    assert_eq!(heights[1], prev_height.add(1));
    Ok(())
}

#[test]
#[serial]
pub fn test_query_update_at_height() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (app, _lb_factory, _deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    let total_bins = get_total_bins(NB_BINS_X as u32, NB_BINS_Y as u32) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, NB_BINS_Y);
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

    // add liquidity already made check the ids
    let heights = lb_pair::query_all_bins_updated(&app, &lb_pair.info.contract, None, None)?;
    assert_eq!(heights.len(), 1);
    assert_eq!(heights[0], app.block_info().height);
    //user have all the heights now

    let height = heights[0];

    let bin_responses =
        lb_pair::query_updated_bins_at_height(&app, &lb_pair.info.contract, height)?;
    let mut bin_ids: Vec<u32> = bin_responses
        .into_iter()
        .map(|response| response.bin_id)
        .collect();
    bin_ids.sort();

    assert_eq!(bin_ids, ids);

    Ok(())
}

#[test]
#[serial]
pub fn test_query_update_at_multiple_heights() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    let total_bins = get_total_bins(NB_BINS_X as u32, NB_BINS_Y as u32) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, NB_BINS_Y);
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

    // add liquidity already made check the ids
    let heights = lb_pair::query_all_bins_updated(&app, &lb_pair.info.contract, None, None)?;
    assert_eq!(heights.len(), 1);
    assert_eq!(heights[0], app.block_info().height);
    //user have all the heights now

    let height = heights[0];

    let query_ids: Vec<u32> =
        lb_pair::query_updated_bins_at_height(&app, &lb_pair.info.contract, height)?
            .into_iter()
            .map(|x| x.bin_id)
            .collect();

    assert_eq!(query_ids, ids);

    roll_blockchain(&mut app, None);

    for _ in 0..49 {
        roll_blockchain(&mut app, None);

        mint_and_add_liquidity(
            &mut app,
            &deployed_contracts,
            &addrs,
            &lb_pair,
            Some(NB_BINS_X),
            Some(NB_BINS_Y),
            DEPOSIT_AMOUNT,
            DEPOSIT_AMOUNT,
        )?;
    }

    // add liquidity already made check the ids
    let mut heights =
        lb_pair::query_all_bins_updated(&app, &lb_pair.info.contract, None, Some(100))?;
    heights.sort();
    assert_eq!(heights.len(), 50);
    //user have all the heights now
    let updated_bins: Vec<u32> = lb_pair::query_updated_bins_at_multiple_heights(
        &app,
        &lb_pair.info.contract,
        heights.clone(),
    )?
    .into_iter()
    .map(|x| x.bin_id)
    .collect();
    assert_eq!(updated_bins, ids);

    Ok(())
}

#[test]
#[serial]
pub fn test_query_update_after_height() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) = lb_pair_setup()?;

    let total_bins = get_total_bins(NB_BINS_X as u32, NB_BINS_Y as u32) as u32;
    let mut balances = vec![Uint256::zero(); total_bins as usize];
    let mut ids = vec![0u32; total_bins as usize];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, NB_BINS_Y);
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

    // add liquidity already made check the ids
    let heights = lb_pair::query_all_bins_updated(&app, &lb_pair.info.contract, None, None)?;
    assert_eq!(heights.len(), 1);
    assert_eq!(heights[0], app.block_info().height);
    //user have all the heights now

    let height = heights[0];

    let query_ids: Vec<u32> =
        lb_pair::query_updated_bins_at_height(&app, &lb_pair.info.contract, height)?
            .into_iter()
            .map(|x| x.bin_id)
            .collect();

    assert_eq!(query_ids, ids);

    for _ in 0..49 {
        roll_blockchain(&mut app, None);

        mint_and_add_liquidity(
            &mut app,
            &deployed_contracts,
            &addrs,
            &lb_pair,
            Some(NB_BINS_X),
            Some(NB_BINS_Y),
            DEPOSIT_AMOUNT,
            DEPOSIT_AMOUNT,
        )?;
    }

    // add liquidity already made check the ids
    let mut heights =
        lb_pair::query_all_bins_updated(&app, &lb_pair.info.contract, None, Some(100))?;
    heights.sort();
    assert_eq!(heights.len(), 50);
    //user have all the heights now
    let (updated_bins, _) = lb_pair::query_updated_bins_after_multiple_heights(
        &app,
        &lb_pair.info.contract,
        heights[0],
        Some(0),
        Some(10),
    )?;

    assert_eq!(updated_bins.len(), 19); //All the bins changed

    let (updated_bins, _) = lb_pair::query_updated_bins_after_multiple_heights(
        &app,
        &lb_pair.info.contract,
        heights[0],
        Some(0),
        Some(20),
    )?;

    assert_eq!(updated_bins.len(), 19);

    let (updated_bins, _) = lb_pair::query_updated_bins_after_multiple_heights(
        &app,
        &lb_pair.info.contract,
        heights[0],
        Some(0),
        Some(100),
    )?;

    assert_eq!(updated_bins.len(), 19);

    Ok(())
}
