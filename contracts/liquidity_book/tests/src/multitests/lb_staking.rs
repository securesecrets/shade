use std::vec;

use anyhow::Ok;
use cosmwasm_std::{StdError, Timestamp, Uint256};
use ethnum::U256;
use shade_multi_test::interfaces::{
    lb_factory,
    lb_pair,
    lb_staking,
    lb_token,
    snip20,
    utils::DeployedContracts,
};
use shade_protocol::{
    c_std::{to_binary, ContractInfo, Uint128},
    lb_libraries::{math::uint256_to_u256::ConvertU256, types::LBPairInformation},
    liquidity_book::{
        lb_staking::{InvokeMsg, QueryTxnType},
        lb_token::SendAction,
    },
    multi_test::App,
};

use super::{lb_pair_fees::ACTIVE_ID, test_helper::*};

pub const DEPOSIT_AMOUNT: u128 = 1_000_000_000_000_000_000_u128;
pub const NB_BINS_X: u32 = 5;
pub const NB_BINS_Y: u32 = 5;

pub fn lb_pair_setup(
    nb_bins_x: Option<u32>,
    nb_bins_y: Option<u32>,
) -> Result<
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
    let (mut app, lb_factory, deployed_contracts, _, _) = setup(None, None)?;

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

    let lb_token = lb_pair::query_lb_token(&app, &lb_pair.info.contract)?;

    lb_token::set_viewing_key(
        &mut app,
        addrs.batman().as_str(),
        &lb_token,
        "viewing_key".to_owned(),
    )?;

    mint_and_add_liquidity(
        &mut app,
        &deployed_contracts,
        &addrs,
        &lb_pair,
        nb_bins_x,
        nb_bins_y,
        DEPOSIT_AMOUNT,
        DEPOSIT_AMOUNT,
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
    lb_pair: &LBPairInformation,
    nb_bins_x: Option<u32>,
    nb_bins_y: Option<u32>,
    deposit_amount_x: u128, // New argument for deposit amount
    deposit_amount_y: u128, // New argument for deposit amount
) -> Result<(), anyhow::Error> {
    let amount_x = Uint128::from(deposit_amount_x);
    let amount_y = Uint128::from(deposit_amount_y);

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let tokens_to_mint = vec![(SHADE, amount_x), (SILK, amount_y)];

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
pub fn staking_contract_init() -> Result<(), anyhow::Error> {
    let (mut app, lb_factory, deployed_contracts, _lb_pair, _lb_token) = lb_pair_setup(None, None)?;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.into(),
        token_y.into(),
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    assert!(lb_token.address.as_str().len() > 0);
    assert!(lb_staking.address.as_str().len() > 0);

    Ok(())
}

#[test]
pub fn stake_simple() -> Result<(), anyhow::Error> {
    let x_bins = NB_BINS_X;
    let y_bins = NB_BINS_Y;
    // should be init with the lb-pair
    //then query it about the contract info
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.into(),
        token_y.into(),
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here
    let total_bins = get_total_bins(x_bins, y_bins) as u32;

    let mut actions = vec![];
    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        println!("id {:?}, balance: {:?}", id, balance);

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    let owner_balance = lb_token::query_all_balances(
        &mut app,
        &lb_token,
        addrs.batman(),
        String::from("viewing_key"),
    )?;

    assert_eq!(owner_balance.len(), 0);

    Ok(())
}

#[test]
pub fn fuzz_stake_simple() -> Result<(), anyhow::Error> {
    let x_bins = generate_random(0, 50);
    let y_bins = generate_random(0, 50);
    // should be init with the lb-pair
    //then query it about the contract info
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.into(),
        token_y.into(),
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here
    let total_bins = get_total_bins(x_bins, y_bins) as u32;

    let mut actions = vec![];
    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    let owner_balance = lb_token::query_all_balances(
        &mut app,
        &lb_token,
        addrs.batman(),
        String::from("viewing_key"),
    )?;

    assert_eq!(owner_balance.len(), 0);

    Ok(())
}

#[test]
pub fn stake_liquidity_with_time() -> Result<(), anyhow::Error> {
    let x_bins = 5;
    let y_bins = 5;

    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.into(),
        token_y.into(),
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here
    let total_bins = get_total_bins(x_bins, y_bins) as u32;
    let mut ids = vec![];
    let mut liqs = vec![];

    let mut actions = vec![];
    let mut balances = vec![];

    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);
        ids.push(id);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
        balances.push(balance);

        liqs.push(balance / Uint256::from_u128(2));

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance / Uint256::from_u128(2),
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 50);

    app.set_time(timestamp);

    let query_auth = generate_auth(addrs.batman().to_string());

    //Check the liquidity after half the time of duration - duration is 100
    let liquidity = lb_staking::query_liquidity(&app, query_auth, &lb_staking, ids.clone(), None)?;

    for (liq, bal) in liquidity.into_iter().zip(liqs.clone()).into_iter() {
        assert_eq!(liq.user_liquidity, bal);
    }

    // add liquduty after 50s or half duration:
    let mut actions = vec![];

    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        liqs[i as usize] = liqs[i as usize]
            + balance.multiply_ratio(Uint256::from(50u128), Uint256::from(100u128));

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;
    let query_auth = generate_auth(addrs.batman().to_string());

    //Check the liquidity after half the time of duration - duration is 100
    let liquidity = lb_staking::query_liquidity(&app, query_auth, &lb_staking, ids.clone(), None)?;

    for (liq, bal) in liquidity.into_iter().zip(liqs.clone()).into_iter() {
        assert_eq!(liq.user_liquidity, bal);
    }
    let mut liqs = vec![];

    //trying to add liquidity after the end_time:
    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 51);
    app.set_time(timestamp);

    mint_and_add_liquidity(
        &mut app,
        &deployed_contracts,
        &addrs,
        &lb_pair,
        Some(x_bins),
        Some(y_bins),
        DEPOSIT_AMOUNT,
        DEPOSIT_AMOUNT,
    )?;

    let mut actions = vec![];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        balances[i as usize] += balance;
        liqs.push(balance.multiply_ratio(Uint256::from(99u128), Uint256::from(100u128)));

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;
    let query_auth = generate_auth(addrs.batman().to_string());

    //Check the liquidity after half the time of duration - duration is 100
    let liquidity = lb_staking::query_liquidity(&app, query_auth, &lb_staking, ids.clone(), None)?;

    for (liq, bal) in liquidity
        .clone()
        .into_iter()
        .zip(balances.clone())
        .into_iter()
    {
        let half_balance = bal.multiply_ratio(Uint256::from(1u128), Uint256::from(2u128));

        assert_eq!(
            liq.user_liquidity,
            half_balance
                + half_balance.multiply_ratio(Uint256::from(99u128), Uint256::from(100u128)),
        );
    }

    Ok(())
}

#[test]
pub fn fuzz_stake_liquidity_with_time() -> Result<(), anyhow::Error> {
    let x_bins = generate_random(0, 50);
    let y_bins = generate_random(0, 50);

    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.into(),
        token_y.into(),
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here
    let total_bins = get_total_bins(x_bins, y_bins) as u32;
    let mut ids = vec![];
    let mut liqs = vec![];

    let mut actions = vec![];
    let mut balances = vec![];

    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);
        ids.push(id);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
        balances.push(balance);

        liqs.push(balance / Uint256::from_u128(2));

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance / Uint256::from_u128(2),
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 50);

    app.set_time(timestamp);

    let query_auth = generate_auth(addrs.batman().to_string());

    //Check the liquidity after half the time of duration - duration is 100
    let liquidity = lb_staking::query_liquidity(&app, query_auth, &lb_staking, ids.clone(), None)?;

    for (liq, bal) in liquidity.into_iter().zip(liqs.clone()).into_iter() {
        assert_eq!(liq.user_liquidity, bal);
    }

    // add liquduty after 50s or half duration:
    let mut actions = vec![];

    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        liqs[i as usize] = liqs[i as usize]
            + balance.multiply_ratio(Uint256::from(50u128), Uint256::from(100u128));

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;
    let query_auth = generate_auth(addrs.batman().to_string());

    //Check the liquidity after half the time of duration - duration is 100
    let liquidity = lb_staking::query_liquidity(&app, query_auth, &lb_staking, ids.clone(), None)?;

    for (liq, bal) in liquidity.into_iter().zip(liqs.clone()).into_iter() {
        assert_eq!(liq.user_liquidity, bal);
    }
    let mut liqs = vec![];

    //trying to add liquidity after the end_time:
    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 51);
    app.set_time(timestamp);

    mint_and_add_liquidity(
        &mut app,
        &deployed_contracts,
        &addrs,
        &lb_pair,
        Some(x_bins),
        Some(y_bins),
        DEPOSIT_AMOUNT,
        DEPOSIT_AMOUNT,
    )?;

    let mut actions = vec![];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        balances[i as usize] += balance;
        liqs.push(balance.multiply_ratio(Uint256::from(99u128), Uint256::from(100u128)));

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;
    let query_auth = generate_auth(addrs.batman().to_string());

    //Check the liquidity after half the time of duration - duration is 100
    let liquidity = lb_staking::query_liquidity(&app, query_auth, &lb_staking, ids.clone(), None)?;

    for (liq, bal) in liquidity
        .clone()
        .into_iter()
        .zip(balances.clone())
        .into_iter()
    {
        let half_balance = bal.multiply_ratio(Uint256::from(1u128), Uint256::from(2u128));
        assert_eq!(
            liq.user_liquidity,
            half_balance
                + half_balance.multiply_ratio(Uint256::from(99u128), Uint256::from(100u128)),
        );
    }

    Ok(())
}

#[test]
pub fn unstake() -> Result<(), anyhow::Error> {
    let x_bins = NB_BINS_X;
    let y_bins = NB_BINS_Y;
    // should be init with the lb-pair
    //then query it about the contract info
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.into(),
        token_y.into(),
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here

    let total_bins = get_total_bins(x_bins, y_bins) as u32;

    let mut actions = vec![];
    let mut balances: Vec<Uint256> = Vec::new();
    let mut ids: Vec<u32> = Vec::new();
    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);
        ids.push(id);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        balances.push(balance);

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    let owner_balance = lb_token::query_all_balances(
        &mut app,
        &lb_token,
        addrs.batman(),
        String::from("viewing_key"),
    )?;

    assert_eq!(owner_balance.len(), 0);

    // unstaking
    lb_staking::unstaking(
        &mut app,
        addrs.batman().as_str(),
        &lb_staking,
        ids.clone(),
        balances.clone(),
    )?;

    for i in 0..total_bins as usize {
        let id = get_id(ACTIVE_ID, i as u32, y_bins);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            ids[i].to_string(),
        )?;

        assert_eq!(balance, balances[i]);
    }

    Ok(())
}

#[test]
pub fn fuzz_unstake() -> Result<(), anyhow::Error> {
    let x_bins = generate_random(0, 50);
    let y_bins = generate_random(0, 50);
    // should be init with the lb-pair
    //then query it about the contract info
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.into(),
        token_y.into(),
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here

    let total_bins = get_total_bins(x_bins, y_bins) as u32;

    let mut actions = vec![];
    let mut balances: Vec<Uint256> = Vec::new();
    let mut ids: Vec<u32> = Vec::new();
    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);
        ids.push(id);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
        balances.push(balance);

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    let owner_balance = lb_token::query_all_balances(
        &mut app,
        &lb_token,
        addrs.batman(),
        String::from("viewing_key"),
    )?;

    assert_eq!(owner_balance.len(), 0);

    // unstaking

    lb_staking::unstaking(
        &mut app,
        addrs.batman().as_str(),
        &lb_staking,
        ids.clone(),
        balances.clone(),
    )?;

    for i in 0..total_bins as usize {
        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            ids[i].to_string(),
        )?;

        assert_eq!(balance, balances[i]);
    }

    Ok(())
}

#[test]
pub fn fuzz_unstake_liquidity_with_time() -> Result<(), anyhow::Error> {
    let x_bins = generate_random(0, 50);
    let y_bins = generate_random(0, 50);
    // should be init with the lb-pair
    //then query it about the contract info
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.into(),
        token_y.into(),
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here

    let total_bins = get_total_bins(x_bins, y_bins) as u32;

    let mut actions = vec![];
    let mut balances: Vec<Uint256> = Vec::new();
    let mut ids: Vec<u32> = Vec::new();
    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);
        ids.push(id);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
        balances.push(balance);

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    //removing liquidity after half duration
    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 50);
    app.set_time(timestamp);

    let owner_balance = lb_token::query_all_balances(
        &mut app,
        &lb_token,
        addrs.batman(),
        String::from("viewing_key"),
    )?;

    assert_eq!(owner_balance.len(), 0);

    // unstaking
    lb_staking::unstaking(
        &mut app,
        addrs.batman().as_str(),
        &lb_staking,
        ids.clone(),
        balances.clone(),
    )?;

    let query_auth = generate_auth(addrs.batman().to_string());

    //Check the liquidity after half the time of duration - duration is 100
    let liquidity = lb_staking::query_liquidity(&app, query_auth, &lb_staking, ids.clone(), None)?;

    for (liq, bal) in liquidity.into_iter().zip(balances.clone()).into_iter() {
        assert_approx_eq_abs(
            liq.user_liquidity,
            bal.multiply_ratio(50u128, 100u128),
            Uint256::from(1u128),
            "ERRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
        );
    }

    Ok(())
}

#[test]
pub fn register_rewards_token() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) =
        lb_pair_setup(Some(NB_BINS_X), Some(NB_BINS_Y))?;
    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //Add the token
    let shade_token = extract_contract_info(&deployed_contracts, SHADE)?;
    let silk_token = extract_contract_info(&deployed_contracts, SILK)?;

    let reward_tokens = vec![shade_token, silk_token];

    lb_staking::register_reward_tokens(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        reward_tokens.clone(),
    )?;

    //query to check the tokens in there
    let q_reward_tokens = lb_staking::query_registered_tokens(&app, &lb_staking)?;

    assert_eq!(q_reward_tokens, reward_tokens);

    //getting an error for trying to add a token again
    let res = lb_staking::register_reward_tokens(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        reward_tokens,
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err("Generic error: Reward token already exists")
    );

    Ok(())
}

#[test]
pub fn add_rewards() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) =
        lb_pair_setup(Some(NB_BINS_X), Some(NB_BINS_Y))?;
    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //Add the token
    let shade_token = extract_contract_info(&deployed_contracts, SHADE)?;
    let silk_token = extract_contract_info(&deployed_contracts, SILK)?;

    let reward_tokens = vec![shade_token, silk_token];

    lb_staking::register_reward_tokens(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        reward_tokens.clone(),
    )?;

    //mint tokens
    snip20::mint_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        &vec![],
        addrs.admin().to_string(),
        DEPOSIT_AMOUNT.into(),
    )?;

    snip20::send_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        lb_staking.address.to_string(),
        DEPOSIT_AMOUNT.into(),
        Some(to_binary(&InvokeMsg::AddRewards {
            start: None,
            end: 20,
        })?),
    )?;

    Ok(())
}

#[test]
pub fn end_epoch() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) =
        lb_pair_setup(Some(NB_BINS_X), Some(NB_BINS_Y))?;
    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //Add the token
    let shade_token = extract_contract_info(&deployed_contracts, SHADE)?;
    let silk_token = extract_contract_info(&deployed_contracts, SILK)?;

    let reward_tokens = vec![shade_token, silk_token];

    lb_staking::register_reward_tokens(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        reward_tokens.clone(),
    )?;

    //mint tokens
    snip20::mint_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        &vec![],
        addrs.admin().to_string(),
        DEPOSIT_AMOUNT.into(),
    )?;

    snip20::send_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        lb_staking.address.to_string(),
        DEPOSIT_AMOUNT.into(),
        Some(to_binary(&InvokeMsg::AddRewards {
            start: None,
            end: 20,
        })?),
    )?;

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?;

    Ok(())
}

#[test]
pub fn fuzz_claim_rewards() -> Result<(), anyhow::Error> {
    let x_bins = generate_random(0, 50);
    let y_bins = generate_random(0, 50);

    // should be init with the lb-pair
    //then query it about the contract info
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.into(),
        token_y.into(),
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here
    let total_bins = get_total_bins(x_bins, y_bins) as u32;

    let mut actions = vec![];
    let mut ids = vec![];
    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);
        ids.push(id);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    let shade_token = extract_contract_info(&deployed_contracts, SHADE)?;
    let silk_token = extract_contract_info(&deployed_contracts, SILK)?;

    let reward_tokens = vec![shade_token, silk_token];

    lb_staking::register_reward_tokens(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        reward_tokens.clone(),
    )?;

    //mint tokens
    snip20::mint_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        &vec![],
        addrs.admin().to_string(),
        DEPOSIT_AMOUNT.into(),
    )?;

    snip20::send_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        lb_staking.address.to_string(),
        DEPOSIT_AMOUNT.into(),
        Some(to_binary(&InvokeMsg::AddRewards {
            start: None,
            end: 11,
        })?),
    )?;

    roll_time(&mut app, Some(100));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?;

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    Ok(())
}

#[test]
pub fn claim_rewards() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let x_bins = 20;
    let y_bins = 20;
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;
    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here
    let total_bins = get_total_bins(x_bins, y_bins) as u32;

    let mut actions = vec![];
    let mut ids = vec![];
    let mut balances: Vec<Uint256> = Vec::new();
    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);
        ids.push(id);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        balances.push(balance);

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    //Added the reward tokens for next 10 rounds
    let shade_token = extract_contract_info(&deployed_contracts, SHADE)?;
    let silk_token = extract_contract_info(&deployed_contracts, SILK)?;

    let reward_tokens = vec![shade_token, silk_token];

    lb_staking::register_reward_tokens(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        reward_tokens.clone(),
    )?;

    //mint tokens
    snip20::mint_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        &vec![],
        addrs.admin().to_string(),
        DEPOSIT_AMOUNT.into(),
    )?;

    snip20::send_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        lb_staking.address.to_string(),
        DEPOSIT_AMOUNT.into(),
        Some(to_binary(&InvokeMsg::AddRewards {
            start: None,
            end: 10,
        })?),
    )?;
    roll_time(&mut app, Some(100));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //1
    roll_time(&mut app, Some(100));
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //2
    roll_time(&mut app, Some(100));

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //3
    roll_time(&mut app, Some(100));

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //4
    roll_time(&mut app, Some(100));

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //5
    roll_time(&mut app, Some(100));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //6
    roll_time(&mut app, Some(100));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //7
    roll_time(&mut app, Some(100));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //8
    roll_time(&mut app, Some(100));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //9
    roll_time(&mut app, Some(100));

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //10
    roll_time(&mut app, Some(100));

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //11
    roll_time(&mut app, Some(100));

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //12 -> 13

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    snip20::set_viewing_key_exec(
        &mut app,
        lb_staking.address.as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance = snip20::balance_query(
        &app,
        lb_staking.address.as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance.u128(), 0);

    //ERROR when staker try to claim rewards again:

    let error = lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking);

    assert_eq!(
        error.unwrap_err(),
        StdError::generic_err(
            "Generic error: You have already claimed rewards for the latest epoch.",
        )
    );

    //staked all:
    let timestamp = Timestamp::from_seconds(app.block_info().time.seconds() + 50);

    app.set_time(timestamp);

    lb_staking::unstaking(
        &mut app,
        addrs.batman().as_str(),
        &lb_staking,
        ids.clone(),
        balances.clone(),
    )?;

    //mint tokens and adding more rewards
    snip20::mint_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        &vec![],
        addrs.admin().to_string(),
        DEPOSIT_AMOUNT.into(),
    )?;

    snip20::send_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        lb_staking.address.to_string(),
        DEPOSIT_AMOUNT.into(),
        Some(to_binary(&InvokeMsg::AddRewards {
            start: None,
            end: 13,
        })?),
    )?;

    //unstake all:
    roll_time(&mut app, Some(100));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //13->14

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    let balance = snip20::balance_query(
        &app,
        &lb_staking.address.as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance.u128(), 0);

    let balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance.u128(), DEPOSIT_AMOUNT + DEPOSIT_AMOUNT);

    Ok(())
}
#[test]
pub fn end_epoch_by_stakers() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let x_bins = 10;
    let y_bins = 10;
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;
    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here
    let total_bins = get_total_bins(x_bins, y_bins) as u32;

    let mut actions = vec![];
    //1) Stake the Snip-1155 tokens
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    let lb_staking_config = lb_staking::query_config(&app, &lb_staking)?;
    assert_eq!(lb_staking_config.epoch_index, 1);

    let epoch_info = lb_staking::query_epoch_info(&app, &lb_staking, None)?;
    assert_eq!(epoch_info.start_time, app.block_info().time.seconds());
    assert_eq!(epoch_info.end_time, app.block_info().time.seconds() + 100);

    //Rolling Time + duration, t=0 + duration
    let timestamp = Timestamp::from_seconds(
        app.block_info().time.seconds() + lb_staking_config.epoch_durations,
    );
    app.set_time(timestamp);

    //2) End Epoch
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //1

    let lb_staking_config = lb_staking::query_config(&app, &lb_staking)?;
    assert_eq!(lb_staking_config.epoch_index, 2);

    let epoch_info = lb_staking::query_epoch_info(&app, &lb_staking, None)?;
    assert_eq!(epoch_info.start_time, app.block_info().time.seconds());
    assert_eq!(epoch_info.end_time, app.block_info().time.seconds() + 100);

    //3 Claim rewards
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    let query_auth = generate_auth(addrs.batman().to_string());

    let staker_info = lb_staking::query_staker_info(&app, &lb_staking, query_auth.clone())?;

    assert_eq!(staker_info.last_claim_rewards_round, Some(1));

    //Rolling Time + duration, t= duration*2
    let timestamp = Timestamp::from_seconds(
        app.block_info().time.seconds() + lb_staking_config.epoch_durations,
    );
    app.set_time(timestamp);

    //4)Stake after one round at t = 2*duration
    mint_and_add_liquidity(
        &mut app,
        &deployed_contracts,
        &addrs,
        &lb_pair,
        Some(x_bins),
        Some(y_bins),
        DEPOSIT_AMOUNT,
        DEPOSIT_AMOUNT,
    )?;

    let mut actions = vec![];
    // Stake the Snip-1155 tokens
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    //Check epoch id
    let lb_staking_config = lb_staking::query_config(&app, &lb_staking)?;
    assert_eq!(lb_staking_config.epoch_index, 3);

    let epoch_info = lb_staking::query_epoch_info(&app, &lb_staking, None)?;
    assert_eq!(epoch_info.start_time, app.block_info().time.seconds());
    assert_eq!(epoch_info.end_time, app.block_info().time.seconds() + 100);

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    let staker_info = lb_staking::query_staker_info(&app, &lb_staking, query_auth.clone())?;
    assert_eq!(staker_info.last_claim_rewards_round, Some(1)); // No rewards distribution yet

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    let staker_info = lb_staking::query_staker_info(&app, &lb_staking, query_auth.clone())?;
    assert_eq!(staker_info.last_claim_rewards_round, Some(1)); // No rewards distribution yet

    let timestamp = Timestamp::from_seconds(
        app.block_info().time.seconds() + lb_staking_config.epoch_durations,
    );
    app.set_time(timestamp);
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //2
    let lb_staking_config = lb_staking::query_config(&app, &lb_staking)?;
    assert_eq!(lb_staking_config.epoch_index, 3);

    let timestamp = Timestamp::from_seconds(
        app.block_info().time.seconds() + lb_staking_config.epoch_durations,
    );
    app.set_time(timestamp);
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //3

    let lb_staking_config = lb_staking::query_config(&app, &lb_staking)?;
    assert_eq!(lb_staking_config.epoch_index, 4);

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    let staker_info = lb_staking::query_staker_info(&app, &lb_staking, query_auth.clone())?;
    assert_eq!(staker_info.last_claim_rewards_round, Some(3)); // No rewards distribution yet

    Ok(())
}

#[test]
pub fn claim_expired_rewards() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let x_bins = 20;
    let y_bins = 20;
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;
    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here
    let total_bins = get_total_bins(x_bins, y_bins) as u32;

    let mut actions = vec![];
    let mut ids = vec![];
    let mut balances: Vec<Uint256> = Vec::new();
    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);
        ids.push(id);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        balances.push(balance);

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    lb_staking::update_config(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        None,
        None,
        Some(200),
        Some(5),
    )?;

    //Added the reward tokens for next 10 rounds
    let shade_token = extract_contract_info(&deployed_contracts, SHADE)?;
    let silk_token = extract_contract_info(&deployed_contracts, SILK)?;

    let reward_tokens = vec![shade_token, silk_token];

    lb_staking::register_reward_tokens(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        reward_tokens.clone(),
    )?;

    //mint tokens
    snip20::mint_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        &vec![],
        addrs.admin().to_string(),
        DEPOSIT_AMOUNT.into(),
    )?;

    snip20::send_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        lb_staking.address.to_string(),
        DEPOSIT_AMOUNT.into(),
        Some(to_binary(&InvokeMsg::AddRewards {
            start: None,
            end: 10,
        })?),
    )?;
    // println!("CHECKING");

    roll_time(&mut app, Some(200));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //1
    roll_time(&mut app, Some(200));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //2
    roll_time(&mut app, Some(200));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //3
    roll_time(&mut app, Some(200));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //4
    roll_time(&mut app, Some(200));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //5
    roll_time(&mut app, Some(200));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //6
    roll_time(&mut app, Some(200));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //7
    roll_time(&mut app, Some(200));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //8
    roll_time(&mut app, Some(200));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //9
    roll_time(&mut app, Some(200));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //10

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    snip20::set_viewing_key_exec(
        &mut app,
        lb_staking.address.as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance = snip20::balance_query(
        &app,
        lb_staking.address.as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert!(balance.u128() > 0);
    let query_auth = generate_auth(addrs.batman().to_string());

    let (claim_rewards_txns, count) = lb_staking::query_txn_history(
        &app,
        &lb_staking,
        query_auth,
        None,
        None,
        QueryTxnType::ClaimRewards,
    )?;

    assert_eq!(claim_rewards_txns.len(), 1);
    assert_eq!(count, 1);

    Ok(())
}

#[test]
pub fn recover_expired_rewards() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let x_bins = 5;
    let y_bins = 5;
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;
    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here
    let total_bins = get_total_bins(x_bins, y_bins) as u32;

    let mut actions = vec![];
    let mut ids = vec![];
    let mut balances: Vec<Uint256> = Vec::new();
    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);
        ids.push(id);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        balances.push(balance);

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    lb_staking::update_config(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        None,
        None,
        Some(200),
        Some(5),
    )?;

    //Added the reward tokens for next 10 rounds
    let shade_token = extract_contract_info(&deployed_contracts, SHADE)?;
    let silk_token = extract_contract_info(&deployed_contracts, SILK)?;

    let reward_tokens = vec![shade_token, silk_token];

    lb_staking::register_reward_tokens(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        reward_tokens.clone(),
    )?;

    //mint tokens
    snip20::mint_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        &vec![],
        addrs.admin().to_string(),
        DEPOSIT_AMOUNT.into(),
    )?;

    snip20::send_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        lb_staking.address.to_string(),
        DEPOSIT_AMOUNT.into(),
        Some(to_binary(&InvokeMsg::AddRewards {
            start: None,
            end: 10,
        })?),
    )?;

    snip20::set_viewing_key_exec(
        &mut app,
        lb_staking.address.as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let balance = snip20::balance_query(
        &app,
        lb_staking.address.as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(balance.u128(), DEPOSIT_AMOUNT);

    let balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(balance.u128(), 0u128);

    roll_time(&mut app, Some(200));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //1 expired at 6
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    roll_time(&mut app, Some(200));
    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //2 expired at 7
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    roll_time(&mut app, Some(200));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //3 expired at 8
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    roll_time(&mut app, Some(200));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //4 expired at 9
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    roll_time(&mut app, Some(200));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //5 expired at 10
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    roll_time(&mut app, Some(200));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //6 expired at 11
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    roll_time(&mut app, Some(200));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //7 expired at 12
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    roll_time(&mut app, Some(200));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //8 expires at 13
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    roll_time(&mut app, Some(200));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //9 expires at 14
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    roll_time(&mut app, Some(200));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //10 expires at 15
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    roll_time(&mut app, Some(200));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //11 expires at 16 
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    roll_time(&mut app, Some(200));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //12 expires at 17
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    roll_time(&mut app, Some(200));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //13 expires at 18 
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;
    roll_time(&mut app, Some(200));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?; //14 expires at 19 
    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    //current round index -> 20

    lb_staking::recover_expired_rewards(&mut app, addrs.admin().as_str(), &lb_staking)?;

    let balance = snip20::balance_query(
        &app,
        lb_staking.address.as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance.u128(), 0u128);

    Ok(())
}

#[test]
pub fn recover_funds() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let x_bins = 5;
    let y_bins = 5;
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) =
        lb_pair_setup(Some(x_bins), Some(y_bins))?;

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.info.contract)?;
    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    //deposit funds here
    let total_bins = get_total_bins(x_bins, y_bins) as u32;

    let mut actions = vec![];
    let mut ids = vec![];
    let mut balances: Vec<Uint256> = Vec::new();
    //Querying all the bins
    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, y_bins);
        ids.push(id);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        balances.push(balance);

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    lb_staking::update_config(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        None,
        None,
        Some(200),
        Some(5),
    )?;

    //Added the reward tokens for next 10 rounds
    let shade_token = extract_contract_info(&deployed_contracts, SHADE)?;
    let shade_token_type = token_type_snip20_generator(&shade_token)?;
    let silk_token = extract_contract_info(&deployed_contracts, SILK)?;

    let reward_tokens = vec![shade_token, silk_token];

    lb_staking::register_reward_tokens(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        reward_tokens.clone(),
    )?;

    //mint tokens
    snip20::mint_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        &vec![],
        addrs.admin().to_string(),
        Uint128::from(200u128),
    )?;

    snip20::send_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        lb_staking.address.to_string(),
        Uint128::from(100u128),
        Some(to_binary(&InvokeMsg::AddRewards {
            start: None,
            end: 10,
        })?),
    )?;
    snip20::transfer_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        lb_staking.address.to_string(),
        Uint128::from(100u128),
    )?;

    let balance = snip20::balance_query(
        &app,
        lb_staking.address.as_str(),
        &deployed_contracts,
        SHADE,
        "SHADE_STAKING_VIEWING_KEY".to_string(),
    )?;

    snip20::set_viewing_key_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let prev_admin_balance = snip20::balance_query(
        &app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance.u128(), 200u128);

    lb_staking::recover_funds(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        shade_token_type.clone(),
        Uint128::from(100u128),
        addrs.admin().to_string(),
    )?;

    let contract_balance = snip20::balance_query(
        &app,
        lb_staking.address.as_str(),
        &deployed_contracts,
        SHADE,
        "SHADE_STAKING_VIEWING_KEY".to_string(),
    )?;

    assert_eq!(contract_balance.u128(), 100u128);

    let admin_balance = snip20::balance_query(
        &app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(admin_balance.u128(), prev_admin_balance.u128() + 100);

    let err = lb_staking::recover_funds(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        shade_token_type,
        Uint128::from(100u128),
        addrs.admin().to_string(),
    );

    assert_eq!(
        err.unwrap_err(),
        StdError::generic_err(
            "Generic error: Trying to recover already staked funds. Extra funds Uint128(0), amount Uint128(100)",
        )
    );
    Ok(())
}

#[test]
pub fn update_config() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, _deployed_contracts, lb_pair, _lb_token) =
        lb_pair_setup(Some(NB_BINS_X), Some(NB_BINS_Y))?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    let config = lb_staking::query_config(&app, &lb_staking)?;
    assert_eq!(config.epoch_durations, (100));

    lb_staking::update_config(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        None,
        None,
        Some(200),
        None,
    )?;
    let config = lb_staking::query_config(&app, &lb_staking)?;
    assert_eq!(config.epoch_durations, (200));

    Ok(())
}

#[test]
fn query_contract_info() -> Result<(), anyhow::Error> {
    let (mut app, _lb_factory, _deployed_contracts, lb_pair, lb_token) =
        lb_pair_setup(Some(NB_BINS_X), Some(NB_BINS_Y))?;
    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;

    let config = lb_staking::query_config(&app, &lb_staking)?;

    assert_eq!(config.lb_pair, lb_pair.info.contract.address);
    assert_eq!(config.lb_token.address, lb_token.address);

    Ok(())
}

#[test]
fn query_id_balance() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, _deployed_contracts, lb_pair, lb_token) =
        lb_pair_setup(Some(NB_BINS_X), Some(NB_BINS_Y))?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;
    let total_bins = get_total_bins(NB_BINS_X, NB_BINS_Y) as u32;

    //stake:
    let mut actions = vec![];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, NB_BINS_Y);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, NB_BINS_Y);
        let (reserves_x, reserves_y, _) =
            lb_pair::query_bin_reserves(&app, &lb_pair.info.contract, id)?;
        let price = lb_pair::query_price_from_id(&app, &lb_pair.info.contract, id)?;

        let expected_balance_x = U256::from(reserves_x);
        let expected_balance_y = U256::from(reserves_y);

        let total: U256 = expected_balance_x * U256::from_str_prefixed(price.to_string().as_str())?
            + (expected_balance_y << 128);

        let balance = lb_staking::query_id_total_balance(&app, &lb_staking, id)?;

        assert_eq!(total.u256_to_uint256(), balance);
        assert!(balance > Uint256::MIN);
    }

    Ok(())
}

#[test]
fn query_balance() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, _deployed_contracts, lb_pair, lb_token) =
        lb_pair_setup(Some(NB_BINS_X), Some(NB_BINS_Y))?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;
    let total_bins = get_total_bins(NB_BINS_X, NB_BINS_Y) as u32;

    //stake:
    let mut actions = vec![];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, NB_BINS_Y);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, NB_BINS_Y);
        let (reserves_x, reserves_y, _) =
            lb_pair::query_bin_reserves(&app, &lb_pair.info.contract, id)?;
        let price = lb_pair::query_price_from_id(&app, &lb_pair.info.contract, id)?;

        let expected_balance_x = U256::from(reserves_x);
        let expected_balance_y = U256::from(reserves_y);

        let total: U256 = expected_balance_x * U256::from_str_prefixed(price.to_string().as_str())?
            + (expected_balance_y << 128);

        let query_auth = generate_auth(addrs.batman().to_string());

        let balance = lb_staking::query_balance(&app, &lb_staking, query_auth, id)?;

        assert_eq!(total.u256_to_uint256(), balance);
        assert!(balance > Uint256::MIN);
    }

    Ok(())
}

#[test]
fn query_all_balance() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, _deployed_contracts, lb_pair, lb_token) =
        lb_pair_setup(Some(NB_BINS_X), Some(NB_BINS_Y))?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;
    let total_bins = get_total_bins(NB_BINS_X, NB_BINS_Y) as u32;

    //stake:
    let mut actions = vec![];

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, NB_BINS_Y);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    let query_auth = generate_auth(addrs.batman().to_string());

    let balances = lb_staking::query_all_balances(&app, &lb_staking, query_auth, None, None)?;

    for owner_balance in balances {
        let id = owner_balance.token_id.parse().unwrap();
        let (reserves_x, reserves_y, _) =
            lb_pair::query_bin_reserves(&app, &lb_pair.info.contract, id)?;
        let price = lb_pair::query_price_from_id(&app, &lb_pair.info.contract, id)?;

        let expected_balance_x = U256::from(reserves_x);
        let expected_balance_y = U256::from(reserves_y);

        let total: U256 = expected_balance_x * U256::from_str_prefixed(price.to_string().as_str())?
            + (expected_balance_y << 128);
        assert_eq!(total.u256_to_uint256(), owner_balance.amount);
        assert!(owner_balance.amount > Uint256::MIN);
    }

    Ok(())
}

#[test]
fn query_txn_history() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, lb_token) =
        lb_pair_setup(Some(NB_BINS_X), Some(NB_BINS_Y))?;

    let lb_staking = lb_pair::query_lb_staking(&mut app, &lb_pair.info.contract)?;
    let total_bins = get_total_bins(NB_BINS_X, NB_BINS_Y) as u32;

    //stake:
    let mut actions = vec![];
    let mut balances: Vec<Uint256> = Vec::new();
    let mut ids: Vec<u32> = Vec::new();

    for i in 0..total_bins {
        let id = get_id(ACTIVE_ID, i, NB_BINS_Y);
        ids.push(id);

        let balance = lb_token::query_balance(
            &app,
            &lb_token,
            addrs.batman(),
            addrs.batman(),
            String::from("viewing_key"),
            id.to_string(),
        )?;
        balances.push(balance);

        actions.push(SendAction {
            token_id: id.to_string(),
            from: addrs.batman(),
            recipient: lb_staking.address.clone(),
            recipient_code_hash: Some(lb_staking.code_hash.clone()),
            amount: balance,
            msg: Some(to_binary(&InvokeMsg::Stake {
                from: Some(addrs.batman().to_string()),
                padding: None,
            })?),
            memo: None,
        })
    }

    lb_token::batch_send(&mut app, addrs.batman().as_str(), &lb_token, actions)?;

    // query all txn history and staking  txn history

    let query_auth = generate_auth(addrs.batman().to_string());

    let (all_txns_1, count) = lb_staking::query_txn_history(
        &app,
        &lb_staking,
        query_auth.clone(),
        None,
        None,
        QueryTxnType::All,
    )?;

    assert_eq!(all_txns_1.len(), total_bins as usize);
    assert_eq!(count, total_bins as u64);

    let (staking_txns, count) = lb_staking::query_txn_history(
        &app,
        &lb_staking,
        query_auth.clone(),
        None,
        None,
        QueryTxnType::Stake,
    )?;

    assert_eq!(staking_txns.len(), total_bins as usize);
    assert_eq!(count, total_bins as u64);

    //Adding the rewards
    let shade_token = extract_contract_info(&deployed_contracts, SHADE)?;
    let silk_token = extract_contract_info(&deployed_contracts, SILK)?;

    let reward_tokens = vec![shade_token, silk_token];

    lb_staking::register_reward_tokens(
        &mut app,
        addrs.admin().as_str(),
        &lb_staking,
        reward_tokens.clone(),
    )?;

    //mint tokens
    snip20::mint_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        &vec![],
        addrs.admin().to_string(),
        DEPOSIT_AMOUNT.into(),
    )?;

    snip20::send_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        SHADE,
        lb_staking.address.to_string(),
        DEPOSIT_AMOUNT.into(),
        Some(to_binary(&InvokeMsg::AddRewards {
            start: None,
            end: 20,
        })?),
    )?;
    roll_time(&mut app, Some(100));

    lb_pair::calculate_rewards(&mut app, addrs.admin().as_str(), &lb_pair.info.contract)?;

    lb_staking::claim_rewards(&mut app, addrs.batman().as_str(), &lb_staking)?;

    //query all txns and claim rewards txns

    let (all_txns_2, count) = lb_staking::query_txn_history(
        &app,
        &lb_staking,
        query_auth.clone(),
        None,
        None,
        QueryTxnType::All,
    )?;

    assert_eq!(all_txns_2.len(), all_txns_1.len() + 1);
    assert_eq!(count, (all_txns_1.len() + 1) as u64);

    let (claim_rewards_txns, count) = lb_staking::query_txn_history(
        &app,
        &lb_staking,
        query_auth.clone(),
        None,
        None,
        QueryTxnType::ClaimRewards,
    )?;

    assert_eq!(claim_rewards_txns.len(), 1);
    assert_eq!(count, 1);

    lb_staking::unstaking(
        &mut app,
        addrs.batman().as_str(),
        &lb_staking,
        ids.clone(),
        balances.clone(),
    )?;

    //query all txns and unstaking txns
    let (all_txns_3, count) = lb_staking::query_txn_history(
        &app,
        &lb_staking,
        query_auth.clone(),
        None,
        None,
        QueryTxnType::All,
    )?;

    assert_eq!(all_txns_3.len(), all_txns_2.len() + 1);
    assert_eq!(count, (all_txns_2.len() + 1) as u64);

    let (unstaking_txns, count) = lb_staking::query_txn_history(
        &app,
        &lb_staking,
        query_auth.clone(),
        None,
        None,
        QueryTxnType::UnStake,
    )?;

    assert_eq!(unstaking_txns.len(), 1);
    assert_eq!(count, 1);

    //checking pagination

    //query all txns and unstaking txns
    let (all_txns, count) = lb_staking::query_txn_history(
        &app,
        &lb_staking,
        query_auth.clone(),
        None,
        Some(10),
        QueryTxnType::All,
    )?;

    assert_eq!(all_txns.len(), 10);
    assert_eq!(count, 11); // total txns

    Ok(())
}
