#[cfg(test)]
mod tests {

    use cosmwasm_std::{
        from_slice,
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
        to_binary, Addr, ContractInfo, ContractResult, Empty, OwnedDeps, Querier, QuerierResult,
        QueryRequest, Response, SystemError, SystemResult, Uint128, Uint256, WasmQuery,
    };

    use shade_protocol::contract_interfaces::liquidity_book::lb_pair::{
        ExecuteMsg, InstantiateMsg, LiquidityParameters, RemoveLiquidity,
    };
    use shade_protocol::lb_libraries::{
        tokens::TokenType,
        types::{ContractInstantiationInfo, StaticFeeParameters},
    };

    use crate::{
        contract::{execute, instantiate},
        msg::TotalSupplyResponse,
    };
    use crate::{error::LBPairError, msg::LbTokenQueryMsg};
    pub fn init_helper() -> (
        Result<Response, LBPairError>,
        OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
    ) {
        let mut deps = mock_dependencies();
        //     let mut deps = mock_dependencies();
        let env = mock_env();

        let mock_message_info = mock_info("", &[]);

        let init_msg = InstantiateMsg {
            factory: ContractInfo {
                address: Addr::unchecked("factory_address"),
                code_hash: "factory_code_hash".to_string(),
            },
            token_x: TokenType::CustomToken {
                contract_addr: Addr::unchecked("token_x_address"),
                token_code_hash: "token_x_code_hash".to_string(),
            },
            token_y: TokenType::CustomToken {
                contract_addr: Addr::unchecked("token_y_address"),
                token_code_hash: "token_y_code_hash".to_string(),
            },
            bin_step: 100,
            pair_parameters: StaticFeeParameters {
                base_factor: 5000,
                filter_period: 30,
                decay_period: 600,
                reduction_factor: 5000,
                variable_fee_control: 40000,
                protocol_share: 1000,
                max_volatility_accumulator: 350000,
            },
            active_id: 8388608,
            lb_token_implementation: ContractInstantiationInfo {
                id: 8388608,
                code_hash: "lb_token_code_hash".to_string(),
            },
            viewing_key: "viewing_key".to_string(),
            pair_name: String::new(),
            entropy: String::new(),
        };

        // uint16 internal constant DEFAULT_BIN_STEP = 10;
        // uint16 internal constant DEFAULT_BASE_FACTOR = 5_000;
        // uint16 internal constant DEFAULT_FILTER_PERIOD = 30;
        // uint16 internal constant DEFAULT_DECAY_PERIOD = 600;
        // uint16 internal constant DEFAULT_REDUCTION_FACTOR = 5_000;
        // uint24 internal constant DEFAULT_VARIABLE_FEE_CONTROL = 40_000;
        // uint16 internal constant DEFAULT_PROTOCOL_SHARE = 1_000;
        // uint24 internal constant DEFAULT_MAX_VOLATILITY_ACCUMULATOR = 350_000;
        // bool internal constant DEFAULT_OPEN_STATE = false;

        let init_results = instantiate(deps.as_mut(), env, mock_message_info, init_msg);
        (init_results, deps)
    }
    #[test]
    fn test_add_liquidity() {
        let (init_response, mut deps) = init_helper();
        deps.querier = custom_querier();

        let array: Vec<f64> = vec![
            0.16666666, 0.16666666, 0.16666666, 0.16666666, 0.16666666, 0.16666666, 0.0, 0.0, 0.0,
            0.0, 0.0,
        ];
        let distribution_y: Vec<u64> = array.into_iter().map(|el| (el * 1e18) as u64).collect();

        let array: Vec<f64> = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.16666666, 0.16666666, 0.16666666, 0.16666666, 0.16666666,
            0.16666666,
        ];
        let distribution_x: Vec<u64> = array.into_iter().map(|el| (el * 1e18) as u64).collect();

        let liquidity_parameters = LiquidityParameters {
            token_x: TokenType::CustomToken {
                contract_addr: Addr::unchecked("token_x_address"),
                token_code_hash: "token_x_code_hash".to_string(),
            },
            token_y: TokenType::CustomToken {
                contract_addr: Addr::unchecked("token_y_address"),
                token_code_hash: "token_y_code_hash".to_string(),
            },
            bin_step: 100,
            amount_x: Uint128::from(100 * 1000000u128),
            amount_y: Uint128::from(100 * 1000000u128),
            amount_x_min: Uint128::from(90 * 1000000u128),
            amount_y_min: Uint128::from(90 * 1000000u128),
            active_id_desired: 8388608,
            id_slippage: 15,
            delta_ids: [-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5].into(),
            distribution_x,
            distribution_y,
            deadline: 9999999999,
        };

        let info = mock_info("HASEEB", &[]);
        let env = mock_env();
        let msg = ExecuteMsg::AddLiquidity {
            liquidity_parameters: liquidity_parameters.clone(),
        };

        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        let info = mock_info("HASEEB", &[]);
        let env = mock_env();
        let msg = ExecuteMsg::AddLiquidity {
            liquidity_parameters,
        };

        let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    }

    #[test]
    fn test_remove_liquidity() {
        let (init_response, mut deps) = init_helper();

        deps.querier = custom_querier();

        let array: Vec<f64> = vec![
            0.181818, 0.181818, 0.181818, 0.181818, 0.181818, 0.090909, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];
        let distribution_y: Vec<u64> = array.into_iter().map(|el| (el * 1e18) as u64).collect();

        let array: Vec<f64> = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.090909, 0.181818, 0.181818, 0.181818, 0.181818, 0.181818,
        ];
        let distribution_x: Vec<u64> = array.into_iter().map(|el| (el * 1e18) as u64).collect();
        let token_x = TokenType::CustomToken {
            contract_addr: Addr::unchecked("token_x_address"),
            token_code_hash: "token_x_code_hash".to_string(),
        };

        let token_y = TokenType::CustomToken {
            contract_addr: Addr::unchecked("token_y_address"),
            token_code_hash: "token_y_code_hash".to_string(),
        };
        let bin_step = 100u16;

        let liquidity_parameters = LiquidityParameters {
            token_x: token_x.clone(),
            token_y: token_y.clone(),
            bin_step,
            amount_x: Uint128::from(1000000000000000000u128),
            amount_y: Uint128::from(1000000000000000000u128),
            amount_x_min: Uint128::from(500000000000000000u128),
            amount_y_min: Uint128::from(500000000000000000u128),
            active_id_desired: 8388608,
            id_slippage: 15,
            delta_ids: [-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5].into(),
            distribution_x,
            distribution_y,
            deadline: 9999999999,
        };

        let info = mock_info("HASEEB", &[]);
        let env = mock_env();
        let msg = ExecuteMsg::AddLiquidity {
            liquidity_parameters,
        };

        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = RemoveLiquidity {
            token_x,
            token_y,
            bin_step,
            amount_x_min: Uint128::from(9500000u128),
            amount_y_min: Uint128::from(9500000u128),
            ids: [8388608].into(),
            amounts: [Uint256::from(3186945938883118954998384437402923u128)].into(),
            deadline: 9999999999,
        };
        let _res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::RemoveLiquidity {
                remove_liquidity_params: msg,
            },
        )
        .unwrap();
    }

    pub fn custom_querier() -> MockQuerier {
        let contract_addr = Addr::unchecked("lb_token");
        let custom_querier: MockQuerier = MockQuerier::new(&[(&contract_addr.as_str(), &[])])
            .with_custom_handler(|query| {
                SystemResult::Ok(cosmwasm_std::ContractResult::Ok(
                    to_binary(&TotalSupplyResponse {
                        total_supply: Uint256::from(6186945938883118954998384437402923u128),
                    })
                    .unwrap(),
                ))
            });

        custom_querier
    }

    struct MyCustomQuerier;

    impl Querier for MyCustomQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request: QueryRequest<Empty> = match from_slice(bin_request) {
                Ok(v) => v,
                Err(e) => {
                    return SystemResult::Err(SystemError::InvalidRequest {
                        error: format!("Parsing query request: {}", e),
                        request: bin_request.into(),
                    });
                }
            };

            match &request {
                QueryRequest::Wasm(WasmQuery::Smart { msg, .. }) => {
                    if msg == &to_binary(&LbTokenQueryMsg::TotalSupply { id: 0 }).unwrap() {
                        return SystemResult::Ok(ContractResult::Ok(
                            to_binary(&TotalSupplyResponse {
                                total_supply: Uint256::from(6186945938883118954998384437402923u128),
                            })
                            .unwrap(),
                        ));
                    }
                    SystemResult::Err(SystemError::UnsupportedRequest {
                        kind: "unhandled query".to_string(),
                    })
                }
                _ => SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: "unhandled query".to_string(),
                }),
            }
        }
    }

    #[test]
    fn test_swap() {
        let (init_response, mut deps) = init_helper();

        let array: Vec<f64> = vec![
            0.181818, 0.181818, 0.181818, 0.181818, 0.181818, 0.090909, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];
        let distribution_y: Vec<u64> = array.into_iter().map(|el| (el * 1e18) as u64).collect();

        let array: Vec<f64> = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.090909, 0.181818, 0.181818, 0.181818, 0.181818, 0.181818,
        ];
        let distribution_x: Vec<u64> = array.into_iter().map(|el| (el * 1e18) as u64).collect();
        let token_x = TokenType::CustomToken {
            contract_addr: Addr::unchecked("token_x_address"),
            token_code_hash: "token_x_code_hash".to_string(),
        };

        let token_y = TokenType::CustomToken {
            contract_addr: Addr::unchecked("token_y_address"),
            token_code_hash: "token_y_code_hash".to_string(),
        };
        let bin_step = 100u16;

        let liquidity_parameters = LiquidityParameters {
            token_x: token_x.clone(),
            token_y: token_y.clone(),
            bin_step,
            amount_x: Uint128::from(1000000000000000000u128),
            amount_y: Uint128::from(1000000000000000000u128),
            amount_x_min: Uint128::from(500000000000000000u128),
            amount_y_min: Uint128::from(500000000000000000u128),
            active_id_desired: 8388608,
            id_slippage: 15,
            delta_ids: [-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5].into(),
            distribution_x,
            distribution_y,
            deadline: 9999999999,
        };

        let info = mock_info("HASEEB", &[]);
        let env = mock_env();
        let msg = ExecuteMsg::AddLiquidity {
            liquidity_parameters,
        };

        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let _res = execute(
            deps.as_mut(),
            env,
            info.clone(),
            ExecuteMsg::Swap {
                swap_for_y: true,
                to: info.sender,
                amount_received: Uint128::from(9999990000000u128),
            },
        )
        .unwrap();
    }
}
