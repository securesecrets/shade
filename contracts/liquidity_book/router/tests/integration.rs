use cosmwasm_std::{to_binary, Addr, Empty};
use secret_multi_test::{App, Contract, ContractWrapper, Executor};
use shadeswap_shared::msg::router::{ExecuteMsg, InitMsg, QueryMsg};

#[cfg(not(target_arch = "wasm32"))]
#[test]
#[ignore = "broken"]
pub fn router_integration_tests() {
    use cosmwasm_std::{Coin, ContractInfo, Uint128};
    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::amm_pairs::amm_pairs_lib::amm_pairs_lib::{
        add_liquidity_to_amm_pairs, amm_pair_contract_store_in, create_amm_settings,
    };
    use multi_test::help_lib::integration_help_lib::{
        configure_block_send_init_funds, convert_to_contract_link, create_token_pair,
        create_token_pair_with_native, increase_allowance, mint_deposit_snip20, roll_blockchain,
        set_viewing_key, snip20_lp_token_contract_store, snip_20_balance_query, TestingExt,
    };
    use multi_test::staking::staking_lib::staking_lib::{
        create_staking_info_contract, staking_contract_store_in,
    };
    use multi_test::util_addr::util_addr::{OWNER, STAKER_A};
    use router::contract::{execute, instantiate, query, reply};

    use shade_protocol::snip20;
    use shade_protocol::utils::asset::RawContract;
    use shadeswap_shared::core::{ContractInstantiationInfo, TokenAmount};
    use shadeswap_shared::msg::amm_pair::InvokeMsg;
    use shadeswap_shared::msg::router::QueryMsgResponse;
    use shadeswap_shared::router::Hop;

    use multi_test::help_lib::integration_help_lib::generate_snip20_contract;
    use shadeswap_shared::core::TokenType;

    use multi_test::factory::factory_lib::factory_lib::{
        create_amm_pairs_to_factory, init_factory, list_amm_pairs_from_factory,
    };

    pub fn router_contract_store() -> Box<dyn Contract<Empty>> {
        let contract =
            ContractWrapper::new_with_empty(execute, instantiate, query).with_reply(reply);
        Box::new(contract)
    }

    let staker_a_addr = Addr::unchecked(STAKER_A.to_owned());
    let owner_addr = Addr::unchecked(OWNER);
    let mut router = App::default();

    configure_block_send_init_funds(&mut router, &owner_addr, Uint128::new(100000000000000u128));
    // GENERATE TOKEN PAIRS & REWARD TOKEN
    let token_0_contract = generate_snip20_contract(
        &mut router,
        "ETH".to_string(),
        "ETH".to_string(),
        18,
        &owner_addr,
    )
    .unwrap();
    let token_1_contract = generate_snip20_contract(
        &mut router,
        "USDT".to_string(),
        "USDT".to_string(),
        18,
        &owner_addr,
    )
    .unwrap();
    let reward_contract = generate_snip20_contract(
        &mut router,
        "RWD".to_string(),
        "RWD".to_string(),
        18,
        &owner_addr,
    )
    .unwrap();

    // MINT AND DEPOSIT FOR LIQUIDITY
    mint_deposit_snip20(
        &mut router,
        &token_0_contract,
        &owner_addr,
        Uint128::new(10000000000u128),
        &owner_addr,
    );
    mint_deposit_snip20(
        &mut router,
        &token_1_contract,
        &owner_addr,
        Uint128::new(10000000000u128),
        &owner_addr,
    );
    mint_deposit_snip20(
        &mut router,
        &reward_contract,
        &owner_addr,
        Uint128::new(10000000000u128),
        &owner_addr,
    );

    roll_blockchain(&mut router, 1).unwrap();

    // INIT LP, STAKING, AMM PAIRS
    let admin_contract = init_admin_contract(&mut router, &owner_addr).unwrap(); //store_init_factory_contract(&mut router, &convert_to_contract_link(&admin_contract)).unwrap();
    let amm_contract_info = router.store_code(amm_pair_contract_store_in());
    let lp_token_info = router.store_code(snip20_lp_token_contract_store());
    let staking_info = router.store_code(staking_contract_store_in());

    // STORE ROUTER CONTRACT
    let router_contract_info = router.store_code(router_contract_store());
    roll_blockchain(&mut router, 1).unwrap();

    // INIT ROUTER CONTRACTs
    let init_msg = InitMsg {
        prng_seed: to_binary("password").unwrap(),
        entropy: to_binary("password").unwrap(),
        admin_auth: convert_to_contract_link(&admin_contract),
        airdrop_address: None,
    };

    roll_blockchain(&mut router, 1).unwrap();
    let router_contract = router
        .instantiate_contract(
            router_contract_info,
            owner_addr.to_owned(),
            &init_msg,
            &[],
            "router",
            Some(OWNER.to_string()),
        )
        .unwrap();

    // CREATE FACTORY
    roll_blockchain(&mut router, 1).unwrap();
    let factory_contract = init_factory(
        &mut router,
        &convert_to_contract_link(&admin_contract),
        &OWNER,
        false,
        create_amm_settings(3, 100, 8, 100, &staker_a_addr),
        ContractInstantiationInfo {
            code_hash: amm_contract_info.code_hash.clone(),
            id: amm_contract_info.code_id,
        },
        ContractInstantiationInfo {
            code_hash: lp_token_info.code_hash.clone(),
            id: lp_token_info.code_id,
        },
        "seed",
        "api_key",
        None,
    )
    .unwrap();

    // CREATE AMM_PAIR SNIP20 vs SNIP20
    create_amm_pairs_to_factory(
        &mut router,
        &factory_contract,
        &create_token_pair(
            &convert_to_contract_link(&token_0_contract),
            &convert_to_contract_link(&token_1_contract),
            false,
        ),
        "seed",
        &create_staking_info_contract(
            staking_info.code_id,
            &staking_info.code_hash,
            Uint128::new(30000u128),
            RawContract {
                address: reward_contract.address.to_string(),
                code_hash: reward_contract.code_hash.clone(),
            },
            30000000000u64,
            None,
        ),
        &router_contract,
        18u8,
        &owner_addr,
        None,
        None,
        None,
    )
    .unwrap();

    // LIST AMM PAIR
    let amm_pairs = list_amm_pairs_from_factory(&mut router, &factory_contract, 0, 30).unwrap();

    // ASSERT AMM PAIRS == 1
    assert_eq!(amm_pairs.len(), 1);

    // INCREASE ALLOWANCE FOR AMM PAIR
    increase_allowance(
        &mut router,
        &token_0_contract,
        Uint128::new(10000000000000000u128),
        &amm_pairs[0].address,
        &owner_addr,
    )
    .unwrap();
    increase_allowance(
        &mut router,
        &token_1_contract,
        Uint128::new(10000000000000000u128),
        &amm_pairs[0].address,
        &owner_addr,
    )
    .unwrap();

    // ADD LIQUIDITY TO AMM_PAIR SNIP20 vs SNIP20
    add_liquidity_to_amm_pairs(
        &mut router,
        &ContractInfo {
            address: amm_pairs[0].address.clone(),
            code_hash: "".to_string(),
        },
        &amm_pairs[0].pair,
        Uint128::new(1000000000u128),
        Uint128::new(1000000000u128),
        Some(Uint128::new(1000000000u128)),
        Some(true),
        &owner_addr,
        &[],
    )
    .unwrap();

    // REGISTER SNIP 20 ROUTER
    roll_blockchain(&mut router, 1).unwrap();
    let msg = ExecuteMsg::RegisterSNIP20Token {
        token_addr: token_0_contract.address.to_string(),
        token_code_hash: token_0_contract.code_hash.to_owned(),
        oracle_key: None,
        padding: None,
    };
    roll_blockchain(&mut router, 1).unwrap();
    let _ = router
        .execute_contract(owner_addr.to_owned(), &router_contract, &msg, &[])
        .unwrap();

    roll_blockchain(&mut router, 1).unwrap();
    let msg = ExecuteMsg::RegisterSNIP20Token {
        token_addr: token_1_contract.address.to_string(),
        token_code_hash: token_1_contract.code_hash.to_owned(),
        oracle_key: None,
        padding: None,
    };
    roll_blockchain(&mut router, 1).unwrap();
    let _ = router
        .execute_contract(owner_addr.to_owned(), &router_contract, &msg, &[])
        .unwrap();

    roll_blockchain(&mut router, 1).unwrap();
    // SWAPSIMULATION - QUERY
    let offer = TokenAmount {
        token: TokenType::CustomToken {
            contract_addr: token_0_contract.address.to_owned(),
            token_code_hash: token_0_contract.code_hash.to_owned(),
        },
        amount: Uint128::new(1000u128),
    };
    let swap_query = QueryMsg::SwapSimulation {
        offer: offer.to_owned(),
        path: vec![Hop {
            addr: amm_pairs[0].address.to_string(),
            code_hash: amm_contract_info.code_hash.clone(),
        }],
        exclude_fee: None,
    };

    // ASSERT SWAPSIMULATION
    let query_response: QueryMsgResponse = router
        .query_test(router_contract.to_owned(), to_binary(&swap_query).unwrap())
        .unwrap();

    match query_response {
        QueryMsgResponse::SwapSimulation {
            total_fee_amount,
            lp_fee_amount,
            shade_dao_fee_amount,
            result,
            price,
        } => {
            // Verify result not actual amount
            assert_ne!(total_fee_amount, Uint128::zero());
            assert_ne!(lp_fee_amount, Uint128::zero());
            assert_ne!(shade_dao_fee_amount, Uint128::zero());
            assert_ne!(result.return_amount, Uint128::zero());
            assert_eq!(price, "1".to_string());
        }
        _ => panic!("Query Responsedoes not match"),
    }

    // ASSERT SWAPTOKENS
    roll_blockchain(&mut router, 1).unwrap();
    let invoke_msg = to_binary(&InvokeMsg::SwapTokens {
        expected_return: Some(Uint128::new(100u128)),
        to: Some(staker_a_addr.to_string()),
        padding: None,
    })
    .unwrap();

    let msg = snip20::ExecuteMsg::Send {
        recipient: amm_pairs[0].address.to_owned().to_string(),
        recipient_code_hash: Some(amm_contract_info.code_hash.clone()),
        amount: Uint128::new(1000u128),
        msg: Some(invoke_msg),
        memo: None,
        padding: None,
    };

    let _response = router
        .execute_contract(
            owner_addr.to_owned(),
            &token_0_contract,
            &msg,
            &[], //
        )
        .unwrap();

    // ASSERT SWAPTOKENSFOREXACT
    roll_blockchain(&mut router, 1).unwrap();
    let execute_swap = ExecuteMsg::SwapTokensForExact {
        offer: offer.to_owned(),
        expected_return: Some(Uint128::new(1000u128)),
        path: vec![Hop {
            addr: amm_pairs[0].address.to_string(),
            code_hash: amm_contract_info.code_hash.clone(),
        }],
        recipient: Some(owner_addr.to_string()),
        padding: None,
    };

    let _response =
        router.execute_contract(owner_addr.to_owned(), &router_contract, &execute_swap, &[]);

    // ASSERT BALANCE TOKEN_1
    let balance =
        snip_20_balance_query(&mut router, &owner_addr, "seed", &token_1_contract).unwrap();
    assert_eq!(balance, Uint128::new(1000019000000000u128));

    // CREATE AMM_PAIR NATIVE - SNIP20
    create_amm_pairs_to_factory(
        &mut router,
        &factory_contract,
        &create_token_pair_with_native(&convert_to_contract_link(&token_1_contract)),
        "seed",
        &create_staking_info_contract(
            staking_info.code_id,
            &staking_info.code_hash,
            Uint128::new(30000u128),
            RawContract {
                address: reward_contract.address.to_string(),
                code_hash: reward_contract.code_hash.clone(),
            },
            30000000000u64,
            None,
        ),
        &router_contract,
        18u8,
        &owner_addr,
        None,
        None,
        None,
    )
    .unwrap();

    // LIST AMM PAIR
    let amm_pairs = list_amm_pairs_from_factory(&mut router, &factory_contract, 0, 30).unwrap();

    // ASSERT AMM PAIRS == 2
    assert_eq!(amm_pairs.len(), 2);
    increase_allowance(
        &mut router,
        &token_1_contract,
        Uint128::new(10000000000000000u128),
        &amm_pairs[1].address,
        &owner_addr,
    )
    .unwrap();
    // ADD LIQUIDITY TO AMM_PAIR NATIVE vs SNIP20
    add_liquidity_to_amm_pairs(
        &mut router,
        &ContractInfo {
            address: amm_pairs[1].address.clone(),
            code_hash: "".to_string(),
        },
        &amm_pairs[1].pair,
        Uint128::new(1000000000u128),
        Uint128::new(1000000000u128),
        Some(Uint128::new(1000000000u128)),
        Some(true),
        &owner_addr,
        &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(1000000000u128),
        }],
    )
    .unwrap();
    roll_blockchain(&mut router, 1).unwrap();

    // SWAP NATIVE TOKEN -> SNIP20
    let native_offer = TokenAmount {
        token: TokenType::NativeToken {
            denom: "uscrt".to_string(),
        },
        amount: Uint128::new(1000u128),
    };
    let _ = router
        .send_tokens(
            owner_addr.clone(),
            staker_a_addr.clone(),
            &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::new(1000),
            }],
        )
        .unwrap();
    let execute_swap = ExecuteMsg::SwapTokensForExact {
        offer: native_offer.to_owned(),
        expected_return: Some(Uint128::new(100u128)),
        path: vec![Hop {
            addr: amm_pairs[1].address.to_string(),
            code_hash: amm_contract_info.code_hash.clone(),
        }],
        recipient: None,
        padding: None,
    };

    let _response = router.execute_contract(
        staker_a_addr.to_owned(),
        &router_contract,
        &execute_swap,
        &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(1000u128),
        }],
    );

    // ASSERT BALANCE TOKEN_1 889
    let _ = set_viewing_key(&mut router, &token_1_contract, "password", &staker_a_addr).unwrap();
    let balance =
        snip_20_balance_query(&mut router, &staker_a_addr, "password", &token_1_contract).unwrap();
    assert_eq!(balance, Uint128::new(970u128));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
#[ignore = "broken"]
pub fn router_integration_tests_stable() {
    use cosmwasm_std::{Coin, ContractInfo, Uint128};
    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::amm_pairs::amm_pairs_lib::amm_pairs_lib::{
        add_liquidity_to_amm_pairs, amm_pair_contract_store_in, create_amm_settings,
    };
    use multi_test::help_lib::integration_help_lib::{
        configure_block_send_init_funds, convert_to_contract_link, create_token_pair,
        create_token_pair_with_native, increase_allowance, mint_deposit_snip20, roll_blockchain,
        set_viewing_key, snip20_lp_token_contract_store, snip_20_balance_query, TestingExt,
    };
    use multi_test::staking::staking_lib::staking_lib::{
        create_staking_info_contract, staking_contract_store_in,
    };
    use multi_test::util_addr::util_addr::{OWNER, STAKER_A};
    use router::contract::{execute, instantiate, query, reply};

    use shade_protocol::snip20;
    use shade_protocol::utils::asset::RawContract;
    use shadeswap_shared::core::{ContractInstantiationInfo, TokenAmount};
    use shadeswap_shared::msg::amm_pair::InvokeMsg;
    use shadeswap_shared::msg::router::QueryMsgResponse;
    use shadeswap_shared::router::Hop;

    use multi_test::help_lib::integration_help_lib::generate_snip20_contract;
    use shadeswap_shared::core::TokenType;

    use multi_test::factory::factory_lib::factory_lib::{
        create_amm_pairs_to_factory, init_factory, list_amm_pairs_from_factory,
    };

    pub fn router_contract_store() -> Box<dyn Contract<Empty>> {
        let contract =
            ContractWrapper::new_with_empty(execute, instantiate, query).with_reply(reply);
        Box::new(contract)
    }

    let staker_a_addr = Addr::unchecked(STAKER_A.to_owned());
    let owner_addr = Addr::unchecked(OWNER);
    let mut router = App::default();

    configure_block_send_init_funds(&mut router, &owner_addr, Uint128::new(100000000000000u128));
    // GENERATE TOKEN PAIRS & REWARD TOKEN
    let token_0_contract = generate_snip20_contract(
        &mut router,
        "ETH".to_string(),
        "ETH".to_string(),
        18,
        &owner_addr,
    )
    .unwrap();
    let token_1_contract = generate_snip20_contract(
        &mut router,
        "USDT".to_string(),
        "USDT".to_string(),
        18,
        &owner_addr,
    )
    .unwrap();
    let reward_contract = generate_snip20_contract(
        &mut router,
        "RWD".to_string(),
        "RWD".to_string(),
        18,
        &owner_addr,
    )
    .unwrap();

    // MINT AND DEPOSIT FOR LIQUIDITY
    mint_deposit_snip20(
        &mut router,
        &token_0_contract,
        &owner_addr,
        Uint128::new(10000000000u128),
        &owner_addr,
    );
    mint_deposit_snip20(
        &mut router,
        &token_1_contract,
        &owner_addr,
        Uint128::new(10000000000u128),
        &owner_addr,
    );
    mint_deposit_snip20(
        &mut router,
        &reward_contract,
        &owner_addr,
        Uint128::new(10000000000u128),
        &owner_addr,
    );

    roll_blockchain(&mut router, 1).unwrap();

    // INIT LP, STAKING, AMM PAIRS
    let admin_contract = init_admin_contract(&mut router, &owner_addr).unwrap(); //store_init_factory_contract(&mut router, &convert_to_contract_link(&admin_contract)).unwrap();
    let amm_contract_info = router.store_code(amm_pair_contract_store_in());
    let lp_token_info = router.store_code(snip20_lp_token_contract_store());
    let staking_info = router.store_code(staking_contract_store_in());

    // STORE ROUTER CONTRACT
    let router_contract_info = router.store_code(router_contract_store());
    roll_blockchain(&mut router, 1).unwrap();

    // INIT ROUTER CONTRACTs
    let init_msg = InitMsg {
        prng_seed: to_binary("password").unwrap(),
        entropy: to_binary("password").unwrap(),
        admin_auth: convert_to_contract_link(&admin_contract),
        airdrop_address: None,
    };

    roll_blockchain(&mut router, 1).unwrap();
    let router_contract = router
        .instantiate_contract(
            router_contract_info,
            owner_addr.to_owned(),
            &init_msg,
            &[],
            "router",
            Some(OWNER.to_string()),
        )
        .unwrap();

    // CREATE FACTORY
    roll_blockchain(&mut router, 1).unwrap();
    let factory_contract = init_factory(
        &mut router,
        &convert_to_contract_link(&admin_contract),
        &OWNER,
        false,
        create_amm_settings(3, 100, 8, 100, &staker_a_addr),
        ContractInstantiationInfo {
            code_hash: amm_contract_info.code_hash.clone(),
            id: amm_contract_info.code_id,
        },
        ContractInstantiationInfo {
            code_hash: lp_token_info.code_hash.clone(),
            id: lp_token_info.code_id,
        },
        "seed",
        "api_key",
        None,
    )
    .unwrap();

    // CREATE AMM_PAIR SNIP20 vs SNIP20
    create_amm_pairs_to_factory(
        &mut router,
        &factory_contract,
        &create_token_pair(
            &convert_to_contract_link(&token_0_contract),
            &convert_to_contract_link(&token_1_contract),
            false,
        ),
        "seed",
        &create_staking_info_contract(
            staking_info.code_id,
            &staking_info.code_hash,
            Uint128::new(30000u128),
            RawContract {
                address: reward_contract.address.to_string(),
                code_hash: reward_contract.code_hash.clone(),
            },
            30000000000u64,
            None,
        ),
        &router_contract,
        18u8,
        &owner_addr,
        None,
        None,
        None,
    )
    .unwrap();

    // LIST AMM PAIR
    let amm_pairs = list_amm_pairs_from_factory(&mut router, &factory_contract, 0, 30).unwrap();

    // ASSERT AMM PAIRS == 1
    assert_eq!(amm_pairs.len(), 1);

    // INCREASE ALLOWANCE FOR AMM PAIR
    increase_allowance(
        &mut router,
        &token_0_contract,
        Uint128::new(10000000000000000u128),
        &amm_pairs[0].address,
        &owner_addr,
    )
    .unwrap();
    increase_allowance(
        &mut router,
        &token_1_contract,
        Uint128::new(10000000000000000u128),
        &amm_pairs[0].address,
        &owner_addr,
    )
    .unwrap();

    // ADD LIQUIDITY TO AMM_PAIR SNIP20 vs SNIP20
    add_liquidity_to_amm_pairs(
        &mut router,
        &ContractInfo {
            address: amm_pairs[0].address.clone(),
            code_hash: "".to_string(),
        },
        &amm_pairs[0].pair,
        Uint128::new(1000000000u128),
        Uint128::new(1000000000u128),
        Some(Uint128::new(1000000000u128)),
        Some(true),
        &owner_addr,
        &[],
    )
    .unwrap();

    // REGISTER SNIP 20 ROUTER
    roll_blockchain(&mut router, 1).unwrap();
    let msg = ExecuteMsg::RegisterSNIP20Token {
        token_addr: token_0_contract.address.to_string(),
        token_code_hash: token_0_contract.code_hash.to_owned(),
        oracle_key: None,
        padding: None,
    };
    roll_blockchain(&mut router, 1).unwrap();
    let _ = router
        .execute_contract(owner_addr.to_owned(), &router_contract, &msg, &[])
        .unwrap();

    roll_blockchain(&mut router, 1).unwrap();
    let msg = ExecuteMsg::RegisterSNIP20Token {
        token_addr: token_1_contract.address.to_string(),
        token_code_hash: token_1_contract.code_hash.to_owned(),
        oracle_key: None,
        padding: None,
    };
    roll_blockchain(&mut router, 1).unwrap();
    let _ = router
        .execute_contract(owner_addr.to_owned(), &router_contract, &msg, &[])
        .unwrap();

    roll_blockchain(&mut router, 1).unwrap();
    // SWAPSIMULATION - QUERY
    let offer = TokenAmount {
        token: TokenType::CustomToken {
            contract_addr: token_0_contract.address.to_owned(),
            token_code_hash: token_0_contract.code_hash.to_owned(),
        },
        amount: Uint128::new(1000u128),
    };
    let swap_query = QueryMsg::SwapSimulation {
        offer: offer.to_owned(),
        path: vec![Hop {
            addr: amm_pairs[0].address.to_string(),
            code_hash: amm_contract_info.code_hash.clone(),
        }],
        exclude_fee: None,
    };

    // ASSERT SWAPSIMULATION
    let query_response: QueryMsgResponse = router
        .query_test(router_contract.to_owned(), to_binary(&swap_query).unwrap())
        .unwrap();

    match query_response {
        QueryMsgResponse::SwapSimulation {
            total_fee_amount,
            lp_fee_amount,
            shade_dao_fee_amount,
            result,
            price,
        } => {
            // Verify result not actual amount
            assert_ne!(total_fee_amount, Uint128::zero());
            assert_ne!(lp_fee_amount, Uint128::zero());
            assert_ne!(shade_dao_fee_amount, Uint128::zero());
            assert_ne!(result.return_amount, Uint128::zero());
            assert_eq!(price, "1".to_string());
        }
        _ => panic!("Query Responsedoes not match"),
    }

    // ASSERT SWAPTOKENS
    roll_blockchain(&mut router, 1).unwrap();
    let invoke_msg = to_binary(&InvokeMsg::SwapTokens {
        expected_return: Some(Uint128::new(100u128)),
        to: Some(staker_a_addr.to_string()),
        padding: None,
    })
    .unwrap();

    let msg = snip20::ExecuteMsg::Send {
        recipient: amm_pairs[0].address.to_owned().to_string(),
        recipient_code_hash: Some(amm_contract_info.code_hash.clone()),
        amount: Uint128::new(1000u128),
        msg: Some(invoke_msg),
        memo: None,
        padding: None,
    };

    let _response = router
        .execute_contract(
            owner_addr.to_owned(),
            &token_0_contract,
            &msg,
            &[], //
        )
        .unwrap();

    // ASSERT SWAPTOKENSFOREXACT
    roll_blockchain(&mut router, 1).unwrap();
    let execute_swap = ExecuteMsg::SwapTokensForExact {
        offer: offer.to_owned(),
        expected_return: Some(Uint128::new(1000u128)),
        path: vec![Hop {
            addr: amm_pairs[0].address.to_string(),
            code_hash: amm_contract_info.code_hash.clone(),
        }],
        recipient: Some(owner_addr.to_string()),
        padding: None,
    };

    let _response =
        router.execute_contract(owner_addr.to_owned(), &router_contract, &execute_swap, &[]);

    // ASSERT BALANCE TOKEN_1
    let balance =
        snip_20_balance_query(&mut router, &owner_addr, "seed", &token_1_contract).unwrap();
    assert_eq!(balance, Uint128::new(1000019000000000u128));

    // CREATE AMM_PAIR NATIVE - SNIP20
    create_amm_pairs_to_factory(
        &mut router,
        &factory_contract,
        &create_token_pair_with_native(&convert_to_contract_link(&token_1_contract)),
        "seed",
        &create_staking_info_contract(
            staking_info.code_id,
            &staking_info.code_hash,
            Uint128::new(30000u128),
            RawContract {
                address: reward_contract.address.to_string(),
                code_hash: reward_contract.code_hash.clone(),
            },
            30000000000u64,
            None,
        ),
        &router_contract,
        18u8,
        &owner_addr,
        None,
        None,
        None,
    )
    .unwrap();

    // LIST AMM PAIR
    let amm_pairs = list_amm_pairs_from_factory(&mut router, &factory_contract, 0, 30).unwrap();

    // ASSERT AMM PAIRS == 2
    assert_eq!(amm_pairs.len(), 2);
    increase_allowance(
        &mut router,
        &token_1_contract,
        Uint128::new(10000000000000000u128),
        &amm_pairs[1].address,
        &owner_addr,
    )
    .unwrap();
    // ADD LIQUIDITY TO AMM_PAIR NATIVE vs SNIP20
    add_liquidity_to_amm_pairs(
        &mut router,
        &ContractInfo {
            address: amm_pairs[1].address.clone(),
            code_hash: "".to_string(),
        },
        &amm_pairs[1].pair,
        Uint128::new(1000000000u128),
        Uint128::new(1000000000u128),
        Some(Uint128::new(1000000000u128)),
        Some(true),
        &owner_addr,
        &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(1000000000u128),
        }],
    )
    .unwrap();
    roll_blockchain(&mut router, 1).unwrap();

    // SWAP NATIVE TOKEN -> SNIP20
    let native_offer = TokenAmount {
        token: TokenType::NativeToken {
            denom: "uscrt".to_string(),
        },
        amount: Uint128::new(1000u128),
    };
    let _ = router
        .send_tokens(
            owner_addr.clone(),
            staker_a_addr.clone(),
            &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::new(1000),
            }],
        )
        .unwrap();
    let execute_swap = ExecuteMsg::SwapTokensForExact {
        offer: native_offer.to_owned(),
        expected_return: Some(Uint128::new(100u128)),
        path: vec![Hop {
            addr: amm_pairs[1].address.to_string(),
            code_hash: amm_contract_info.code_hash.clone(),
        }],
        recipient: None,
        padding: None,
    };

    let _response = router.execute_contract(
        staker_a_addr.to_owned(),
        &router_contract,
        &execute_swap,
        &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(1000u128),
        }],
    );

    // ASSERT BALANCE TOKEN_1 889
    let _ = set_viewing_key(&mut router, &token_1_contract, "password", &staker_a_addr).unwrap();
    let balance =
        snip_20_balance_query(&mut router, &staker_a_addr, "password", &token_1_contract).unwrap();
    assert_eq!(balance, Uint128::new(889u128));
}
