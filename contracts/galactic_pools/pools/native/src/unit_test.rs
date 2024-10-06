#[cfg(test)]
mod tests {
    use crate::{
        constants::*,
        contract::{execute, instantiate, query},
        helper::*,
        msg::{
            ContractConfigResponse,
            ContractStatus,
            ContractStatusResponse,
            CurrentRewardsResponse,
            DelegatedResponse,
            GalacticPoolsPermissions,
            HandleAnswer,
            HandleMsg,
            InstantiateMsg,
            LiquidityResponse,
            PoolStateInfoResponse,
            PoolStateLiquidityStatsResponse,
            QueryMsg,
            QueryWithPermit,
            RecordsResponse,
            RemoveSponsorCredentialsDecisions,
            ResponseStatus,
            ResponseStatus::Success,
            Review,
            RoundResponse,
            SponsorMessageReqResponse,
            SponsorsResponse,
            UnbondingsResponse,
            ValidatorInfo,
            ViewingKeyErrorResponse,
            WithdrawablelResponse,
        },
        rand::sha_256,
        state::*,
        viewing_key::{ViewingKey, VIEWING_KEY_SIZE},
    };
    use c_std::{
        coin,
        coins,
        from_binary,
        testing::{mock_env, mock_info, *},
        to_binary,
        Addr,
        Api,
        Binary,
        BlockInfo,
        Coin,
        ContractInfo,
        Decimal,
        DepsMut,
        Empty,
        Env,
        FullDelegation,
        OwnedDeps,
        Response,
        StdError,
        StdResult,
        Storage,
        Timestamp,
        TransactionInfo,
        Uint128,
        Validator,
    };
    use s_toolkit::permit::{validate, Permit, PermitParams, PermitSignature, PubKey};
    use serde::{Deserialize, Serialize};
    use shade_protocol::{c_std, s_toolkit};
    use std::ops::{Add, AddAssign, Sub};

    const REWARDS_RETURNED_FROM_VALIDATOR_PER_ACTION: u128 = 10u128;
    const COMMON_DIVISOR: u64 = 10000;
    const SCRT_TO_USCRT: u128 = 1000000;

    //////////////////////////////// Helper functions ////////////////////////////////
    pub fn init_helper(
        contract_balance: Option<u128>,
    ) -> (
        Result<Response, c_std::StdError>,
        OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
    ) {
        let mut deps = mock_dependencies_with_balance(&[Coin {
            amount: Uint128::from(contract_balance.unwrap_or_default()),
            denom: "uscrt".to_string(),
        }]);
        //     let mut deps = mock_dependencies();
        let env = mock_env();

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

        let mut validators: Vec<Validator> = Vec::new();

        for validator_address in &validator_vector {
            validators.push(Validator {
                address: (validator_address.address.clone()),
                commission: Decimal::percent(1),
                max_commission: Decimal::percent(2),
                max_change_rate: Decimal::percent(3),
            })
        }

        let mut delegation_vec: Vec<FullDelegation> = Vec::new();
        // and another one on val2
        let delegation1 = FullDelegation {
            delegator: deps.api.addr_validate("cosmos2contract").unwrap(),
            validator: "galacticPools".to_string().clone(),
            amount: coin(8888, "uscrt"),
            can_redelegate: coin(4567, "uscrt"),
            accumulated_rewards: coins(10 * SCRT_TO_USCRT, "uscrt"),
        };
        delegation_vec.push(delegation1);
        let delegation2 = FullDelegation {
            delegator: deps.api.addr_validate("cosmos2contract").unwrap(),
            validator: "secureSecret".to_string().clone(),
            amount: coin(8888, "uscrt"),
            can_redelegate: coin(4567, "uscrt"),
            accumulated_rewards: coins(10 * SCRT_TO_USCRT, "uscrt"),
        };
        delegation_vec.push(delegation2);

        let delegation3 = FullDelegation {
            delegator: deps.api.addr_validate("cosmos2contract").unwrap(),
            validator: "xavierCapital".to_string().clone(),
            amount: coin(8888, "uscrt"),
            can_redelegate: coin(4567, "uscrt"),
            accumulated_rewards: coins(10 * SCRT_TO_USCRT, "uscrt"),
        };
        delegation_vec.push(delegation3);

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

        deps.querier
            .update_staking("uscrt", &validators, &delegation_vec);

        let init_msg = InstantiateMsg {
            admins: Option::from(vec![Addr::unchecked("admin")]),
            triggerers: Option::from(vec![Addr::unchecked("triggerer")]),
            reviewers: Option::from(vec![Addr::unchecked("reviewer")]),
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
            shade_rewards_address: Addr::unchecked("shade"),
            galactic_pools_rewards_address: Addr::unchecked("galactic_pools"),
            reserve_percentage: (60 * common_divisor) / 100,
            is_sponosorship_admin_controlled: false,
            unbonding_batch_duration: 3600 * 24 * 3,
            minimum_deposit_amount: None,
            grand_prize_address: Addr::unchecked("grand_prize"),
            /// setting number of number_of_tickers that can be run on txn send to avoid potential errors
            number_of_tickers_per_transaction: Uint128::from(1000000u128),
            sponsor_msg_edit_fee: Some(Uint128::from(1000000u128)),
            exp_contract: None,
        };

        let mock_message_info = mock_info("", &[]);

        let init_results = instantiate(deps.as_mut(), env, mock_message_info, init_msg);
        (init_results, deps)
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct ValidatorRewards {
        pub validator_address: String,
        pub reward: Vec<Coin>,
    }

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
                address: Addr::unchecked(MOCK_CONTRACT_ADDR),
                code_hash: "".to_string(),
            },
        }
    }

    pub fn deposits_filler_unit_test_helper(
        contact_balance: Option<u128>,
    ) -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
        let (_init_result, mut deps) = init_helper(contact_balance);

        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(30000 * SCRT_TO_USCRT),
            Some(0u64),
            Some(round_obj.start_time),
            "secretbatman",
            None,
        );
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(5000 * (SCRT_TO_USCRT)),
            Some(0u64),
            Some(round_obj.start_time),
            "Superman",
            None,
        );
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(5000 * (SCRT_TO_USCRT)),
            Some(0u64),
            Some(round_obj.start_time),
            "Spider-man",
            None,
        );
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(5000 * (SCRT_TO_USCRT)),
            Some(0u64),
            Some(round_obj.start_time),
            "Wonder-Women",
            None,
        );
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(5000 * (SCRT_TO_USCRT)),
            Some(0u64),
            Some(round_obj.start_time),
            "Thor",
            None,
        );
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(5000 * (SCRT_TO_USCRT)),
            Some(0u64),
            Some(round_obj.start_time),
            "Captain-America",
            None,
        );
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(5000 * (SCRT_TO_USCRT)),
            Some(0u64),
            Some(round_obj.start_time),
            "Ironman",
            None,
        );
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(5000 * (SCRT_TO_USCRT)),
            Some(0u64),
            Some(round_obj.start_time),
            "Loki",
            None,
        );
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(5000 * (SCRT_TO_USCRT)),
            Some(0u64),
            Some(round_obj.start_time),
            "Aqua-man",
            None,
        );
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(30000 * (SCRT_TO_USCRT)),
            Some(0u64),
            Some((round_obj.start_time + round_obj.end_time) / 2),
            "secretbatman",
            None,
        );

        return deps;
    }

    pub fn deposit_unit_test_helper(
        mut_deps: DepsMut,
        deposit_amount: Uint128,
        height: Option<u64>,
        time: Option<u64>,
        deposit_sender: &str,
        denom_default_uscrt: Option<&str>,
    ) -> StdResult<()> {
        let handle_msg = HandleMsg::Deposit {};
        let mut denom: &str = "uscrt";
        if denom_default_uscrt.is_some() {
            denom = denom_default_uscrt.unwrap();
        }

        let handle_result = execute(
            mut_deps,
            custom_mock_env(height, time, None, None),
            mock_info(deposit_sender, &[Coin {
                amount: deposit_amount,
                denom: denom.to_string(),
            }]),
            handle_msg,
        );

        if handle_result.is_err() {
            return Err(handle_result.unwrap_err());
        } else {
            Ok(())
        }
    }

    pub fn request_withdraw_unit_test_helper(
        mut_deps: DepsMut,
        request_withdraw_amount: Uint128,
        height: Option<u64>,
        time: Option<u64>,
        request_sender: &str,
    ) -> StdResult<()> {
        let handle_msg = HandleMsg::RequestWithdraw {
            amount: request_withdraw_amount,
        };

        let handle_result = execute(
            mut_deps,
            custom_mock_env(height, time, None, None),
            mock_info(request_sender, &[]),
            handle_msg,
        );

        if handle_result.is_err() {
            return Err(handle_result.unwrap_err());
        } else {
            Ok(())
        }
    }

    pub fn withdraw_unit_test_helper(
        mut_deps: DepsMut,
        withdraw_amount: Uint128,
        height: Option<u64>,
        time: Option<u64>,
        request_sender: &str,
    ) -> StdResult<()> {
        let handle_msg = HandleMsg::Withdraw {
            amount: withdraw_amount,
        };

        let handle_result = execute(
            mut_deps,
            custom_mock_env(height, time, None, None),
            mock_info(request_sender, &[]),
            handle_msg,
        );

        if handle_result.is_err() {
            return Err(handle_result.unwrap_err());
        } else {
            Ok(())
        }
    }

    pub fn claim_rewards_unit_test_helper(
        mut_deps: DepsMut,
        height: Option<u64>,
        time: Option<u64>,
        request_sender: &str,
    ) -> StdResult<()> {
        let handle_msg = HandleMsg::ClaimRewards {};
        let handle_result = execute(
            mut_deps,
            custom_mock_env(height, time, None, None),
            mock_info(request_sender, &[]),
            handle_msg,
        );

        if handle_result.is_err() {
            return Err(handle_result.unwrap_err());
        } else {
            Ok(())
        }
    }

    pub fn create_viewking_key_unit_test_helper(
        mut_deps: DepsMut,
        request_sender: &str,
    ) -> StdResult<()> {
        let handle_msg = HandleMsg::CreateViewingKey {
            entropy: "".to_string(),
        };
        let handle_result = execute(
            mut_deps,
            custom_mock_env(None, None, None, None),
            mock_info(request_sender, &[]),
            handle_msg,
        );

        if handle_result.is_err() {
            return Err(handle_result.unwrap_err());
        } else {
            Ok(())
        }
    }

    pub fn set_viewing_key_unit_test_helper(
        mut_deps: DepsMut,
        request_sender: &str,
        key: &str,
    ) -> StdResult<()> {
        // Set VK
        let handle_msg = HandleMsg::SetViewingKey {
            key: key.to_string(),
        };
        let handle_result = execute(
            mut_deps,
            custom_mock_env(None, None, None, None),
            mock_info(request_sender, &[]),
            handle_msg,
        );
        if handle_result.is_err() {
            return Err(handle_result.unwrap_err());
        } else {
            Ok(())
        }
    }

    pub fn sponsor_unit_test_helper(
        mut_deps: DepsMut,
        deposit_amount: Uint128,
        height: Option<u64>,
        time: Option<u64>,
        sponsor: &str,
        denom_default_uscrt: Option<&str>,
    ) -> StdResult<()> {
        let handle_msg = HandleMsg::Sponsor {
            title: None,
            message: None,
        };
        let mut denom: &str = "uscrt";
        if denom_default_uscrt.is_some() {
            denom = denom_default_uscrt.unwrap();
        }

        let handle_result = execute(
            mut_deps,
            custom_mock_env(height, time, None, None),
            mock_info(sponsor, &[Coin {
                amount: deposit_amount,
                denom: denom.to_string(),
            }]),
            handle_msg,
        );

        if handle_result.is_err() {
            return Err(handle_result.unwrap_err());
        } else {
            Ok(())
        }
    }

    pub fn sponsor_request_withdraw_unit_test_helper(
        mut_deps: DepsMut,
        request_withdraw_amount: Uint128,
        height: Option<u64>,
        time: Option<u64>,
        request_sender: &str,
    ) -> StdResult<()> {
        let handle_msg = HandleMsg::SponsorRequestWithdraw {
            amount: request_withdraw_amount,
        };

        let handle_result = execute(
            mut_deps,
            custom_mock_env(height, time, None, None),
            mock_info(request_sender, &[]),
            handle_msg,
        );

        if handle_result.is_err() {
            return Err(handle_result.unwrap_err());
        } else {
            Ok(())
        }
    }

    pub fn sponsor_withdraw_unit_test_helper(
        mut_deps: DepsMut,
        withdraw_amount: Uint128,
        height: Option<u64>,
        time: Option<u64>,
        request_sender: &str,
    ) -> StdResult<()> {
        let handle_msg = HandleMsg::SponsorWithdraw {
            amount: withdraw_amount,
        };

        let handle_result = execute(
            mut_deps,
            custom_mock_env(height, time, None, None),
            mock_info(request_sender, &[]),
            handle_msg,
        );

        if handle_result.is_err() {
            return Err(handle_result.unwrap_err());
        } else {
            Ok(())
        }
    }

    pub fn end_round_unit_test_helper(deps: DepsMut) -> StdResult<Response> {
        let round_obj = round_read_only_unit_test_helper(deps.storage);

        let handle_msg = HandleMsg::EndRound {};
        let mocked_env = custom_mock_env(None, Some(round_obj.end_time), None, None);
        let mocked_info = mock_info("triggerer", &[]);

        let handle_result = execute(deps, mocked_env, mocked_info, handle_msg);

        if handle_result.is_err() {
            return Err(handle_result.unwrap_err());
        } else {
            Ok(handle_result.unwrap())
        }
    }

    //     //////////////////////////////// Readonly State Helper functions ////////////////////////////////
    pub fn user_info_read_only_unit_test_helper(
        user: &Addr,
        deps_storage: &dyn Storage,
    ) -> UserInfo {
        let user_info_obj = USER_INFO_STORE
            .load(deps_storage, user)
            .unwrap_or(UserInfo {
                amount_delegated: Uint128::zero(),
                amount_withdrawable: Uint128::zero(),
                starting_round: None,
                total_won: Uint128::zero(),
                last_claim_rewards_round: None,
                amount_unbonding: Uint128::zero(),
                unbonding_batches: Vec::new(),
            });

        return user_info_obj;
    }

    pub fn user_liquidity_stats_read_only_unit_test_helper(
        user: &Addr,
        deps_storage: &dyn Storage,
        round_index: u64,
    ) -> UserLiqState {
        let user_liquidity_snapshot_obj = USER_LIQUIDITY_STATS_STORE
            .load(deps_storage, (user, round_index))
            .unwrap_or(UserLiqState {
                amount_delegated: None,
                liquidity: None,
                tickets_used: None,
            });
        user_liquidity_snapshot_obj
    }

    pub fn config_read_only_unit_test_helper(deps_storage: &dyn Storage) -> ConfigInfo {
        CONFIG_STORE.load(deps_storage).unwrap()
    }

    pub fn pool_state_read_only_unit_test_helper(deps_storage: &dyn Storage) -> PoolState {
        POOL_STATE_STORE.load(deps_storage).unwrap()
    }

    pub fn pool_state_liquidity_snapshot_read_only_unit_test_helper(
        deps_storage: &dyn Storage,
        current_round_index: u64,
    ) -> PoolLiqState {
        let pool_state_liquidity_snapshot_obj: PoolLiqState = POOL_STATE_LIQUIDITY_STATS_STORE
            .load(deps_storage, current_round_index)
            .unwrap_or(PoolLiqState {
                total_delegated: None,
                total_liquidity: None,
            });
        pool_state_liquidity_snapshot_obj
    }

    pub fn round_read_only_unit_test_helper(deps_storage: &dyn Storage) -> RoundInfo {
        ROUND_STORE.load(deps_storage).unwrap()
    }

    pub fn rewards_stats_for_nth_round_read_only_unit_test_helper(
        deps_storage: &dyn Storage,
        round: u64,
    ) -> RewardsState {
        let reward_stats_for_nth_round = REWARDS_STATS_FOR_NTH_ROUND_STORE
            .load(deps_storage, round)
            .unwrap_or(Default::default());

        reward_stats_for_nth_round
    }

    pub fn sponsor_info_unit_test_read_only_helper(
        deps_storage: &dyn Storage,
        sender: &Addr,
    ) -> StdResult<SponsorInfo> {
        let sponsor_info_obj = SPONSOR_INFO_STORE
            .load(deps_storage, sender)
            .unwrap_or_default();

        return Ok(sponsor_info_obj);
    }

    pub fn sponsor_state_unit_test_read_only_helper(
        deps_storage: &dyn Storage,
    ) -> StdResult<GlobalSponsorState> {
        return Ok(SPONSOR_STATS_STORE.load(deps_storage)?);
    }

    ////////////////////////////////////// Tests //////////////////////////////////////
    ////////////////////////////////////// User Tests //////////////////////////////////////

    #[test]
    fn test_init() -> StdResult<()> {
        // test default
        let (init_result, deps) = init_helper(None);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.admins[0], Addr::unchecked("admin"));

        assert_eq!(config.triggerers[0], Addr::unchecked("triggerer"));

        assert_eq!(config.common_divisor, COMMON_DIVISOR);
        assert_eq!(config.denom, "uscrt".to_string());
        let prng_seed_hashed = sha_256(&Binary::from("I'm secretbatman".as_bytes()).0);

        assert_eq!(config.prng_seed, prng_seed_hashed.to_vec());
        assert_eq!(config.contract_address, Addr::unchecked(MOCK_CONTRACT_ADDR));
        assert_eq!(config.validators[0].address, "galacticPools".to_string());
        assert_eq!(config.validators[1].address, "secureSecret".to_string());
        assert_eq!(config.validators[2].address, "xavierCapital".to_string());
        assert_eq!(config.unbonding_duration, 3600 * 24 * 21); // Seconds in 21 days
        assert_eq!(config.minimum_deposit_amount, None);
        assert_eq!(config.status, ContractStatus::Normal.to_u8());

        let pool_state_obj = pool_state_read_only_unit_test_helper(&deps.storage);
        assert_eq!(pool_state_obj.total_delegated, Uint128::zero());
        assert_eq!(pool_state_obj.rewards_returned_to_contract, Uint128::zero());
        assert_eq!(pool_state_obj.total_reserves, Uint128::zero());
        assert_eq!(pool_state_obj.total_sponsored, Uint128::zero());

        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            round_obj.entropy,
            sha_256(&sha_256(&Binary::from("I'm secretbatman".as_bytes()).0).to_vec())
        );
        assert_eq!(
            round_obj.seed,
            sha_256(&Binary::from("I'm secretbatman".as_bytes()).0).to_vec()
        );
        assert_eq!(round_obj.start_time, 1571797419u64);
        assert_eq!(round_obj.end_time, 1571797419 + 3600 * 24 * 7); // Seconds in 7 days
        assert_eq!(round_obj.current_round_index, 1u64);
        assert_eq!(round_obj.ticket_price, Uint128::from(1u128 * SCRT_TO_USCRT)); // 1 SCRT
        assert_eq!(round_obj.rewards_expiry_duration, 3600 * 24 * 45); // Seconds in 45 days
        assert_eq!(
            round_obj.admin_share.shade_percentage_share,
            (60 * COMMON_DIVISOR) / 100
        );
        assert_eq!(
            round_obj.admin_share.galactic_pools_percentage_share,
            (40 * COMMON_DIVISOR) / 100
        );
        assert_eq!(
            round_obj.triggerer_share_percentage,
            (1 * COMMON_DIVISOR) / 100
        );
        assert_eq!(round_obj.shade_rewards_address, Addr::unchecked("shade"));
        assert_eq!(
            round_obj.galactic_pools_rewards_address,
            Addr::unchecked("galactic_pools")
        );
        assert_eq!(round_obj.unclaimed_rewards_last_claimed_round, None);

        Ok(())
    }

    #[test]
    fn test_deposit() -> StdResult<()> {
        //0) Checking validity of the deposit
        //0.1) Error Check: Denom send must me uscrt
        //0.2) Error Check: Minimum amount must be 1 uscrt
        //0.3) Error Check: Minimum amount must be 1 scrt or 1000000 uscrt
        //0.4) Checking: Maximum possible amount that can be deposited
        let (_init_result, mut deps) = init_helper(None);
        //0.1) Error Check: Denom send must me uscrt
        let deposit_results = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(30000 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
            Some("uatom"),
        )
        .unwrap_err();
        assert_eq!(
            deposit_results,
            StdError::generic_err("Wrong token given, expected uscrt found uatom")
        );
        //0.2) Error Check: Minimum amount must be 1 uscrt
        let deposit_results = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(0u128),
            None,
            None,
            "secretbatman",
            Some("uscrt"),
        )
        .unwrap_err();
        assert_eq!(
            deposit_results,
            StdError::generic_err("Must deposit atleast one uscrt")
        );
        //0.3) Error Check: Minimum amount must be 1 scrt or 1000000 uscrt
        let handle_msg = HandleMsg::UpdateConfig {
            unbonding_batch_duration: None,
            unbonding_duration: None,
            minimum_deposit_amount: Some(Uint128::from(1 * SCRT_TO_USCRT)),
            exp_contract: None,
        };
        let _handle_result = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("admin", &[]),
            handle_msg,
        )
        .unwrap();
        let deposit_results = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(30000u128),
            None,
            None,
            "secretbatman",
            Some("uscrt"),
        )
        .unwrap_err();
        assert_eq!(
            deposit_results,
            StdError::generic_err("Must deposit a minimum of 1000000 uscrt",)
        );
        //0.2) Checking: Maximum possible amount that can be deposited
        //340282366920938463463374607431768211455 39 Digits u128
        //18446744073709551615 20 digits u64
        //200*MILLION*1000000(to uscrt) 100,000,000,000,000  total uscrt in secret Network - 16 digits
        //340282366920938463463374607431768 is the largest possible number user can deposit.
        let (_init_result, mut deps) = init_helper(None);
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(340282366920938463463374607431768u128),
            Some(0),
            Some(0),
            "secretbatman",
            Some("uscrt"),
        )?;
        let user = user_info_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
        );
        assert_eq!(
            user.amount_delegated,
            Uint128::from(340282366920938463463374607431768u128)
        );

        //1) Checking user
        //1.1) Amount Delegated
        //1.2) Liquidity Provided
        //1.3) Amount Delegated and liquidity Provided after the end lottery time
        let (_init_result, deps) = init_helper(None);
        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        //1.1) Calculating secretbatman's Delegated Amount. He delegated 30000 SCRT at t0 and 30000 SCRT at t1/2; so he delegated 60000 SCRT.
        let mut deps = deposits_filler_unit_test_helper(None);
        let user = user_info_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            deps.as_ref().storage,
        );
        assert_eq!(
            user.amount_delegated,
            Uint128::from((30000 + 30000) * SCRT_TO_USCRT)
        );
        //1.2) Calculating Liquidity Provided.
        //30000+15000 since 30000 Scrt were delegated at the start and 30000 were delegated at the middle.
        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            deps.as_ref().storage,
            round_obj.current_round_index,
        );
        assert_eq!(
            user_liquidity_snapshot_obj.liquidity.unwrap(),
            Uint128::from((30000 + 15000) * SCRT_TO_USCRT)
        );
        //1.3) Amount Delegated and liquidity Provided after the end lottery time
        let round_obj = round_read_only_unit_test_helper(deps.as_ref().storage);
        deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10000 * SCRT_TO_USCRT),
            None,
            Some(round_obj.end_time),
            "secretbatman",
            None,
        )?;
        let user = user_info_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            deps.as_ref().storage,
        );
        assert_eq!(
            user.amount_delegated,
            Uint128::from((30000 + 30000 + 10000) * SCRT_TO_USCRT)
        );
        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            deps.as_ref().storage,
            round_obj.current_round_index,
        );
        assert_eq!(
            user_liquidity_snapshot_obj.liquidity.unwrap(),
            Uint128::from((30000 + 15000 + 0) * SCRT_TO_USCRT)
        );

        //2) Checking:
        //2.1) Total deposits - pool state
        //2.2) Total liquidity provided by the users -  pool state liquidity
        //2.3) Total Rewards returned -  pool state
        //2.4) Total liquidity in  pool state when no-deposits are made -  pool state liquidity
        //2.5) Total liquidity in  pool state when deposits are made -  pool state liquidity
        let mut deps = deposits_filler_unit_test_helper(None);
        let round_obj = round_read_only_unit_test_helper(deps.as_ref().storage);
        //2.1) Total Amount delegated
        //secretbatman      30000    @ t0 and 30000 @ t1/2. So 60000 SCRT.
        //Superman          5000     @ t0
        //Spider-man        5000     @ t0
        //Wonder-Women      5000     @ t0
        //Thor              5000     @ t0
        //Captain-America   5000     @ t0
        //Iron-man          5000     @ t0
        //Loki              5000     @ t0
        //Aqua-man          5000     @ t0
        let pool_state = pool_state_read_only_unit_test_helper(deps.as_ref().storage);
        assert_eq!(
            pool_state.total_delegated,
            Uint128::from(100000 * SCRT_TO_USCRT)
        );
        //2.2) Total Liquidity provided
        let pool_state_liquidity_snapshot_obj: PoolLiqState =
            pool_state_liquidity_snapshot_read_only_unit_test_helper(
                deps.as_ref().storage,
                round_obj.current_round_index,
            );
        //70000 were delegated at the start and 30000 were delegated at the middle. hence
        assert_eq!(
            pool_state_liquidity_snapshot_obj.total_liquidity.unwrap(),
            Uint128::from((70000 + (30000 / 2)) * SCRT_TO_USCRT)
        );
        //2.3) Checking: Total Rewards returned
        //Rewards returned are usually variable. But for this test, we kept rewards constant.
        assert_eq!(
            pool_state.rewards_returned_to_contract,
            Uint128::from(REWARDS_RETURNED_FROM_VALIDATOR_PER_ACTION * 10 * SCRT_TO_USCRT)
        ); // 10 scrt on each deposit and a total of 10 deposits

        //Checking liquidity after
        //2.4)No-deposit during round 2. So total liquidity equals total amount delegated.
        end_round_unit_test_helper(deps.as_mut())?; //round 1 -> 2 liq:85000
        end_round_unit_test_helper(deps.as_mut())?; //round 2 -> 3 liq: 100000
        let round_obj = round_read_only_unit_test_helper(deps.as_ref().storage);
        let pool_state_liquidity_snapshot_obj: PoolLiqState =
            pool_state_liquidity_snapshot_read_only_unit_test_helper(
                deps.as_ref().storage,
                round_obj.current_round_index.sub(1u64),
            );
        assert_eq!(
            pool_state_liquidity_snapshot_obj.total_liquidity.unwrap(),
            Uint128::from(100000 * SCRT_TO_USCRT)
        );

        //2.5)With a deposit during round
        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10000 * SCRT_TO_USCRT),
            None,
            Some(round_obj.start_time),
            "secretbatman",
            None,
        )?;
        end_round_unit_test_helper(deps.as_mut())?; //round 3 -> 4 liq:110000
        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        let pool_state_liquidity_snapshot_obj: PoolLiqState =
            pool_state_liquidity_snapshot_read_only_unit_test_helper(
                &deps.storage,
                round_obj.current_round_index.sub(1u64),
            );
        assert_eq!(
            pool_state_liquidity_snapshot_obj.total_liquidity.unwrap(),
            Uint128::from((100000 + 10000) * SCRT_TO_USCRT)
        );
        Ok(())
    }

    #[test]
    fn test_request_withdraw() -> StdResult<()> {
        //0) Checking validity of the deposit
        //0.1) Checking what happens if user request_withdraw more than deposited
        //0.2) Checking validity of the deposit
        let mut deps = deposits_filler_unit_test_helper(None);
        let res = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((150000 * SCRT_TO_USCRT) as u128),
            None,
            None,
            "secretbatman",
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err(format!(
                "insufficient funds to redeem: balance=60000000000, required=150000000000",
            ))
        );

        //1) Checking USERINFO after requesting the amount
        //1.1)Checking delegated amount after request_withdraw
        let mut deps = deposits_filler_unit_test_helper(None);
        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((50000 * SCRT_TO_USCRT) as u128),
            None,
            Some((round_obj.start_time + round_obj.end_time) / 2),
            "secretbatman",
        )?;
        let user = user_info_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
        );
        assert_eq!(user.amount_delegated.u128(), 10000 * SCRT_TO_USCRT);
        //1.2) Checking unbonding information
        assert_eq!(user.unbonding_batches[0], 1);
        assert_eq!(user.unbonding_batches.len(), 1);

        //1.3) Calculating Liquidity Provided.
        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
            round_obj.current_round_index,
        );
        assert_eq!(
            user_liquidity_snapshot_obj.liquidity.unwrap(),
            Uint128::from((45000 - 25000) * SCRT_TO_USCRT)
        );

        //2) Checking  poolstate  after request withdraw
        let mut deps = deposits_filler_unit_test_helper(None);
        let round_obj = round_read_only_unit_test_helper(&deps.storage);

        let pool_obj = pool_state_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            pool_obj.total_delegated,
            Uint128::from(100000 * SCRT_TO_USCRT)
        );
        assert_eq!(
            pool_obj.rewards_returned_to_contract.u128(),
            100 * SCRT_TO_USCRT
        );

        //2.1) Checking  poolstate & LiquidityStats after request withdraw
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((50000 * SCRT_TO_USCRT) as u128),
            None,
            Some((round_obj.start_time + round_obj.end_time) / 2),
            "secretbatman",
        )?;
        let pool_obj = pool_state_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            pool_obj.total_delegated,
            Uint128::from(50000 * SCRT_TO_USCRT)
        );

        //2.2) Total Liquidity provided
        let pool_state_liquidity_snapshot_obj: PoolLiqState =
            pool_state_liquidity_snapshot_read_only_unit_test_helper(
                &deps.storage,
                round_obj.current_round_index,
            );
        assert_eq!(
            pool_state_liquidity_snapshot_obj.total_liquidity.unwrap(),
            Uint128::from((85000 - 25000) * SCRT_TO_USCRT)
        );
        //2.3) Checking: Total Rewards returned
        assert_eq!(
            pool_obj.rewards_returned_to_contract.u128(),
            100 * SCRT_TO_USCRT
        );

        //3) Request Withdraw after round ends
        let mut deps = deposits_filler_unit_test_helper(None);
        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((50000 * SCRT_TO_USCRT) as u128),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        )?;
        let user = user_info_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
        );
        assert_eq!(user.amount_delegated.u128(), 10000 * SCRT_TO_USCRT);
        //3.2) Checking unbonding information
        assert_eq!(user.unbonding_batches[0], 1);
        assert_eq!(user.unbonding_batches.len(), 1);

        //3.3) Calculating Liquidity Provided.
        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
            round_obj.current_round_index,
        );
        // Liquidity will not change
        assert_eq!(
            user_liquidity_snapshot_obj.liquidity.unwrap(),
            Uint128::from((45000) * SCRT_TO_USCRT)
        );

        //Checking the validator stats
        // Done in test_validator_walk_through
        Ok(())
    }

    #[test]
    fn test_withdraw() -> StdResult<()> {
        //1)Checking: Withdraw more than contract balance
        let (_init_result, mut deps) = init_helper(Some(60000000));
        //Deposit
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(70 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
            None,
        );
        //Request withdraw
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((70 * SCRT_TO_USCRT) as u128),
            None,
            None,
            "secretbatman",
        )?;
        let user = user_info_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
        );
        assert_eq!(user.unbonding_batches.len(), 1);
        assert_eq!(user.unbonding_batches[0], 1);
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        //1.1) Withdraw
        let res = withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((70 * SCRT_TO_USCRT) as u128),
            None,
            None,
            "secretbatman",
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("Trying to withdraw more than available")
        );

        //2)Error Check: Amount available for withdraw is less than withdraw amount
        let (_init_result, mut deps) = init_helper(Some(800 * SCRT_TO_USCRT));

        //Deposit
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(100 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
            None,
        )?;
        //Request withdraw
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((60 * SCRT_TO_USCRT) as u128),
            None,
            None,
            "secretbatman",
        )?;
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        //2.1)Withdraw
        let res = withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((70 * SCRT_TO_USCRT) as u128),
            None,
            Some(config.next_unbonding_batch_time + config.unbonding_batch_duration),
            "secretbatman",
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("Trying to withdraw more than available")
        );

        //3)Error Check: Amount available for withdraw is less than withdraw amount
        let (_init_result, mut deps) = init_helper(Some(800 * SCRT_TO_USCRT));
        //Deposit
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(100 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
            None,
        )?;
        //Request withdraw
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((50 * SCRT_TO_USCRT) as u128),
            None,
            None,
            "secretbatman",
        )?;
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((50 * SCRT_TO_USCRT) as u128),
            None,
            Some(10),
            "secretbatman",
        )?;
        let user = user_info_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
        );
        assert_eq!(user.unbonding_batches.len(), 1);
        assert_eq!(user.unbonding_batches[0], 1);

        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        //3.1)Withdraw
        let res = withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((60 * SCRT_TO_USCRT) as u128),
            None,
            Some(config.next_unbonding_batch_time + config.unbonding_batch_duration),
            "secretbatman",
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("Trying to withdraw more than available")
        );

        //4) Checking user obj
        let (_init_result, mut deps) = init_helper(Some(100 * SCRT_TO_USCRT));
        //Deposit
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(100 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
            None,
        )?;
        //Request withdraw
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((100 * SCRT_TO_USCRT) as u128),
            None,
            None,
            "secretbatman",
        )?;
        //4.1) user obj check after request withdraw
        let user = user_info_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
        );
        assert_eq!(user.unbonding_batches.len(), 1);
        assert_eq!(user.unbonding_batches[0], 1);

        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;

        //Withdraw
        let config = config_read_only_unit_test_helper(&deps.storage);
        let _ = withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((100 * SCRT_TO_USCRT) as u128),
            None,
            Some(config.next_unbonding_batch_time + config.unbonding_duration),
            "secretbatman",
        )?;
        //4.2) user obj check after  withdraw
        let user = user_info_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
        );
        assert_eq!(user.unbonding_batches.len(), 0);
        assert_eq!(user.amount_delegated, Uint128::from(0 * SCRT_TO_USCRT));

        Ok(())
    }

    #[test]
    fn test_claim_rewards() -> StdResult<()> {
        //1) Testing the checks used
        //1.1) Error Check: When user just deposited right this round
        //1.2) Error Check: When user just claimed previous round and this round has not end yet
        //1) Testing the checks used
        let mut deps = deposits_filler_unit_test_helper(None);
        //1.1) Error Check: When user just deposited right this round
        let res =
            claim_rewards_unit_test_helper(deps.as_mut(), None, Some(1814400), "secretbatman");
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err(format!(
                "You are not yet able to claim rewards. Wait for this round to end"
            ))
        );
        //1.2) Error Check: When user just claimed previous round and this round has not end yet
        let _ = end_round_unit_test_helper(deps.as_mut());
        //1.2.1)Claimed first time
        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        );
        //1.2.2)Trying to claim again in the same round
        let res = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err(format!("You claimed recently!. Wait for this round to end"))
        );

        //2)Checking if the round just expired.
        let mut deps = deposits_filler_unit_test_helper(None);
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-1 day-0
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-2 day-7
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-3 day-14
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-4 day-21
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-5 day-28
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-6 day-35
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-7 day-49 Round 1 expired at day 45.
        // So round 1 will be skipped when rewards are claimed
        let round_obj = round_read_only_unit_test_helper(deps.as_ref().storage);
        let rewards_stats_for_nth_round_obj =
            rewards_stats_for_nth_round_read_only_unit_test_helper(&deps.storage, 1);
        assert!(
            round_obj.end_time
                >= rewards_stats_for_nth_round_obj
                    .rewards_expiration_date
                    .unwrap()
        );

        //2) Checking liquidity
        //2.1) User deposits once and didn't deposit for few rounds. But keeps claiming rewards
        let mut deps = deposits_filler_unit_test_helper(None); //round-1
        let _ = end_round_unit_test_helper(deps.as_mut()); //round-1
        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
            1,
        );
        assert_eq!(
            user_liquidity_snapshot_obj.liquidity.unwrap().u128(),
            45000 * SCRT_TO_USCRT
        );

        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-2
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-3
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-4
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-5
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-6
        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
            6,
        );
        assert!(user_liquidity_snapshot_obj.liquidity.is_none()); //Since no deposit are made user liquidity is zero for round 6 until user deposits/withdraw  or claim rewards

        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10000 * SCRT_TO_USCRT),
            None,
            Some((round_obj.start_time + round_obj.end_time) / 2),
            "secretbatman",
            None,
        );
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-7
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-8
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-9
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-10
        let round_obj = round_read_only_unit_test_helper(deps.as_ref().storage);
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        );
        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
            10,
        );
        assert_eq!(
            user_liquidity_snapshot_obj.liquidity.unwrap().u128(),
            70000 * SCRT_TO_USCRT
        );

        //2.2) User deposits once in round-1 and request withdraw all in round 1 as well. Then try claiming in far future round
        let mut deps = deposits_filler_unit_test_helper(None); //round-1
        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((60000 * SCRT_TO_USCRT) as u128),
            None,
            Some((round_obj.start_time + round_obj.end_time) / 2),
            "secretbatman",
        )?;
        for _ in 1..6 {
            let _ = end_round_unit_test_helper(deps.as_mut())?;
            //round-1 to 5
        }

        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        );
        for _ in 6..10 {
            let _ = end_round_unit_test_helper(deps.as_mut())?;
            //round-6 to 9
        }
        //User rewards log will exist for round-1 but not after that round
        // for i in 2..10 {
        //     let user_rewards_log_obj = user_rewards_log_unit_test_helper(
        //         &deps.storage,
        //         Uint128(i),
        //         HumanAddr("secretbatman".to_string()),
        //     );
        //     assert!(user_rewards_log_obj.liquidity.is_none());
        // }

        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10000 * SCRT_TO_USCRT),
            None,
            Some(round_obj.start_time),
            "secretbatman",
            None,
        )?;
        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        )?;
        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
            10,
        );
        assert_eq!(
            user_liquidity_snapshot_obj.liquidity.unwrap().u128(),
            10000 * SCRT_TO_USCRT
        );

        //starts round
        let (_init_result, mut deps) = init_helper(None);
        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        //deposit 2 million scrt at t0
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(2000000 * SCRT_TO_USCRT),
            Some(0u64),
            Some(round_obj.start_time),
            "secretbatman",
            None,
        );
        //end_round x 2
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-1
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-1

        //claim_rewards
        // must take 4 iterations
        // 1-  last claim round == None
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        )?;
        let user_info =
            user_info_read_only_unit_test_helper(&Addr::unchecked("secretbatman"), &deps.storage);
        assert_eq!(user_info.last_claim_rewards_round, None);

        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
            1,
        );
        assert_eq!(
            user_liquidity_snapshot_obj.tickets_used.unwrap().u128(),
            1000000
        );
        // 2- last claim round == 1
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        )?;
        let user_info =
            user_info_read_only_unit_test_helper(&Addr::unchecked("secretbatman"), &deps.storage);
        assert_eq!(user_info.last_claim_rewards_round.unwrap(), 1);

        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
            1,
        );
        assert_eq!(
            user_liquidity_snapshot_obj.tickets_used.unwrap().u128(),
            2000000
        );

        // 3- last claim round == 1
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        )?;
        let user_info =
            user_info_read_only_unit_test_helper(&Addr::unchecked("secretbatman"), &deps.storage);
        assert_eq!(user_info.last_claim_rewards_round.unwrap(), 1);

        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
            2,
        );
        assert_eq!(
            user_liquidity_snapshot_obj.tickets_used.unwrap().u128(),
            1000000
        );
        // 4- last claim round == 2`
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        )?;
        let user_info =
            user_info_read_only_unit_test_helper(&Addr::unchecked("secretbatman"), &deps.storage);

        assert_eq!(user_info.last_claim_rewards_round.unwrap(), 2);

        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps.api.addr_validate("secretbatman")?,
            &deps.storage,
            2,
        );
        assert_eq!(
            user_liquidity_snapshot_obj.tickets_used.unwrap().u128(),
            2000000
        );

        //Checking
        let (_init_result, mut deps) = init_helper(None);
        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        //deposit 2 million scrt at t0
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(2000000 * SCRT_TO_USCRT),
            Some(0u64),
            Some(round_obj.start_time),
            "secretbatman",
            None,
        );
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-1
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(
                round_obj
                    .end_time
                    .add(round_obj.rewards_expiry_duration + 1),
            ),
            "secretbatman",
        )?;

        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-2
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(
                round_obj
                    .end_time
                    .add(round_obj.rewards_expiry_duration + 1),
            ),
            "secretbatman",
        )?;
        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-3
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        )?;
        assert_eq!(
            user_liquidity_snapshot_obj.liquidity.unwrap().u128(),
            2000000 * SCRT_TO_USCRT
        );
        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-4
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        )?;
        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-5
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        )?;
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        )?;
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        )?;
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secretbatman",
        )?;

        Ok(())
    }

    #[test]
    fn test_create_viewing_key() -> StdResult<()> {
        let (init_result, mut deps) = init_helper(None);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );
        // Checking the CreateViewingKey
        let handle_msg = HandleMsg::CreateViewingKey {
            entropy: "".to_string(),
        };
        let mock_info = mock_info("secretbatman", &[]);

        let handle_result = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info.clone(),
            handle_msg,
        );
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );
        let answer: HandleAnswer = from_binary(&handle_result?.data.unwrap())?;
        // Checking if the viewing key matches
        let key = match answer {
            HandleAnswer::CreateViewingKey { key } => key,
            _ => panic!("NOPE"),
        };
        let saved_vk = read_viewing_key(&deps.storage, &mock_info.sender).unwrap();
        assert!(key.check_viewing_key(saved_vk.as_slice()));

        Ok(())
    }

    #[test]
    fn test_set_viewing_key() -> StdResult<()> {
        let (init_result, mut deps) = init_helper(None);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // Setting VK
        let handle_msg = HandleMsg::SetViewingKey {
            key: "hi lol".to_string(),
        };
        let mock_info = mock_info("bob", &[]);

        let handle_result = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info.clone(),
            handle_msg,
        );
        let unwrapped_result: HandleAnswer = from_binary(&handle_result?.data.unwrap())?;
        assert_eq!(
            to_binary(&unwrapped_result)?,
            to_binary(&HandleAnswer::SetViewingKey {
                status: ResponseStatus::Success
            })?,
        );

        // Set valid VK
        let actual_vk = ViewingKey("x".to_string().repeat(VIEWING_KEY_SIZE));
        let handle_msg = HandleMsg::SetViewingKey {
            key: actual_vk.to_string().clone(),
        };

        let handle_result = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info.clone(),
            handle_msg,
        );
        let unwrapped_result: HandleAnswer = from_binary(&handle_result?.data.unwrap())?;
        assert_eq!(
            to_binary(&unwrapped_result)?,
            to_binary(&HandleAnswer::SetViewingKey { status: Success })?,
        );
        let saved_vk = read_viewing_key(&deps.storage, &mock_info.sender).unwrap();
        //Checking set viewing key
        assert!(actual_vk.check_viewing_key(&saved_vk));
        Ok(())
    }

    //Revoke Permit has a bug in secret-toolkit waiting for it to be fixed.
    //TODO comments
    #[test]
    fn test_revoke_permit() -> StdResult<()> {
        let (init_result, mut deps) = init_helper(None);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = HandleMsg::RevokePermit {
            permit_name: "galactic_pools_batman".to_string(),
        };
        let mock_info = mock_info("secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7", &[]);

        let handle_result = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info.clone(),
            handle_msg,
        );
        let unwrapped_result: HandleAnswer = from_binary(&handle_result?.data.unwrap())?;
        assert_eq!(
            to_binary(&unwrapped_result)?,
            to_binary(&HandleAnswer::RevokePermit {
                status: ResponseStatus::Success
            })?,
        );

        //2) Checking the results of the query when permission is owner

        let token = "cosmos2contract".to_string();
        let permit: Permit<GalacticPoolsPermissions> = Permit {
            params: PermitParams {
                allowed_tokens: vec![token.clone()],
                permit_name: "galactic_pools_batman".to_string(),
                chain_id: "pulsar-2".to_string(),
                permissions: vec![GalacticPoolsPermissions::Owner],
            },
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64("AihRsQtwF56zrW7J1VP4KqTNBF5GTMXoFusm6zfIixvD")?,
                },
                signature: Binary::from_base64(
                    "q7MQVPXCwA89cBMl/dCQhZ87dxzrNhxlQUUEznf4JvhWluAhRaNvblSofu79lYGUJ0+mfH1KMCsmF+kkARHYpQ==",
                )?,
            },
        };

        let query_msg = QueryMsg::WithPermit {
            permit,
            query: QueryWithPermit::Delegated {},
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert_eq!(
            query_result.unwrap_err(),
            StdError::generic_err(format!(
                "Permit \"galactic_pools_batman\" was revoked by account \"secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7\""
            ))
        );

        Ok(())
    }

    ////////////////////////////////////// Sponsors //////////////////////////////////////
    #[test]
    fn test_sponsor() -> StdResult<()> {
        //Initializing
        let (_, mut deps) = init_helper(None);

        //0) Error Check: Denom send must me uscrt
        let deposit_results = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(30000 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
            Some("uatom"),
        )
        .unwrap_err();
        assert_eq!(
            deposit_results,
            StdError::generic_err("Wrong token given, expected uscrt found uatom")
        );

        //1)Sponsoring
        let handle_msg = HandleMsg::Sponsor {
            title: None,
            message: None,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretrichierich", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(10000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;
        //1.1)Checking pool state
        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            pool_state.total_sponsored,
            Uint128::from(10000 * SCRT_TO_USCRT)
        );

        //1.2)Total amount deposited and sponsored should be equal to 100,000 SCRT (delegated) and 10,000 SCRT(Sponsored).
        let config_obj = config_read_only_unit_test_helper(&deps.storage);
        let mut total_deposited_and_sponsored = Uint128::zero();
        for val in config_obj.validators {
            total_deposited_and_sponsored.add_assign(val.delegated);
        }
        assert_eq!(
            total_deposited_and_sponsored,
            Uint128::from((10000) * SCRT_TO_USCRT)
        );

        let sponsor_info_obj = sponsor_info_unit_test_read_only_helper(
            &deps.storage,
            &deps.api.addr_validate("secretrichierich")?,
        )?;
        assert_eq!(
            sponsor_info_obj.amount_sponsored,
            Uint128::from(10000 * SCRT_TO_USCRT)
        );

        //2) Checking global request list and global sponsors list
        //2.1)Initializing
        let (_, mut deps) = init_helper(None);
        //2.2)
        //2.2.1)Sponsoring -> User 1
        let handle_msg = HandleMsg::Sponsor {
            title: None,
            message: None,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretuser1", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(10000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;
        let sponsor_info_obj = sponsor_info_unit_test_read_only_helper(
            &deps.storage,
            &deps.api.addr_validate("secretuser1")?,
        )?;
        assert_eq!(sponsor_info_obj.addr_list_index, Some(0));
        let sponsor_state_obj = sponsor_state_unit_test_read_only_helper(&deps.storage)?;
        assert_eq!(sponsor_state_obj.offset, 1);
        //2.2.2)Sponsoring -> User 2
        let handle_msg = HandleMsg::Sponsor {
            title: Some("user2 title".to_string()),
            message: Some("user2 message".to_string()),
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretuser2", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(10000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;
        let sponsor_info_obj = sponsor_info_unit_test_read_only_helper(
            &deps.storage,
            &deps.api.addr_validate("secretuser2")?,
        )?;
        assert_eq!(sponsor_info_obj.addr_list_index, Some(1));
        let sponsor_state_obj = sponsor_state_unit_test_read_only_helper(&deps.storage)?;
        assert_eq!(sponsor_state_obj.offset, 2);

        //2.2.3)Sponsoring -> User 2 does it again
        let handle_msg = HandleMsg::Sponsor {
            title: Some("user2 changed title".to_string()),
            message: Some("user2 changed message".to_string()),
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretuser2", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(10000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;
        let sponsor_info_obj = sponsor_info_unit_test_read_only_helper(
            &deps.storage,
            &deps.api.addr_validate("secretuser2")?,
        )?;
        assert_eq!(sponsor_info_obj.addr_list_index, Some(1));
        let sponsor_state_obj = sponsor_state_unit_test_read_only_helper(&deps.storage)?;
        assert_eq!(sponsor_state_obj.offset, 2);

        //Fetch Global Request list
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
                // Messages can only be changed by message edit
                assert_eq!(vec[0].title, Some("user2 title".to_string()));
                assert_eq!(vec[0].message, Some("user2 message".to_string()));
            }
        }
        Ok(())
    }

    #[test]
    fn test_req_withdraw_sponsor() -> StdResult<()> {
        //Initializing
        let (_, mut deps) = init_helper(None);
        //Sponsoring some amount
        let handle_msg = HandleMsg::Sponsor {
            title: None,
            message: None,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretrichierich", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(10000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;

        //1)Checking validity of the amount
        let handle_msg = HandleMsg::SponsorRequestWithdraw {
            amount: Uint128::from(150000 * SCRT_TO_USCRT),
        };
        let res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretrichierich", &[]),
            handle_msg,
        );

        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err(format!(
                "insufficient funds to redeem: balance=10000000000, required=150000000000",
            ))
        );

        //2)Making request to unbond
        let handle_msg = HandleMsg::SponsorRequestWithdraw {
            amount: Uint128::from(10000 * SCRT_TO_USCRT),
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretrichierich", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(15000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;

        //2.1)Unbonding batch
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        //2.2)Checking pool state
        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        assert_eq!(pool_state.total_sponsored, Uint128::from(0 * SCRT_TO_USCRT));
        assert_eq!(
            pool_state.rewards_returned_to_contract,
            Uint128::from((20) * SCRT_TO_USCRT)
        );

        let sponsor = sponsor_info_helper_read_only(
            &deps.storage,
            &deps.api.addr_validate("secretrichierich")?,
        )?;
        assert_eq!(sponsor.amount_sponsored.u128(), 0 * SCRT_TO_USCRT);
        //3.2) Checking unbonding information
        assert_eq!(sponsor.unbonding_batches[0], 1);
        assert_eq!(sponsor.unbonding_batches.len(), 1);

        //3.3)Checking validators  Total amount deposited and sponsored should be equal to 100,000 SCRT (delegated)
        let config_obj = config_read_only_unit_test_helper(&deps.storage);
        let mut total_deposited_and_sponsored = Uint128::zero();
        for val in config_obj.validators {
            total_deposited_and_sponsored = total_deposited_and_sponsored.add(val.delegated);
        }
        assert_eq!(
            total_deposited_and_sponsored,
            Uint128::from((0) * SCRT_TO_USCRT)
        );

        //3.4)Checking the validator stats
        //Done -> check test_validator_walk_through
        Ok(())
    }

    #[test]
    fn test_withdraw_sponsor() -> StdResult<()> {
        let mut deps = deposits_filler_unit_test_helper(Some(10000 * SCRT_TO_USCRT));
        //Deposit
        let handle_msg = HandleMsg::Sponsor {
            title: None,
            message: None,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretrichierich", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(10000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;
        //Request withdraw
        let handle_msg = HandleMsg::SponsorRequestWithdraw {
            amount: Uint128::from(10000 * SCRT_TO_USCRT),
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretrichierich", &[]),
            handle_msg,
        )?;
        //Batch unbond
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;

        //1) Error Check: Amount available for withdraw is less than withdraw amount
        let handle_msg = HandleMsg::SponsorWithdraw {
            amount: Uint128::from(1000000 * SCRT_TO_USCRT),
        };

        let res = execute(
            deps.as_mut(),
            custom_mock_env(
                None,
                Some(config.next_unbonding_batch_time + config.unbonding_duration.add(1)),
                None,
                None,
            ),
            mock_info("secretrichierich", &[]),
            handle_msg,
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("Trying to withdraw more than available")
        );
        //1.1)sponsor check after request withdraw
        let sponsor = sponsor_info_helper_read_only(
            &deps.storage,
            &deps.api.addr_validate("secretrichierich")?,
        )?;
        assert_eq!(sponsor.amount_sponsored.u128(), 0 * SCRT_TO_USCRT);
        //1.2) Checking unbonding information
        assert_eq!(sponsor.unbonding_batches[0], 1);
        assert_eq!(sponsor.unbonding_batches.len(), 1);

        //2) Withdraw
        let handle_msg = HandleMsg::SponsorWithdraw {
            amount: Uint128::from(10000 * SCRT_TO_USCRT),
        };

        let _res = execute(
            deps.as_mut(),
            custom_mock_env(
                None,
                Some(config.next_unbonding_batch_time + config.unbonding_duration.add(1)),
                None,
                None,
            ),
            mock_info("secretrichierich", &[]),
            handle_msg,
        )?;

        //2.1)sponsor check after request withdraw
        let sponsor = sponsor_info_helper_read_only(
            &deps.storage,
            &deps.api.addr_validate("secretrichierich")?,
        )?;
        assert_eq!(sponsor.amount_sponsored.u128(), 0 * SCRT_TO_USCRT);
        //2.2) Checking unbonding information
        assert_eq!(sponsor.unbonding_batches.len(), 0);

        //2.3) Checking pool state
        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        assert_eq!(pool_state.total_sponsored, Uint128::from(0 * SCRT_TO_USCRT));

        Ok(())
    }

    ////////////////////////////////////// Sponsors + Admin //////////////////////////////////////
    /// try_sponsor + try_sponsor_request_withdraw +
    /// try_sponsor_withdraw + try_sponsor_message_edit +
    /// try_review_sponsor_messages -> Admin
    #[test]
    fn test_sponsors_walkthrough() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        //1)Deposit with and without the message
        //1.1) Deposit with the message
        let handle_msg = HandleMsg::Sponsor {
            title: Some("Bruce Wayne".to_string()),
            message: Some("I'm rich".to_string()),
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretbrucewayne", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(10000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;
        //1.2) Query the Sponsor Message Request
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
            }
        }
        //1.3) Deposit without the message
        let handle_msg = HandleMsg::Sponsor {
            title: None,
            message: None,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretbrucewayne", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(10000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;
        //1.4) Query the Sponsor Message Request
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
            }
        }

        //1.5) Query the Sponsors List
        let query_msg = QueryMsg::Sponsors {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
                for v in vec {
                    assert_eq!(v.amount_sponsored.u128(), 20000 * SCRT_TO_USCRT);
                    assert_eq!(v.message, None);
                    assert_eq!(v.title, None);
                    assert_eq!(v.addr_list_index, Some(0));
                }
            }
        }

        //2) Review Sponsor Request Messages
        let mut decision_vec: Vec<Review> = vec![];
        decision_vec.push(Review {
            index: 0,
            is_accpeted: true,
        });
        let handle_msg = HandleMsg::ReviewSponsors {
            decisions: decision_vec,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("reviewer", &[]),
            handle_msg,
        )?;

        //2.1) Query the sponsor message request
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 0);
                assert_eq!(len, 0);
            }
        }

        //2.2) Query the Sponsors List
        let query_msg = QueryMsg::Sponsors {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
                for v in vec {
                    assert_eq!(v.amount_sponsored.u128(), 20000 * SCRT_TO_USCRT);
                    assert_eq!(v.title, Some("Bruce Wayne".to_string()));
                    assert_eq!(v.message, Some("I'm rich".to_string()));
                    assert_eq!(v.addr_list_index, Some(0));
                }
            }
        }

        let sponsor_stats = SPONSOR_STATS_STORE.load(deps.as_ref().storage)?;
        assert_eq!(sponsor_stats.offset, 1);

        //3) Sponsor Request withdraw all amount
        let handle_msg = HandleMsg::SponsorRequestWithdraw {
            amount: Uint128::from(20000 * SCRT_TO_USCRT),
        };
        execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretbrucewayne", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(20000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;
        let sponsor_stats = SPONSOR_STATS_STORE.load(deps.as_ref().storage)?;
        assert_eq!(sponsor_stats.offset, 0);

        //3.1) Query the Sponsors List
        let query_msg = QueryMsg::Sponsors {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 0);
                assert_eq!(len, 0);
            }
        }

        //4) try to edit message after you sponsor with and without the message
        let (_init_result, mut deps) = init_helper(None);

        let handle_msg = HandleMsg::Sponsor {
            title: None,
            message: None,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretbrucewayne", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(10000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 0);
                assert_eq!(len, 0);
            }
        }

        let handle_msg = HandleMsg::SponsorMessageEdit {
            title: Some("Bruce Wayne".to_string()),
            message: Some("I'm rich".to_string()),
            delete_title: false,
            delete_message: false,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretbrucewayne", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(1 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
            }
        }

        Ok(())
    }

    ////////////////////////////////////// Validators //////////////////////////////////////
    #[test]
    fn test_validator_walk_through() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        //1)Checking validators after deposit 1
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
            None,
        );
        let config = config_read_only_unit_test_helper(&deps.storage);

        assert_eq!(config.validators[0].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 0);
        assert_eq!(config.validators[2].delegated.u128(), 0 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_delegation, 1);

        //deposit 10 scrt
        //2)Checking validators after deposit 2
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
            None,
        );
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.validators[0].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[2].delegated.u128(), 00 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_delegation, 2);

        //3)Checking validators after deposit 3
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
            None,
        );
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.validators[0].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[2].delegated.u128(), 10 * SCRT_TO_USCRT);
        //Since there are 3 validators so 3%3 == 0
        assert_eq!(config.next_validator_for_delegation, 0);

        //4) User check
        let user_info =
            user_info_read_only_unit_test_helper(&Addr::unchecked("secretbatman"), &deps.storage);
        assert_eq!(user_info.amount_delegated.u128(), 30 * SCRT_TO_USCRT);

        //Request withdraw
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((10 * SCRT_TO_USCRT) as u128),
            None,
            None,
            "secretbatman",
        );
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.validators[0].delegated.u128(), 0 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[2].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_unbonding, 1);

        //Request withdraw
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((10 * SCRT_TO_USCRT) as u128),
            None,
            None,
            "secretbatman",
        );
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.validators[0].delegated.u128(), 0 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 0);
        assert_eq!(config.validators[2].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_unbonding, 2);

        //Request withdraw
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((10 * SCRT_TO_USCRT) as u128),
            None,
            None,
            "secretbatman",
        );
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.validators[0].delegated.u128(), 0 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 0);
        assert_eq!(config.validators[2].delegated.u128(), 0 * SCRT_TO_USCRT);
        //Since there are 3 validators so 3%3 == 0
        assert_eq!(config.next_validator_for_unbonding, 0);

        //5)Checking user,sponsor and admin reserves in combinations.
        let (_init_result, mut deps) = init_helper(None);

        //1)Checking validators after sponsor
        let _ = sponsor_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "Bruce Wayne",
            None,
        );
        let config = config_read_only_unit_test_helper(&deps.storage);

        assert_eq!(config.validators[0].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 0);
        assert_eq!(config.validators[2].delegated.u128(), 0 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_delegation, 1);

        //deposit 10 scrt
        // 2)Checking validators after sponsor 2
        let _ = sponsor_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "Bill gates",
            None,
        );
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.validators[0].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[2].delegated.u128(), 0 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_delegation, 2);

        let _ = sponsor_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "Jordan Xavier",
            None,
        );
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.validators[0].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[2].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_delegation, 0);

        //4) Checking sponsors
        let sponsor_info = sponsor_info_unit_test_read_only_helper(
            &deps.storage,
            &Addr::unchecked("Bruce Wayne"),
        )?;
        assert_eq!(sponsor_info.amount_sponsored.u128(), 10 * SCRT_TO_USCRT);
        let sponsor_info =
            sponsor_info_unit_test_read_only_helper(&deps.storage, &Addr::unchecked("Bill gates"))?;
        assert_eq!(sponsor_info.amount_sponsored.u128(), 10 * SCRT_TO_USCRT);
        let sponsor_info = sponsor_info_unit_test_read_only_helper(
            &deps.storage,
            &Addr::unchecked("Jordan Xavier"),
        )?;
        assert_eq!(sponsor_info.amount_sponsored.u128(), 10 * SCRT_TO_USCRT);

        let _ = sponsor_request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((10 * SCRT_TO_USCRT) as u128),
            None,
            None,
            "Bruce Wayne",
        )?;
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.validators[0].delegated.u128(), 00 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[2].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_unbonding, 1);

        let _ = sponsor_request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((10 * SCRT_TO_USCRT) as u128),
            None,
            None,
            "Bill gates",
        )?;
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.validators[0].delegated.u128(), 00 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 00 * SCRT_TO_USCRT);
        assert_eq!(config.validators[2].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_unbonding, 2);

        let _ = sponsor_request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from((10 * SCRT_TO_USCRT) as u128),
            None,
            None,
            "Jordan Xavier",
        )?;
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.validators[0].delegated.u128(), 00 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 00 * SCRT_TO_USCRT);
        assert_eq!(config.validators[2].delegated.u128(), 00 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_unbonding, 0);

        //Checking reserves and validators
        let mut deps = deposits_filler_unit_test_helper(Some(800 * SCRT_TO_USCRT));
        for _ in 0..6 {
            let _ = end_round_unit_test_helper(deps.as_mut())?;
        }
        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);

        let config_obj = config_read_only_unit_test_helper(&deps.storage);
        let mut total_delegated_sponsored_reserves = Uint128::zero();
        for val in config_obj.validators {
            total_delegated_sponsored_reserves =
                total_delegated_sponsored_reserves.add(val.delegated);
        }
        assert_eq!(
            total_delegated_sponsored_reserves,
            Uint128::from((100000) * SCRT_TO_USCRT).add(pool_state.total_reserves)
        );

        //Testing user deposits, sponsors and reserves in unity
        let (_init_result, mut deps) = init_helper(None);

        //1)deposits
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
            None,
        );
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.validators[0].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 0);
        assert_eq!(config.validators[2].delegated.u128(), 0 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_delegation, 1);

        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            pool_state.total_delegated,
            Uint128::from(10 * SCRT_TO_USCRT)
        );

        //2)sponsors
        let _ = sponsor_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "Bruce Wayne",
            None,
        );
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.validators[0].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[1].delegated.u128(), 10 * SCRT_TO_USCRT);
        assert_eq!(config.validators[2].delegated.u128(), 0 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_delegation, 2);

        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            pool_state.total_sponsored,
            Uint128::from(10 * SCRT_TO_USCRT)
        );

        //3)reserves
        for _ in 0..10 {
            deposit_unit_test_helper(
                deps.as_mut(),
                Uint128::from(1 * SCRT_TO_USCRT),
                None,
                None,
                "Jordan Xavier",
                None,
            )?;
        }
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            config.validators[0].delegated.u128(),
            (10 + 3) * SCRT_TO_USCRT
        );
        assert_eq!(
            config.validators[1].delegated.u128(),
            (10 + 3) * SCRT_TO_USCRT
        );
        assert_eq!(config.validators[2].delegated.u128(), 4 * SCRT_TO_USCRT);
        assert_eq!(config.next_validator_for_delegation, 0);

        let mut total_delegated_sponsored_reserves: Uint128 = Uint128::zero();
        for val in config.validators {
            total_delegated_sponsored_reserves.add_assign(val.delegated);
        }

        assert_eq!(
            total_delegated_sponsored_reserves,
            Uint128::from((30) * SCRT_TO_USCRT)
        );

        for _ in 0..6 {
            let _ = end_round_unit_test_helper(deps.as_mut())?;
        }
        let config = config_read_only_unit_test_helper(&deps.storage);
        let mut total_delegated_sponsored_reserves: Uint128 = Uint128::zero();
        for val in &config.validators {
            total_delegated_sponsored_reserves =
                total_delegated_sponsored_reserves.add(val.delegated);
        }
        // Still 0 6%3==0
        assert_eq!(config.next_validator_for_delegation, 0);

        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            total_delegated_sponsored_reserves,
            Uint128::from((30) * SCRT_TO_USCRT).add(pool_state.total_reserves)
        );

        //Testing user user_request_withdraw, sponsors_request_withdraw and reserves_request_withdraw
        //1)user_request_withdraw
        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        let _ = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            Some(round_obj.start_time),
            "secretbatman",
        )?;
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        let mut total_delegated_sponsored_reserves: Uint128 = Uint128::zero();
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.next_validator_for_unbonding, 1);

        for val in &config.validators {
            total_delegated_sponsored_reserves =
                total_delegated_sponsored_reserves.add(val.delegated);
        }
        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            total_delegated_sponsored_reserves,
            Uint128::from((20) * SCRT_TO_USCRT).add(pool_state.total_reserves)
        );

        //2)sponsor_request_withdraw
        sponsor_request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "Bruce Wayne",
        )?;

        let mut total_delegated_sponsored_reserves: Uint128 = Uint128::zero();
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.next_validator_for_unbonding, 1);

        for val in &config.validators {
            total_delegated_sponsored_reserves =
                total_delegated_sponsored_reserves.add(val.delegated);
        }
        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            total_delegated_sponsored_reserves,
            Uint128::from((20) * SCRT_TO_USCRT).add(pool_state.total_reserves)
        );

        //3)reserves
        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        let reserves_req_withdraw = pool_state.total_reserves;
        if !reserves_req_withdraw.is_zero() {
            let handle_msg = HandleMsg::RequestReservesWithdraw {
                amount: reserves_req_withdraw,
            };
            execute(
                deps.as_mut(),
                custom_mock_env(None, None, None, None),
                mock_info("admin", &[]),
                handle_msg,
            )?;
        }
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        let mut total_delegated_sponsored_reserves: Uint128 = Uint128::zero();
        let config = config_read_only_unit_test_helper(&deps.storage);
        for val in &config.validators {
            total_delegated_sponsored_reserves =
                total_delegated_sponsored_reserves.add(val.delegated);
        }
        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            total_delegated_sponsored_reserves,
            Uint128::from((10) * SCRT_TO_USCRT)
        );
        assert_eq!(
            total_delegated_sponsored_reserves,
            pool_state
                .total_delegated
                .add(pool_state.total_reserves)
                .add(pool_state.total_sponsored)
        );
        Ok(())
    }

    #[test]
    fn test_sponsor_edit_message() -> StdResult<()> {
        let (_, mut deps) = init_helper(None);

        //1)Sending edit message without any sponsoring
        let handle_msg = HandleMsg::SponsorMessageEdit {
            title: None,
            message: None,
            delete_message: false,
            delete_title: false,
        };
        let res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretuser1", &[]),
            handle_msg,
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err(format!("Sponsor to avail this option"))
        );

        //2)Sponsoring
        let handle_msg = HandleMsg::Sponsor {
            title: Some("user1 title".to_string()),
            message: Some("user1 message".to_string()),
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretuser1", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(10000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;
        //Fetch Global Request list
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
                assert_eq!(vec[0].title, Some("user1 title".to_string()));
                assert_eq!(vec[0].message, Some("user1 message".to_string()));
            }
        }

        //2.1)Sending edit message  sponsoring
        let handle_msg = HandleMsg::SponsorMessageEdit {
            title: Some("user1 title updated".to_string()),
            message: Some("user1 message updated".to_string()),
            delete_message: false,
            delete_title: false,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretuser1", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(1 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        );
        //Fetch Global Request list
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
                assert_eq!(vec[0].title, Some("user1 title updated".to_string()));
                assert_eq!(vec[0].message, Some("user1 message updated".to_string()));
            }
        }

        //2) Review Sponsor Request Messages
        let mut decision_vec: Vec<Review> = vec![];
        decision_vec.push(Review {
            index: 0,
            is_accpeted: true,
        });
        let handle_msg = HandleMsg::ReviewSponsors {
            decisions: decision_vec,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("reviewer", &[]),
            handle_msg,
        )?;

        //Fetch Global Request list
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 0);
                assert_eq!(len, 0);
            }
        }

        let query_msg = QueryMsg::Sponsors {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
                for v in vec {
                    assert_eq!(v.amount_sponsored.u128(), 10000 * SCRT_TO_USCRT);
                    assert_eq!(v.message, Some("user1 message updated".to_string()));
                    assert_eq!(v.title, Some("user1 title updated".to_string()));
                    assert_eq!(v.addr_list_index, Some(0));
                }
            }
        }

        //2.2)deleteing edit message  sponsoring
        let handle_msg = HandleMsg::SponsorMessageEdit {
            title: None,
            message: None,
            delete_message: true,
            delete_title: false,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretuser1", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(1 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        );

        let query_msg = QueryMsg::Sponsors {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
                for v in vec {
                    assert_eq!(v.amount_sponsored.u128(), 10000 * SCRT_TO_USCRT);
                    assert_eq!(v.message, None);
                    assert_eq!(v.title, Some("user1 title updated".to_string()));
                    assert_eq!(v.addr_list_index, Some(0));
                }
            }
        }

        Ok(())
    }

    ////////////////////////////////////// Admin //////////////////////////////////////
    #[test]
    fn test_review_sponsor_messages() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        //1)Deposit with and without the message
        //1.1) Deposit with the message
        let handle_msg = HandleMsg::Sponsor {
            title: Some("Bruce Wayne".to_string()),
            message: Some("I'm rich".to_string()),
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("secretbrucewayne", &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128::from(10000 * SCRT_TO_USCRT),
            }]),
            handle_msg,
        )?;
        //1.2) Query the Sponsor Message Request
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(vec[0].title, Some("Bruce Wayne".to_string()));
                assert_eq!(vec[0].message, Some("I'm rich".to_string()));
                assert_eq!(len, 1);
            }
        }

        //1.3) Query the Sponsors List
        let query_msg = QueryMsg::Sponsors {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
                for v in vec {
                    assert_eq!(v.amount_sponsored.u128(), 10000 * SCRT_TO_USCRT);
                    assert_eq!(v.message, None);
                    assert_eq!(v.title, None);
                    assert_eq!(v.addr_list_index, Some(0));
                }
            }
        }

        //2) Review Sponsor Request Messages
        let mut decision_vec: Vec<Review> = vec![];
        decision_vec.push(Review {
            index: 0,
            is_accpeted: true,
        });
        let handle_msg = HandleMsg::ReviewSponsors {
            decisions: decision_vec,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("reviewer", &[]),
            handle_msg,
        )?;

        //2.1) Query the sponsor message request
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 0);
                assert_eq!(len, 0);
            }
        }

        //1.3) Query the Sponsors List
        let query_msg = QueryMsg::Sponsors {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
                for v in vec {
                    assert_eq!(v.amount_sponsored.u128(), 10000 * SCRT_TO_USCRT);
                    assert_eq!(v.message, Some("I'm rich".to_string()));
                    assert_eq!(v.title, Some("Bruce Wayne".to_string()));
                    assert_eq!(v.addr_list_index, Some(0));
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_remove_sponsor_credentials() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        //1)Sponsoring by 10 users
        for i in 0..10 {
            let sponsor = format!("user{}", i);

            let handle_msg = HandleMsg::Sponsor {
                title: Some(format!("user{} Title", i)),
                message: Some(format!("user{} Message", i)),
            };
            let _res = execute(
                deps.as_mut(),
                custom_mock_env(None, None, None, None),
                mock_info(&sponsor, &[Coin {
                    denom: "uscrt".to_string(),
                    amount: Uint128::from(10000 * SCRT_TO_USCRT),
                }]),
                handle_msg,
            )?;
        }

        //2)Accepting their sponsor requests
        let mut decision_vec: Vec<Review> = vec![];
        for i in 0..10 {
            decision_vec.push(Review {
                index: i,
                is_accpeted: true,
            });
        }
        let handle_msg = HandleMsg::ReviewSponsors {
            decisions: decision_vec,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("reviewer", &[]),
            handle_msg,
        )?;

        //3)Removing credentials of even numbers only
        let mut decisions_vec = vec![];
        for i in 0..10 {
            if i % 2 == 0 {
                decisions_vec.push(RemoveSponsorCredentialsDecisions {
                    index: i,
                    remove_sponsor_title: true,
                    remove_sponsor_message: true,
                })
            }
        }
        let handle_msg = HandleMsg::RemoveSponsorCredentials {
            decisions: decisions_vec,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("reviewer", &[]),
            handle_msg,
        )?;

        //4)Checking if removing actually worked
        for i in 0..10 {
            let sponsor = format!("user{}", i);

            let sponsor_obj =
                sponsor_info_helper_read_only(&deps.storage, &deps.api.addr_validate(&sponsor)?)?;

            if i % 2 == 0 {
                assert_eq!(sponsor_obj.title, None);
                assert_eq!(sponsor_obj.message, None);
            } else {
                assert_eq!(sponsor_obj.title, Some(format!("user{} Title", i)));
                assert_eq!(sponsor_obj.message, Some(format!("user{} Message", i)));
            }
        }

        Ok(())
    }

    #[test]
    fn test_end_round() -> StdResult<()> {
        let mut deps = deposits_filler_unit_test_helper(None);

        //1)Checking if the round is closed by Triggerer/Admin
        let handle_msg = HandleMsg::EndRound {};

        let handle_result = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(3600 * 24 * 7), None, None),
            mock_info("not-triggerer", &[]),
            handle_msg,
        );
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::generic_err(
                "This is an triggerer command. Triggerer commands can only be run from triggerer address",
            )
        );

        //2)Checking if the round can be closed -- time-wise
        let handle_msg = HandleMsg::EndRound {};
        let handle_result = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(3600 * 24 * 7 - 1), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        );
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::generic_err("Round end time is in the future",)
        );

        // DEBUG  pool state rewards checking -- replace in code
        // for reward in rewards_obj {
        //     dbg!("Reward {}", reward.reward);
        //     total_rewards.add_assign(reward.reward);
        // }

        //Calculating total rewards recieved
        //*DEBUG
        //dbg!("Total Rewards{}", total_rewards);

        //3) Checking when rewards are zero
        let (_init_result, mut deps) = init_helper(None);
        deps.querier.update_staking("uscrt", &[], &[]);
        let respose = end_round_unit_test_helper(deps.as_mut())?;
        let raw = respose.data.unwrap();
        let status: HandleAnswer = from_binary(&raw)?;
        let msg = HandleAnswer::EndRound { status: Success };
        assert_eq!(msg, status);
        let rewards_stats_for_nth_round_obj =
            rewards_stats_for_nth_round_read_only_unit_test_helper(&deps.storage, 1);
        assert_eq!(
            rewards_stats_for_nth_round_obj.total_rewards,
            Uint128::zero()
        );

        //4) Checking total rewards distribution --> winning_amount + triggerer's share + shade's share + Galactic Pools share.
        let mut deps = deposits_filler_unit_test_helper(None);
        //*Total rewards are 130 scrts -> calculated as 10 scrts per one deposit, total 10 deposits are made
        //and 10scrt when ending round so withdrawing rewards from all three validators hence 30 scrts.
        let res = end_round_unit_test_helper(deps.as_mut())?;
        assert_eq!(res.messages.len(), 6);
        //*Debug
        // println!("Shade share {}", shade_share);
        // println!("Galactic Pool share {}", galactic_pools_share);
        // println!("Triggerer share {}", trigger_share);
        // println!("Winning amount {}", winning_amount);
        // let total = shade_share + galactic_pools_share + trigger_share + winning_amount;
        // println!(
        //     "Original: {}, calculated: {}, Difference: {}",
        //     test_total_rewards,
        //     total,
        //     test_total_rewards - total
        // );

        //5) Checking config + round + pool
        let (_init_result, deps) = init_helper(None);
        let config_obj = config_read_only_unit_test_helper(deps.as_ref().storage);
        assert_eq!(config_obj.next_validator_for_delegation, 0);
        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        assert_eq!(round_obj.entropy.len(), 32);
        //* After round ends
        let mut deps = deposits_filler_unit_test_helper(None);
        let _ = end_round_unit_test_helper(deps.as_mut())?;
        //*Total 10 delegations made in deposit_unit_test_simple_helper_filler() and end_round_my_mocked_querier_unit_test_helper()
        //* 10%3 = 1
        let config_obj = config_read_only_unit_test_helper(deps.as_ref().storage);
        assert_eq!(config_obj.next_validator_for_delegation, 1);
        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        assert_eq!(round_obj.entropy.len(), 80);

        //6)Checking increase in reserves
        //Checking  pool state
        let pool_state_obj = pool_state_read_only_unit_test_helper(deps.as_ref().storage);
        assert_eq!(pool_state_obj.total_reserves, Uint128::zero());
        //*Expiry is almost 45 days after round ended. round ends after 7 days. 7*8 = 48days
        //*since on claims are made all the winning amount comes back as reserves and some is propogated.
        for _ in 0..7 {
            let _ = end_round_unit_test_helper(deps.as_mut())?;
        }
        let pool_state_obj = pool_state_read_only_unit_test_helper(deps.as_ref().storage);
        assert_ne!(pool_state_obj.total_reserves, Uint128::zero());

        //7) Checking  pool state liquidity
        let mut deps = deposits_filler_unit_test_helper(None);
        let _ = end_round_unit_test_helper(deps.as_mut())?;
        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        let pool_state_liquidity = pool_state_liquidity_snapshot_read_only_unit_test_helper(
            deps.as_ref().storage,
            round_obj.current_round_index - 1,
        );
        //We already know that in filler helper function, secretbatman deposits 30,000 scrt in the middle average liquidity to 85000
        //while total amount delegated is 100,000
        assert_eq!(
            pool_state_liquidity.total_liquidity.unwrap(),
            Uint128::from(85000 * SCRT_TO_USCRT)
        );
        assert_eq!(
            pool_state_liquidity.total_delegated.unwrap(),
            Uint128::from(100000 * SCRT_TO_USCRT)
        );
        //7.1) if no delegations are made during the round
        let _ = end_round_unit_test_helper(deps.as_mut())?;
        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        let pool_state_liquidity = pool_state_liquidity_snapshot_read_only_unit_test_helper(
            deps.as_ref().storage,
            round_obj.current_round_index - 1,
        );
        assert_eq!(
            pool_state_liquidity.total_liquidity.unwrap(),
            Uint128::from(100000 * SCRT_TO_USCRT)
        );
        assert_eq!(
            pool_state_liquidity.total_delegated.unwrap(),
            Uint128::from(100000 * SCRT_TO_USCRT)
        );

        //8)Claim the unclaimed rewards that have been expired and checking rewards_stats_for_nth_round_unit_test_helper
        //*Checking expiration date
        let mut deps = deposits_filler_unit_test_helper(None);
        let _ = end_round_unit_test_helper(deps.as_mut())?;

        let rewards_stats_for_nth_round_obj =
            rewards_stats_for_nth_round_read_only_unit_test_helper(deps.as_ref().storage, 1);

        let round_obj = round_read_only_unit_test_helper(deps.as_ref().storage);
        let time_stamp = Timestamp::from_seconds(round_obj.start_time);

        assert_eq!(
            rewards_stats_for_nth_round_obj
                .rewards_expiration_date
                .unwrap(),
            3600 * 24 * 45 + time_stamp.seconds()
        );

        let mut deps = deposits_filler_unit_test_helper(None);
        let _ = end_round_unit_test_helper(deps.as_mut())?;
        let pool_state = pool_state_read_only_unit_test_helper(deps.as_ref().storage);
        assert_eq!(pool_state.rewards_returned_to_contract, Uint128::zero());
        assert_eq!(pool_state.total_reserves, Uint128::zero());
        //*Checking increase in reserves
        let pool_state_obj = pool_state_read_only_unit_test_helper(deps.as_ref().storage);
        assert_eq!(pool_state_obj.total_reserves, Uint128::zero());
        //* Expiry is almost 45 days after round ended. round ends after 7 days. 7*8 = 48days
        //* since on claims are made all the winning amount comes back as reserves and some is propogated.
        for _ in 0..2 {
            for _ in 0..7 {
                let _ = end_round_unit_test_helper(deps.as_mut())?;
            }
        }
        let pool_state_obj = pool_state_read_only_unit_test_helper(deps.as_ref().storage);
        assert_ne!(pool_state_obj.total_reserves, Uint128::zero());

        Ok(())
    }

    #[test]
    fn test_add_admin() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        // 1) Error caused by not admin command
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("not-admin", &[]);
        let handle_msg = HandleMsg::AddAdmin {
            admin: Addr::unchecked("admin2"),
        };
        let res = execute(deps.as_mut(), env, info, handle_msg).unwrap_err();
        assert_eq!(
            res,
            StdError::generic_err(
                "This is an admin command. Admin commands can only be run from admin address",
            )
        );

        // 2) Adding admin2
        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.admins.len(), 1);

        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);
        let handle_msg = HandleMsg::AddAdmin {
            admin: Addr::unchecked("admin2"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg)?;

        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.admins.len(), 2);
        assert!(config.admins.contains(&Addr::unchecked("admin")));
        assert!(config.admins.contains(&Addr::unchecked("admin2")));

        // 3)Error adding admin2 again
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);
        let handle_msg = HandleMsg::AddAdmin {
            admin: Addr::unchecked("admin2"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg).unwrap_err();
        assert_eq!(_res, StdError::generic_err("This address already exisits",));

        //4)Adding admin3 from admin2
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin2", &[]);
        let handle_msg = HandleMsg::AddAdmin {
            admin: Addr::unchecked("admin3"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg)?;
        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.admins.len(), 3);
        assert!(config.admins.contains(&Addr::unchecked("admin")));
        assert!(config.admins.contains(&Addr::unchecked("admin2")));
        assert!(config.admins.contains(&Addr::unchecked("admin3")));
        Ok(())
    }

    #[test]
    fn test_remove_admin() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        //1) Error caused by not admin command
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("not-admin", &[]);
        let handle_msg = HandleMsg::RemoveAdmin {
            admin: Addr::unchecked("admin"),
        };
        let res = execute(deps.as_mut(), env, info, handle_msg).unwrap_err();
        assert_eq!(
            res,
            StdError::generic_err(
                "This is an admin command. Admin commands can only be run from admin address",
            )
        );

        //2)Adding admin2
        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.admins.len(), 1);

        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);
        let handle_msg = HandleMsg::AddAdmin {
            admin: Addr::unchecked("admin2"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg)?;

        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.admins.len(), 2);
        assert!(config.admins.contains(&Addr::unchecked("admin")));
        assert!(config.admins.contains(&Addr::unchecked("admin2")));

        //3)Remove admin from admin2
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin2", &[]);
        let handle_msg = HandleMsg::RemoveAdmin {
            admin: Addr::unchecked("admin"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg)?;
        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.admins.len(), 1);
        assert!(config.admins.contains(&Addr::unchecked("admin2")));

        Ok(())
    }

    #[test]
    fn test_add_triggerer() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        //1)Error caused by not admin command
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("not-admin", &[]);
        let handle_msg = HandleMsg::AddTriggerer {
            triggerer: Addr::unchecked("triggerer2"),
        };
        let res = execute(deps.as_mut(), env, info, handle_msg).unwrap_err();
        assert_eq!(
            res,
            StdError::generic_err(
                "This is an admin command. Admin commands can only be run from admin address",
            )
        );

        //2)Adding triggerer2
        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.triggerers.len(), 1);

        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);
        let handle_msg = HandleMsg::AddTriggerer {
            triggerer: Addr::unchecked("triggerer2"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg)?;

        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.triggerers.len(), 2);
        assert!(config.triggerers.contains(&Addr::unchecked("triggerer")));
        assert!(config.triggerers.contains(&Addr::unchecked("triggerer2")));

        // 3)Error adding triggerer2 again
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);
        let handle_msg = HandleMsg::AddTriggerer {
            triggerer: Addr::unchecked("triggerer2"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg).unwrap_err();
        assert_eq!(_res, StdError::generic_err("This address already exisits",));
        Ok(())
    }

    #[test]
    fn test_remove_triggerer() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        //1) Error caused by not admin command
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("not-admin", &[]);
        let handle_msg = HandleMsg::RemoveTriggerer {
            triggerer: Addr::unchecked("triggerer2"),
        };
        let res = execute(deps.as_mut(), env, info, handle_msg).unwrap_err();
        assert_eq!(
            res,
            StdError::generic_err(
                "This is an admin command. Admin commands can only be run from admin address",
            )
        );

        //2)Adding triggerer2
        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.triggerers.len(), 1);
        assert!(config.triggerers.contains(&Addr::unchecked("triggerer")));

        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);
        let handle_msg = HandleMsg::AddTriggerer {
            triggerer: Addr::unchecked("triggerer2"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg)?;

        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.triggerers.len(), 2);
        assert!(config.triggerers.contains(&Addr::unchecked("triggerer")));
        assert!(config.triggerers.contains(&Addr::unchecked("triggerer2")));

        //3)Remove admin from triggerer2
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);
        let handle_msg = HandleMsg::RemoveTriggerer {
            triggerer: Addr::unchecked("triggerer"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg)?;
        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.triggerers.len(), 1);
        assert!(config.triggerers.contains(&Addr::unchecked("triggerer2")));

        Ok(())
    }

    #[test]
    fn test_add_reviewer() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        //1)Error caused by not admin command
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("not-admin", &[]);
        let handle_msg = HandleMsg::AddReviewer {
            reviewer: Addr::unchecked("reviewer2"),
        };
        let res = execute(deps.as_mut(), env, info, handle_msg).unwrap_err();
        assert_eq!(
            res,
            StdError::generic_err(
                "This is an admin command. Admin commands can only be run from admin address",
            )
        );

        //2)Adding reviewer2
        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.reviewers.len(), 1);

        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);
        let handle_msg = HandleMsg::AddReviewer {
            reviewer: Addr::unchecked("reviewer2"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg)?;

        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.reviewers.len(), 2);
        assert!(config.reviewers.contains(&Addr::unchecked("reviewer")));
        assert!(config.reviewers.contains(&Addr::unchecked("reviewer2")));

        // 3)Error adding reviewer2 again
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);
        let handle_msg = HandleMsg::AddReviewer {
            reviewer: Addr::unchecked("reviewer2"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg).unwrap_err();
        assert_eq!(_res, StdError::generic_err("This address already exisits",));
        Ok(())
    }

    #[test]
    fn test_remove_reviewer() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        //1) Error caused by not admin command
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("not-admin", &[]);
        let handle_msg = HandleMsg::RemoveReviewer {
            reviewer: Addr::unchecked("reviewer2"),
        };
        let res = execute(deps.as_mut(), env, info, handle_msg).unwrap_err();
        assert_eq!(
            res,
            StdError::generic_err(
                "This is an admin command. Admin commands can only be run from admin address",
            )
        );

        //2)Adding reviewer2
        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.reviewers.len(), 1);
        assert!(config.reviewers.contains(&Addr::unchecked("reviewer")));

        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);
        let handle_msg = HandleMsg::AddReviewer {
            reviewer: Addr::unchecked("reviewer2"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg)?;

        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.reviewers.len(), 2);
        assert!(config.reviewers.contains(&Addr::unchecked("reviewer")));
        assert!(config.reviewers.contains(&Addr::unchecked("reviewer2")));

        //3)Remove admin from reviewer2
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);
        let handle_msg = HandleMsg::RemoveReviewer {
            reviewer: Addr::unchecked("reviewer"),
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg)?;
        let config = config_helper_read_only(&deps.storage)?;
        assert_eq!(config.reviewers.len(), 1);
        assert!(config.reviewers.contains(&Addr::unchecked("reviewer2")));

        Ok(())
    }

    #[test]
    fn test_update_round_change_admin_share() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        // change admin
        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("not-admin", &[]);
        let handle_msg = HandleMsg::UpdateRound {
            admin_share: Some(AdminShareInfo {
                total_percentage_share: (7 * COMMON_DIVISOR) / 100 as u64,
                shade_percentage_share: (50 * COMMON_DIVISOR) / 100 as u64,
                galactic_pools_percentage_share: (40 * COMMON_DIVISOR) / 100 as u64,
            }),
            duration: None,
            rewards_distribution: None,
            ticket_price: None,
            rewards_expiry_duration: None,
            triggerer_share_percentage: None,
            shade_rewards_address: None,
            galactic_pools_rewards_address: None,
            grand_prize_address: None,
            unclaimed_distribution: None,
        };
        let res = execute(deps.as_mut(), env, info, handle_msg).unwrap_err();
        assert_eq!(
            res,
            StdError::generic_err(
                "This is an admin command. Admin commands can only be run from admin address",
            )
        );

        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);
        let handle_msg = HandleMsg::UpdateRound {
            admin_share: Some(AdminShareInfo {
                total_percentage_share: (7 * COMMON_DIVISOR) / 100 as u64,
                shade_percentage_share: (50 * COMMON_DIVISOR) / 100 as u64,
                galactic_pools_percentage_share: (40 * COMMON_DIVISOR) / 100 as u64,
            }),
            duration: None,
            rewards_distribution: None,
            ticket_price: None,
            rewards_expiry_duration: None,
            triggerer_share_percentage: None,
            shade_rewards_address: None,
            galactic_pools_rewards_address: None,
            grand_prize_address: None,
            unclaimed_distribution: None,
        };
        let res = execute(deps.as_mut(), env, info, handle_msg).unwrap_err();
        assert_eq!(
            res,
            StdError::generic_err("Total percentage shares don't add up to 100%",)
        );

        let env = custom_mock_env(None, None, None, None);
        let info = mock_info("admin", &[]);

        let handle_msg = HandleMsg::UpdateRound {
            admin_share: Some(AdminShareInfo {
                total_percentage_share: (7 * COMMON_DIVISOR) / 100 as u64,
                shade_percentage_share: (50 * COMMON_DIVISOR) / 100 as u64,
                galactic_pools_percentage_share: (50 * COMMON_DIVISOR) / 100 as u64,
            }),
            duration: None,
            rewards_distribution: None,
            ticket_price: None,
            rewards_expiry_duration: None,
            triggerer_share_percentage: None,
            shade_rewards_address: None,
            galactic_pools_rewards_address: None,
            grand_prize_address: None,
            unclaimed_distribution: None,
        };
        let _res = execute(deps.as_mut(), env, info, handle_msg)?;

        let round_obj: RoundInfo = round_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            round_obj.admin_share.total_percentage_share,
            (7 * COMMON_DIVISOR) / 100 as u64
        );
        assert_eq!(
            round_obj.admin_share.galactic_pools_percentage_share,
            (50 * COMMON_DIVISOR) / 100 as u64
        );
        assert_eq!(
            round_obj.admin_share.shade_percentage_share,
            (50 * COMMON_DIVISOR) / 100 as u64
        );
        Ok(())
    }

    #[test]
    fn test_update_round_change_round_duration() -> StdResult<()> {
        //Depositing amount
        let (_init_result, mut deps) = init_helper(Some(800 * SCRT_TO_USCRT));
        let handle_msg = HandleMsg::UpdateRound {
            admin_share: None,
            duration: Some(1),
            rewards_distribution: None,
            ticket_price: None,
            rewards_expiry_duration: None,
            triggerer_share_percentage: None,
            shade_rewards_address: None,
            galactic_pools_rewards_address: None,
            grand_prize_address: None,
            unclaimed_distribution: None,
        };
        let res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("non-admin", &[]),
            handle_msg,
        )
        .unwrap_err();
        assert_eq!(
            res,
            StdError::generic_err(
                "This is an admin command. Admin commands can only be run from admin address",
            )
        );

        //Depositing amount
        let (_init_result, mut deps) = init_helper(Some(800 * SCRT_TO_USCRT));
        let handle_msg = HandleMsg::UpdateRound {
            admin_share: None,
            duration: Some(1),
            rewards_distribution: None,
            ticket_price: None,
            rewards_expiry_duration: None,
            triggerer_share_percentage: None,
            shade_rewards_address: None,
            galactic_pools_rewards_address: None,
            grand_prize_address: None,
            unclaimed_distribution: None,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("admin", &[]),
            handle_msg,
        );
        let round_obj = round_read_only_unit_test_helper(&deps.storage);
        assert_eq!(round_obj.duration, 1);

        Ok(())
    }

    #[test]
    fn test_update_round_change_unbonding_duration() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(Some(800 * SCRT_TO_USCRT));
        let handle_msg = HandleMsg::UpdateConfig {
            unbonding_batch_duration: None,
            unbonding_duration: Some(1),
            minimum_deposit_amount: None,
            exp_contract: None,
        };
        let res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("non-admin", &[]),
            handle_msg,
        )
        .unwrap_err();
        assert_eq!(
            res,
            StdError::generic_err(
                "This is an admin command. Admin commands can only be run from admin address",
            )
        );

        let (_init_result, mut deps) = init_helper(Some(800 * SCRT_TO_USCRT));
        let handle_msg = HandleMsg::UpdateConfig {
            unbonding_batch_duration: None,
            unbonding_duration: Some(1),
            minimum_deposit_amount: None,
            exp_contract: None,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("admin", &[]),
            handle_msg,
        );
        let config_obj = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config_obj.unbonding_duration, 1);

        Ok(())
    }

    #[test]
    fn test_unbond_batch() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);
        //1) User Deposits
        for _ in 0..10 {
            let _ = deposit_unit_test_helper(
                deps.as_mut(),
                Uint128::from(10 * SCRT_TO_USCRT),
                None,
                None,
                "secretbatman",
                None,
            );
            //1.1)Request withdraw
            let _ = request_withdraw_unit_test_helper(
                deps.as_mut(),
                Uint128::from(10 * SCRT_TO_USCRT),
                None,
                None,
                "secretbatman",
            );

            end_round_unit_test_helper(deps.as_mut())?;
        }

        //2) Sponsors Deposits
        let _ = sponsor_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "Bruce Wayne",
            None,
        );
        //2.1) Sponsor_request_withdraw
        sponsor_request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "Bruce Wayne",
        )?;

        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        let reserves_req_withdraw = pool_state.total_reserves;
        let handle_msg = HandleMsg::RequestReservesWithdraw {
            amount: reserves_req_withdraw,
        };

        execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("admin", &[]),
            handle_msg,
        )?;

        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(
            config.next_unbonding_batch_amount.u128(),
            110 * SCRT_TO_USCRT + pool_state.total_reserves.u128()
        );
        assert_eq!(config.next_unbonding_batch_index, 1);
        assert_eq!(config.unbonding_batch_duration, 3600 * 24 * 3);
        assert_eq!(
            config.next_unbonding_batch_time,
            mock_env().block.time.seconds() + config.unbonding_batch_duration
        );

        let handle_msg = HandleMsg::UnbondBatch {};
        //3) When not triggerer
        let handle_result = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("non-triggerer", &[]),
            handle_msg,
        );

        assert_eq!(
            handle_result.unwrap_err(),
            StdError::generic_err(
                "This is an triggerer command. Triggerer commands can only be run from triggerer address",
            )
        );

        //3.1) When triggerer
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _handle_result = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        );

        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.next_unbonding_batch_amount.u128(), 0 * SCRT_TO_USCRT);
        assert_eq!(config.next_unbonding_batch_index, 2);

        assert_eq!(config.unbonding_batch_duration, 3600 * 24 * 3);
        assert_eq!(
            config.next_unbonding_batch_time,
            mock_env().block.time.seconds()
                + config.unbonding_batch_duration
                + config.unbonding_batch_duration
        );

        //4) When unbonding amount == 0
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _handle_result = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        );
        let config = config_read_only_unit_test_helper(&deps.storage);
        assert_eq!(config.next_unbonding_batch_amount.u128(), 0 * SCRT_TO_USCRT);
        assert_eq!(config.next_unbonding_batch_index, 3);

        Ok(())
    }

    #[test]
    fn test_request_reserves_withdraw() -> StdResult<()> {
        //9 different dummy users deposit 100000 SCRT using the helper filler function below.
        let mut deps = deposits_filler_unit_test_helper(Some(800 * SCRT_TO_USCRT));

        for _ in 0..8 {
            let _ = end_round_unit_test_helper(deps.as_mut())?;
        }
        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        let req_withdraw_amount = pool_state.total_reserves;

        let config_obj = config_read_only_unit_test_helper(&deps.storage);
        let mut total_delegated_sponsored_reserves = Uint128::zero();
        for val in config_obj.validators {
            total_delegated_sponsored_reserves =
                total_delegated_sponsored_reserves.add(val.delegated);
        }
        assert_eq!(
            total_delegated_sponsored_reserves,
            Uint128::from((100000) * SCRT_TO_USCRT).add(req_withdraw_amount)
        );

        let handle_msg = HandleMsg::RequestReservesWithdraw {
            amount: req_withdraw_amount,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("admin", &[]),
            handle_msg,
        )?;

        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        //Total amount deposited and sponsored should be equal to 100,000 SCRT (delegated)
        let config_obj = config_read_only_unit_test_helper(&deps.storage);
        let mut total_delegated_sponsored_reserves = Uint128::zero();
        for val in config_obj.validators {
            total_delegated_sponsored_reserves =
                total_delegated_sponsored_reserves.add(val.delegated);
        }
        assert_eq!(
            total_delegated_sponsored_reserves,
            Uint128::from((100000) * SCRT_TO_USCRT).add(pool_state.total_reserves)
        );
        Ok(())
    }

    #[test]
    fn test_reserves_withdraw() -> StdResult<()> {
        let mut deps = deposits_filler_unit_test_helper(Some(800 * SCRT_TO_USCRT));

        for _ in 0..10 {
            let _ = end_round_unit_test_helper(deps.as_mut())?;
        }

        let pool_state = pool_state_read_only_unit_test_helper(&deps.storage);
        let req_withdraw_amount = pool_state.total_reserves;
        let handle_msg = HandleMsg::RequestReservesWithdraw {
            amount: req_withdraw_amount,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("admin", &[]),
            handle_msg,
        )?;

        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        //Total amount deposited and sponsored should be equal to 100,000 SCRT (delegated)
        let config_obj = config_read_only_unit_test_helper(&deps.storage);

        let handle_msg = HandleMsg::ReservesWithdraw {
            amount: req_withdraw_amount,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(
                None,
                Some(config.next_unbonding_batch_time + config_obj.unbonding_duration.add(1)),
                None,
                None,
            ),
            mock_info("admin", &[]),
            handle_msg,
        )?;

        Ok(())
    }

    #[test]
    fn test_rebalance_validator_set() -> StdResult<()> {
        let mut deps = deposits_filler_unit_test_helper(None);

        let handle_msg = HandleMsg::RebalanceValidatorSet {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("admin", &[]),
            handle_msg,
        )?;

        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let pool_state = pool_state_read_only_unit_test_helper(deps.as_ref().storage);

        assert_eq!(
            config.validators[0].delegated,
            pool_state.total_delegated.multiply_ratio(
                config.validators[0].weightage as u128,
                config.common_divisor as u128,
            )
        );
        assert_eq!(
            config.validators[1].delegated,
            pool_state.total_delegated.multiply_ratio(
                config.validators[1].weightage as u128,
                config.common_divisor as u128,
            )
        );
        assert_eq!(
            config.validators[2].delegated,
            pool_state.total_delegated.multiply_ratio(
                config.validators[2].weightage as u128,
                config.common_divisor as u128,
            )
        );

        assert_eq!(config.next_validator_for_delegation, 0);
        assert_eq!(config.next_validator_for_unbonding, 0);

        Ok(())
    }

    #[test]
    fn test_update_validator_set() {
        let mut deps = deposits_filler_unit_test_helper(Some(800 * SCRT_TO_USCRT));

        let mut validator_vector: Vec<ValidatorInfo> = Vec::new();
        validator_vector.push(ValidatorInfo {
            address: "galacticPools".to_string(),
            weightage: (40 * COMMON_DIVISOR) / 100,
        });
        validator_vector.push(ValidatorInfo {
            address: "secureSecret".to_string(),
            weightage: (10 * COMMON_DIVISOR) / 100,
        });
        validator_vector.push(ValidatorInfo {
            address: "xavierCapital".to_string(),
            weightage: (0 * COMMON_DIVISOR) / 100,
        });
        validator_vector.push(ValidatorInfo {
            address: "IDK".to_string(),
            weightage: (30 * COMMON_DIVISOR) / 100,
        });
        validator_vector.push(ValidatorInfo {
            address: "IDK2".to_string(),
            weightage: (20 * COMMON_DIVISOR) / 100,
        });

        let handle_msg = HandleMsg::UpdateValidatorSet {
            updated_validator_set: validator_vector,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("admin", &[]),
            handle_msg,
        );

        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let pool_state = pool_state_read_only_unit_test_helper(deps.as_ref().storage);

        assert_eq!(
            config.validators[0].delegated,
            pool_state.total_delegated.multiply_ratio(
                config.validators[0].weightage as u128,
                config.common_divisor as u128,
            )
        );
        assert_eq!(config.validators[0].address, "galacticPools".to_string());
        assert_eq!(
            config.validators[1].delegated,
            pool_state.total_delegated.multiply_ratio(
                config.validators[1].weightage as u128,
                config.common_divisor as u128,
            )
        );
        assert_eq!(config.validators[1].address, "secureSecret".to_string());

        assert_eq!(
            config.validators[2].delegated,
            pool_state.total_delegated.multiply_ratio(
                config.validators[2].weightage as u128,
                config.common_divisor as u128,
            )
        );
        assert_eq!(config.validators[2].address, "IDK".to_string());

        assert_eq!(
            config.validators[3].delegated,
            pool_state.total_delegated.multiply_ratio(
                config.validators[3].weightage as u128,
                config.common_divisor as u128,
            )
        );
        assert_eq!(config.validators[3].address, "IDK2".to_string());
        assert_eq!(config.next_validator_for_delegation, 0);
        assert_eq!(config.next_validator_for_unbonding, 0);
    }

    #[test]
    fn test_set_contract_status() -> StdResult<()> {
        //Deposit : Normal
        //Sponsor: Normal
        //Request_withdraw: StopTransactions
        //Withdraw: StopTransactions
        //End_Round: StopTransactions
        //ClaimRewards: StopTransactions
        //CreateViewingKey: StopTransactions
        //SetViewingKey: StopTransactions
        //Request_withdraw_sponsor: StopTransactions
        //Withdraw_sponsor: StopTransactions

        let (_init_result, mut deps) = init_helper(None);
        let handle_msg = HandleMsg::SetContractStatus {
            level: ContractStatus::StopTransactions,
        };
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("non-admin", &[]),
            handle_msg,
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err(
                "This is an admin command. Admin commands can only be run from admin address",
            )
        );

        //StopTransactions
        let handle_msg = HandleMsg::SetContractStatus {
            level: ContractStatus::StopTransactions,
        };
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            handle_msg,
        )?;

        //Deposit: Normal
        let res = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(1 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
            None,
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("The contract admin has temporarily disabled this action")
        );
        //Sponsor : Normal
        let res = sponsor_unit_test_helper(
            deps.as_mut(),
            Uint128::from(1 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
            None,
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("The contract admin has temporarily disabled this action")
        );

        //StopAll
        let handle_msg = HandleMsg::SetContractStatus {
            level: ContractStatus::StopAll,
        };
        execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("admin", &[]),
            handle_msg,
        )?;
        //Request_withdraw: StopTransactions
        let res = request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from(1 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("The contract admin has temporarily disabled this action")
        );
        //Withdraw: StopTransaction
        let res = withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::from(1 * SCRT_TO_USCRT),
            None,
            None,
            "secretbatman",
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("The contract admin has temporarily disabled this action")
        );
        //End_Round: StopTransactions
        let res = end_round_unit_test_helper(deps.as_mut());
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("The contract admin has temporarily disabled this action")
        );
        //ClaimRewards: StopTransactions
        let res = claim_rewards_unit_test_helper(deps.as_mut(), None, None, "secretbatman");
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("The contract admin has temporarily disabled this action")
        );
        //CreateViewingKey: StopTransactions
        let res = create_viewking_key_unit_test_helper(deps.as_mut(), "secretbatman");
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("The contract admin has temporarily disabled this action")
        );
        //SetViewingKey: StopTransactions
        let res = set_viewing_key_unit_test_helper(deps.as_mut(), "secretbatman", "hi lol");
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("The contract admin has temporarily disabled this action")
        );

        //Request_withdraw_sponsor: StopTransactions
        let res = sponsor_request_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::zero(),
            None,
            None,
            "secretbatman",
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("The contract admin has temporarily disabled this action")
        );
        //Withdraw_sponsor: StopTransactions
        let res = sponsor_withdraw_unit_test_helper(
            deps.as_mut(),
            Uint128::zero(),
            None,
            None,
            "secretbatman",
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("The contract admin has temporarily disabled this action")
        );

        Ok(())
    }

    ////////////////////////////////////// Queries //////////////////////////////////////
    #[test]
    fn test_query_contract_config() -> StdResult<()> {
        // Error check
        let (init_result, deps) = init_helper(None);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        //Checking query results
        let query_msg = QueryMsg::ContractConfig {};
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: ContractConfigResponse = from_binary(&query_result.unwrap())?;
        match query_answer {
            ContractConfigResponse {
                admins,
                triggerers,
                reviewers,

                denom,
                contract_address,
                validators,
                next_unbonding_batch_time,
                next_unbonding_batch_amount,
                unbonding_batch_duration,
                unbonding_duration,
                minimum_deposit_amount,
                exp_contract,
            } => {
                assert_eq!(admins[0], ("admin".to_string()));
                assert_eq!(triggerers[0], ("triggerer".to_string()));
                assert_eq!(reviewers[0], ("reviewer".to_string()));
                assert_eq!(denom, "uscrt".to_string());
                assert_eq!(contract_address, ("cosmos2contract".to_string()));
                assert_eq!(validators[0].address, ("galacticPools".to_string()));
                assert_eq!(validators[1].address, ("secureSecret".to_string()));
                assert_eq!(validators[2].address, ("xavierCapital".to_string()));

                assert_eq!(
                    next_unbonding_batch_time,
                    mock_env().block.time.seconds() + 3600 * 24 * 3
                );
                assert_eq!(
                    next_unbonding_batch_amount,
                    Uint128::from(0 * SCRT_TO_USCRT)
                );
                assert_eq!(unbonding_batch_duration, 3600 * 24 * 3);
                assert_eq!(unbonding_duration, 3600 * 24 * 21);
                assert_eq!(minimum_deposit_amount, None);
                assert_eq!(exp_contract, None);
            } // _ => panic!("unexpected"),
        }

        Ok(())
    }

    #[test]
    fn test_query_contract_status() -> StdResult<()> {
        // Error check
        let (init_result, mut deps) = init_helper(None);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        //Checking query results
        let query_msg = QueryMsg::ContractStatus {};
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: ContractStatusResponse = from_binary(&query_result?)?;
        match query_answer {
            ContractStatusResponse { status } => {
                assert_eq!(status, ContractStatus::Normal)
            } // _ => panic!("unexpected"),
        }

        //Setting contract status to StopTransactions
        let handle_msg = HandleMsg::SetContractStatus {
            level: ContractStatus::StopTransactions,
        };
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            handle_msg,
        )?;

        //Checking query results
        let query_msg = QueryMsg::ContractStatus {};
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: ContractStatusResponse = from_binary(&query_result?)?;
        match query_answer {
            ContractStatusResponse { status } => {
                assert_eq!(status, ContractStatus::StopTransactions)
            } // _ => panic!("unexpected"),
        }

        Ok(())
    }

    #[test]
    fn test_query_round_obj() -> StdResult<()> {
        // Error check
        let (init_result, deps) = init_helper(None);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        //Checking query results
        let query_msg = QueryMsg::Round {};
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: RoundResponse = from_binary(&query_result?)?;
        match query_answer {
            RoundResponse {
                duration,
                start_time,
                end_time,
                rewards_distribution,
                current_round_index,
                ticket_price,
                rewards_expiry_duration,
                admin_share,
                triggerer_share_percentage,
                unclaimed_distribution,
            } => {
                assert_eq!(duration, 3600 * 24 * 7);
                assert_eq!(start_time, mock_env().block.time.seconds());
                assert_eq!(end_time, mock_env().block.time.seconds() + 3600 * 24 * 7);
                let r_d: RewardsDistInfo = RewardsDistInfo {
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
                assert_eq!(rewards_distribution, r_d);
                assert_eq!(current_round_index, 1u64);
                assert_eq!(ticket_price.u128(), 1 * SCRT_TO_USCRT);
                assert_eq!(rewards_expiry_duration, 3600 * 24 * 45);
                assert_eq!(
                    admin_share.galactic_pools_percentage_share,
                    (40 * COMMON_DIVISOR) / 100 as u64
                );
                assert_eq!(
                    admin_share.shade_percentage_share,
                    (60 * COMMON_DIVISOR) / 100 as u64
                );
                assert_eq!(triggerer_share_percentage, (1 * COMMON_DIVISOR) / 100);
                assert_eq!(unclaimed_distribution, UnclaimedDistInfo {
                    reserves_percentage: (60 * COMMON_DIVISOR) / 100,
                    propagate_percentage: COMMON_DIVISOR.sub((60 * COMMON_DIVISOR) / 100),
                });
            } // _ => panic!("unexpected"),
        }

        Ok(())
    }

    #[test]
    fn test_query_pool_state() -> StdResult<()> {
        let deps = deposits_filler_unit_test_helper(None);

        //Checking query results
        let query_msg = QueryMsg::PoolState {};
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: PoolStateInfoResponse = from_binary(&query_result?)?;
        match query_answer {
            PoolStateInfoResponse {
                total_delegated,
                rewards_returned_to_contract,
                total_reserves,
                total_sponsored,
            } => {
                assert_eq!(total_delegated.u128(), 100000 * SCRT_TO_USCRT);
                assert_eq!(rewards_returned_to_contract.u128(), 10 * 10 * SCRT_TO_USCRT);
                assert_eq!(total_reserves.u128(), 0 * SCRT_TO_USCRT);
                assert_eq!(total_sponsored.u128(), 0 * SCRT_TO_USCRT);
            }
        }

        Ok(())
    }

    #[test]
    fn test_query_pool_state_liquidity_stats() -> StdResult<()> {
        let deps = deposits_filler_unit_test_helper(None);

        //Checking query results
        let query_msg = QueryMsg::PoolStateLiquidityStats {};
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: PoolStateLiquidityStatsResponse = from_binary(&query_result?)?;
        match query_answer {
            PoolStateLiquidityStatsResponse { total_liquidity } => {
                // 70k was deposit at t0 and 30k was deposited at t(1/2) hence the avg liquidit is 85k
                assert_eq!(total_liquidity.u128(), 85000 * SCRT_TO_USCRT);
            }
        }
        Ok(())
    }
    #[test]
    fn test_query_pool_state_liquidity_stats_specific() -> StdResult<()> {
        let deps = deposits_filler_unit_test_helper(None);

        //Checking query results
        let query_msg = QueryMsg::PoolStateLiquidityStatsSpecific { round_index: 1u64 };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: PoolStateLiquidityStatsResponse = from_binary(&query_result?)?;
        match query_answer {
            PoolStateLiquidityStatsResponse { total_liquidity } => {
                // 70k was deposit at t0 and 30k was deposited at t(1/2) hence the avg liquidit is 85k
                assert_eq!(total_liquidity.u128(), 85000 * SCRT_TO_USCRT);
            }
        }
        Ok(())
    }

    #[test]
    fn test_query_current_rewards() -> StdResult<()> {
        let deps = deposits_filler_unit_test_helper(None);

        //Checking query results
        let query_msg = QueryMsg::CurrentRewards {};
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: CurrentRewardsResponse = from_binary(&query_result?)?;
        match query_answer {
            CurrentRewardsResponse { rewards } => {
                // 10 rewards of 10 tokens each were returned. And 3 rewards of 10 tokens each were pending/unclaimed at each validator
                assert_eq!(rewards.u128(), (10 * 10 + 10 * 3) * SCRT_TO_USCRT);
            }
        }
        Ok(())
    }

    //Authenticated + Permit based Queries

    #[test]
    fn test_query_permits() -> StdResult<()> {
        let deps = deposits_filler_unit_test_helper(None);

        let token = "cosmos2contract".to_string();
        //1) Checking signature validity
        let permit: Permit<GalacticPoolsPermissions> = Permit {
            params: PermitParams {
                allowed_tokens: vec![token.clone()],
                permit_name: "galactic_pools_batman".to_string(),
                chain_id: "pulsar-2".to_string(),
                permissions: vec![GalacticPoolsPermissions::Delegated],
            },
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64("AihRsQtwF56zrW7J1VP4KqTNBF5GTMXoFusm6zfIixvD")?,
                },
                signature: Binary::from_base64(
                    "mU+0UZJoFbWS2sqPoN75ed0KiMHj+Mie520IhJa6Zcxux9ky5v/jWatLugEhb8JruJ0c4Bi0ZlA+tK7ydppTug==",
                )?,
            },
        };

        let address = validate::<GalacticPoolsPermissions>(
            deps.as_ref(),
            PREFIX_REVOKED_PERMITS,
            &permit,
            token.clone(),
            None,
        )?;

        assert_eq!(
            address,
            "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7".to_string()
        );

        //2) Check error if the signature is not correct in reference to the permit
        let deps = mock_dependencies();

        let permit: Permit<GalacticPoolsPermissions> = Permit {
            params: PermitParams {
                allowed_tokens: vec![token.clone()],
                permit_name: "galactic_pools_batman".to_string(),
                chain_id: "pulsar-2".to_string(),
                permissions: vec![GalacticPoolsPermissions::Owner],
            },
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64("AihRsQtwF56zrW7J1VP4KqTNBF5GTMXoFusm6zfIixvD")?,
                },
                signature: Binary::from_base64(
                    "rhHv9JwULRoxm4/CnAJSWMa76Q40U8MKZFKgDrEAkk8m2yvUHIOYmF/oc6VsJICDePhbzHyzlg35X5pjw8Yn9A==",
                )?,
            },
        };
        let address = validate::<GalacticPoolsPermissions>(
            deps.as_ref(),
            PREFIX_REVOKED_PERMITS,
            &permit,
            token.clone(),
            None,
        );

        assert_eq!(
            address,
            Err(StdError::generic_err(
                "Failed to verify signatures for the given permit",
            ))
        );

        //3) Check error if the user doesn't have the permission to particular query function
        let deps = deposits_filler_unit_test_helper(None);
        let token = "cosmos2contract".to_string();

        let permit: Permit<GalacticPoolsPermissions> = Permit {
            params: PermitParams {
                allowed_tokens: vec![token.clone()],
                permit_name: "galactic_pools_batman".to_string(),
                chain_id: "pulsar-2".to_string(),
                permissions: vec![GalacticPoolsPermissions::Unbondings],
            },
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64("AihRsQtwF56zrW7J1VP4KqTNBF5GTMXoFusm6zfIixvD")?,
                },
                signature: Binary::from_base64(
                    "QSMrJNzYIbaFGNronY8IEjndWtPEFnaOyYdG7mbPdQE6uonPfuhK3oBXM5Sf+TAcASEj4b+8aS1lJYfToH1O+w==",
                )?,
            },
        };

        let query_msg = QueryMsg::WithPermit {
            permit,
            query: QueryWithPermit::Delegated {},
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert_eq!(
            query_result.unwrap_err(),
            StdError::generic_err(
                "Owner or Delegated permission is required for queries, got permissions [Unbondings]"
            )
        );

        //4) Checking the results of the query when wrong query_message
        let permit: Permit<GalacticPoolsPermissions> = Permit {
            params: PermitParams {
                allowed_tokens: vec![token.clone()],
                permit_name: "galactic_pools_batman".to_string(),
                chain_id: "pulsar-2".to_string(),
                permissions: vec![GalacticPoolsPermissions::Owner],
            },
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64("AihRsQtwF56zrW7J1VP4KqTNBF5GTMXoFusm6zfIixvD")?,
                },
                signature: Binary::from_base64(
                    "q7MQVPXCwA89cBMl/dCQhZ87dxzrNhxlQUUEznf4JvhWluAhRaNvblSofu79lYGUJ0+mfH1KMCsmF+kkARHYpQ==",
                )?,
            },
        };

        let query_msg = QueryMsg::WithPermit {
            permit,
            query: QueryWithPermit::Test {},
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(query_result.is_err());

        Ok(())
    }

    #[test]
    fn test_authenticated_queries() -> StdResult<()> {
        //1) Checking error: if the vk is not set
        let _env = custom_mock_env(None, None, None, None);
        let mut deps = deposits_filler_unit_test_helper(None);

        let no_vk_yet_query_msg = QueryMsg::Delegated {
            address: "secretbatman".to_string(),
            key: "no_vk_yet".to_string(),
        };
        let query_result: ViewingKeyErrorResponse =
            from_binary(&query(deps.as_ref(), _env, no_vk_yet_query_msg)?)?;

        assert_eq!(query_result, ViewingKeyErrorResponse {
            msg: "Wrong viewing key for this address or viewing key not set".to_string(),
        });

        //2) Checking after the vk is set.
        let create_vk_msg = HandleMsg::CreateViewingKey {
            entropy: "heheeehe".to_string(),
        };
        let _env = custom_mock_env(None, None, None, None);
        let info = mock_info("secretbatman", &[]);
        let handle_response = execute(deps.as_mut(), _env, info, create_vk_msg)?;
        let vk = match from_binary(&handle_response.data.unwrap())? {
            HandleAnswer::CreateViewingKey { key } => key,
            _ => panic!("Unexpected result from handle"),
        };

        let query_balance_msg = QueryMsg::Delegated {
            address: ("secretbatman".to_string()),
            key: vk.0,
        };

        let _env = custom_mock_env(None, None, None, None);
        let query_response = query(deps.as_ref(), _env, query_balance_msg)?;
        let balance = match from_binary(&query_response)? {
            DelegatedResponse { amount } => amount,
        };
        assert_eq!(balance, Uint128::from(60000 * SCRT_TO_USCRT));

        //3) Checking if the vk is wrong
        let wrong_vk_query_msg = QueryMsg::Delegated {
            address: ("secretbatman".to_string()),
            key: "wrong_vk".to_string(),
        };
        let _env = custom_mock_env(None, None, None, None);
        let query_result: ViewingKeyErrorResponse =
            from_binary(&query(deps.as_ref(), _env, wrong_vk_query_msg)?)?;
        assert_eq!(query_result, ViewingKeyErrorResponse {
            msg: "Wrong viewing key for this address or viewing key not set".to_string(),
        });

        Ok(())
    }

    #[test]
    fn test_query_delegated() -> StdResult<()> {
        let mut deps = deposits_filler_unit_test_helper(None);
        let token = "cosmos2contract".to_string();

        //Permit Queries
        //1) Depositing just for the example
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
            None,
        );

        //1.1) Checking the results of the query when permission is delegated
        let permit: Permit<GalacticPoolsPermissions> = Permit {
            params: PermitParams {
                allowed_tokens: vec![token.clone()],
                permit_name: "galactic_pools_batman".to_string(),
                chain_id: "pulsar-2".to_string(),
                permissions: vec![GalacticPoolsPermissions::Delegated],
            },
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64("AihRsQtwF56zrW7J1VP4KqTNBF5GTMXoFusm6zfIixvD")?,
                },
                signature: Binary::from_base64(
                    "mU+0UZJoFbWS2sqPoN75ed0KiMHj+Mie520IhJa6Zcxux9ky5v/jWatLugEhb8JruJ0c4Bi0ZlA+tK7ydppTug==",
                )?,
            },
        };

        let query_msg = QueryMsg::WithPermit {
            permit,
            query: QueryWithPermit::Delegated {},
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: DelegatedResponse = from_binary(&query_result?)?;
        match query_answer {
            DelegatedResponse { amount } => {
                // 10 rewards of 10 tokens each were returned. And 3 rewards of 10 tokens each were pending/unclaimed at each validator
                assert_eq!(amount.u128(), 10 * SCRT_TO_USCRT);
            }
        }

        //2) Checking the results of the query when permission is owner
        let permit: Permit<GalacticPoolsPermissions> = Permit {
            params: PermitParams {
                allowed_tokens: vec![token.clone()],
                permit_name: "galactic_pools_batman".to_string(),
                chain_id: "pulsar-2".to_string(),
                permissions: vec![GalacticPoolsPermissions::Owner],
            },
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64("AihRsQtwF56zrW7J1VP4KqTNBF5GTMXoFusm6zfIixvD")?,
                },
                signature: Binary::from_base64(
                    "q7MQVPXCwA89cBMl/dCQhZ87dxzrNhxlQUUEznf4JvhWluAhRaNvblSofu79lYGUJ0+mfH1KMCsmF+kkARHYpQ==",
                )?,
            },
        };

        let query_msg = QueryMsg::WithPermit {
            permit,
            query: QueryWithPermit::Delegated {},
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: DelegatedResponse = from_binary(&query_result?)?;
        match query_answer {
            DelegatedResponse { amount } => {
                // 10 rewards of 10 tokens each were returned. And 3 rewards of 10 tokens each were pending/unclaimed at each validator
                assert_eq!(amount.u128(), 10 * SCRT_TO_USCRT);
            }
        }

        //Authenticated Queries

        Ok(())
    }

    #[test]
    fn test_query_liquidity() -> StdResult<()> {
        //starts round
        let (_init_result, mut deps) = init_helper(None);
        let round_obj = round_read_only_unit_test_helper(&mut deps.storage);
        let token = "cosmos2contract".to_string();
        //Permit Queries
        //1) Depositing just for the example
        //1.1) Checking the results of the query when permission is delegated
        let permit: Permit<GalacticPoolsPermissions> = Permit {
            params: PermitParams {
                allowed_tokens: vec![token.clone()],
                permit_name: "galactic_pools_batman".to_string(),
                chain_id: "pulsar-2".to_string(),
                permissions: vec![GalacticPoolsPermissions::Owner],
            },
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64("AihRsQtwF56zrW7J1VP4KqTNBF5GTMXoFusm6zfIixvD")?,
                },
                signature: Binary::from_base64(
                    "q7MQVPXCwA89cBMl/dCQhZ87dxzrNhxlQUUEznf4JvhWluAhRaNvblSofu79lYGUJ0+mfH1KMCsmF+kkARHYpQ==",
                )?,
            },
        };

        let query_msg = QueryMsg::WithPermit {
            permit: permit.clone(),
            query: QueryWithPermit::Liquidity { round_index: 2 },
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: LiquidityResponse = from_binary(&query_result?)?;
        match query_answer {
            LiquidityResponse {
                total_liq,
                total_tickets,
                ticket_price,
                user_liq,
                user_tickets,
                tickets_used,
                expiry_date,
                total_rewards,
                unclaimed_rewards,
            } => {
                // 10 rewards of 10 tokens each were returned. And 3 rewards of 10 tokens each were pending/unclaimed at each validator
                assert_eq!(total_liq.u128(), 0 * SCRT_TO_USCRT);
                assert_eq!(total_tickets.u128(), 0);
                assert_eq!(ticket_price.u128(), 1 * SCRT_TO_USCRT);
                assert_eq!(user_liq.u128(), 0 * SCRT_TO_USCRT);
                assert_eq!(user_tickets.u128(), 0);
                assert_eq!(tickets_used.u128(), 0);
                assert_eq!(expiry_date, None);
                assert_eq!(total_rewards.u128(), 0);
                assert_eq!(unclaimed_rewards.u128(), 0);
            }
        }
        //deposit 2 million scrt at t0
        let env = custom_mock_env(None, None, None, None);
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(2000000 * SCRT_TO_USCRT),
            Some(0u64),
            Some(round_obj.start_time),
            "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
            None,
        );
        let query_msg = QueryMsg::WithPermit {
            permit: permit.clone(),
            query: QueryWithPermit::Liquidity { round_index: 1 },
        };
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: LiquidityResponse = from_binary(&query_result?)?;
        match query_answer {
            LiquidityResponse {
                total_liq,
                total_tickets,
                ticket_price,
                user_liq,
                user_tickets,
                tickets_used,
                expiry_date,
                total_rewards,
                unclaimed_rewards,
            } => {
                // 10 rewards of 10 tokens each were returned. And 3 rewards of 10 tokens each were pending/unclaimed at each validator
                assert_eq!(total_liq.u128(), 2000000 * SCRT_TO_USCRT);
                assert_eq!(total_tickets.u128(), 2000000);
                assert_eq!(ticket_price.u128(), 1 * SCRT_TO_USCRT);
                assert_eq!(user_liq.u128(), 2000000 * SCRT_TO_USCRT);
                assert_eq!(user_tickets.u128(), 2000000);
                assert_eq!(tickets_used.u128(), 0);
                assert_eq!(expiry_date, None);
                assert_eq!(total_rewards.u128(), 0);
                assert_eq!(unclaimed_rewards.u128(), 0);
            }
        }
        //End_Round x 2
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-1
        let _ = end_round_unit_test_helper(deps.as_mut())?; //round-1

        //claim_rewards
        // must take 4 iterations
        // 1-  last claim round == None
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
        )?;
        let user_info = user_info_read_only_unit_test_helper(
            &Addr::unchecked("secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7"),
            &deps.storage,
        );
        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps
                .api
                .addr_validate("secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7")?,
            &deps.storage,
            1,
        );
        assert_eq!(
            user_liquidity_snapshot_obj.tickets_used.unwrap().u128(),
            1000000
        );

        //Testing
        let query_msg = QueryMsg::WithPermit {
            permit,
            query: QueryWithPermit::Liquidity { round_index: 1 },
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: LiquidityResponse = from_binary(&query_result?)?;
        match query_answer {
            LiquidityResponse {
                total_liq,
                total_tickets,
                ticket_price,
                user_liq,
                user_tickets,
                tickets_used,
                ..
            } => {
                // 10 rewards of 10 tokens each were returned. And 3 rewards of 10 tokens each were pending/unclaimed at each validator
                assert_eq!(total_liq.u128(), 2000000 * SCRT_TO_USCRT);
                assert_eq!(total_tickets.u128(), 2000000);
                assert_eq!(ticket_price.u128(), 1 * SCRT_TO_USCRT);
                assert_eq!(user_liq.u128(), 2000000 * SCRT_TO_USCRT);
                assert_eq!(user_tickets.u128(), 2000000);
                assert_eq!(tickets_used.u128(), 1000000);
            }
        }

        assert_eq!(user_info.last_claim_rewards_round, None);
        // 2- last claim round == 1
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
        )?;
        let user_info = user_info_read_only_unit_test_helper(
            &Addr::unchecked("secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7"),
            &deps.storage,
        );
        assert_eq!(user_info.last_claim_rewards_round.unwrap(), 1);

        // 3- last claim round == 1
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
        )?;
        let user_info = user_info_read_only_unit_test_helper(
            &Addr::unchecked("secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7"),
            &deps.storage,
        );
        assert_eq!(user_info.last_claim_rewards_round.unwrap(), 1);

        let user_liquidity_snapshot_obj = user_liquidity_stats_read_only_unit_test_helper(
            &deps
                .api
                .addr_validate("secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7")?,
            &deps.storage,
            2,
        );
        assert_eq!(
            user_liquidity_snapshot_obj.tickets_used.unwrap().u128(),
            1000000
        );

        // 4- last claim round == 2`
        let _ = claim_rewards_unit_test_helper(
            deps.as_mut(),
            None,
            Some(round_obj.end_time),
            "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
        )?;
        let _user_info = user_info_read_only_unit_test_helper(
            &Addr::unchecked("secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7"),
            &deps.storage,
        );
        Ok(())
    }

    #[test]
    fn test_query_withdrawable() -> StdResult<()> {
        //Permit Queries
        let mut deps = deposits_filler_unit_test_helper(None);
        let token = "cosmos2contract".to_string();
        //1) Depositing just for the example
        let env = mock_env();
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
            None,
        );
        for _ in 0..10 {
            let _ = request_withdraw_unit_test_helper(
                deps.as_mut(),
                Uint128::from(1 * SCRT_TO_USCRT),
                None,
                Some(env.block.time.seconds()),
                "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
            );
        }
        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;

        //1.1) Checking the results of the query when permission is delegated
        let permit: Permit<GalacticPoolsPermissions> = Permit {
            params: PermitParams {
                allowed_tokens: vec![token.clone()],
                permit_name: "galactic_pools_batman".to_string(),
                chain_id: "pulsar-2".to_string(),
                permissions: vec![GalacticPoolsPermissions::Withdrawable],
            },
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64("AihRsQtwF56zrW7J1VP4KqTNBF5GTMXoFusm6zfIixvD")?,
                },
                signature: Binary::from_base64(
                    "T7jQblf+v5UN3lFOZwGdvxis3Ryqc2NKvf6bdPP96NNFs7bkLUou0bKg8x4XxGILII7rRNRsjX1kxwnf5vp5aw==",
                )?,
            },
        };
        validate::<GalacticPoolsPermissions>(
            deps.as_ref(),
            PREFIX_REVOKED_PERMITS,
            &permit,
            token.clone(),
            None,
        )?;

        let query_msg = QueryMsg::WithPermit {
            permit,
            query: QueryWithPermit::Withdrawable {},
        };
        let env = custom_mock_env(
            None,
            Some(
                mock_env()
                    .block
                    .time
                    .seconds()
                    .add(config.next_unbonding_batch_time)
                    + 3600 * 24 * 3
                    + 3600 * 2400 * 21,
            ),
            None,
            None,
        );
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: WithdrawablelResponse = from_binary(&query_result?)?;
        match query_answer {
            WithdrawablelResponse { amount } => {
                assert_eq!(amount.u128(), 10 * SCRT_TO_USCRT);
            }
        }

        Ok(())
    }

    #[test]
    fn test_query_unbondings() -> StdResult<()> {
        //Permit Queries
        let mut deps = deposits_filler_unit_test_helper(None);
        let token = "cosmos2contract".to_string();
        //1) Depositing just for the example
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
            None,
        );
        for _ in 0..10 {
            let _ = request_withdraw_unit_test_helper(
                deps.as_mut(),
                Uint128::from(1 * SCRT_TO_USCRT),
                None,
                None,
                "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
            );
        }

        let config = config_read_only_unit_test_helper(deps.as_ref().storage);
        let env = custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None);
        let handle_msg = HandleMsg::UnbondBatch {};
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("triggerer", &[]),
            handle_msg,
        )?;
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(10 * SCRT_TO_USCRT),
            None,
            None,
            "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
            None,
        );
        for _ in 0..10 {
            let _ = request_withdraw_unit_test_helper(
                deps.as_mut(),
                Uint128::from(1 * SCRT_TO_USCRT),
                None,
                None,
                "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
            );
        }

        //1.1) Checking the results of the query when permission is unbondings
        let permit: Permit<GalacticPoolsPermissions> = Permit {
            params: PermitParams {
                allowed_tokens: vec![token.clone()],
                permit_name: "galactic_pools_batman".to_string(),
                chain_id: "pulsar-2".to_string(),
                permissions: vec![GalacticPoolsPermissions::Unbondings],
            },
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64("AihRsQtwF56zrW7J1VP4KqTNBF5GTMXoFusm6zfIixvD")?,
                },
                signature: Binary::from_base64(
                    "QSMrJNzYIbaFGNronY8IEjndWtPEFnaOyYdG7mbPdQE6uonPfuhK3oBXM5Sf+TAcASEj4b+8aS1lJYfToH1O+w==",
                )?,
            },
        };
        validate::<GalacticPoolsPermissions>(
            deps.as_ref(),
            PREFIX_REVOKED_PERMITS,
            &permit,
            token.clone(),
            None,
        )?;

        let query_msg = QueryMsg::WithPermit {
            permit,
            query: QueryWithPermit::Unbondings {},
        };
        let query_result = query(deps.as_ref(), env.clone(), query_msg.clone());
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: UnbondingsResponse = from_binary(&query_result?)?;
        match query_answer {
            UnbondingsResponse { vec, len } => {
                assert_eq!(vec.len(), 2);
                assert_eq!(vec[0].batch_index, 1);
                assert_eq!(vec[0].amount.u128(), 10 * SCRT_TO_USCRT);
                assert_eq!(
                    vec[0].unbonding_time.unwrap(),
                    env.clone().block.time.seconds() + 3600 * 24 * 21
                );
                assert_eq!(vec[0].next_batch_unbonding_time, None);

                assert_eq!(vec[1].batch_index, 2);
                assert_eq!(vec[1].amount.u128(), 10 * SCRT_TO_USCRT);
                assert_eq!(vec[1].unbonding_time, None);
                assert_eq!(
                    vec[1].next_batch_unbonding_time.unwrap(),
                    env.clone().block.time.seconds() + 3600 * 24 * 3
                );

                assert_eq!(len, 2);
            }
        }

        //After 21 days

        let query_result = query(deps.as_ref(), env.clone(), query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: UnbondingsResponse = from_binary(&query_result?)?;
        match query_answer {
            UnbondingsResponse { vec, len } => {
                assert_eq!(vec.len(), 2);
                assert_eq!(vec[0].batch_index, 1);
                assert_eq!(vec[0].amount.u128(), 10 * SCRT_TO_USCRT);
                assert_eq!(
                    vec[0].unbonding_time.unwrap(),
                    env.clone().block.time.seconds() + 3600 * 24 * 21
                );
                assert_eq!(vec[0].next_batch_unbonding_time, None);

                assert_eq!(vec[1].batch_index, 2);
                assert_eq!(vec[1].amount.u128(), 10 * SCRT_TO_USCRT);
                assert_eq!(vec[1].unbonding_time, None);
                assert_eq!(
                    vec[1].next_batch_unbonding_time.unwrap(),
                    env.clone().block.time.seconds() + 3600 * 24 * 3
                );

                assert_eq!(len, 2);
            }
        }

        Ok(())
    }

    #[test]
    fn test_query_records() -> StdResult<()> {
        //Permit Queries
        let mut deps = deposits_filler_unit_test_helper(None);
        let token = "cosmos2contract".to_string();
        //1) Depositing just for the example
        let _ = deposit_unit_test_helper(
            deps.as_mut(),
            Uint128::from(5000 * SCRT_TO_USCRT),
            None,
            None,
            "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
            None,
        );

        for _ in 0..2 {
            let round_obj = round_read_only_unit_test_helper(deps.as_ref().storage);

            let _ = end_round_unit_test_helper(deps.as_mut())?;
            let _ = claim_rewards_unit_test_helper(
                deps.as_mut(),
                None,
                Some(round_obj.end_time),
                "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
            )?;
        }

        let permit: Permit<GalacticPoolsPermissions> = Permit {
            params: PermitParams {
                allowed_tokens: vec![token.clone()],
                permit_name: "galactic_pools_batman".to_string(),
                chain_id: "pulsar-2".to_string(),
                permissions: vec![GalacticPoolsPermissions::Records],
            },
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64("AihRsQtwF56zrW7J1VP4KqTNBF5GTMXoFusm6zfIixvD")?,
                },
                signature: Binary::from_base64(
                    "9WNoBfC8G4aJHdDBh4u8ATTZSt0xZZJyf5N+TcaxxZd1RW1pufT/80FSBWAF7ENFp7oSaW5qbOH9erd4kz9byA==",
                )?,
            },
        };
        validate::<GalacticPoolsPermissions>(
            deps.as_ref(),
            PREFIX_REVOKED_PERMITS,
            &permit,
            token.clone(),
            None,
        )?;

        let query_msg = QueryMsg::WithPermit {
            permit,
            query: QueryWithPermit::Records {
                page_size: Some(100),
                start_page: None,
            },
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg);
        assert!(
            query_result.is_ok(),
            "query failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: RecordsResponse = from_binary(&query_result?)?;
        match query_answer {
            RecordsResponse { vec, len } => {
                assert!(vec.len() != 0);
                assert!(len != 0);
            }
        }

        Ok(())
    }

    #[test]
    fn test_query_sponsors() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        //1.1) Deposit with the message
        for i in 0..13 {
            let sponsor = format!("user{}", i);
            let handle_msg = HandleMsg::Sponsor {
                title: Some("Bruce Wayne".to_string()),
                message: Some("I'm rich".to_string()),
            };
            let _res = execute(
                deps.as_mut(),
                custom_mock_env(None, None, None, None),
                mock_info(&sponsor, &[Coin {
                    denom: "uscrt".to_string(),
                    amount: Uint128::from(10000 * SCRT_TO_USCRT),
                }]),
                handle_msg,
            )?;
        }

        //1.2) Query the Sponsors List with page size and page index == NONE
        let query_msg = QueryMsg::Sponsors {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 5);
                assert_eq!(len, 13);
                //returns page = 0 & page_size = 5

                for index in 0..5 {
                    assert_eq!(vec[index].addr_list_index, Some(index as u32));
                }
            }
        }
        //1.3) Query the Sponsors List page 0 page_size 5
        let query_msg = QueryMsg::Sponsors {
            page_size: Some(15),
            start_page: Some(0),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 13);
                assert_eq!(len, 13);
                let mut index = 0;
                for val in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] {
                    assert_eq!(vec[index].addr_list_index, Some((val) as u32));
                    index += 1;
                }
            }
        }

        //1.3) Query the Sponsors List page 0 page_size 5
        let query_msg = QueryMsg::Sponsors {
            page_size: Some(5),
            start_page: Some(0),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 5);
                assert_eq!(len, 13);
                for index in 0..5 {
                    assert_eq!(vec[index].addr_list_index, Some(index as u32));
                }
            }
        }

        //1.4) Query the Sponsors List page 1 page_size 5
        let query_msg = QueryMsg::Sponsors {
            page_size: Some(5),
            start_page: Some(1),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 5);
                assert_eq!(len, 13);
                for index in 0..5 {
                    assert_eq!(vec[index].addr_list_index, Some((index + 5) as u32));
                }
            }
        }

        //1.5) Query the Sponsors List page 1 page_size 5
        let query_msg = QueryMsg::Sponsors {
            page_size: Some(5),
            start_page: Some(2),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 3);
                assert_eq!(len, 13);
                for index in 0..3 {
                    assert_eq!(vec[index].addr_list_index, Some((index + 10) as u32));
                }
            }
        }

        //2 Now Querying after deleting sponsors from storage
        //user 0,2,4,6,8,10 are removed
        for i in 0..12 {
            if i % 2 == 0 {
                let sponsor = format!("user{}", i);
                sponsor_request_withdraw_unit_test_helper(
                    deps.as_mut(),
                    Uint128::from(10000 * SCRT_TO_USCRT),
                    None,
                    None,
                    &sponsor,
                )?;
            }
        }

        //2.1) Query the Sponsors List page 0 page_size 5 [1,3,5,7,9]
        let query_msg = QueryMsg::Sponsors {
            page_size: Some(5),
            start_page: Some(0),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 5);
                assert_eq!(len, 7);
                let mut index = 0;
                for val in [1, 3, 5, 7, 9] {
                    assert_eq!(vec[index].addr_list_index, Some((val) as u32));
                    index += 1;
                }
            }
        }

        //2.2) Query the Sponsors List page 1 page_size 5 [11,12]
        let query_msg = QueryMsg::Sponsors {
            page_size: Some(5),
            start_page: Some(1),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 2);
                assert_eq!(len, 7);
                let mut index = 0;
                for val in [11, 12] {
                    assert_eq!(vec[index].addr_list_index, Some((val) as u32));
                    index += 1;
                }
            }
        }

        //2.3) Query the Sponsors List page 0 page_size 10 [1,3,5,7,9,11,12]
        let query_msg = QueryMsg::Sponsors {
            page_size: Some(10),
            start_page: Some(0),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 7);
                assert_eq!(len, 7);
                let mut index = 0;
                for val in [1, 3, 5, 7, 9, 11, 12] {
                    assert_eq!(vec[index].addr_list_index, Some((val) as u32));
                    index += 1;
                }
            }
        }

        //2.3) Query the Sponsors List page 01 page_size 10 []
        let query_msg = QueryMsg::Sponsors {
            page_size: Some(10),
            start_page: Some(01),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorsResponse { vec, len } => {
                assert_eq!(vec.len(), 0);
                assert_eq!(len, 7);
            }
        }

        Ok(())
    }

    #[test]
    fn test_query_sponsor_msg_request() -> StdResult<()> {
        let (_init_result, mut deps) = init_helper(None);

        //1.1) Deposit with the message
        for i in 0..13 {
            let sponsor = format!("user{}", i);
            let handle_msg = HandleMsg::Sponsor {
                title: Some("Bruce Wayne".to_string()),
                message: Some("I'm rich".to_string()),
            };
            let _res = execute(
                deps.as_mut(),
                custom_mock_env(
                    None,
                    Some(mock_env().block.time.seconds().add(i)),
                    None,
                    None,
                ),
                mock_info(&sponsor, &[Coin {
                    denom: "uscrt".to_string(),
                    amount: Uint128::from(10000 * SCRT_TO_USCRT),
                }]),
                handle_msg,
            )?;
        }

        //1.2) Query the Sponsors List with page size and page index == NONE
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 5);
                assert_eq!(len, 13);
                //returns page = 0 & page_size = 5
                for index in 0..5 {
                    assert_eq!(vec[index].index, Some(index as u32));
                }
            }
        }

        //1.3) Query the Sponsors List page 0 page_size 5
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: Some(5),
            start_page: Some(0),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 5);
                assert_eq!(len, 13);
                for index in 0..5 {
                    assert_eq!(vec[index].index, Some(index as u32));
                }
            }
        }

        //1.4) Query the Sponsors List page 1 page_size 5
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: Some(5),
            start_page: Some(1),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 5);
                assert_eq!(len, 13);
                for index in 0..5 {
                    assert_eq!(vec[index].index, Some((index + 5) as u32));
                }
            }
        }

        //1.5) Query the Sponsors List page 1 page_size 5
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: Some(5),
            start_page: Some(2),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 3);
                assert_eq!(len, 13);
                for index in 0..3 {
                    assert_eq!(vec[index].index, Some((index + 10) as u32));
                    assert_eq!(vec[index].deque_store_index, Some((index + 10) as u32));
                }
            }
        }

        //2) Reviewing requests
        let (_init_result, mut deps) = init_helper(None);

        for i in 0..13 {
            let sponsor = format!("user{}", i);
            let title = format!("user {} title", i);
            let handle_msg = HandleMsg::Sponsor {
                title: Some(title),
                message: Some("Some message".to_string()),
            };
            let _res = execute(
                deps.as_mut(),
                custom_mock_env(
                    None,
                    Some(mock_env().block.time.seconds().add(i)),
                    None,
                    None,
                ),
                mock_info(&sponsor, &[Coin {
                    denom: "uscrt".to_string(),
                    amount: Uint128::from(10000 * SCRT_TO_USCRT),
                }]),
                handle_msg,
            )?;
        }

        // User addr [user0,user1,user2,user3,user4,user5,user6,user7,user8,user9,user10,user11,user12]
        // User index [0.  ,  1. ,  2. ,  3. ,  4. ,  5. ,  6. ,  7. ,  8. ,  9. ,  10. ,  11. ,  12. ]

        //2) Review Sponsor Request Messages
        let mut decision_vec: Vec<Review> = vec![];

        for i in (0..13).rev() {
            if i % 2 == 0 {
                decision_vec.push(Review {
                    index: i,
                    is_accpeted: true,
                });
            }
        }
        let handle_msg = HandleMsg::ReviewSponsors {
            decisions: decision_vec,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("reviewer", &[]),
            handle_msg,
        )?;

        // User addr [user1,user3,user5,user7,user9,user11]
        // User index [0.  ,  1. ,  2. ,  3. ,  4. ,  5.  ]

        let sponsor_state_obj = sponsor_state_unit_test_read_only_helper(&deps.storage)?;
        assert_eq!(sponsor_state_obj.offset, 13);
        // assert_eq!(sponsor_state_obj.sponsor_msg_req_empty_slots.len(), 6);
        // assert_eq!(sponsor_state_obj.sponsor_msg_req_offset, 12);

        //2.1) Review Sponsor Request Messages
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: None,
            start_page: None,
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 5);
                assert_eq!(len, 6);
                let mut ind = 0;

                // [1,3,7,9,11] -> index [1,3,4,2,0]
                // User addr [user1,user3,user5,user7,user9,user11]
                // User index [0.  ,  1. ,  2. ,  3. ,  4. ,  5.  ]

                for user in [1, 3, 5, 7, 9] {
                    let addr = format!("user{}", user);
                    assert_eq!(vec[ind].index, Some(user as u32));
                    assert_eq!(vec[ind].addr, addr);
                    ind += 1;
                }
            }
        }

        //2.1) Review Sponsor Request Messages
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: Some(5),
            start_page: Some(0),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 5);
                assert_eq!(len, 6);
                let mut ind = 0;

                for user in [1, 3, 5, 7, 9] {
                    let addr = format!("user{}", user);
                    assert_eq!(vec[ind].index, Some(user as u32));
                    assert_eq!(vec[ind].addr, addr);
                    ind += 1;
                }
            }
        }

        // 2.1) Review Sponsor Request Messages
        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: Some(5),
            start_page: Some(1),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 6);
                // [5] -> index [11]
                // User addr  [user11]
                // User index [  11  ]
                let mut ind = 0;
                for user in [11] {
                    let addr = format!("user{}", user);
                    assert_eq!(vec[ind].index, Some((user) as u32));
                    assert_eq!(vec[ind].addr, addr);
                    ind += 1;
                }
            }
        }

        let mut decision_vec: Vec<Review> = vec![];
        for i in 0..5 {
            decision_vec.push(Review {
                index: i,
                is_accpeted: true,
            });
        }
        let handle_msg = HandleMsg::ReviewSponsors {
            decisions: decision_vec,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("reviewer", &[]),
            handle_msg,
        )?;

        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: Some(5),
            start_page: Some(0),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 1);
                assert_eq!(len, 1);
                let mut ind = 0;
                for user in [11] {
                    let addr = format!("user{}", user);
                    assert_eq!(vec[ind].addr, addr);
                    assert_eq!(vec[ind].index, Some(user as u32));
                    ind += 1;
                }
            }
        }

        for i in 13..15 {
            let sponsor = format!("user{}", i);
            let title = format!("user {} title", i);
            let handle_msg = HandleMsg::Sponsor {
                title: Some(title),
                message: Some("Some message".to_string()),
            };
            let _res = execute(
                deps.as_mut(),
                custom_mock_env(
                    None,
                    Some(mock_env().block.time.seconds().add(i)),
                    None,
                    None,
                ),
                mock_info(&sponsor, &[Coin {
                    denom: "uscrt".to_string(),
                    amount: Uint128::from(10000 * SCRT_TO_USCRT),
                }]),
                handle_msg,
            )?;
        }

        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: Some(5),
            start_page: Some(0),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 3);
                assert_eq!(len, 3);
                let mut ind = 0;
                for user in [11, 13, 14] {
                    let addr = format!("user{}", user);
                    assert_eq!(vec[ind].addr, addr);
                    assert_eq!(vec[ind].index, Some(user));
                    ind += 1;
                }
            }
        }

        let mut decision_vec: Vec<Review> = vec![];
        for i in 0..3 {
            decision_vec.push(Review {
                index: i,
                is_accpeted: true,
            });
        }
        let handle_msg = HandleMsg::ReviewSponsors {
            decisions: decision_vec,
        };
        let _res = execute(
            deps.as_mut(),
            custom_mock_env(None, None, None, None),
            mock_info("reviewer", &[]),
            handle_msg,
        )?;

        let query_msg = QueryMsg::SponsorMessageRequestCheck {
            page_size: Some(5),
            start_page: Some(0),
        };
        let env = custom_mock_env(None, None, None, None);
        let query_result = query(deps.as_ref(), env, query_msg)?;

        let query_answer = from_binary(&query_result)?;
        match query_answer {
            //no increase in Request because the no request was made.
            SponsorMessageReqResponse { vec, len } => {
                assert_eq!(vec.len(), 0);
                assert_eq!(len, 0);
            }
        }

        Ok(())
    }

    //     ////////////////////////////////////// Simulations //////////////////////////////////////

    // #[test]
    // fn claim_rewards_liq() -> StdResult<()> {
    //     let (init_result, mut deps) = init_helper(None);

    //     let round_obj = round_obj_read_only_unit_test_helper(deps.as_ref().storage);
    //     //deposit after the round_time
    //     let _ = deposit_unit_test_helper(
    //         deps.as_mut(),
    //         Uint128::from(10 * SCRT_TO_USCRT),
    //         Some(0),
    //         Some(round_obj.end_time.add(3600)),
    //         "secretbatman",
    //         Some("uscrt"),
    //     )?;

    //     //ends round
    //     let handle_msg = HandleMsg::EndRound {};
    //     let mocked_env =
    //         custom_mock_env(None, Some(round_obj.end_time.add(3600 * 2)), None, None);
    //     let mocked_info = mock_info("triggerer", &[]);
    //     let handle_result = execute(deps.as_mut(), mocked_env, mocked_info, handle_msg)?;

    //     //claims rewards
    //     let res = claim_rewards_unit_test_helper(
    //         deps.as_mut(),
    //         None,
    //         Some(round_obj.end_time.add(3600 * 3)),
    //         "secretbatman",
    //     )?;

    //     let round_obj = round_obj_read_only_unit_test_helper(deps.as_ref().storage);

    //     //ends round
    //     let handle_msg = HandleMsg::EndRound {};
    //     let mocked_env =
    //         custom_mock_env(None, Some(round_obj.end_time.add(3600 * 2)), None, None);
    //     let mocked_info = mock_info("triggerer", &[]);

    //     let handle_result = execute(deps.as_mut(), mocked_env, mocked_info, handle_msg)?;

    //     //claims rewards
    //     let res = claim_rewards_unit_test_helper(
    //         deps.as_mut(),
    //         None,
    //         Some(round_obj.end_time.add(3600 * 3)),
    //         "secretbatman",
    //     )?;
    //     Ok(())
    // }

    // #[test]
    // fn claim_rewards_sim() -> StdResult<()> {
    //     let (init_result, mut deps) = init_helper(None);

    //     //deposit after the end round_time
    //     for i in 0..50 {
    //         let round_obj = round_obj_read_only_unit_test_helper(deps.as_ref().storage);

    //         let _ = deposit_unit_test_helper(
    //             deps.as_mut(),
    //             Uint128::from(50000 * SCRT_TO_USCRT),
    //             Some(0),
    //             Some(round_obj.end_time.add(3600)),
    //             "secretbatman",
    //             Some("uscrt"),
    //         )?;

    //         //ends round
    //         let handle_msg = HandleMsg::EndRound {};
    //         let mocked_env =
    //             custom_mock_env(None, Some(round_obj.end_time.add(3600 * 2)), None, None);
    //         let mocked_info = mock_info("triggerer", &[]);
    //         let handle_result = execute(deps.as_mut(), mocked_env, mocked_info, handle_msg)?;

    //         // println!("{}", i);
    //         //claims rewards
    //         // let res = claim_rewards_unit_test_helper(
    //         //     deps.as_mut(),
    //         //     None,
    //         //     Some(round_obj.end_time.add(3600 * 3)),
    //         //     "secretbatman",
    //         // )?;
    //     }

    //     // let round_obj = round_obj_read_only_unit_test_helper(deps.as_ref().storage);

    //     // //ends round
    //     // let handle_msg = HandleMsg::EndRound {};
    //     // let mocked_env =
    //     //     custom_mock_env(None, Some(round_obj.end_time.add(3600 * 2)), None, None);
    //     // let mocked_info = mock_info("triggerer", &[]);

    //     // let handle_result = execute(deps.as_mut(), mocked_env, mocked_info, handle_msg)?;

    //     // //claims rewards
    //     // let res = claim_rewards_unit_test_helper(
    //     //     deps.as_mut(),
    //     //     None,
    //     //     Some(round_obj.end_time.add(3600 * 3)),
    //     //     "secretbatman",
    //     // )?;
    //     Ok(())
    // }

    // #[test]
    // fn testing_a_error() -> StdResult<()> {
    //     let (init_result, mut deps) = init_helper(None);
    //     let round_obj = round_obj_read_only_unit_test_helper(deps.as_ref().storage);

    //     let _ = deposit_unit_test_helper(
    //         deps.as_mut(),
    //         Uint128::from(44500000u128),
    //         Some(0),
    //         None,
    //         "secretbatman",
    //         Some("uscrt"),
    //     )?;

    //     let _ = deposit_unit_test_helper(
    //         deps.as_mut(),
    //         Uint128::from(32000001u128),
    //         Some(0),
    //         None,
    //         "secretbatman",
    //         Some("uscrt"),
    //     )?;

    //     let res = request_withdraw_unit_test_helper(
    //         deps.as_mut(),
    //         Uint128::from(76500001 as u128),
    //         None,
    //         None,
    //         "secretbatman",
    //     );

    //     let config = config_read_only_unit_test_helper(deps.as_ref().storage);
    //     let handle_msg = HandleMsg::UnbondBatch {};
    //     let _res = execute(
    //         deps.as_mut(),
    //         custom_mock_env(None, Some(config.next_unbonding_batch_time), None, None),
    //         mock_info("triggerer", &[]),
    //         handle_msg,
    //     )?;

    //     Ok(())
    // }

    //     // #[test]
    //     // fn testing_claim_percentage_worst_case() -> StdResult<()> {
    //     //     let (_init_result, mut deps) = init_helper(None);
    //     //     let mut tier_0: u128 = 0;
    //     //     let mut tier_1: u128 = 0;
    //     //     let mut tier_2: u128 = 0;
    //     //     let mut tier_3: u128 = 0;
    //     //     let mut tier_4: u128 = 0;
    //     //     let mut tier_5: u128 = 0;

    //     //     let total_number_of_rounds = 2;
    //     //     let number_of_deposits = 100000;

    //     //     for i in 0..number_of_deposits {
    //     //         let amount_to_delegate = Uint128::from(1 * SCRT_TO_USCRT);
    //     //         deposit_unit_test_helper(
    //     //             deps.as_mut(),
    //     //             amount_to_delegate,
    //     //             None,
    //     //             None,
    //     //             i.to_string().as_str(),
    //     //             None,
    //     //         )?;
    //     //         if i % 1000 == 0 {
    //     //             print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    //     //             print!("Deposit: {:?}", i);
    //     //         }
    //     //     }
    //     //     for round in 1..total_number_of_rounds {
    //     //         let _ = End_Round_my_mocked_querier_unit_test_helper(deps.as_mut()).unwrap();
    //     //         let round_obj = round_obj_unit_test_helper(&deps.storage);
    //     //         for i in 0..number_of_deposits {
    //     //             if i % 1000 == 0 {
    //     //                 print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

    //     //                 println!("Claim Rewards: {:?}", i);
    //     //             }

    //     //             let _ = claim_rewards_unit_test_helper(
    //     //                 deps.as_mut(),
    //     //                 None,
    //     //                 Some(round_obj.end_time),
    //     //                 i.to_string().as_str(),
    //     //             )?;
    //     //         }

    //     //         let rewards_stats_for_nth_round_obj =
    //     //             rewards_stats_for_nth_round_unit_test_helper(&deps.storage, round);

    //     //         tier_0 += rewards_stats_for_nth_round_obj
    //     //             .distribution_per_tiers
    //     //             .tier_0
    //     //             .total_prizes_claimed
    //     //             .u128();
    //     //         tier_1 += rewards_stats_for_nth_round_obj
    //     //             .distribution_per_tiers
    //     //             .tier_1
    //     //             .total_prizes_claimed
    //     //             .u128();
    //     //         tier_2 += rewards_stats_for_nth_round_obj
    //     //             .distribution_per_tiers
    //     //             .tier_2
    //     //             .total_prizes_claimed
    //     //             .u128();
    //     //         tier_3 += rewards_stats_for_nth_round_obj
    //     //             .distribution_per_tiers
    //     //             .tier_3
    //     //             .total_prizes_claimed
    //     //             .u128();
    //     //         tier_4 += rewards_stats_for_nth_round_obj
    //     //             .distribution_per_tiers
    //     //             .tier_4
    //     //             .total_prizes_claimed
    //     //             .u128();
    //     //         tier_5 += rewards_stats_for_nth_round_obj
    //     //             .distribution_per_tiers
    //     //             .tier_5
    //     //             .total_prizes_claimed
    //     //             .u128();
    //     //     }

    //     //     println!(
    //     //         "Average Tier 5 prizes claimed: {} of 243, 20% of total rewards",
    //     //         (tier_5 as f64 / total_number_of_rounds as f64)
    //     //     );
    //     //     println!(
    //     //         "Average Tier 4 prizes claimed: {} of 81, 10% of total rewards",
    //     //         (tier_4 as f64 / total_number_of_rounds as f64)
    //     //     );
    //     //     println!(
    //     //         "Average Tier 3 prizes claimed: {} of 27, 14% of total rewards",
    //     //         (tier_3 as f64 / total_number_of_rounds as f64)
    //     //     );
    //     //     println!(
    //     //         "Average Tier 2 prizes claimed: {} of 9, 12% of total rewards",
    //     //         (tier_2 as f64 / total_number_of_rounds as f64)
    //     //     );
    //     //     println!(
    //     //         "Average Tier 1 prizes claimed: {} of 3, 19% of total rewards",
    //     //         (tier_1 as f64 / total_number_of_rounds as f64)
    //     //     );
    //     //     println!(
    //     //         "Average Tier 0 prizes claimed: {} of 1, 25% of total rewards",
    //     //         (tier_0 as f64 / total_number_of_rounds as f64)
    //     //     );
    //     //     println!(
    //     //         "% of Total Prizes claimed: {:.2}%",
    //     //         ((tier_5 as f64 / total_number_of_rounds as f64) / 243 as f64) * 20 as f64
    //     //             + ((tier_4 as f64 / total_number_of_rounds as f64) / 81 as f64) * 10 as f64
    //     //             + ((tier_3 as f64 / total_number_of_rounds as f64) / 27 as f64) * 14 as f64
    //     //             + ((tier_2 as f64 / total_number_of_rounds as f64) / 9 as f64) * 12 as f64
    //     //             + ((tier_1 as f64 / total_number_of_rounds as f64) / 3 as f64) * 19 as f64
    //     //             + ((tier_0 as f64 / total_number_of_rounds as f64) / 1 as f64) * 25 as f64
    //     //     );
    //     //     Ok(())
    //     // }

    //     // #[test]
    //     // fn testing_claim_percentage() -> StdResult<()> {
    //     //     //Calculating the average claim rate for the rewards
    //     //     //Setting dynamic ticket multiplier
    //     //     let mut deps = deposit_unit_test_simple_helper_filler(None);

    //     //     let mut tier_0: u128 = 0;
    //     //     let mut tier_1: u128 = 0;
    //     //     let mut tier_2: u128 = 0;
    //     //     let mut tier_3: u128 = 0;
    //     //     let mut tier_4: u128 = 0;
    //     //     let mut tier_5: u128 = 0;

    //     //     for round in 1..100 {
    //     //         let _ = end_round_my_mocked_querier_unit_test_helper(deps.as_mut()).unwrap();
    //     //         let round_obj = round_unit_test_helper(&deps.storage);
    //     //         let _ = claim_rewards_unit_test_helper(
    //     //             deps.as_mut(),
    //     //             None,
    //     //             Some(round_obj.end_time),
    //     //             "Superman",
    //     //         )?;
    //     //         let _ = claim_rewards_unit_test_helper(
    //     //             deps.as_mut(),
    //     //             None,
    //     //             Some(round_obj.end_time),
    //     //             "Spider-man",
    //     //         )?;
    //     //         let _ = claim_rewards_unit_test_helper(
    //     //             deps.as_mut(),
    //     //             None,
    //     //             Some(round_obj.end_time),
    //     //             "Wonder-Women",
    //     //         )?;
    //     //         let _ = claim_rewards_unit_test_helper(
    //     //             deps.as_mut(),
    //     //             None,
    //     //             Some(round_obj.end_time),
    //     //             "Aqua-man",
    //     //         )?;
    //     //         let _ = claim_rewards_unit_test_helper(
    //     //             deps.as_mut(),
    //     //             None,
    //     //             Some(round_obj.end_time),
    //     //             "Ironman",
    //     //         )?;
    //     //         let _ = claim_rewards_unit_test_helper(
    //     //             deps.as_mut(),
    //     //             None,
    //     //             Some(round_obj.end_time),
    //     //             "Loki",
    //     //         )?;
    //     //         let _ = claim_rewards_unit_test_helper(
    //     //             deps.as_mut(),
    //     //             None,
    //     //             Some(round_obj.end_time),
    //     //             "Captain-America",
    //     //         )?;
    //     //         let _ = claim_rewards_unit_test_helper(
    //     //             deps.as_mut(),
    //     //             None,
    //     //             Some(round_obj.end_time),
    //     //             "Thor",
    //     //         )?;
    //     //         let _ = claim_rewards_unit_test_helper(
    //     //             deps.as_mut(),
    //     //             None,
    //     //             Some(round_obj.end_time),
    //     //             "secretbatman",
    //     //         )?;

    //     //         let rewards_stats_for_nth_round_obj =
    //     //             rewards_stats_for_nth_round_unit_test_helper(&deps.storage, round);

    //     //         println!(
    //     //             "Tier 0:{} 1:{} 2:{} 3:{} 4:{} 5:{}",
    //     //             rewards_stats_for_nth_round_obj
    //     //                 .distribution_per_tiers
    //     //                 .tier_0
    //     //                 .total_prizes_claimed
    //     //                 .u128(),
    //     //             rewards_stats_for_nth_round_obj
    //     //                 .distribution_per_tiers
    //     //                 .tier_1
    //     //                 .total_prizes_claimed
    //     //                 .u128(),
    //     //             rewards_stats_for_nth_round_obj
    //     //                 .distribution_per_tiers
    //     //                 .tier_2
    //     //                 .total_prizes_claimed
    //     //                 .u128(),
    //     //             rewards_stats_for_nth_round_obj
    //     //                 .distribution_per_tiers
    //     //                 .tier_3
    //     //                 .total_prizes_claimed
    //     //                 .u128(),
    //     //             rewards_stats_for_nth_round_obj
    //     //                 .distribution_per_tiers
    //     //                 .tier_4
    //     //                 .total_prizes_claimed
    //     //                 .u128(),
    //     //             rewards_stats_for_nth_round_obj
    //     //                 .distribution_per_tiers
    //     //                 .tier_5
    //     //                 .total_prizes_claimed
    //     //                 .u128()
    //     //         );

    //     //         tier_0 += rewards_stats_for_nth_round_obj
    //     //             .distribution_per_tiers
    //     //             .tier_0
    //     //             .total_prizes_claimed
    //     //             .u128();
    //     //         tier_1 += rewards_stats_for_nth_round_obj
    //     //             .distribution_per_tiers
    //     //             .tier_1
    //     //             .total_prizes_claimed
    //     //             .u128();
    //     //         tier_2 += rewards_stats_for_nth_round_obj
    //     //             .distribution_per_tiers
    //     //             .tier_2
    //     //             .total_prizes_claimed
    //     //             .u128();
    //     //         tier_3 += rewards_stats_for_nth_round_obj
    //     //             .distribution_per_tiers
    //     //             .tier_3
    //     //             .total_prizes_claimed
    //     //             .u128();
    //     //         tier_4 += rewards_stats_for_nth_round_obj
    //     //             .distribution_per_tiers
    //     //             .tier_4
    //     //             .total_prizes_claimed
    //     //             .u128();
    //     //         tier_5 += rewards_stats_for_nth_round_obj
    //     //             .distribution_per_tiers
    //     //             .tier_5
    //     //             .total_prizes_claimed
    //     //             .u128();
    //     //     }

    //     //     println!(
    //     //         "Average Tier 5 prizes claimed: {} of 243, 20% of total rewards",
    //     //         (tier_5 as f64 / 100 as f64)
    //     //     );
    //     //     println!(
    //     //         "Average Tier 4 prizes claimed: {} of 81, 10% of total rewards",
    //     //         (tier_4 as f64 / 100 as f64)
    //     //     );
    //     //     println!(
    //     //         "Average Tier 3 prizes claimed: {} of 27, 14% of total rewards",
    //     //         (tier_3 as f64 / 100 as f64)
    //     //     );
    //     //     println!(
    //     //         "Average Tier 2 prizes claimed: {} of 9, 12% of total rewards",
    //     //         (tier_2 as f64 / 100 as f64)
    //     //     );
    //     //     println!(
    //     //         "Average Tier 1 prizes claimed: {} of 3, 19% of total rewards",
    //     //         (tier_1 as f64 / 100 as f64)
    //     //     );
    //     //     println!(
    //     //         "Average Tier 0 prizes claimed: {} of 1, 25% of total rewards",
    //     //         (tier_0 as f64 / 100 as f64)
    //     //     );
    //     //     println!(
    //     //         "% of Total Prizes claimed: {:.2}%",
    //     //         ((tier_5 as f64 / 100 as f64) / 243 as f64) * 20 as f64
    //     //             + ((tier_4 as f64 / 100 as f64) / 81 as f64) * 10 as f64
    //     //             + ((tier_3 as f64 / 100 as f64) / 27 as f64) * 14 as f64
    //     //             + ((tier_2 as f64 / 100 as f64) / 9 as f64) * 12 as f64
    //     //             + ((tier_1 as f64 / 100 as f64) / 3 as f64) * 19 as f64
    //     //             + ((tier_0 as f64 / 100 as f64) / 1 as f64) * 25 as f64
    //     //     );
    //     //     Ok(())
    //     // }
}
