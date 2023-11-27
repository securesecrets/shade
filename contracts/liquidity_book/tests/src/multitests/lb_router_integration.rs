use anyhow::Ok;
use serial_test::serial;
use shade_multi_test::interfaces::{
    lb_factory,
    lb_pair,
    router::{self},
    snip20,
    utils::SupportedContracts,
};
use shade_protocol::{
    c_std::{to_binary, BalanceResponse, BankQuery, Coin, QueryRequest, StdError, Uint128},
    swap::{
        core::{TokenAmount, TokenType},
        router::{Hop, InvokeMsg},
    },
};

use super::lb_pair_fees::DEPOSIT_AMOUNT;
use crate::multitests::test_helper::*;

const SWAP_AMOUNT: u128 = 1000;

#[test]
#[serial]
pub fn router_integration() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, lb_factory, mut deployed_contracts) = setup(None, None)?;

    let starting_number_of_pairs =
        lb_factory::query_number_of_lb_pairs(&mut app, &lb_factory.clone().into())?;

    let starting_number_of_pairs =
        lb_factory::query_number_of_lb_pairs(&mut app, &lb_factory.clone().into())?;

    //test the registered tokens

    //     1. IMPORT necessary modules and components from various libraries.
    //     2. DEFINE function router_contract_store that returns a new contract box.
    //     3. INITIALIZE:
    //        a. Addresses for `staker_a` and `owner`.
    //        b. A default router.
    //     4. CONFIGURE the blockchain and send initial funds to the owner address.
    router::init(&mut app, addrs.admin().as_str(), &mut deployed_contracts)?;

    let router = match deployed_contracts.clone().get(&SupportedContracts::Router) {
        Some(router) => router,
        None => panic!("Router init failed"),
    }
    .clone()
    .into();

    //     5. GENERATE three token contracts:
    //        a. ETH token
    //        b. USDT token
    //        c. RWD token (reward token)
    let shd = match deployed_contracts.get(&SupportedContracts::Snip20(SHADE.to_string())) {
        Some(shd) => shd,
        None => panic!("Shade not registered"),
    };

    let silk = match deployed_contracts.get(&SupportedContracts::Snip20(SILK.to_string())) {
        Some(silk) => silk,
        None => panic!("Silk not registered"),
    };

    router::register_snip20_token(
        &mut app,
        addrs.admin().as_str(),
        &router,
        &shd.clone().into(),
    )?;

    router::register_snip20_token(
        &mut app,
        addrs.admin().as_str(),
        &router,
        &silk.clone().into(),
    )?;
    //     6. MINT and DEPOSIT funds for each of the three generated tokens using the owner address.
    let tokens_to_mint = vec![
        (SHADE, Uint128::from(DEPOSIT_AMOUNT + SWAP_AMOUNT)),
        (SILK, Uint128::from(DEPOSIT_AMOUNT + SWAP_AMOUNT)),
    ];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.batman().into_string(),
        tokens_to_mint.clone(),
    )?;

    //     7. ROLL the blockchain forward by 1 block.

    //     8. INITIALIZE contracts and store their info:
    //        a. Admin contract -> already initialized in router
    //        b. AMM pair contract
    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let shade = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_x = token_type_snip20_generator(&shade)?;
    let token_y = token_type_snip20_generator(&silk)?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    )?;
    //        c. LP token contract -> initializated with lb_pair

    //     13. LIST the AMM pairs and ASSERT that there's only 1 AMM pair.
    let all_pairs =
        lb_factory::query_all_lb_pairs(&mut app, &lb_factory.clone().into(), token_x, token_y)?;
    assert_eq!(all_pairs.len(), 1);
    let shd_silk_lb_pair = all_pairs[0].clone();

    //     14. INCREASE the allowance for both tokens for the created AMM pair.
    increase_allowance_helper(
        &mut app,
        &deployed_contracts,
        addrs.batman().into_string(),
        shd_silk_lb_pair.info.contract.address.to_string(),
        tokens_to_mint,
    )?;

    //     15. ADD liquidity to the AMM pair.
    let amount_x = Uint128::from(DEPOSIT_AMOUNT);
    let amount_y = Uint128::from(DEPOSIT_AMOUNT);
    let nb_bins_x = 10;
    let nb_bins_y = 10;
    let liquidity_parameters = liquidity_parameters_generator(
        &deployed_contracts,
        ID_ONE,
        shade.clone(),
        silk,
        amount_x,
        amount_y,
        nb_bins_x,
        nb_bins_y,
    )?;

    lb_pair::add_liquidity(
        &mut app,
        addrs.batman().as_str(),
        &shd_silk_lb_pair.info.contract,
        liquidity_parameters,
    )?;

    //     17. QUERY the router for a swap simulation and ASSERT the expected results.
    let offer = TokenAmount {
        token: TokenType::CustomToken {
            contract_addr: shade.address.clone(),
            token_code_hash: shade.code_hash,
        },
        amount: Uint128::new(SWAP_AMOUNT),
    };

    // ASSERT SWAPSIMULATION
    let (_total_fee_amount, _lp_fee_amount, _shade_dao_fee_amount, result, price) =
        router::query_swap_simulation(
            &app,
            &router,
            offer.to_owned(),
            vec![Hop {
                addr: all_pairs[0].info.contract.address.to_string(),
                code_hash: all_pairs[0].info.contract.code_hash.clone(),
            }],
            None,
        )?;

    // Verify result not actual amount
    // println!("total_fee_amount {}", total_fee_amount);
    // println!("lp_fee_amount {}", lp_fee_amount);
    // println!("shade_dao_fee_amount {}", shade_dao_fee_amount);

    // assert_ne!(total_fee_amount, Uint128::zero());
    // assert_eq!(
    //     lp_fee_amount,
    //     total_fee_amount.multiply_ratio(9u128, 10u128)
    // );
    // assert_eq!(
    //     shade_dao_fee_amount,
    //     total_fee_amount.multiply_ratio(1u128, 10u128)
    // );
    assert_ne!(result.return_amount, Uint128::zero());
    assert_eq!(price, "0".to_string());

    //     18. EXECUTE a token swap operation.
    let router_invoke_msg = to_binary(&InvokeMsg::SwapTokensForExact {
        expected_return: Some(Uint128::new(999u128)),
        path: vec![Hop {
            addr: shd_silk_lb_pair.info.contract.address.to_string(),
            code_hash: shd_silk_lb_pair.info.contract.code_hash.clone(),
        }],
        recipient: Some(addrs.scare_crow().to_string()),
    })
    .unwrap();

    snip20::send_exec(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        router.address.to_string(),
        Uint128::new(SWAP_AMOUNT),
        Some(router_invoke_msg),
    )?;

    snip20::set_viewing_key_exec(
        &mut app,
        addrs.scare_crow().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    let scare_crow_balance = snip20::balance_query(
        &app,
        addrs.scare_crow().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(scare_crow_balance, Uint128::from(999u128));

    //     19. EXECUTE a swap for exact tokens operation and check the resulting balance of a token.
    let res = router::swap_tokens_for_exact_tokens(
        &mut app,
        addrs.batman().as_str(),
        &router,
        offer,
        Some(Uint128::from(999u128)),
        vec![Hop {
            addr: all_pairs[0].info.contract.address.to_string(),
            code_hash: all_pairs[0].info.contract.code_hash.clone(),
        }],
        Some(addrs.scare_crow().to_string()),
    );
    assert_eq!(
        res,
        Err(StdError::GenericErr {
            msg: "Generic error: Sent a non-native token. Should use the receive interface in SNIP20.".to_string()
        })
    );

    snip20::set_viewing_key_exec(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    let batman_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SHADE,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(batman_balance, Uint128::zero());

    snip20::set_viewing_key_exec(
        &mut app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    let batman_balance = snip20::balance_query(
        &app,
        addrs.batman().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    assert_eq!(batman_balance, Uint128::from(SWAP_AMOUNT));

    // add quote_asset:
    lb_factory::add_quote_asset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        TokenType::NativeToken {
            denom: "uscrt".to_string(),
        },
    )?;

    //     20. CREATE another AMM pair between a native token(SSCRT) and a SNIP20 token(SILK)
    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let token_x = token_type_native_generator("uscrt".to_string())?;
    let token_y = token_type_snip20_generator(&silk)?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    )?;

    //     21. LIST the AMM pairs and ASSERT there are now 2 AMM pairs.
    let number_of_pairs =
        lb_factory::query_number_of_lb_pairs(&mut app, &lb_factory.clone().into())?;

    assert_eq!(number_of_pairs, starting_number_of_pairs + 2);

    let all_pairs = lb_factory::query_all_lb_pairs(
        &mut app,
        &lb_factory.clone().into(),
        token_x.clone(),
        token_y.clone(),
    )?;
    assert_eq!(all_pairs.len(), 1);
    let scrt_silk_lb_pair = all_pairs[0].clone();

    //     22. INCREASE the allowance for the SNIP20 token for the new AMM pair.
    let tokens_to_mint = vec![(SILK, Uint128::from(DEPOSIT_AMOUNT + SWAP_AMOUNT))];

    mint_token_helper(
        &mut app,
        &deployed_contracts,
        &addrs,
        addrs.joker().into_string(),
        tokens_to_mint.clone(),
    )?;

    increase_allowance_helper(
        &mut app,
        &deployed_contracts,
        addrs.joker().into_string(),
        scrt_silk_lb_pair.info.contract.address.to_string(),
        tokens_to_mint,
    )?;

    //     23. ADD liquidity to the new AMM pair.
    let amount_x = Uint128::from(DEPOSIT_AMOUNT);
    let amount_y = Uint128::from(DEPOSIT_AMOUNT);
    let nb_bins_x = 10;
    let nb_bins_y = 10;
    let liquidity_parameters = liquidity_parameters_generator_with_native(
        &deployed_contracts,
        ID_ONE,
        token_x,
        token_y,
        amount_x,
        amount_y,
        nb_bins_x,
        nb_bins_y,
    )?;

    app.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &addrs.joker(), vec![Coin {
                denom: "uscrt".into(),
                amount: amount_x + Uint128::from(SWAP_AMOUNT),
            }])
            .unwrap();
    });

    lb_pair::add_native_liquidity(
        &mut app,
        addrs.joker().as_str(),
        &scrt_silk_lb_pair.info.contract,
        liquidity_parameters,
        vec![Coin {
            denom: String::from("uscrt"),
            amount: amount_x,
        }],
    )?;

    let res: BalanceResponse = app
        .wrap()
        .query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
            address: addrs.joker().to_string(),
            denom: "uscrt".to_string(),
        }))
        .unwrap();

    assert_eq!(res, BalanceResponse {
        amount: Coin {
            amount: Uint128::new(SWAP_AMOUNT),
            denom: "uscrt".to_string(),
        },
    });

    //     25. SWAP a native token for a SNIP20 token and ASSERT the resulting balance of the SNIP20 token.
    let offer = TokenAmount {
        token: TokenType::CustomToken {
            contract_addr: silk.address.clone(),
            token_code_hash: silk.code_hash,
        },
        amount: Uint128::new(SWAP_AMOUNT),
    };
    let (_total_fee_amount, _lp_fee_amount, _shade_dao_fee_amount, _result, price) =
        router::query_swap_simulation(
            &app,
            &router,
            offer,
            vec![Hop {
                addr: scrt_silk_lb_pair.info.contract.address.to_string(),
                code_hash: scrt_silk_lb_pair.info.contract.code_hash.clone(),
            }],
            None,
        )?;

    // Verify result not actual amount
    // assert_ne!(total_fee_amount, Uint128::zero());
    // assert_ne!(lp_fee_amount, Uint128::zero());
    // assert_ne!(shade_dao_fee_amount, Uint128::zero());
    // assert_ne!(result.return_amount, Uint128::zero());
    assert_eq!(price, "0".to_string());

    //Swapping SILK -> USCRT
    let router_invoke_msg = to_binary(&InvokeMsg::SwapTokensForExact {
        expected_return: Some(Uint128::new(999u128)),
        path: vec![Hop {
            addr: scrt_silk_lb_pair.info.contract.address.to_string(),
            code_hash: scrt_silk_lb_pair.info.contract.code_hash.clone(),
        }],
        recipient: None,
    })
    .unwrap();

    snip20::send_exec(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SILK,
        router.address.to_string(),
        Uint128::new(SWAP_AMOUNT),
        Some(router_invoke_msg),
    )?;

    //Query SILK and uscrt balance
    snip20::set_viewing_key_exec(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    let altaf_bhai_balance = snip20::balance_query(
        &app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(altaf_bhai_balance, Uint128::zero());

    let res: BalanceResponse = app
        .wrap()
        .query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
            address: addrs.joker().to_string(),
            denom: "uscrt".to_string(),
        }))
        .unwrap();

    assert_eq!(res, BalanceResponse {
        amount: Coin {
            amount: Uint128::new(SWAP_AMOUNT + 999),
            denom: "uscrt".to_string(),
        },
    });

    //Swapping USCRT -> SILK
    let offer = TokenAmount {
        token: TokenType::NativeToken {
            denom: "uscrt".to_string(),
        },
        amount: Uint128::new(SWAP_AMOUNT),
    };

    router::swap_tokens_for_exact_tokens(
        &mut app,
        addrs.joker().as_str(),
        &router,
        offer,
        Some(Uint128::new(999u128)),
        vec![Hop {
            addr: scrt_silk_lb_pair.info.contract.address.to_string(),
            code_hash: scrt_silk_lb_pair.info.contract.code_hash.clone(),
        }],
        None,
    )?;

    //Query SILK and uscrt balance
    snip20::set_viewing_key_exec(
        &mut app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;

    let altaf_bhai_balance = snip20::balance_query(
        &app,
        addrs.joker().as_str(),
        &deployed_contracts,
        SILK,
        "viewing_key".to_owned(),
    )?;
    assert_eq!(altaf_bhai_balance, Uint128::from(999u128));

    let res: BalanceResponse = app
        .wrap()
        .query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
            address: addrs.joker().to_string(),
            denom: "uscrt".to_string(),
        }))
        .unwrap();

    assert_eq!(res, BalanceResponse {
        amount: Coin {
            amount: Uint128::from(999u128),
            denom: "uscrt".to_string(),
        },
    });

    //     20. CREATE another AMM pair between a native token(SSCRT) and a SNIP20 token(SILK)
    let shade = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_x = token_type_snip20_generator(&shade)?;
    let token_y = token_type_native_generator("uscrt".to_string())?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ID_ONE,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    )?;

    //     21. LIST the AMM pairs and ASSERT there are now 2 AMM pairs.
    let number_of_pairs =
        lb_factory::query_number_of_lb_pairs(&mut app, &lb_factory.clone().into())?;

    assert_eq!(number_of_pairs, starting_number_of_pairs + 3);

    Ok(())
}
