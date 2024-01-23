use ::lb_pair::state::ORACLE;
use anyhow::Ok;
use cosmwasm_std::Coin;
use serial_test::serial;
use shade_multi_test::interfaces::{lb_factory, lb_pair, lb_token, utils::DeployedContracts};
use shade_protocol::{
    c_std::{ContractInfo, StdError::GenericErr, Uint128},
    lb_libraries::types::LBPairInformation,
    liquidity_book::lb_pair::RemoveLiquidity,
    multi_test::{App, BankSudo, SudoMsg},
    swap::core::{TokenAmount, TokenType},
};

use crate::multitests::test_helper::*;

pub const ACTIVE_ID: u32 = ID_ONE - 24647;
pub const DEPOSIT_AMOUNT: u128 = 1_000_000_000_000;

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
pub fn test_contract_status() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    let amount_x = Uint128::from(600 * 100_000_000_u128); //10^8
    let amount_y = Uint128::from(100 * 100_000_000_u128);
    let nb_bins_x = 6u32;
    let nb_bins_y = 6u32;

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
        token_x.clone(),
        token_y.clone(),
        amount_x,
        amount_y,
        nb_bins_x,
        nb_bins_y,
    )?;

    // Set to lb_withdraw only
    lb_pair::set_contract_status(
        &mut app,
        addrs.admin().as_str(),
        &lb_pair.info.contract,
        shade_protocol::liquidity_book::lb_pair::ContractStatus::LpWithdrawOnly,
    )?;

    let res = lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        liquidity_parameters.clone(),
    );

    // query balance for token_minted and calculating the residue
    assert_eq!(
        res,
        Err(GenericErr {
            msg: "Transaction is blocked by contract status".to_string()
        })
    );

    // Set back to ACTIVE
    lb_pair::set_contract_status(
        &mut app,
        addrs.admin().as_str(),
        &lb_pair.info.contract,
        shade_protocol::liquidity_book::lb_pair::ContractStatus::Active,
    )?;

    //add_liquidity workds fine now:
    lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        liquidity_parameters.clone(),
    )?;

    // Set to FREEZEALL
    lb_pair::set_contract_status(
        &mut app,
        addrs.admin().as_str(),
        &lb_pair.info.contract,
        shade_protocol::liquidity_book::lb_pair::ContractStatus::FreezeAll,
    )?;

    let res = lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        liquidity_parameters.clone(),
    );

    assert_eq!(
        res,
        Err(GenericErr {
            msg: "Transaction is blocked by contract status".to_string()
        })
    );

    let res: Result<(), cosmwasm_std::StdError> = lb_pair::remove_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        RemoveLiquidity {
            token_x: token_x.into(),
            token_y: token_y.into(),
            bin_step: 0,
            amount_x_min: Uint128::default(),
            amount_y_min: Uint128::default(),
            ids: Vec::new(),     // doesn't matter
            amounts: Vec::new(), // doesn't matter
            deadline: 0,
        },
    );

    assert_eq!(
        res,
        Err(GenericErr {
            msg: "Transaction is blocked by contract status".to_string()
        })
    );

    Ok(())
}

#[test]
#[serial]
pub fn test_native_tokens_error() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    let token_x = extract_contract_info(&deployed_contracts, SHADE)?;

    let deposit_coin = Coin {
        denom: "uscrt".into(),
        amount: DEPOSIT_AMOUNT.into(),
    };
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: addrs.batman().to_string().clone(),
        amount: vec![deposit_coin.clone()],
    }))
    .unwrap();
    let res = lb_pair::swap_native(
        &mut app,
        addrs.batman().as_str(),
        &lb_pair.info.contract,
        Some(addrs.joker().to_string()),
        TokenAmount {
            token: TokenType::CustomToken {
                contract_addr: token_x.address,
                token_code_hash: token_x.code_hash,
            },
            amount: Uint128::one(),
        },
    );
    // query balance for token_minted and calculating the residue
    assert_eq!(
        res,
        Err(GenericErr {
            msg: "Use the receive interface".to_string()
        })
    );

    Ok(())
}

#[test]
#[serial]
pub fn test_increase_oracle_lenght() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, _deployed_contracts, lb_pair, _lb_token) = lb_pair_setup()?;

    app.deps(&lb_pair.info.contract.address, |storage| {
        assert_eq!(ORACLE.load(storage).unwrap().samples.len(), 0);
    })?;

    // update oracle lenght

    lb_pair::increase_oracle_length(&mut app, addrs.admin().as_str(), &lb_pair.info.contract, 20)?;

    // query_oracle lenght
    app.deps(&lb_pair.info.contract.address, |storage| {
        assert_eq!(ORACLE.load(storage).unwrap().samples.len(), 20);
    })?;

    Ok(())
}
