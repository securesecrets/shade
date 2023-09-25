use cosmwasm_std::{to_binary, Addr, Empty};
use secret_multi_test::{App, Contract, ContractWrapper, Executor};
use shadeswap_shared::msg::router::{ExecuteMsg, InitMsg, QueryMsg};

#[test]
pub fn router_registered_tokens() {
    use cosmwasm_std::{Coin, ContractInfo, Uint128};
    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::amm_pairs::amm_pairs_lib::amm_pairs_lib::{
        add_liquidity_to_amm_pairs, amm_pair_contract_store_in, create_amm_settings,
    };
    use multi_test::help_lib::integration_help_lib::{
        configure_block_send_init_funds, convert_to_contract_link, create_token_pair,
        create_token_pair_with_native, increase_allowance, mint_deposit_snip20, set_viewing_key,
        snip20_lp_token_contract_store, snip_20_balance_query, TestingExt,
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

    pub fn router_store() -> Box<dyn Contract<Empty>> {
        let contract =
            ContractWrapper::new_with_empty(execute, instantiate, query).with_reply(reply);
        Box::new(contract)
    }

    let owner = Addr::unchecked(OWNER);
    let mut app = App::default();

    configure_block_send_init_funds(&mut app, &owner, Uint128::new(100000000000000u128));
    // GENERATE TOKEN PAIRS & REWARD TOKEN
    let token_0 =
        generate_snip20_contract(&mut app, "ETH".to_string(), "ETH".to_string(), 18, &owner)
            .unwrap();
    let token_1 =
        generate_snip20_contract(&mut app, "USDT".to_string(), "USDT".to_string(), 18, &owner)
            .unwrap();

    // INIT LP, STAKING, AMM PAIRS
    let admin_contract = init_admin_contract(&mut app, &owner).unwrap();
    let staking_info = app.store_code(staking_contract_store_in());

    // STORE ROUTER CONTRACT
    let router_info = app.store_code(router_store());

    // INIT ROUTER CONTRACT
    let router = app
        .instantiate_contract(
            router_info,
            owner.to_owned(),
            &InitMsg {
                prng_seed: to_binary("password").unwrap(),
                entropy: to_binary("password").unwrap(),
                admin_auth: convert_to_contract_link(&admin_contract),
                airdrop_address: None,
            },
            &[],
            "router",
            Some(OWNER.to_string()),
        )
        .unwrap();

    match app
        .query_test(
            router.to_owned(),
            to_binary(&QueryMsg::RegisteredTokens {}).unwrap(),
        )
        .unwrap()
    {
        QueryMsgResponse::RegisteredTokens { tokens } => {
            assert_eq!(
                tokens,
                Vec::<Addr>::new(),
                "Empty registered tokens after init"
            );
        }
        _ => {
            panic!("Query Failed");
        }
    }

    app.execute_contract(
        owner.to_owned(),
        &router,
        &ExecuteMsg::RegisterSNIP20Token {
            token_addr: token_0.address.to_string(),
            token_code_hash: token_0.code_hash.to_owned(),
            oracle_key: None,
            padding: None,
        },
        &[],
    )
    .unwrap();

    match app
        .query_test(
            router.to_owned(),
            to_binary(&QueryMsg::RegisteredTokens {}).unwrap(),
        )
        .unwrap()
    {
        QueryMsgResponse::RegisteredTokens { tokens } => {
            assert_eq!(
                tokens,
                vec![token_0.address.clone()],
                "First token registered successfully"
            );
        }
        _ => {
            panic!("Query Failed");
        }
    }

    app.execute_contract(
        owner.to_owned(),
        &router,
        &ExecuteMsg::RegisterSNIP20Token {
            token_addr: token_1.address.to_string(),
            token_code_hash: token_1.code_hash.to_owned(),
            oracle_key: None,
            padding: None,
        },
        &[],
    )
    .unwrap();

    match app
        .query_test(
            router.to_owned(),
            to_binary(&QueryMsg::RegisteredTokens {}).unwrap(),
        )
        .unwrap()
    {
        QueryMsgResponse::RegisteredTokens { tokens } => {
            assert_eq!(
                tokens,
                vec![token_0.address, token_1.address],
                "Second token registered successfully"
            );
        }
        _ => {
            panic!("Query Failed");
        }
    }
}
