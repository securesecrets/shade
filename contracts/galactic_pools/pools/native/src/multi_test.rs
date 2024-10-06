#[cfg(test)]
mod tests {

    use std::ops::Add;

    use c_std::{coins, Addr, Binary, BlockInfo, Coin, ContractInfo, Empty, StdResult, Uint128};

    use shade_protocol::{c_std, s_toolkit};

    use experience_contract::msg::AddContract;
    use shade_protocol::multi_test::{
        App,
        AppBuilder,
        Contract,
        ContractWrapper,
        Executor,
        StakingSudo,
        SudoMsg,
    };

    use crate::{
        msg::{ExpContract, InstantiateMsg, ValidatorInfo},
        state::{DistInfo, RewardsDistInfo},
    };

    const ADMIN: &str = "admin00001";
    const TRIGGERER: &str = "trigger00001";

    const SCRT_TO_USCRT: u128 = 1000000;

    pub fn pool_info() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    fn mock_app(init_funds: &[Coin]) -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(ADMIN), init_funds.to_vec())
                .unwrap();
        })
    }

    pub fn exp_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(
            experience_contract::contract::execute,
            experience_contract::contract::instantiate,
            experience_contract::contract::query,
        );
        Box::new(contract)
    }

    #[track_caller]
    fn instantiate_exp(app: &mut App, address: &String) -> StdResult<ContractInfo> {
        let flex_id = app.store_code(exp_contract());

        let msg = experience_contract::msg::InstantiateMsg {
            entropy: "entropy lol 123".to_string(),
            admin: Some(vec![Addr::unchecked(address.clone())]),
            schedules: vec![experience_contract::msg::MintingScheduleUint {
                mint_per_block: Uint128::from(1u128),
                duration: 1000,
                start_after: None,
                continue_with_current_season: false,
            }],

            season_ending_block: 0,
            grand_prize_contract: None,
        };
        Ok(app
            .instantiate_contract(flex_id, Addr::unchecked(ADMIN), &msg, &[], "exp", None)
            .unwrap())
    }

    fn add_validator(app: &mut App, validator: &str) -> StdResult<()> {
        let _res = app
            .sudo(SudoMsg::Staking(StakingSudo::AddValidator {
                validator: validator.to_string(),
            }))
            .unwrap();
        Ok(())
    }

    #[track_caller]
    fn instantiate_pool(
        mut app: &mut App,
        _contract_balance: Option<u128>,
        exp_contract_obj: &ContractInfo,
    ) -> StdResult<ContractInfo> {
        let contract_id = app.store_code(pool_info());
        let common_divisor = 10000u64;
        let mut validator_vector: Vec<ValidatorInfo> = Vec::new();
        validator_vector.push(ValidatorInfo {
            address: "galacticPools".to_string(),
            weightage: (60 * common_divisor) / 100,
        });
        validator_vector.push(ValidatorInfo {
            address: "secureSecret".to_string(),
            weightage: (20 * common_divisor) / 100,
        });
        validator_vector.push(ValidatorInfo {
            address: "xavierCapital".to_string(),
            weightage: (20 * common_divisor) / 100,
        });

        add_validator(&mut app, "galacticPools")?;
        add_validator(&mut app, "secureSecret")?;
        add_validator(&mut app, "xavierCapital")?;

        let rewards_distribution: RewardsDistInfo = RewardsDistInfo {
            tier_0: DistInfo {
                total_number_of_winners: Uint128::from(1u128),
                percentage_of_rewards: (20 * 10000) / 100,
            },
            tier_1: DistInfo {
                total_number_of_winners: Uint128::from(3u128),
                percentage_of_rewards: (10 * 10000) / 100,
            },
            tier_2: DistInfo {
                total_number_of_winners: Uint128::from(9u128),
                percentage_of_rewards: (14 * 10000) / 100,
            },
            tier_3: DistInfo {
                total_number_of_winners: Uint128::from(27u128),
                percentage_of_rewards: (12 * 10000) / 100,
            },
            tier_4: DistInfo {
                total_number_of_winners: Uint128::from(81u128),
                percentage_of_rewards: (19 * 10000) / 100,
            },
            tier_5: DistInfo {
                total_number_of_winners: Uint128::from(243u128),
                percentage_of_rewards: (25 * 10000) / 100,
            },
        };

        let init_msg = InstantiateMsg {
            admins: Option::from(vec![
                Addr::unchecked("admin".to_string()),
                Addr::unchecked(ADMIN.to_string()),
            ]),
            triggerers: Option::from(vec![Addr::unchecked(TRIGGERER.to_string())]),
            reviewers: Option::from(vec![Addr::unchecked("reviewer".to_string())]),
            triggerer_share_percentage: (1 * common_divisor) / 100, //dividing by 1/100 * common_divisor
            denom: "uscrt".to_string(),
            prng_seed: Binary::from("I'm secretbatman".as_bytes()),
            validator: validator_vector,
            unbonding_duration: 3600 * 24 * 21, //21 days
            round_duration: 3600 * 24 * 7,      //7 days
            rewards_distribution,
            ticket_price: Uint128::from(1 * SCRT_TO_USCRT),
            rewards_expiry_duration: 3888000, // 45 days
            common_divisor,
            total_admin_share: (10 * common_divisor) / 100,
            shade_percentage_share: (60 * common_divisor) / 100,
            galactic_pools_percentage_share: (40 * common_divisor) / 100,
            shade_rewards_address: Addr::unchecked(("shade".to_string())),
            galactic_pools_rewards_address: Addr::unchecked(("galactic_pools".to_string())),
            reserve_percentage: (60 * common_divisor) / 100,
            is_sponosorship_admin_controlled: false,
            unbonding_batch_duration: 3600 * 24 * 3,
            minimum_deposit_amount: None,
            grand_prize_address: Addr::unchecked("grand_prize".to_string()),
            /// setting number of number_of_tickers that can be run on txn send to avoid potential errors
            number_of_tickers_per_transaction: Uint128::from(1000000u128),
            sponsor_msg_edit_fee: Some(Uint128::from(1000000u128)),
            exp_contract: Some(ExpContract {
                contract: s_toolkit::utils::types::Contract {
                    address: exp_contract_obj.address.to_string(),
                    hash: exp_contract_obj.code_hash.clone(),
                },
                vk: "vk_1".to_string(),
            }),
        };

        Ok(app
            .instantiate_contract(
                contract_id,
                Addr::unchecked(ADMIN),
                &init_msg,
                &[],
                "GalacticPools",
                None,
            )
            .unwrap())
    }

    //1)Init pool contract

    #[test]
    fn test_initialize_exp_contract() -> StdResult<()> {
        let mut app = mock_app(&coins(100 * SCRT_TO_USCRT, "uscrt"));
        instantiate_exp(&mut app, &ADMIN.to_string())?;
        Ok(())
    }

    #[test]
    fn test_initialize_pool_contract() -> StdResult<()> {
        let mut app = mock_app(&coins(100 * SCRT_TO_USCRT, "uscrt"));
        let exp_contract_obj = instantiate_exp(&mut app, &ADMIN.to_string())?;
        instantiate_pool(&mut app, Some(100 * SCRT_TO_USCRT), &exp_contract_obj)?;
        Ok(())
    }

    #[test]
    fn test_set_exp_contract() -> StdResult<()> {
        let mut app = mock_app(&coins(100 * SCRT_TO_USCRT, "uscrt"));

        //1) Initialize exp contract
        let exp_contract_obj = instantiate_exp(&mut app, &ADMIN.to_string())?;

        //1.1) Init pool contract with exp information
        let pool_contract_obj =
            instantiate_pool(&mut app, Some(100 * SCRT_TO_USCRT), &exp_contract_obj)?;

        //1.2) Set exp contract
        let _res = app
            .execute_contract(
                Addr::unchecked(&ADMIN.to_string()),
                &pool_contract_obj,
                &crate::msg::HandleMsg::UpdateConfig {
                    unbonding_batch_duration: None,
                    unbonding_duration: None,
                    minimum_deposit_amount: None,
                    exp_contract: Some(ExpContract {
                        contract: s_toolkit::utils::types::Contract {
                            address: exp_contract_obj.address.to_string(),
                            hash: exp_contract_obj.code_hash.clone(),
                        },
                        vk: "vk_1".to_string(),
                    }),
                },
                &[],
            )
            .unwrap();

        //1.3) Query exp contract info

        let contract_config: crate::msg::ContractConfigResponse = app
            .wrap()
            .query_wasm_smart(
                &pool_contract_obj.code_hash,
                &pool_contract_obj.address,
                &crate::msg::QueryMsg::ContractConfig {},
            )
            .unwrap();

        assert_eq!(
            contract_config.exp_contract,
            Some(s_toolkit::utils::types::Contract {
                address: exp_contract_obj.address.to_string(),
                hash: exp_contract_obj.code_hash.clone(),
            })
        );

        Ok(())
    }

    #[test]
    fn test_set_weights() -> StdResult<()> {
        let mut app = mock_app(&coins(100 * SCRT_TO_USCRT, "uscrt"));

        //1) Initialize exp contract
        let exp_contract_obj = instantiate_exp(&mut app, &ADMIN.to_string())?;

        //1.1) Init pool contract with exp information
        let pool_contract_obj =
            instantiate_pool(&mut app, Some(100 * SCRT_TO_USCRT), &exp_contract_obj)?;

        //1.2) Set exp contract
        let _res = app
            .execute_contract(
                Addr::unchecked(&ADMIN.to_string()),
                &pool_contract_obj,
                &crate::msg::HandleMsg::UpdateConfig {
                    unbonding_batch_duration: None,
                    unbonding_duration: None,
                    minimum_deposit_amount: None,
                    exp_contract: Some(ExpContract {
                        contract: s_toolkit::utils::types::Contract {
                            address: exp_contract_obj.address.to_string(),
                            hash: exp_contract_obj.code_hash.clone(),
                        },
                        vk: "vk_1".to_string(),
                    }),
                },
                &[],
            )
            .unwrap();

        let _res = app
            .execute_contract(
                Addr::unchecked(&ADMIN.to_string()),
                &exp_contract_obj.clone(),
                &experience_contract::msg::ExecuteMsg::AddContract {
                    contracts: [AddContract {
                        address: Addr::unchecked(pool_contract_obj.address.to_string()),
                        code_hash: pool_contract_obj.code_hash.to_string(),
                        weight: 0,
                    }]
                    .to_vec(),
                },
                &[],
            )
            .unwrap();
        let _res = app
            .execute_contract(
                Addr::unchecked(&ADMIN.to_string()),
                &exp_contract_obj.clone(),
                &experience_contract::msg::ExecuteMsg::UpdateWeights {
                    weights: vec![experience_contract::msg::WeightUpdate {
                        address: Addr::unchecked(pool_contract_obj.address.to_string()),
                        weight: 100,
                    }],
                },
                &[],
            )
            .unwrap();

        //1.3) Query exp contract info

        let verified_contracts: experience_contract::msg::QueryAnswer = app
            .wrap()
            .query_wasm_smart(
                &exp_contract_obj.code_hash,
                &exp_contract_obj.address,
                &experience_contract::msg::QueryMsg::VerifiedContracts {
                    start_page: None,
                    page_size: None,
                },
            )
            .unwrap();

        let contracts = match verified_contracts {
            experience_contract::msg::QueryAnswer::VerifiedContractsResponse { contracts } => {
                contracts
            }
            _ => panic!("Unexpected QueryAnswer variant"),
        };

        match &contracts[0] {
            experience_contract::msg::VerifiedContractRes {
                address,
                weight,
                code_hash,
                ..
            } => {
                assert_eq!(address, &pool_contract_obj.address.to_string());
                assert_eq!(code_hash, &pool_contract_obj.code_hash.to_string());
                assert_eq!(address, &pool_contract_obj.address.to_string());
                assert_eq!(weight, &100u64);
            }
        }
        Ok(())
    }

    #[test]
    fn test_multi_tests_walkthrough() -> StdResult<()> {
        let mut app = mock_app(&coins(100 * SCRT_TO_USCRT, "uscrt"));

        //1) Initialize exp contract
        let exp_contract_obj = instantiate_exp(&mut app, &ADMIN.to_string())?;

        //1.1) Init pool contract with exp information
        let pool_contract_obj =
            instantiate_pool(&mut app, Some(100 * SCRT_TO_USCRT), &exp_contract_obj)?;

        //1.2) Set exp contract
        let _res = app
            .execute_contract(
                Addr::unchecked(&ADMIN.to_string()),
                &pool_contract_obj,
                &crate::msg::HandleMsg::UpdateConfig {
                    unbonding_batch_duration: None,
                    unbonding_duration: None,
                    minimum_deposit_amount: None,
                    exp_contract: Some(ExpContract {
                        contract: s_toolkit::utils::types::Contract {
                            address: exp_contract_obj.address.to_string(),
                            hash: exp_contract_obj.code_hash.clone(),
                        },
                        vk: "vk_1".to_string(),
                    }),
                },
                &[],
            )
            .unwrap();

        let _res = app
            .execute_contract(
                Addr::unchecked(&ADMIN.to_string()),
                &exp_contract_obj.clone(),
                &experience_contract::msg::ExecuteMsg::AddContract {
                    contracts: [AddContract {
                        address: Addr::unchecked(pool_contract_obj.address.to_string()),
                        code_hash: pool_contract_obj.code_hash.to_string(),
                        weight: 10,
                    }]
                    .to_vec(),
                },
                &[],
            )
            .unwrap();

        let _res = app
            .execute_contract(
                Addr::unchecked(&ADMIN.to_string()),
                &exp_contract_obj.clone(),
                &experience_contract::msg::ExecuteMsg::UpdateWeights {
                    weights: vec![experience_contract::msg::WeightUpdate {
                        address: Addr::unchecked(pool_contract_obj.address.to_string()),
                        weight: 100,
                    }],
                },
                &[],
            )
            .unwrap();

        //3) Set some rewards
        let _res = app.sudo(SudoMsg::Staking(StakingSudo::AddRewards {
            amount: Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(10 * SCRT_TO_USCRT),
            },
        }));

        //2) Deposit some $$$ by 2 users
        let _res = app
            .send_tokens(
                Addr::unchecked(ADMIN),
                Addr::unchecked(&"user_1".to_string()),
                &coins(10 * SCRT_TO_USCRT, "uscrt"),
            )
            .unwrap();
        let _res = app
            .execute_contract(
                Addr::unchecked(&"user_1".to_string()),
                &pool_contract_obj,
                &crate::msg::HandleMsg::Deposit {},
                &coins(10 * SCRT_TO_USCRT, "uscrt"),
            )
            .unwrap();

        let _res = app.send_tokens(
            Addr::unchecked(ADMIN),
            Addr::unchecked(&"user_2".to_string()),
            &coins(10 * SCRT_TO_USCRT, "uscrt"),
        );
        let _res = app
            .execute_contract(
                Addr::unchecked(&"user_2".to_string()),
                &pool_contract_obj,
                &crate::msg::HandleMsg::Deposit {},
                &coins(10 * SCRT_TO_USCRT, "uscrt"),
            )
            .unwrap();

        //4) End round
        //4.1) Update block and time
        app.set_block(BlockInfo {
            height: app.block_info().height.add(71),
            time: app.block_info().time.plus_seconds(3600 * 24 * 7),
            chain_id: app.block_info().chain_id,
            random: None,
        });

        //4.3) Check total exp avaiable for the pool contract
        let available_exp: experience_contract::msg::QueryAnswer = app
            .wrap()
            .query_wasm_smart(
                &exp_contract_obj.code_hash,
                &exp_contract_obj.address,
                &experience_contract::msg::QueryMsg::Contract {
                    key: "vk_1".to_string(),
                    address: Addr::unchecked(pool_contract_obj.clone().address.into_string()),
                },
            )
            .unwrap();

        if let experience_contract::msg::QueryAnswer::ContractResponse {
            available_exp,
            unclaimed_exp,
            ..
        } = available_exp
        {
            assert_eq!(available_exp, Uint128::zero());
            assert_eq!(unclaimed_exp, Uint128::from(71u128));
        }

        // 5) End round
        let _res = app
            .execute_contract(
                Addr::unchecked(&TRIGGERER.to_string()),
                &pool_contract_obj,
                &crate::msg::HandleMsg::EndRound {},
                &[],
            )
            .unwrap();

        //5.1) Query Xp given to the contract(POOL CONTRACT)
        let rewards_stat_obj: crate::msg::RewardStatsResponse = app
            .wrap()
            .query_wasm_smart(
                &pool_contract_obj.code_hash,
                &pool_contract_obj.address,
                &crate::msg::QueryMsg::RewardsStats {},
            )
            .unwrap();

        assert_eq!(rewards_stat_obj.total_exp, Some(Uint128::from(71u128)));
        assert_eq!(rewards_stat_obj.total_exp_claimed, Some(Uint128::zero()));

        //5.2) Check total exp avaiable for the pool contract(EXP POOL)
        let available_exp: experience_contract::msg::QueryAnswer = app
            .wrap()
            .query_wasm_smart(
                &exp_contract_obj.code_hash,
                &exp_contract_obj.address,
                &experience_contract::msg::QueryMsg::Contract {
                    key: "vk_1".to_string(),
                    address: Addr::unchecked(pool_contract_obj.clone().address.into_string()),
                },
            )
            .unwrap();

        if let experience_contract::msg::QueryAnswer::ContractResponse {
            available_exp,
            unclaimed_exp,
            ..
        } = available_exp
        {
            assert_eq!(available_exp, Uint128::from(71u128));
            assert_eq!(unclaimed_exp, Uint128::from(0u128));
        }
        //6) Claim rewards by u1 side
        let _res = app
            .execute_contract(
                Addr::unchecked(&"user_1".to_string()),
                &pool_contract_obj,
                &crate::msg::HandleMsg::ClaimRewards {},
                &[],
            )
            .unwrap();

        //6.1) Check the exp given to user(POOL CONTRACT).
        let rewards_stat_obj: crate::msg::RewardStatsResponse = app
            .wrap()
            .query_wasm_smart(
                &pool_contract_obj.code_hash,
                &pool_contract_obj.address,
                &crate::msg::QueryMsg::RewardsStats {},
            )
            .unwrap();

        assert_eq!(rewards_stat_obj.total_exp, Some(Uint128::from(71u128)));
        assert_eq!(
            rewards_stat_obj.total_exp_claimed,
            Some(Uint128::from(35u128))
        );

        //6.2) Check total exp avaiable for the pool contract(EXP CONTRACT).
        let available_exp: experience_contract::msg::QueryAnswer = app
            .wrap()
            .query_wasm_smart(
                &exp_contract_obj.code_hash,
                &exp_contract_obj.address,
                &experience_contract::msg::QueryMsg::Contract {
                    key: "vk_1".to_string(),
                    address: Addr::unchecked(pool_contract_obj.clone().address.into_string()),
                },
            )
            .unwrap();

        if let experience_contract::msg::QueryAnswer::ContractResponse {
            available_exp,
            unclaimed_exp,
            ..
        } = available_exp
        {
            assert_eq!(available_exp, Uint128::from(36u128));
            assert_eq!(unclaimed_exp, Uint128::from(0u128));
        }
        //6.3) Check total exp avaiable for the user(EXP CONTRACT).
        //6.3.1) Set user1 vk
        let _res = app
            .execute_contract(
                Addr::unchecked(&"user_1".to_string()),
                &exp_contract_obj,
                &crate::msg::HandleMsg::SetViewingKey {
                    key: "vk".to_string(),
                },
                &[],
            )
            .unwrap();
        //6.3.2) Query exp
        let available_exp: experience_contract::msg::QueryAnswer = app
            .wrap()
            .query_wasm_smart(
                &exp_contract_obj.code_hash,
                &exp_contract_obj.address,
                &experience_contract::msg::QueryMsg::UserExp {
                    key: "vk".to_string(),
                    address: Addr::unchecked("user_1".to_string()),
                    season: None,
                },
            )
            .unwrap();

        if let experience_contract::msg::QueryAnswer::UserExp { exp } = available_exp {
            assert_eq!(exp, Uint128::from(35u128));
        }

        //7) Claim rewards by u2 side
        let _res = app
            .execute_contract(
                Addr::unchecked(&"user_2".to_string()),
                &pool_contract_obj,
                &crate::msg::HandleMsg::ClaimRewards {},
                &[],
            )
            .unwrap();

        //7.1) Check the exp given to user 2.(POOL CONTRACT)
        let rewards_stat_obj: crate::msg::RewardStatsResponse = app
            .wrap()
            .query_wasm_smart(
                &pool_contract_obj.code_hash,
                &pool_contract_obj.address,
                &crate::msg::QueryMsg::RewardsStats {},
            )
            .unwrap();

        assert_eq!(rewards_stat_obj.total_exp, Some(Uint128::from(71u128)));
        assert_eq!(
            rewards_stat_obj.total_exp_claimed,
            Some(Uint128::from(70u128))
        );

        //7.2) Check total exp avaiable for the pool contract
        let available_exp: experience_contract::msg::QueryAnswer = app
            .wrap()
            .query_wasm_smart(
                &exp_contract_obj.code_hash,
                &exp_contract_obj.address,
                &experience_contract::msg::QueryMsg::Contract {
                    key: "vk_1".to_string(),
                    address: Addr::unchecked(pool_contract_obj.clone().address.into_string()),
                },
            )
            .unwrap();

        if let experience_contract::msg::QueryAnswer::ContractResponse {
            available_exp,
            unclaimed_exp,
            ..
        } = available_exp
        {
            assert_eq!(available_exp, Uint128::from(1u128));
            assert_eq!(unclaimed_exp, Uint128::from(0u128));
        }
        //7.3) Check total exp avaiable for the user(EXP CONTRACT).
        //7.3.1) Set user2 vk
        let _res = app
            .execute_contract(
                Addr::unchecked(&"user_2".to_string()),
                &exp_contract_obj,
                &crate::msg::HandleMsg::SetViewingKey {
                    key: "vk".to_string(),
                },
                &[],
            )
            .unwrap();
        //7.3.2) Query exp
        let available_exp: experience_contract::msg::QueryAnswer = app
            .wrap()
            .query_wasm_smart(
                &exp_contract_obj.code_hash,
                &exp_contract_obj.address,
                &experience_contract::msg::QueryMsg::UserExp {
                    key: "vk".to_string(),
                    address: Addr::unchecked("user_2".to_string()),
                    season: None,
                },
            )
            .unwrap();

        if let experience_contract::msg::QueryAnswer::UserExp { exp } = available_exp {
            assert_eq!(exp, Uint128::from(35u128));
        }

        Ok(())
    }
}
