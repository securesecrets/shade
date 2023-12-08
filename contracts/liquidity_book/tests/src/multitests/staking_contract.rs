use cosmwasm_std::{to_binary, ContractInfo, Uint128, Uint256};
use ethnum::U256;
use shade_multi_test::interfaces::{lb_factory, lb_pair, lb_token, utils::DeployedContracts};
use shade_protocol::{
    lb_libraries::{math::uint256_to_u256::ConvertU256, types::LBPairInformation},
    liquidity_book::{
        lb_token::SendAction,
        staking::{ExecuteMsg, InvokeMsg},
    },
    multi_test::App,
};

use super::{lb_pair_fees::ACTIVE_ID, test_helper::*};

pub const DEPOSIT_AMOUNT: u128 = 1_000_000_000_000_000_000_u128;
pub const NB_BINS_X: u8 = 50;
pub const NB_BINS_Y: u8 = 50;

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

    let lb_token = lb_pair::query_lb_token(&app, &lb_pair.lb_pair.contract)?;

    lb_token::set_viewing_key(
        &mut app,
        addrs.batman().as_str(),
        &lb_token,
        "viewing_key".to_owned(),
    )?;

    let amount_x = Uint128::from(DEPOSIT_AMOUNT);
    let amount_y = Uint128::from(DEPOSIT_AMOUNT);

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
        NB_BINS_X,
        NB_BINS_Y,
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
pub fn staking_contract_init() -> Result<(), anyhow::Error> {
    let (mut app, lb_factory, deployed_contracts, _lb_pair, _lb_token) = lb_pair_setup()?;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.into(),
        token_y.into(),
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.lb_pair.contract)?;

    let staking_contract = lb_pair::query_staking_contract(&mut app, &lb_pair.lb_pair.contract)?;

    assert!(lb_token.address.as_str().len() > 0);
    assert!(staking_contract.address.as_str().len() > 0);

    Ok(())
}

#[test]
pub fn stake() -> Result<(), anyhow::Error> {
    // should be init with the lb-pair
    //then query it about the contract info
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts, _lb_pair, _lb_token) = lb_pair_setup()?;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_y = extract_contract_info(&deployed_contracts, SILK)?;

    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.into(),
        token_y.into(),
    )?;
    let lb_pair = all_pairs[0].clone();

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.lb_pair.contract)?;

    let staking_contract = lb_pair::query_staking_contract(&mut app, &lb_pair.lb_pair.contract)?;

    //deposit funds here

    let total_bins = get_total_bins(NB_BINS_X, NB_BINS_Y) as u32;

    let mut actions = vec![];
    //Querying all the bins
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
            recipient: staking_contract.address.clone(),
            recipient_code_hash: Some(staking_contract.code_hash.clone()),
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
