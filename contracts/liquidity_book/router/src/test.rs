#[cfg(test)]
pub mod tests {
    use crate::{
        contract::{execute, instantiate, SWAP_REPLY_ID},
        state::{config_r, epheral_storage_r, epheral_storage_w, Config, CurrentSwapInfo},
    };
    use cosmwasm_std::{
        from_slice,
        testing::{mock_env, mock_info, MockApi, MockStorage},
        to_binary,
        Addr,
        OwnedDeps,
        Response,
        StdResult,
        SubMsg,
    };

    use cosmwasm_std::{Api, Coin};
    use serde::{Deserialize, Serialize};
    use shade_protocol::{snip20::Snip20ReceiveMsg, utils::liquidity_book::tokens::TokenType};
    use shadeswap_shared::{
        admin::ValidateAdminPermissionResponse,
        amm_pair::FeeInfo,
        core::TokenPair,
        msg::{
            amm_pair::{
                ExecuteMsg as AMMPairExecuteMsg,
                QueryMsgResponse as AMMPairQueryMsgResponse,
            },
            factory::QueryResponse as FactoryQueryResponse,
        },
    };

    use cosmwasm_std::{
        Empty,
        Env,
        Querier,
        QuerierResult,
        QueryRequest,
        StdError,
        Storage,
        Uint128,
        WasmMsg,
        WasmQuery,
    };
    use shade_protocol::Contract;
    use shadeswap_shared::{
        core::{ContractInstantiationInfo, Fee, TokenAmount},
        router::{ExecuteMsg, Hop, InitMsg, InvokeMsg},
    };

    use shadeswap_shared::snip20::manager::Balance;

    pub const FACTORY_ADDRESS: &str = "FACTORY_ADDRESS";
    pub const PAIR_CONTRACT_1: &str = "paircontracta";
    pub const PAIR_CONTRACT_2: &str = "paircontractb";
    pub const CUSTOM_TOKEN_1: &str = "secret1h92kweqxaxwwf3wny8qmcalsxe366s940eqd54";

    #[test]
    fn ok_init() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let env = mock_env();
        let config = mkconfig(env.clone(), 0);
        let mock_info = mock_info("admin", &[]);
        assert!(instantiate(deps.as_mut(), env.clone(), mock_info, (&config).into()).is_ok());
        assert_eq!(config, config_r(deps.as_mut().storage).load()?);
        Ok(())
    }

    #[test]
    fn swap_native_for_snip20_tokens_ok() -> StdResult<()> {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::new(10u128),
            }]),
            ExecuteMsg::SwapTokensForExact {
                offer: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".to_string(),
                    },
                    amount: Uint128::new(10u128),
                },
                expected_return: None,
                path: vec![Hop {
                    addr: PAIR_CONTRACT_1.to_string(),
                    code_hash: "".to_string(),
                }],
                recipient: None,
                padding: None,
            },
        )
        .unwrap();

        assert!(result.messages.len() > 0);
        let result = epheral_storage_r(&deps.storage).load();
        match result {
            Ok(info) => {
                assert_eq!(info.amount, TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".to_string(),
                    },
                    amount: Uint128::new(10u128),
                });

                assert_eq!(info.path, vec![Hop {
                    addr: PAIR_CONTRACT_1.to_string(),
                    code_hash: "".to_string()
                }]);
            }
            Err(_) => panic!("Ephemeral storage should not be empty!"),
        }

        Ok(())
    }

    #[test]
    fn swap_snip20_native_for_tokens_ok() -> StdResult<()> {
        let (init_result, mut deps) = init_helper();
        let env = mock_env();
        let mock_info = mock_info("admin", &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(10u128),
        }]);

        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let result = execute(
            deps.as_mut(),
            env,
            mock_info,
            ExecuteMsg::SwapTokensForExact {
                offer: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".to_string(),
                    },
                    amount: Uint128::new(10u128),
                },
                expected_return: None,
                path: vec![Hop {
                    addr: PAIR_CONTRACT_1.to_string(),
                    code_hash: "".to_string(),
                }],
                recipient: Some("sender_addr".to_string()),
                padding: None,
            },
        )
        .unwrap();

        assert!(result.messages.len() > 0);
        let result = epheral_storage_r(&deps.storage).load();
        match result {
            Ok(info) => {
                assert_eq!(info.amount, TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".to_string(),
                    },
                    amount: Uint128::new(10u128),
                });
                assert_eq!(info.path, vec![Hop {
                    addr: PAIR_CONTRACT_1.to_string(),
                    code_hash: "".to_string()
                }]);
            }
            Err(_) => panic!("Ephemeral storage should not be empty!"),
        }

        Ok(())
    }

    #[test]
    fn snip20_swap() -> StdResult<()> {
        let mock_info = mock_info("admin", &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(1000000000000000u128),
        }]);
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        epheral_storage_w(&mut deps.storage).save(&CurrentSwapInfo {
            amount_out_min: Some(Uint128::new(10u128)),
            amount: TokenAmount {
                token: TokenType::NativeToken {
                    denom: "uscrt".to_string(),
                },
                amount: Uint128::new(10u128),
            },
            path: vec![
                Hop {
                    addr: PAIR_CONTRACT_1.to_string(),
                    code_hash: "".to_string(),
                },
                Hop {
                    addr: PAIR_CONTRACT_2.to_string(),
                    code_hash: "".to_string(),
                },
            ],
            next_token_in: TokenType::CustomToken {
                contract_addr: Addr::unchecked("token_1"),
                token_code_hash: "".to_string(),
            },
            recipient: Addr::unchecked("recipient".to_string()),
            current_index: 0,
        })?;

        let result = execute(
            deps.as_mut(),
            mock_env(),
            mock_info,
            ExecuteMsg::Receive(Snip20ReceiveMsg {
                from: "recipient".to_string(),
                msg: Some(
                    to_binary(&InvokeMsg::SwapTokensForExact {
                        expected_return: Some(Uint128::new(1000u128)),
                        path: vec![Hop {
                            addr: PAIR_CONTRACT_1.to_string(),
                            code_hash: "".to_string(),
                        }],
                        recipient: None,
                    })
                    .unwrap(),
                ),
                amount: Uint128::new(100u128),
                sender: "recipient".to_string(),
                memo: None,
            }),
        );

        match result {
            Ok(info) => {
                println!("{:?}", info.messages);
            }
            Err(err) => {
                let _test = err.to_string();
                assert_eq!(StdError::generic_err("No matching token in pair"), err);
            }
        }

        Ok(())
    }

    #[test]
    fn first_swap_callback_with_one_more_ok() -> StdResult<()> {
        let (init_result, mut deps) = init_helper();
        let env = mock_env();

        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        epheral_storage_w(&mut deps.storage).save(&CurrentSwapInfo {
            amount: TokenAmount {
                token: TokenType::NativeToken {
                    denom: "uscrt".into(),
                },
                amount: Uint128::new(10u128),
            },
            amount_out_min: Some(Uint128::new(10u128)),
            path: vec![
                Hop {
                    addr: PAIR_CONTRACT_1.to_string(),
                    code_hash: "".to_string(),
                },
                Hop {
                    addr: PAIR_CONTRACT_2.to_string(),
                    code_hash: "".to_string(),
                },
            ],
            recipient: Addr::unchecked("recipient".to_string()),
            current_index: 0,
            next_token_in: TokenType::NativeToken {
                denom: "uscrt".into(),
            },
        })?;

        let result = execute(
            deps.as_mut(),
            env,
            mock_info("admin", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::new(10u128),
            }]),
            ExecuteMsg::SwapTokensForExact {
                offer: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".to_string(),
                    },
                    amount: Uint128::new(10u128),
                },
                expected_return: None,
                path: vec![Hop {
                    addr: PAIR_CONTRACT_1.to_string(),
                    code_hash: "".to_string(),
                }],
                recipient: None,
                padding: None,
            },
        )
        .unwrap();

        let msg = to_binary(&AMMPairExecuteMsg::SwapTokens {
            expected_return: None,
            to: None,
            offer: TokenAmount {
                token: TokenType::NativeToken {
                    denom: "uscrt".to_string(),
                },
                amount: Uint128::new(10u128),
            },
            padding: None,
        })?;

        assert_eq!(
            result.messages[0],
            SubMsg::reply_always(
                WasmMsg::Execute {
                    contract_addr: PAIR_CONTRACT_1.to_string(),
                    code_hash: "".to_string(),
                    msg,
                    funds: vec![Coin {
                        denom: "uscrt".to_string(),
                        amount: Uint128::new(10u128),
                    }],
                },
                SWAP_REPLY_ID,
            )
        );

        Ok(())
    }

    #[test]
    fn first_swap_callback_with_no_more_ok() -> StdResult<()> {
        let (init_result, mut deps) = init_helper();
        let env = mock_env();

        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        epheral_storage_w(&mut deps.storage).save(&CurrentSwapInfo {
            amount_out_min: Some(Uint128::new(10u128)),
            amount: TokenAmount {
                token: TokenType::NativeToken {
                    denom: "uscrt".into(),
                },
                amount: Uint128::new(10u128),
            },
            path: vec![Hop {
                addr: PAIR_CONTRACT_1.to_string(),
                code_hash: "".to_string(),
            }],
            recipient: Addr::unchecked("recipient".to_string()),
            current_index: 0,
            next_token_in: TokenType::NativeToken {
                denom: "uscrt".into(),
            },
        })?;

        let result = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("admin", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::new(10u128),
            }]),
            ExecuteMsg::SwapTokensForExact {
                offer: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".to_string(),
                    },
                    amount: Uint128::new(10u128),
                },
                expected_return: None,
                path: vec![Hop {
                    addr: PAIR_CONTRACT_1.to_string(),
                    code_hash: "".to_string(),
                }],
                recipient: None,
                padding: None,
            },
        )
        .unwrap();

        let msg = to_binary(&AMMPairExecuteMsg::SwapTokens {
            expected_return: None,
            to: None,
            offer: TokenAmount {
                token: TokenType::NativeToken {
                    denom: "uscrt".to_string(),
                },
                amount: Uint128::new(10u128),
            },
            padding: None,
        })?;
        assert_eq!(
            result.messages[0],
            SubMsg::reply_always(
                WasmMsg::Execute {
                    contract_addr: PAIR_CONTRACT_1.to_string(),
                    code_hash: "".to_string(),
                    msg,
                    funds: vec![Coin {
                        denom: "uscrt".to_string(),
                        amount: Uint128::new(10u128),
                    }],
                },
                SWAP_REPLY_ID,
            )
        );

        Ok(())
    }

    #[test]
    fn first_swap_callback_with_no_more_not_enough_return() -> StdResult<()> {
        let (init_result, mut deps) = init_helper();
        let env = mock_env();

        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        epheral_storage_w(&mut deps.storage).save(&CurrentSwapInfo {
            amount_out_min: Some(Uint128::new(100)),
            amount: TokenAmount {
                token: TokenType::NativeToken {
                    denom: "uscrt".into(),
                },
                amount: Uint128::new(10),
            },
            path: vec![Hop {
                addr: PAIR_CONTRACT_1.to_string(),
                code_hash: "".to_string(),
            }],
            recipient: Addr::unchecked("recipient".to_string()),
            current_index: 0,
            next_token_in: TokenType::NativeToken {
                denom: "uscrt".into(),
            },
        })?;

        let result = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("admin", &[]),
            ExecuteMsg::SwapTokensForExact {
                offer: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".to_string(),
                    },
                    amount: Uint128::new(10u128),
                },
                expected_return: None,
                path: vec![Hop {
                    addr: "token_addr".to_string(),
                    code_hash: "".to_string(),
                }],
                recipient: None,
                padding: None,
            },
        );

        match result {
            Err(StdError::GenericErr { .. }) => {}
            _ => panic!("Must return error"),
        }
        Ok(())
    }

    fn mkconfig(_env: Env, _id: u64) -> Config {
        Config {
            viewing_key: "SHADE_ROUTER_KEY".to_string(),
            admin_auth: Contract {
                address: Addr::unchecked("".to_string()),
                code_hash: "".to_string(),
            },
            airdrop_address: None,
        }
    }
    fn mkdeps() -> OwnedDeps<impl Storage, impl Api, impl Querier> {
        mock_dependencies(&[])
    }

    impl Into<InitMsg> for &Config {
        fn into(self) -> InitMsg {
            InitMsg {
                prng_seed: to_binary(&"prng").unwrap(),
                entropy: to_binary(&"entropy").unwrap(),
                admin_auth: Contract {
                    address: Addr::unchecked("".to_string()),
                    code_hash: "".to_string(),
                },
                airdrop_address: None,
            }
        }
    }

    fn init_helper() -> (
        StdResult<Response>,
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();

        let init_msg = InitMsg {
            prng_seed: to_binary(&"prng").unwrap(),
            entropy: to_binary(&"entropy").unwrap(),
            admin_auth: Contract {
                address: Addr::unchecked("admin_auth".to_string()),
                code_hash: "".to_string(),
            },
            airdrop_address: None,
        };

        let init_result = instantiate(deps.as_mut(), env, mock_info("admin", &[]), init_msg);
        // register token
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::RegisterSNIP20Token {
                token_addr: CUSTOM_TOKEN_1.to_string(),
                token_code_hash: "hash".to_string(),
                oracle_key: None,
                padding: None,
            },
        )
        .unwrap();

        (init_result, deps)
    }

    pub fn mock_dependencies(
        _contract_balance: &[Coin],
    ) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier { portion: 100 },
            custom_query_type: std::marker::PhantomData,
        }
    }

    #[derive(Serialize, Deserialize)]
    struct IntBalanceResponse {
        pub balance: Balance,
    }

    pub struct MockQuerier {
        portion: u128,
    }

    impl Querier for MockQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request: QueryRequest<Empty> = from_slice(bin_request).unwrap();
            match &request {
                QueryRequest::Wasm(msg) => match msg {
                    WasmQuery::Smart {
                        contract_addr,
                        code_hash: _,
                        msg: _,
                    } => {
                        println!("{}", contract_addr);
                        match contract_addr.as_str() {
                            FACTORY_ADDRESS => QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(
                                to_binary(&FactoryQueryResponse::GetConfig {
                                    pair_contract: ContractInstantiationInfo {
                                        code_hash: "".to_string(),
                                        id: 1,
                                    },
                                    amm_settings: shadeswap_shared::amm_pair::AMMSettings {
                                        lp_fee: Fee::new(28, 10000),
                                        shade_dao_fee: Fee::new(2, 10000),
                                        stable_lp_fee: Fee::new(28, 10000),
                                        stable_shade_dao_fee: Fee::new(2, 10000),
                                        shade_dao_address: Contract {
                                            address: Addr::unchecked(String::from("DAO")),
                                            code_hash: "".to_string(),
                                        },
                                    },
                                    lp_token_contract: ContractInstantiationInfo {
                                        code_hash: "".to_string(),
                                        id: 1,
                                    },
                                    authenticator: None,
                                    admin_auth: Contract {
                                        address: Addr::unchecked("admin_auth".to_string()),
                                        code_hash: "".to_string(),
                                    },
                                })
                                .unwrap(),
                            )),
                            "admin_auth" => QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(
                                to_binary(&ValidateAdminPermissionResponse {
                                    has_permission: true,
                                })
                                .unwrap(),
                            )),
                            PAIR_CONTRACT_1 => QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(
                                to_binary(&AMMPairQueryMsgResponse::GetPairInfo {
                                    liquidity_token: Contract {
                                        address: Addr::unchecked("asd"),
                                        code_hash: "".to_string(),
                                    },
                                    factory: Some(Contract {
                                        address: Addr::unchecked("asd"),
                                        code_hash: "".to_string(),
                                    }),
                                    pair: TokenPair(
                                        TokenType::CustomToken {
                                            contract_addr: Addr::unchecked(
                                                CUSTOM_TOKEN_1.to_string(),
                                            ),
                                            token_code_hash: "hash".into(),
                                        },
                                        TokenType::NativeToken {
                                            denom: "denom".into(),
                                        },
                                        false,
                                    ),
                                    amount_0: Uint128::new(100),
                                    amount_1: Uint128::new(101),
                                    total_liquidity: Uint128::new(100),
                                    contract_version: 1,
                                    fee_info: FeeInfo {
                                        shade_dao_address: Addr::unchecked("".to_string()),
                                        lp_fee: Fee {
                                            nom: 2u64,
                                            denom: 100u64,
                                        },
                                        shade_dao_fee: Fee {
                                            nom: 2u64,
                                            denom: 100u64,
                                        },
                                        stable_lp_fee: Fee {
                                            nom: 2u64,
                                            denom: 100u64,
                                        },
                                        stable_shade_dao_fee: Fee {
                                            nom: 2u64,
                                            denom: 100u64,
                                        },
                                    },
                                    stable_info: None,
                                })
                                .unwrap(),
                            )),
                            CUSTOM_TOKEN_1 => QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(
                                to_binary(&IntBalanceResponse {
                                    balance: Balance(Uint128::new(100)),
                                })
                                .unwrap(),
                            )),
                            _ => unimplemented!(),
                        }
                    }
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        }
    }
}
