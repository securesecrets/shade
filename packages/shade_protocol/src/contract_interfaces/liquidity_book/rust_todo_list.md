  - lb_factory
    - src
        - contract.rs
            - instantiate
            - execute
            - try_set_lb_pair_implementation
            - try_set_lb_token_implementation
            - try_set_staking_contract_implementation
            - try_create_lb_pair
            - try_set_lb_pair_ignored
            - try_set_pair_preset
            - try_set_preset_open_state
            - try_remove_preset
            - try_set_fee_parameters_on_pair
            - try_set_fee_recipient
            - try_add_quote_asset
            - try_remove_quote_asset
            - try_force_decay
            - query
            - query_min_bin_step
            - query_fee_recipient
            - query_lb_pair_implementation
            - query_lb_token_implementation
            - query_number_of_lb_pairs
            - query_lb_pair_at_index
            - query_number_of_quote_assets
            - query_quote_asset_at_index
            - query_is_quote_asset
            - query_lb_pair_information
            - _get_lb_pair_information
            - _sort_tokens
            - query_preset
            - query_all_bin_steps
            - query_open_bin_steps
            - _is_preset_open
            - query_all_lb_pairs
            - reply
        - error.rs
        - lib.rs
        - prelude.rs
        - state.rs
            - ephemeral_storage_w
            - ephemeral_storage_r
        - types.rs
  - lb_pair
    - src
        - contract.rs
            - instantiate
            - execute
            - receiver_callback
            - query
            - reply
        - error.rs
        - execute.rs
            - try_swap
            - update_fee_map_tree
            - updating_reward_stats
            - updating_oracles
            - updating_bin_reserves
            - try_add_liquidity
            - add_liquidity_internal
            - mint
            - mint_bins
            - update_bin
            - try_remove_liquidity
            - remove_liquidity
            - burn
            - try_collect_protocol_fees
            - try_set_static_fee_parameters
            - try_force_decay
            - try_calculate_rewards_distribution
            - calculate_default_distribution
            - calculate_time_based_rewards_distribution
            - calculate_volume_based_rewards_distribution
            - try_reset_rewards_config
        - helper.rs
            - register_pair_token
            - match_lengths
            - check_ids_bounds
            - check_active_id_slippage
            - calculate_id
            - _query_total_supply
            - query_token_symbol
            - _get_next_non_empty_bin
            - only_factory
        - lib.rs
        - prelude.rs
        - query.rs
            - query_pair_info
            - query_swap_simulation
            - query_factory
            - query_lb_token
            - query_staking
            - query_tokens
            - query_token_x
            - query_token_y
            - query_bin_step
            - query_reserves
            - query_active_id
            - query_all_bins_reserves
            - query_bins_reserves
            - query_updated_bins_at_height
            - query_updated_bins_at_multiple_heights
            - query_updated_bins_after_height
            - query_bins_updating_heights
            - query_bin_reserves
            - query_next_non_empty_bin
            - query_protocol_fees
            - query_static_fee_params
            - query_variable_fee_params
            - query_oracle_params
            - query_oracle_sample
            - query_oracle_samples
            - query_oracle_samples_after
            - query_price_from_id
            - query_id_from_price
            - query_swap_in
            - query_swap_out
            - query_total_supply
            - query_rewards_distribution
        - state.rs
  - lb_staking
    - src
        - contract.rs
            - instantiate
            - execute
            - authenticate
            - query
        - execute.rs
            - receiver_callback_snip_1155
            - receiver_callback
            - try_stake
            - try_unstake
            - update_staker_and_total_liquidity
            - try_end_epoch
            - try_claim_rewards
            - try_register_reward_tokens
            - try_update_config
            - process_epoch
            - try_add_rewards
            - try_recover_expired_funds
            - try_recover_funds
        - helper.rs
            - register_reward_tokens
            - store_empty_reward_set
            - require_lb_token
            - staker_init_checker
            - assert_lb_pair
            - check_if_claimable
            - finding_user_liquidity
            - finding_total_liquidity
            - eq
            - hash<H: Hasher>
            - get_txs
        - lib.rs
        - query.rs
            - query_contract_info
            - query_epoch_info
            - query_registered_tokens
            - query_token_id_balance
            - query_staker_info
            - query_balance
            - query_all_balances
            - query_transaction_history
            - query_liquidity
        - state.rs
            - store_stake
            - store_unstake
            - store_claim_rewards
            - append_tx_for_addr
            - append_stake_tx_for_addr
            - append_unstake_tx_for_addr
            - append_claim_rewards_tx_for_addr
  - lb_token
    - src
        - contract.rs
            - instantiate
            - execute
            - query
            - permit_queries
            - viewing_keys_queries
        - execute.rs
            - try_curate_token_ids
            - try_mint_tokens
            - try_burn_tokens
            - try_change_metadata
            - try_transfer
            - try_batch_transfer
            - try_send
            - try_batch_send
            - try_give_permission
            - try_revoke_permission
            - try_create_viewing_key
            - try_set_viewing_key
            - try_revoke_permit
            - try_add_curators
            - try_remove_curators
            - try_add_minters
            - try_remove_minters
            - try_change_admin
            - try_remove_admin
            - try_register_receive
            - pad_response
            - is_valid_name
            - is_valid_symbol
            - verify_admin
            - verify_curator
            - verify_curator_of_token_id
            - verify_minter
            - exec_curate_token_id
            - impl_send
            - impl_transfer
            - exec_change_balance
            - try_add_receiver_api_callback
        - lib.rs
        - query.rs
            - query_contract_info
            - query_id_balance
            - query_token_id_public_info
            - query_token_id_private_info
            - query_registered_code_hash
            - query_balance
            - query_all_balances
            - query_transactions
            - query_permission
            - query_all_permissions
        - state
        - mod.rs
            - contr_conf_w
            - contr_conf_r
            - blockinfo_w
            - blockinfo_r
            - tkn_info_w
            - tkn_info_r
            - tkn_tot_supply_w
            - tkn_tot_supply_r
            - balances_w<'a>
            - balances_r<'a>
            - permission_w<'a>
            - permission_r<'a>
            - perm_r<'a>
            - get_receiver_hash
            - set_receiver_hash
        - permissions.rs
            - new_permission
            - update_permission_unchecked
            - update_permission
            - may_load_any_permission
            - may_load_active_permission
            - list_owner_permission_keys
            - append_permission_for_addr
        - save_load_functions.rs
            - json_save<T: Serialize>
            - json_load<T: DeserializeOwned>
            - json_may_load<T: DeserializeOwned, S: ReadonlyStorage>
        - txhistory.rs
            - get_txs
            - store_transfer
            - store_mint
            - store_burn
            - append_tx_for_addr
            - append_new_owner
            - may_get_current_owner
  - router
    - src
        - contract.rs
            - instantiate
            - execute
            - receiver_callback
            - query
            - reply
        - execute.rs
            - refresh_tokens
            - next_swap
            - swap_tokens_for_exact_tokens
            - update_viewing_key
            - get_trade_with_callback
            - register_pair_token
        - lib.rs
        - query.rs
            - pair_contract_config
            - swap_simulation
        - state.rs
            - config_w
            - config_r
            - registered_tokens_w
            - registered_tokens_r
            - registered_tokens_list_w
            - registered_tokens_list_r
            - epheral_storage_w
            - epheral_storage_r
  - tests
    - src
        - lib.rs
        - multitests
        - lb_factory.rs
            - test_setup
            - test_set_lb_pair_implementation
            - test_revert_set_lb_pair_implementation
            - test_set_lb_token_implementation
            - test_create_lb_pair
            - test_create_lb_pair_factory_unlocked
            - test_revert_create_lb_pair
            - test_fuzz_set_preset
            - test_remove_preset
            - test_set_fees_parameters_on_pair
            - test_set_fee_recipient
            - test_fuzz_open_presets
            - test_add_quote_asset
            - test_remove_quote_asset
            - test_force_decay
            - test_get_all_lb_pair
        - lb_pair_fees.rs
            - lb_pair_setup
            - test_fuzz_swap_in_x
            - test_fuzz_swap_in_y
            - test_fuzz_swap_out_for_x
            - test_fuzz_swap_out_for_y
            - test_fuzz_swap_in_x_and_y
            - test_fuzz_swap_in_y_and_x
            - test_fuzz_swap_out_x_and_y
            - test_fuzz_swap_out_y_and_x
            - test_fee_x_lp
            - test_fee_y_lp
            - test_collect_protocol_fees_x_tokens
            - test_collect_protocol_fees_y_tokens
            - test_collect_protocol_fees_both_tokens
            - test_collect_protocol_fees_after_swap
            - test_revert_total_fee_exceeded
            - test_fuzz_swap_in_x_and_y_btc_silk
            - test_fuzz_base_fee_only
            - test_base_and_variable_fee_only
        - lb_pair_initial_state.rs
            - lb_pair_setup
            - test_query_factory
            - test_query_token_x
            - test_query_token_y
            - test_query_bin_step
            - test_query_bin_reserves
            - test_query_active_id
            - test_fuzz_query_bin
            - test_query_next_non_empty_bin
            - test_query_protocol_fees
            - test_query_static_fee_parameters
            - test_query_variable_fee_parameters
            - test_query_oracle_parameters
            - test_query_oracle_sample_at
            - test_query_price_from_id
            - test_query_id_from_price
            - test_fuzz_query_swap_out
            - test_fuzz_query_swap_in
            - test_invalid_reward_bins_error
        - lb_pair_liquidity.rs
            - lb_pair_setup
            - test_simple_mint_repeat
            - test_simple_mint
            - test_mint_twice
            - test_mint_with_different_bins
            - test_simple_burn
            - test_burn_half_twice
            - test_query_next_non_empty_bin
            - test_revert_mint_zero_shares
            - test_revert_burn_empty_array
            - test_revert_burn_more_than_balance
            - test_revert_burn_zero
            - test_revert_on_deadline
            - test_revert_on_wrong_pair
            - test_revert_on_amount_slippage
            - test_revert_on_length_mismatch
            - test_revert_on_id_desired_overflow
            - test_revert_on_id_slippage_caught
            - test_revert_on_delta_ids_overflow
            - test_revert_on_empty_liquidity_config
            - testing_implicit_swap
            - test_revert_burn_on_wrong_pair
        - lb_pair_oracle.rs
            - lb_pair_setup
            - test_query_oracle_parameters
            - test_query_oracle_sample_at_init
            - test_query_oracle_sample_at_one_swap
            - test_fuzz_query_oracle_sample_at_one_swap
            - test_fuzz_update_oracle_id
            - test_fuzz_update_cumm_txns
            - test_fuzz_query_oracle_sample_after
        - lb_pair_queries.rs
            - lb_pair_setup
            - mint_and_add_liquidity
            - test_query_bin_reserves
            - test_query_bins_reserves
            - test_query_all_bins_reserves
            - test_query_all_bins_updated_add_liquidity
            - test_query_total_supply
            - test_query_tokens
            - test_query_all_bins_updated_swap
            - test_query_all_bins_updated_remove_liquidity
            - test_query_update_at_height
            - test_query_update_at_multiple_heights
            - test_query_update_after_height
        - lb_pair_rewards.rs
            - test_fuzz_calculate_volume_based_rewards
            - test_calculate_volume_based_rewards
            - test_calculate_time_based_rewards
            - test_fuzz_calculate_time_based_rewards
            - test_reset_rewards_config
        - lb_pair_swap.rs
            - lb_pair_setup
            - test_fuzz_swap_in_x
            - test_fuzz_swap_in_y
            - test_fuzz_swap_out_for_y
            - test_fuzz_swap_out_for_y_send_someone
            - test_fuzz_swap_out_for_x
            - test_revert_swap_insufficient_amount_in
            - test_revert_swap_insufficient_amount_out
            - test_revert_swap_out_of_liquidity
            - test_revert_zero_bin_reserves
        - lb_pair_trivial.rs
            - lb_pair_setup
            - test_contract_status
            - test_native_tokens_error
        - lb_router_integration.rs
            - router_integration
        - lb_router_register_tokens.rs
            - router_registered_tokens
        - lb_staking.rs
            - lb_pair_setup
            - mint_and_add_liquidity
            - staking_contract_init
            - fuzz_stake_simple
            - fuzz_stake_liquidity_with_time
            - fuzz_unstake
            - fuzz_unstake_liquidity_with_time
            - register_rewards_token
            - add_rewards
            - end_epoch
            - fuzz_claim_rewards
            - claim_rewards
            - end_epoch_by_stakers
            - claim_expired_rewards
            - recover_expired_rewards
            - recover_funds
            - update_config
            - query_contract_info
            - query_id_balance
            - query_balance
            - query_all_balance
            - query_txn_history
        - lb_token.rs
            - init_setup
            - test_simple_mint
            - test_mint_twice
            - test_mint_with_different_bins
            - test_simple_burn
            - test_burn_half_twice
            - test_revert_mint_zero_tokens
            - test_revert_burn_empty_array
            - test_revert_burn_more_than_balance
            - test_revert_burn_zero
        - mod.rs
        - test_helper.rs
            - admin
            - user1
            - user2
            - batman
            - scare_crow
            - joker
            - all
            - a_hash
            - b_hash
            - c_hash
            - d_hash
            - init_addrs
            - assert_approx_eq_rel
            - assert_approx_eq_abs
            - generate_auth
            - setup
            - roll_blockchain
            - roll_time
            - extract_contract_info
            - token_type_snip20_generator
            - token_type_native_generator
            - safe64_divide
            - get_id
            - get_total_bins
            - safe24
            - bound<T: PartialOrd>
            - generate_random<T>
            - liquidity_parameters_generator
            - liquidity_parameters_generator_custom
            - liquidity_parameters_generator_with_native
            - mint_increase_allowance_helper
            - mint_token_helper
            - increase_allowance_helper