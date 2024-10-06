#[cfg(test)]
mod tests {
    use std::{
        ops::{Add, Sub},
        vec,
    };

    use c_std::{
        coins, from_binary,
        testing::{mock_dependencies_with_balance, MOCK_CONTRACT_ADDR},
        Addr, Api, BlockInfo, ContractInfo, Env, StdResult, Timestamp, TransactionInfo, Uint128,
    };
    use s_toolkit::utils::types::Contract;
    use shade_protocol::{c_std, s_toolkit};

    use crate::{
        contract::execute,
        error::{self, ContractError},
        msg::{
            AddContract, ExecuteMsg, InstantiateMsg, MintingScheduleUint,
            QueryAnswer::{self, ContractInfoResponse},
            QueryMsg, WeightUpdate,
        },
        state::{ScheduleUnit, CONFIG, SUPPLY_POOL},
    };

    use crate::contract::{instantiate, query};

    use c_std::testing::mock_info;

    const OWNER: &str = "admin0000000001";
    const DENOM: &str = "uscrt";

    /// Returns a default enviroment with height, time, chain_id, and contract address
    /// You can submit as is to most contracts, or modify height/time if you want to
    /// test for expiration.
    ///
    /// This is intended for use in test code only.
    pub fn custom_mock_env(
        height: Option<u64>,
        time: Option<u64>,
        chain_id: Option<&str>,
        transaction_index: Option<u32>,
        contract_address: Option<&str>,
    ) -> Env {
        Env {
            block: BlockInfo {
                height: height.unwrap_or_default(),
                time: Timestamp::from_seconds(time.unwrap_or_default()),
                chain_id: chain_id.unwrap_or_default().to_string(),
                random: None,
            },
            transaction: Some(TransactionInfo {
                index: transaction_index.unwrap_or_default(),
                hash: String::new(),
            }),
            contract: ContractInfo {
                address: Addr::unchecked(contract_address.unwrap_or(MOCK_CONTRACT_ADDR)),
                code_hash: "".to_string(),
            },
        }
    }

    #[test]
    fn test_instantiate_works() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),
            season_ending_block: 0,
            grand_prize_contract: None,
        };
        let message_info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));

        let _res = instantiate(deps.as_mut(), env.clone(), message_info.clone(), msg).unwrap();

        let query_msg = QueryMsg::ContractInfo {};
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap())?;

        match res {
            ContractInfoResponse { info } => {
                assert_eq!(info.admins, vec![OWNER]);
                assert_eq!(&info.contract_address.to_string(), "exp_contract");
                assert_eq!(info.minting_schedule, []);
                assert_eq!(info.total_weight, 0);
                assert_eq!(info.season_total_xp_cap, Uint128::zero());
            }
            _ => {}
        }

        Ok(())
    }

    #[test]
    fn test_add_admin() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let config = CONFIG.load(&deps.storage)?;
        assert_eq!(config.admins.len(), 1);

        // Test Case: Trying with non-admin address
        let info = mock_info("NOT-OWNER", &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let msg = ExecuteMsg::AddAdmin {
            address: Addr::unchecked(String::from("admin2")),
        };

        let res = execute(deps.as_mut(), env, info, msg);

        assert_eq!(
            res.unwrap_err(),
            error::ContractError::CustomError {
                val: format!("Not an admin: {}", "NOT-OWNER"),
            }
        );

        // Test Case: Executing with admin address
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let msg = ExecuteMsg::AddAdmin {
            address: Addr::unchecked(String::from("admin2")),
        };

        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        let config = CONFIG.load(&deps.storage)?;
        assert_eq!(config.admins.len(), 2);

        Ok(())
    }

    #[test]
    fn test_remove_admin() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![
                Addr::unchecked(OWNER.to_owned()),
                Addr::unchecked("admin2".to_owned()),
            ]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let config = CONFIG.load(&deps.storage)?;
        assert_eq!(config.admins.len(), 2);

        // Test Case: Trying with non-admin address
        let info = mock_info("NOT-OWNER", &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let msg = ExecuteMsg::RemoveAdmin {
            address: Addr::unchecked(String::from("admin2")),
        };

        let res = execute(deps.as_mut(), env, info, msg);

        assert_eq!(
            res.unwrap_err(),
            error::ContractError::CustomError {
                val: format!("Not an admin: {}", "NOT-OWNER"),
            }
        );

        // Test Case: Executing with admin address
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let msg = ExecuteMsg::RemoveAdmin {
            address: Addr::unchecked(String::from("admin2")),
        };

        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        let config = CONFIG.load(&deps.storage)?;
        assert_eq!(config.admins.len(), 1);

        // Test Case: Attempt to remove the last admin
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let msg = ExecuteMsg::RemoveAdmin {
            address: Addr::unchecked(String::from(OWNER)),
        };

        let res = execute(deps.as_mut(), env, info, msg);

        assert_eq!(
            res.unwrap_err(),
            error::ContractError::CustomError {
                val: "Cannot remove the last admin".to_string(),
            }
        );

        // Test Case: Trying with an invalid address
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let msg = ExecuteMsg::RemoveAdmin {
            address: Addr::unchecked(String::from("**")),
        };

        let res = execute(deps.as_mut(), env, info, msg);

        assert_eq!(
            res.unwrap_err(),
            error::ContractError::CustomError {
                val: format!("Address not found in admins: {}", "**"),
            }
        );

        Ok(())
    }

    #[test]
    fn test_add_contract() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };

        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let schedule = [ScheduleUnit {
            end_block: env.block.height.add(100),
            mint_per_block: Uint128::one(),
            duration: 100,
            start_block: env.block.height,
            start_after: None,
        }];

        let sch = [MintingScheduleUint {
            duration: 100,
            mint_per_block: Uint128::one(),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };

        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::AddContract {
            contracts: [
                AddContract {
                    address: Addr::unchecked("pool1".to_string()),
                    code_hash: "pool1 hash".to_string(),
                    weight: 60,
                },
                AddContract {
                    address: Addr::unchecked("pool2".to_string()),
                    code_hash: "pool2 hash".to_string(),
                    weight: 20,
                },
            ]
            .to_vec(),
        };

        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        //adding contract at block height h
        let msg = ExecuteMsg::AddContract {
            contracts: [AddContract {
                address: Addr::unchecked("pool3".to_string()),
                code_hash: "pool3 hash".to_string(),
                weight: 20,
            }]
            .to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Get contract info query
        let query_msg = QueryMsg::VerifiedContracts {
            start_page: None,
            page_size: None,
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        if let crate::msg::QueryAnswer::VerifiedContractsResponse { contracts } = res {
            assert_eq!(contracts[0].address, "pool1".to_string());
            assert_eq!(contracts[0].available_xp, Uint128::zero());
            assert_eq!(contracts[0].weight, 60);
            assert_eq!(contracts[0].last_claimed, env.block.height);
            assert_eq!(contracts[1].address, "pool2".to_string());
            assert_eq!(contracts[1].available_xp, Uint128::zero());
            assert_eq!(contracts[1].weight, 20);
            assert_eq!(contracts[1].last_claimed, env.block.height);
            assert_eq!(contracts[2].address, "pool3".to_string());
            assert_eq!(contracts[2].available_xp, Uint128::zero());
            assert_eq!(contracts[2].weight, 20);
            assert_eq!(contracts[2].last_claimed, env.block.height);
        }

        //adding contract at block height h + 100
        //setting block to block + 100
        let env = custom_mock_env(
            Some(env.block.height.add(100)),
            None,
            None,
            None,
            Some("exp_contract"),
        );

        let msg = ExecuteMsg::AddContract {
            contracts: [AddContract {
                address: Addr::unchecked("pool4".to_string()),
                code_hash: "pool4 hash".to_string(),
                weight: 100,
            }]
            .to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Get contract info query
        let query_msg = QueryMsg::VerifiedContracts {
            start_page: None,
            page_size: None,
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        if let crate::msg::QueryAnswer::VerifiedContractsResponse { contracts } = res {
            assert_eq!(contracts[0].address, "pool1".to_string());
            assert_eq!(contracts[0].available_xp, Uint128::from(60u128));
            assert_eq!(contracts[0].weight, 60);
            assert_eq!(contracts[0].last_claimed, env.block.height);
            assert_eq!(contracts[1].address, "pool2".to_string());
            assert_eq!(contracts[1].available_xp, Uint128::from(20u128));
            assert_eq!(contracts[1].weight, 20);
            assert_eq!(contracts[1].last_claimed, env.block.height);
            assert_eq!(contracts[2].address, "pool3".to_string());
            assert_eq!(contracts[2].available_xp, Uint128::from(20u128));
            assert_eq!(contracts[2].weight, 20);
            assert_eq!(contracts[2].last_claimed, env.block.height);
            assert_eq!(contracts[3].address, "pool4".to_string());
            assert_eq!(contracts[3].available_xp, Uint128::from(0u128));
            assert_eq!(contracts[3].weight, 100);
            assert_eq!(contracts[3].last_claimed, env.block.height);
        }

        let query_msg = QueryMsg::ContractInfo {};
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            ContractInfoResponse { info } => {
                assert_eq!(info.admins, vec![OWNER]);
                assert_eq!(&info.contract_address.to_string(), "exp_contract");
                assert_eq!(info.minting_schedule, schedule);
                assert_eq!(info.total_weight, 200);
            }
            _ => {}
        }
        Ok(())
    }

    #[test]
    fn test_remove_contract() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };
        let info = mock_info(OWNER, &[]);
        let mut env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _exp_obj = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Set schedule

        let schedule = [ScheduleUnit {
            end_block: env.block.height.add(100),
            mint_per_block: Uint128::one(),
            duration: 100,
            start_block: env.block.height,
            start_after: None,
        }];

        let sch = [MintingScheduleUint {
            duration: 100,
            mint_per_block: Uint128::one(),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Set weights

        let msg = ExecuteMsg::AddContract {
            contracts: [
                AddContract {
                    address: Addr::unchecked("pool1".to_string()),
                    code_hash: "pool1 hash".to_string(),
                    weight: 60,
                },
                AddContract {
                    address: Addr::unchecked("pool2".to_string()),
                    code_hash: "pool2 hash".to_string(),
                    weight: 40,
                },
            ]
            .to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Advance block height
        env.block.height += 100;

        // Remove contract
        let msg = ExecuteMsg::RemoveContract {
            contracts: [Addr::unchecked("pool2".to_string())].to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Check verified contracts query
        let query_msg = QueryMsg::VerifiedContracts {
            start_page: None,
            page_size: None,
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::VerifiedContractsResponse { contracts } => {
                assert_eq!(contracts.len(), 2);
                assert_eq!(contracts[0].address, "pool1".to_string());
                assert_eq!(contracts[0].available_xp, Uint128::from(60u128));
                assert_eq!(contracts[0].weight, 60);
                assert_eq!(contracts[0].last_claimed, env.block.height);
                assert_eq!(contracts[1].address, "pool2".to_string());
                assert_eq!(contracts[1].available_xp, Uint128::from(40u128));
                assert_eq!(contracts[1].weight, 0);
                assert_eq!(contracts[1].last_claimed, env.block.height);
            }
            _ => {}
        }

        // Check contract info query
        let query_msg = QueryMsg::ContractInfo {};
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            ContractInfoResponse { info } => {
                assert_eq!(info.admins, vec![OWNER]);
                assert_eq!(&info.contract_address.to_string(), "exp_contract");
                assert_eq!(info.minting_schedule, schedule);
                assert_eq!(info.total_weight, 60);
            }
            _ => {}
        }
        Ok(())
    }

    #[test]
    fn test_set_grand_prize_contract() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let config = CONFIG.load(&deps.storage)?;
        assert_eq!(config.grand_prize_contract, None);

        // Trying with non-admin address
        let info = mock_info("NOT-OWNER", &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let msg = ExecuteMsg::SetGrandPrizeContract {
            address: Addr::unchecked(String::from("grand_prize_contract")),
        };

        let res = execute(deps.as_mut(), env, info, msg);

        assert_eq!(
            res.unwrap_err(),
            error::ContractError::CustomError {
                val: format!("Not an admin: {}", "NOT-OWNER"),
            }
        );

        // Executing with admin address
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let msg = ExecuteMsg::SetGrandPrizeContract {
            address: Addr::unchecked(String::from("grand_prize_contract")),
        };

        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        let config = CONFIG.load(&deps.storage)?;
        assert_eq!(
            config.grand_prize_contract.unwrap().to_string(),
            String::from("grand_prize_contract")
        );

        // Trying with an invalid address
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let msg = ExecuteMsg::RemoveAdmin {
            address: Addr::unchecked(String::from("**")),
        };

        let res = execute(deps.as_mut(), env, info, msg);

        assert_eq!(
            res.unwrap_err(),
            error::ContractError::CustomError {
                val: format!("Address not found in admins: {}", "**"),
            }
        );
        Ok(())
    }

    #[test]
    fn test_set_schedule() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };

        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::AddContract {
            contracts: [
                AddContract {
                    address: Addr::unchecked("pool1".to_string()),
                    code_hash: "pool1 hash".to_string(),
                    weight: 60,
                },
                AddContract {
                    address: Addr::unchecked("pool2".to_string()),
                    code_hash: "pool2 hash".to_string(),
                    weight: 40,
                },
            ]
            .to_vec(),
        };

        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let mut duration = 100u64;
        let sch = [MintingScheduleUint {
            duration,
            mint_per_block: Uint128::one(),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let info = mock_info("Not-Owner", &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());

        if let Err(err) = _res {
            assert_eq!(
                err,
                ContractError::CustomError {
                    val: format!("Not an admin: Not-Owner"),
                }
            )
        }

        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        let schdule = vec![ScheduleUnit {
            end_block: env.block.height.add(duration),
            mint_per_block: Uint128::one(),
            duration,
            start_block: env.block.height,
            start_after: None,
        }];
        let query_msg = QueryMsg::ContractInfo {};
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        match res {
            ContractInfoResponse { info } => {
                assert_eq!(info.admins, vec![OWNER]);
                assert_eq!(&info.contract_address.to_string(), "exp_contract");
                assert_eq!(info.minting_schedule, schdule);
                assert_eq!(info.season_count, 1);
                assert_eq!(info.season_duration, 100);
                assert_eq!(info.season_starting_block, env.block.height);
                assert_eq!(info.season_ending_block, env.block.height.add(100));
                assert_eq!(info.total_weight, 100);
                assert_eq!(
                    info.season_total_xp_cap,
                    Uint128::from(duration as u128 * 1u128)
                );
            }
            _ => {}
        }

        //ability to extend or reduce the size of season.
        //extend season
        duration = 1000;
        let sch = [MintingScheduleUint {
            duration,
            mint_per_block: Uint128::one(),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());

        let query_msg = QueryMsg::ContractInfo {};
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        let schdule = vec![ScheduleUnit {
            end_block: env.block.height.add(duration),
            mint_per_block: Uint128::one(),
            duration,
            start_block: env.block.height,
            start_after: None,
        }];
        match res {
            ContractInfoResponse { info } => {
                assert_eq!(info.minting_schedule, schdule);
                assert_eq!(info.season_count, 1);
                assert_eq!(info.season_duration, duration);
                assert_eq!(info.season_starting_block, env.block.height);
                assert_eq!(info.season_ending_block, env.block.height.add(duration));
                assert_eq!(info.total_weight, 100);
                assert_eq!(
                    info.season_total_xp_cap,
                    Uint128::from(duration as u128 * 1u128)
                );
            }
            _ => {}
        }

        //reduce season
        duration = 10;
        let sch = [MintingScheduleUint {
            duration,
            mint_per_block: Uint128::one(),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let info = mock_info(OWNER, &[]);
        let mut env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());

        let query_msg = QueryMsg::ContractInfo {};
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        let schdule = vec![ScheduleUnit {
            end_block: env.block.height.add(duration),
            mint_per_block: Uint128::one(),
            duration,
            start_block: env.block.height,
            start_after: None,
        }];
        match res {
            ContractInfoResponse { info } => {
                assert_eq!(info.minting_schedule, schdule);
                assert_eq!(info.season_count, 1);
                assert_eq!(info.season_duration, duration);
                assert_eq!(info.season_starting_block, env.block.height);
                assert_eq!(info.season_ending_block, env.block.height.add(duration));
                assert_eq!(
                    info.season_total_xp_cap,
                    Uint128::from(duration as u128 * 1u128)
                );
                assert_eq!(info.total_weight, 100);
            }
            _ => {}
        }
        env.block.height = env.block.height.add(duration);

        duration = 1000;
        let sch = [MintingScheduleUint {
            duration,
            mint_per_block: Uint128::one(),
            continue_with_current_season: false,
            start_after: None,
        }];
        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let info = mock_info(OWNER, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        let query_msg = QueryMsg::ContractInfo {};
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        let schdule = vec![ScheduleUnit {
            end_block: env.block.height.add(duration),
            mint_per_block: Uint128::one(),
            duration,
            start_block: env.block.height,
            start_after: None,
        }];
        let old_duration: u64 = 10;
        match res {
            ContractInfoResponse { info } => {
                assert_eq!(info.minting_schedule, schdule);
                assert_eq!(info.season_count, 1);
                assert_eq!(info.season_duration, duration.add(old_duration));
                assert_eq!(
                    info.season_starting_block,
                    env.block.height - (old_duration)
                );
                assert_eq!(info.season_ending_block, env.block.height.add(duration));
                assert_eq!(
                    info.season_total_xp_cap,
                    Uint128::from(duration as u128 * 1u128)
                        .add(Uint128::from(old_duration as u128 * 1u128))
                );
                assert_eq!(info.total_weight, 100);
            }
            _ => {}
        }

        Ok(())
    }

    #[test]
    fn test_update_weight() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::AddContract {
            contracts: [
                AddContract {
                    address: Addr::unchecked("pool1".to_string()),
                    code_hash: "pool1 hash".to_string(),
                    weight: 60,
                },
                AddContract {
                    address: Addr::unchecked("pool2".to_string()),
                    code_hash: "pool2 hash".to_string(),
                    weight: 40,
                },
            ]
            .to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let query_msg = QueryMsg::VerifiedContracts {
            start_page: None,
            page_size: None,
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        match res {
            crate::msg::QueryAnswer::VerifiedContractsResponse { contracts } => {
                assert_eq!(contracts[0].address, "pool1".to_string());
                assert_eq!(contracts[0].available_xp, Uint128::zero());
                assert_eq!(contracts[0].weight, 60);
                assert_eq!(contracts[0].last_claimed, env.block.height);
                assert_eq!(contracts[1].address, "pool2".to_string());
                assert_eq!(contracts[1].available_xp, Uint128::zero());
                assert_eq!(contracts[1].weight, 40);
                assert_eq!(contracts[1].last_claimed, env.block.height);
            }
            _ => {}
        }

        let query_msg = QueryMsg::ContractInfo {};
        let res: QueryAnswer = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap())?;

        if let QueryAnswer::ContractInfoResponse { info } = res {
            assert_eq!(info.total_weight, 100); // Updated total weight after setting pool1 weight to zero
        }

        //Updating contracts

        //updating without adding
        let w = [WeightUpdate {
            address: Addr::unchecked("pool3".to_string()),
            weight: 60,
        }];
        let msg = ExecuteMsg::UpdateWeights {
            weights: w.to_vec(),
        };
        let error_res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        if let Err(err) = error_res {
            assert_eq!(
                err,
                ContractError::CustomError {
                    val: format!(
                        "Contract address pool3 is not a a verified contract. Add contract first",
                    ),
                }
            )
        }

        //updating

        // Updating the weight to zero
        let w = [WeightUpdate {
            address: Addr::unchecked("pool1".to_string()),
            weight: 0,
        }];
        let msg = ExecuteMsg::UpdateWeights {
            weights: w.to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        //Total weight
        let query_msg = QueryMsg::ContractInfo {};
        let res: QueryAnswer = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap())?;

        if let QueryAnswer::ContractInfoResponse { info } = res {
            assert_eq!(info.total_weight, 40); // Updated total weight after setting pool1 weight to zero
        }

        let query_msg = QueryMsg::VerifiedContracts {
            start_page: None,
            page_size: None,
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::VerifiedContractsResponse { contracts } => {
                assert_eq!(contracts[0].address, "pool1".to_string());
                assert_eq!(contracts[0].weight, 0); // Updated weight for pool1 (zero)
                assert_eq!(contracts[1].address, "pool2".to_string());
                assert_eq!(contracts[1].weight, 40);
            }
            _ => {}
        }

        Ok(())
    }

    #[test]
    fn test_add_exp() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);
        let info = mock_info(OWNER, &[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };

        let mut env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _exp_obj = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let sch = [MintingScheduleUint {
            duration: 100,
            mint_per_block: Uint128::one(),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::AddContract {
            contracts: [
                AddContract {
                    address: Addr::unchecked("pool1".to_string()),
                    code_hash: "pool1 hash".to_string(),
                    weight: 60,
                },
                AddContract {
                    address: Addr::unchecked("pool2".to_string()),
                    code_hash: "pool2 hash".to_string(),
                    weight: 40,
                },
            ]
            .to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        env.block.height += 100;

        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("pool1", &coins(0, DENOM)),
            msg,
        )
        .unwrap();

        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_1".to_string(),
        };
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("pool1", &coins(0, DENOM)),
            msg,
        )
        .unwrap();

        let contract_exp_obj = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::Contract {
                address: Addr::unchecked("pool1".to_string()),
                key: "vk_1".to_string(),
            },
        )
        .unwrap();
        let res = from_binary(&contract_exp_obj).unwrap();

        if let crate::msg::QueryAnswer::ContractResponse {
            available_exp,
            unclaimed_exp,
            ..
        } = res
        {
            assert_eq!(available_exp, Uint128::from(60u128));
            assert_eq!(unclaimed_exp, Uint128::zero());
        }

        let msg = ExecuteMsg::AddExp {
            address: Addr::unchecked("user1".to_string()),
            exp: Uint128::from(20u128),
        };
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("pool1", &coins(0, DENOM)),
            msg,
        )
        .unwrap();

        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_1".to_string(),
        };
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("user1", &coins(0, DENOM)),
            msg,
        )
        .unwrap();
        let user_exp_obj = query(
            deps.as_ref(),
            env.clone(),
            crate::msg::QueryMsg::UserExp {
                address: Addr::unchecked("user1".to_string()),
                key: String::from("vk_1"),
                season: None,
            },
        )
        .unwrap();
        let query_answer = from_binary(&user_exp_obj).unwrap();

        if let crate::msg::QueryAnswer::UserExp { exp } = query_answer {
            assert_eq!(exp, Uint128::from(20u128));
        }
        Ok(())
    }

    #[test]
    fn test_update_last_claimed() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };
        let info = mock_info(OWNER, &[]);
        let mut env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _exp_obj = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Set schedule

        let sch = [MintingScheduleUint {
            duration: 100,
            mint_per_block: Uint128::one(),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Set weights

        let msg = ExecuteMsg::AddContract {
            contracts: [
                AddContract {
                    address: Addr::unchecked("pool1".to_string()),
                    code_hash: "pool1 hash".to_string(),
                    weight: 60,
                },
                AddContract {
                    address: Addr::unchecked("pool2".to_string()),
                    code_hash: "pool2 hash".to_string(),
                    weight: 40,
                },
            ]
            .to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Advance block height
        env.block.height += 100;

        // pool1 updates its last claimed
        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool1", &[]), msg).unwrap();

        // Set vk
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_1".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool1", &[]), msg).unwrap();

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(60u128));
                assert_eq!(unclaimed_exp, Uint128::zero());
            }
            _ => {}
        }
        Ok(())
    }

    #[test]
    fn test_get_xp() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);

        let init_msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };

        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        let msg = ExecuteMsg::AddContract {
            contracts: [
                AddContract {
                    address: Addr::unchecked("pool1".to_string()),
                    code_hash: "pool1 hash".to_string(),
                    weight: 50,
                },
                AddContract {
                    address: Addr::unchecked("pool2".to_string()),
                    code_hash: "pool2 hash".to_string(),
                    weight: 50,
                },
            ]
            .to_vec(),
        };

        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Scenario 1:

        // Schedule:
        // Schedule 1: start_block = 10, end_block = 20, mint_per_block = 2
        // Schedule 2: start_block = 25, end_block = 35, mint_per_block = 4
        // current_block = 15
        // last_claimed = 0
        // total_weight = 100
        // contract: weight = 50
        // Expected result: XP earned during Schedule 1
        let sch = [
            MintingScheduleUint {
                duration: 10,
                mint_per_block: Uint128::from(2u128),
                continue_with_current_season: false,
                start_after: Some(10u64),
            },
            MintingScheduleUint {
                duration: 10,
                mint_per_block: Uint128::from(4u128),
                continue_with_current_season: false,
                start_after: Some(25u64),
            },
        ];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        //Set vk
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_1".to_string(),
        };

        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None, Some("pool1")),
            mock_info("pool1", &[]),
            msg,
        )
        .unwrap();

        let env = custom_mock_env(Some(15u64), None, None, None, Some("exp_contract"));
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        match res {
            crate::msg::QueryAnswer::ContractResponse { unclaimed_exp, .. } => {
                assert_eq!(unclaimed_exp, Uint128::from(5u128)); // 10 / 2
            }
            _ => {}
        }

        // Scenario 2:
        // Schedule:
        // Schedule 1: start_block = 10, end_block = 20, mint_per_block = 2
        // Schedule 2: start_block = 25, end_block = 35, mint_per_block = 4
        // current_block = 30
        // last_claimed = 0
        // total_weight = 100
        // contract: weight = 50
        // Expected result: XP earned during Schedule 1 and a part of Schedule 2
        let env = custom_mock_env(Some(30u64), None, None, None, Some("exp_contract"));
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        match res {
            crate::msg::QueryAnswer::ContractResponse { unclaimed_exp, .. } => {
                assert_eq!(unclaimed_exp, Uint128::from(20u128)); // (20 + 20)/2
            }
            _ => {}
        }

        // Scenario 3:

        // Schedule:
        // Schedule 1: start_block = 10, end_block = 30, mint_per_block = 2
        // Schedule 2: start_block = 20, end_block = 40, mint_per_block = 4
        // current_block = 25
        // last_claimed = 0
        // total_weight = 100
        // contract: weight = 50
        // Expected result: XP earned during Schedule 1, and the overlapping part with Schedule 2
        let sch = [
            MintingScheduleUint {
                duration: 20,
                mint_per_block: Uint128::from(2u128),
                continue_with_current_season: false,
                start_after: Some(10u64),
            },
            MintingScheduleUint {
                duration: 20,
                mint_per_block: Uint128::from(4u128),
                continue_with_current_season: false,
                start_after: Some(20u64),
            },
        ];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        let env = custom_mock_env(Some(25u64), None, None, None, Some("exp_contract"));
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        match res {
            crate::msg::QueryAnswer::ContractResponse { unclaimed_exp, .. } => {
                assert_eq!(unclaimed_exp, Uint128::from(25u128)); // (30 + 20)/2
            }
            _ => {}
        }

        // Scenario 4:

        // Schedule:
        // Schedule 1: start_block = 10, end_block = 20, mint_per_block = 2
        // Schedule 2: start_block = 25, end_block = 35, mint_per_block = 4
        // current_block = 40
        // last_claimed = 0
        // total_weight = 100
        // contract: weight = 50
        // Expected result: XP earned during Schedule 1 and Schedule 2
        let sch = [
            MintingScheduleUint {
                duration: 10,
                mint_per_block: Uint128::from(2u128),
                continue_with_current_season: false,
                start_after: Some(10u64),
            },
            MintingScheduleUint {
                duration: 10,
                mint_per_block: Uint128::from(4u128),
                continue_with_current_season: false,
                start_after: Some(25u64),
            },
        ];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        let env = custom_mock_env(Some(40u64), None, None, None, Some("exp_contract"));
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        match res {
            crate::msg::QueryAnswer::ContractResponse { unclaimed_exp, .. } => {
                assert_eq!(unclaimed_exp, Uint128::from(30u128)); // (30 + 20)/2
            }
            _ => {}
        }

        // Scenario 5:

        // Schedule:
        // Schedule 1: start_block = 10, end_block = 30, mint_per_block = 2
        // Schedule 2: start_block = 20, end_block = 40, mint_per_block = 4
        // current_block = 45
        // last_claimed = 0
        // total_weight = 100
        // contract: weight = 50
        // Expected result: XP earned during Schedule 1 and Schedule 2
        let sch = [
            MintingScheduleUint {
                duration: 20,
                mint_per_block: Uint128::from(2u128),
                continue_with_current_season: false,
                start_after: Some(10u64),
            },
            MintingScheduleUint {
                duration: 20,
                mint_per_block: Uint128::from(4u128),
                continue_with_current_season: false,
                start_after: Some(20u64),
            },
        ];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        let env = custom_mock_env(Some(45u64), None, None, None, Some("exp_contract"));
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        match res {
            crate::msg::QueryAnswer::ContractResponse { unclaimed_exp, .. } => {
                assert_eq!(unclaimed_exp, Uint128::from(60u128)); // (30 + 20)/2
            }
            _ => {}
        }

        Ok(())
    }

    #[test]
    fn test_query_verified_contracts() -> StdResult<()> {
        // test_set_weight();
        Ok(())
    }

    #[test]
    fn season_walkthrough() -> StdResult<()> {
        // Initialized
        let mut deps = mock_dependencies_with_balance(&[]);
        let info = mock_info(OWNER, &[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };

        let mut env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _exp_obj = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        // Minting 1 xp/block.
        let sch = [MintingScheduleUint {
            duration: 100,
            mint_per_block: Uint128::from(100u128),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Adding a new contract pool1  @ b0 duration = b10, w = 50
        let msg = ExecuteMsg::AddContract {
            contracts: [AddContract {
                address: Addr::unchecked("pool1".to_string()),
                code_hash: "pool1 hash".to_string(),
                weight: 50,
            }]
            .to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Checking at b10. Claim some xp, assert xp claimed = 10 * 100.
        // Advance block height
        env.block.height += 10;

        // pool1 updates its last claimed
        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool1", &[]), msg).unwrap();
        let config = CONFIG.load(&deps.storage)?;
        let supply_pool = SUPPLY_POOL.load(&deps.storage, config.season_counter)?;
        assert_eq!(
            supply_pool.xp_claimed_by_contracts,
            Uint128::from(10 * 100u128)
        );

        // Set vk
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_1".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool1", &[]), msg).unwrap();

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(1000u128));
                assert_eq!(unclaimed_exp, Uint128::zero());
            }
            _ => {}
        }

        // Adding a new contract @ b11, duration = b10, w = 50
        env.block.height += 1;

        let msg = ExecuteMsg::AddContract {
            contracts: [AddContract {
                address: Addr::unchecked("pool2".to_string()),
                code_hash: "pool2 hash".to_string(),
                weight: 50,
            }]
            .to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let config = CONFIG.load(&deps.storage)?;
        assert_eq!(config.total_weight, 100u64);

        // Checking at b20. Claim some xp, assert xp claimed = 4.5 * 100, total xp claimed = 20
        // pool1 updates its last claimed
        env.block.height += 9;

        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool2", &[]), msg).unwrap();
        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool1", &[]), msg).unwrap();
        let supply_pool = SUPPLY_POOL.load(&deps.storage, config.season_counter)?;
        assert_eq!(
            supply_pool.xp_claimed_by_contracts,
            Uint128::from(20 * 100u128)
        );

        // Set vk
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_2".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool2", &[]), msg).unwrap();

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool2".to_string()),
            key: "vk_2".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(450u128));
                assert_eq!(unclaimed_exp, Uint128::zero());
            }
            _ => {}
        }

        // Simulating at b100. assert pool 1 xp = (11+ 44.5) , assert pool2 xp (44.5)
        env.block.height += 80;

        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool2".to_string()),
            key: "vk_2".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(450u128));
                assert_eq!(unclaimed_exp, Uint128::from(4000u128));
            }
            _ => {}
        }

        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool1", &[]), msg).unwrap();

        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool2", &[]), msg).unwrap();

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(5550u128));
                assert_eq!(unclaimed_exp, Uint128::zero());
            }
            _ => {}
        }

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool2".to_string()),
            key: "vk_2".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                last_claimed,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(4450u128));
                assert_eq!(unclaimed_exp, Uint128::zero());
                assert_eq!(last_claimed, env.block.height);
            }
            _ => {}
        }

        let supply_pool = SUPPLY_POOL.load(&deps.storage, config.season_counter)?;
        assert_eq!(
            supply_pool.season_total_xp_cap,
            supply_pool.xp_claimed_by_contracts
        );
        //Trying to add more xp after season ends. hence all stats remain constant.
        env.block.height += 10;
        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool1", &[]), msg).unwrap();

        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool2", &[]), msg).unwrap();

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(5550u128));
                assert_eq!(unclaimed_exp, Uint128::zero());
            }
            _ => {}
        }

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool2".to_string()),
            key: "vk_2".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                last_claimed,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(4450u128));
                assert_eq!(unclaimed_exp, Uint128::zero());
                assert_eq!(last_claimed, env.block.height.sub(10u64));
            }
            _ => {}
        }

        let supply_pool = SUPPLY_POOL.load(&deps.storage, config.season_counter)?;
        assert_eq!(
            supply_pool.season_total_xp_cap,
            supply_pool.xp_claimed_by_contracts
        );

        // Adding pool @ b0. Remove Pool 2 @ b20, assert xp claimed = 10 * 100, total xp claimed = 20
        let mut deps = mock_dependencies_with_balance(&[]);
        let info = mock_info(OWNER, &[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };

        let mut env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _exp_obj = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        // Minting 1 xp/block.
        let sch = [MintingScheduleUint {
            duration: 100,
            mint_per_block: Uint128::from(100u128),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Adding a new contract pool1  @ b0 duration = b10, w = 50
        let msg = ExecuteMsg::AddContract {
            contracts: [
                AddContract {
                    address: Addr::unchecked("pool1".to_string()),
                    code_hash: "pool1 hash".to_string(),
                    weight: 50,
                },
                AddContract {
                    address: Addr::unchecked("pool2".to_string()),
                    code_hash: "pool2 hash".to_string(),
                    weight: 50,
                },
            ]
            .to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Remove contract @ b20.
        env.block.height += 20;

        let msg = ExecuteMsg::RemoveContract {
            contracts: [Addr::unchecked("pool2".to_string())].to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        //Query verified contracts
        let query_msg = QueryMsg::VerifiedContracts {
            start_page: None,
            page_size: None,
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        // We're not removing contracts to avoid any errors on the pool contract sides
        match res {
            crate::msg::QueryAnswer::VerifiedContractsResponse { contracts } => {
                assert_eq!(contracts.len(), 2);
            }
            _ => {}
        }
        // Simulating at b100.

        env.block.height += 80;

        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool1", &[]), msg).unwrap();

        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool2", &[]), msg).unwrap();

        // Set vk
        // Set vk
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_1".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool1", &[]), msg).unwrap();
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_2".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool2", &[]), msg).unwrap();

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(9000u128));
                assert_eq!(unclaimed_exp, Uint128::zero());
            }
            _ => {}
        }

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool2".to_string()),
            key: "vk_2".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                last_claimed,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(1000u128));
                assert_eq!(unclaimed_exp, Uint128::zero());
                assert_eq!(last_claimed, env.block.height);
            }
            _ => {}
        }

        // Recap to b20. Update Pool 2 weightage to 10 and pool 1 weightage to 90
        let mut deps = mock_dependencies_with_balance(&[]);
        let info = mock_info(OWNER, &[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };

        let mut env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _exp_obj = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        // Minting 1 xp/block.
        let sch = [MintingScheduleUint {
            duration: 100,
            mint_per_block: Uint128::from(100u128),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Adding a new contract pool1  @ b0 duration = b10, w = 50
        let msg = ExecuteMsg::AddContract {
            contracts: [
                AddContract {
                    address: Addr::unchecked("pool1".to_string()),
                    code_hash: "pool1 hash".to_string(),
                    weight: 50,
                },
                AddContract {
                    address: Addr::unchecked("pool2".to_string()),
                    code_hash: "pool2 hash".to_string(),
                    weight: 50,
                },
            ]
            .to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        env.block.height += 20;

        // Simulating at b100. assert pool 1 xp = (91.1) , assert pool2 xp (8.9)

        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_1".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool1", &[]), msg).unwrap();
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_2".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool2", &[]), msg).unwrap();

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(0u128));
                assert_eq!(unclaimed_exp, Uint128::from(1000u128));
            }
            _ => {}
        }

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool2".to_string()),
            key: "vk_2".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(available_exp, Uint128::zero());
                assert_eq!(unclaimed_exp, Uint128::from(1000u128));
            }
            _ => {}
        }

        let msg = ExecuteMsg::UpdateWeights {
            weights: [
                WeightUpdate {
                    address: Addr::unchecked("pool1".to_string()),
                    weight: 90,
                },
                WeightUpdate {
                    address: Addr::unchecked("pool2".to_string()),
                    weight: 10,
                },
            ]
            .to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        env.block.height += 80;

        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool1", &[]), msg).unwrap();

        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool2", &[]), msg).unwrap();

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(8200u128));
                assert_eq!(unclaimed_exp, Uint128::zero());
            }
            _ => {}
        }

        // Query contract exp
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool2".to_string()),
            key: "vk_2".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                last_claimed,
                ..
            } => {
                assert_eq!(available_exp, Uint128::from(1800u128));
                assert_eq!(unclaimed_exp, Uint128::zero());
                assert_eq!(last_claimed, env.block.height);
            }
            _ => {}
        }

        Ok(())
    }

    #[test]
    fn test_query_contract_info() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);
        let info = mock_info(OWNER, &[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };

        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let contract_info_obj =
            query(deps.as_ref(), env.clone(), QueryMsg::ContractInfo {}).unwrap();
        let res = from_binary(&contract_info_obj).unwrap();
        if let crate::msg::QueryAnswer::ContractInfoResponse { info } = res {
            assert_eq!(info.admins, vec![OWNER]);
            assert_eq!(
                info.contract_address.to_string(),
                "exp_contract".to_string()
            );
            assert_eq!(info.minting_schedule, []);
            assert_eq!(info.total_weight, 0);
        }
        Ok(())
    }

    #[test]
    fn test_query_contract_exp_available() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _exp_obj = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let sch = [MintingScheduleUint {
            duration: 100,
            mint_per_block: Uint128::one(),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };

        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::AddContract {
            contracts: [
                AddContract {
                    address: Addr::unchecked("pool1".to_string()),
                    code_hash: "pool1 hash".to_string(),
                    weight: 60,
                },
                AddContract {
                    address: Addr::unchecked("pool2".to_string()),
                    code_hash: "pool2 hash".to_string(),
                    weight: 40,
                },
            ]
            .to_vec(),
        };

        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        //Set vk
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_1".to_string(),
        };

        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None, Some("pool1")),
            mock_info("pool1", &[]),
            msg,
        )
        .unwrap();

        // Get contract info query
        //querying at the same block as contract was added
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(available_exp, Uint128::zero());
                assert_eq!(unclaimed_exp, Uint128::zero());
            }
            _ => {}
        }

        //setting block to block + 100
        let new_env = custom_mock_env(
            Some(env.block.height.add(100)),
            None,
            None,
            None,
            Some("exp_contract"),
        );
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), new_env.clone(), query_msg).unwrap()).unwrap();

        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(unclaimed_exp, Uint128::from(60u128));
                assert_eq!(available_exp, Uint128::zero());
            }
            _ => {}
        }

        //Set vk
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_2".to_string(),
        };
        let _res = execute(deps.as_mut(), env, mock_info("pool2", &[]), msg).unwrap();

        // Get contract info query
        //querying at the same block as contract was added
        let query_msg = QueryMsg::Contract {
            address: Addr::unchecked("pool2".to_string()),
            key: "vk_2".to_string(),
        };
        let res = from_binary(&query(deps.as_ref(), new_env.clone(), query_msg).unwrap()).unwrap();

        match res {
            crate::msg::QueryAnswer::ContractResponse {
                available_exp,
                unclaimed_exp,
                ..
            } => {
                assert_eq!(unclaimed_exp, Uint128::from(40u128));
                assert_eq!(available_exp, Uint128::zero());
            }
            _ => {}
        }
        Ok(())
    }

    #[test]
    fn test_query_user_exp() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);
        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };

        let _exp_obj = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let sch = [MintingScheduleUint {
            duration: 100,
            mint_per_block: Uint128::one(),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::AddContract {
            contracts: [
                AddContract {
                    address: Addr::unchecked("pool1".to_string()),
                    code_hash: "pool1 hash".to_string(),
                    weight: 60,
                },
                AddContract {
                    address: Addr::unchecked("pool2".to_string()),
                    code_hash: "pool2 hash".to_string(),
                    weight: 20,
                },
            ]
            .to_vec(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        //pool1 updating it's last claimed
        //adding contract at block height h + 100
        let new_env = custom_mock_env(
            Some(env.block.height.add(100)),
            None,
            None,
            None,
            Some("pool1"),
        );
        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), new_env.clone(), mock_info("pool1", &[]), msg).unwrap();

        //Rewarding 20 exp to user1
        let msg = ExecuteMsg::AddExp {
            address: Addr::unchecked("user1".to_string()),
            exp: Uint128::from(20u128),
        };
        let _res = execute(deps.as_mut(), new_env.clone(), mock_info("pool1", &[]), msg).unwrap();

        //Get user exp
        //set vk
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_1".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), mock_info("user1", &[]), msg).unwrap();

        //querying at the block + 100
        let query_msg = QueryMsg::UserExp {
            address: Addr::unchecked("user1".to_string()),
            key: "vk_1".to_string(),
            season: None,
        };
        let user_exp_obj =
            from_binary(&query(deps.as_ref(), new_env.clone(), query_msg).unwrap()).unwrap();

        if let crate::msg::QueryAnswer::UserExp { exp } = user_exp_obj {
            assert_eq!(exp, Uint128::from(20u128));
        }
        Ok(())
    }

    #[test]
    fn test_query_user_exp_by_authoritize_address() -> StdResult<()> {
        let mut deps = mock_dependencies_with_balance(&[]);

        let msg = InstantiateMsg {
            // Your instantiate message fields here
            entropy: "LOL".to_string(),
            admin: Some(vec![Addr::unchecked(OWNER.to_owned())]),
            schedules: Vec::new(),

            grand_prize_contract: None,
            season_ending_block: 0,
        };

        let info = mock_info(OWNER, &[]);
        let env = custom_mock_env(None, None, None, None, Some("exp_contract"));
        let _exp_obj = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let sch = [MintingScheduleUint {
            duration: 100,
            mint_per_block: Uint128::one(),
            continue_with_current_season: false,
            start_after: None,
        }];

        let msg = ExecuteMsg::SetSchedule {
            schedule: sch.to_vec(),
        };

        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::AddContract {
            contracts: [
                AddContract {
                    address: Addr::unchecked("pool1".to_string()),
                    code_hash: "pool1 hash".to_string(),
                    weight: 60,
                },
                AddContract {
                    address: Addr::unchecked("pool2".to_string()),
                    code_hash: "pool2 hash".to_string(),
                    weight: 20,
                },
            ]
            .to_vec(),
        };

        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        //pool1 updating it's last claimed
        //adding contract at block height h + 100
        let new_env = custom_mock_env(
            Some(env.block.height.add(100)),
            None,
            None,
            None,
            Some("pool1"),
        );
        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = execute(deps.as_mut(), new_env.clone(), mock_info("pool1", &[]), msg).unwrap();

        //Rewarding 20 exp to user1
        let msg = ExecuteMsg::AddExp {
            address: Addr::unchecked("user1".to_string()),
            exp: Uint128::from(20u128),
        };
        let _res = execute(deps.as_mut(), new_env.clone(), mock_info("pool1", &[]), msg).unwrap();

        // Get user exp
        // set vk
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_1".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), mock_info("user1", &[]), msg).unwrap();

        // querying at the block + 100
        let query_msg = QueryMsg::UserExp {
            address: Addr::unchecked("user1".to_string()),
            key: "vk_1".to_string(),
            season: None,
        };
        let user_exp_obj: crate::msg::QueryAnswer =
            from_binary(&query(deps.as_ref(), new_env.clone(), query_msg).unwrap()).unwrap();

        if let crate::msg::QueryAnswer::UserExp { exp } = user_exp_obj {
            assert_eq!(exp, Uint128::from(20u128));
        }

        // Getting user exp by authoritized addresses
        // Get user exp
        // set vk
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_1".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), mock_info("pool1", &[]), msg).unwrap();

        //querying at the block + 100
        let query_msg = QueryMsg::CheckUserExp {
            address: Addr::unchecked("pool1".to_string()),
            key: "vk_1".to_string(),
            user_address: Addr::unchecked("user1".to_string()),
            season: None,
        };
        let user_exp_obj =
            from_binary(&query(deps.as_ref(), new_env.clone(), query_msg).unwrap()).unwrap();

        if let crate::msg::QueryAnswer::UserExp { exp } = user_exp_obj {
            assert_eq!(exp, Uint128::from(20u128));
        }
        Ok(())
    }
}
