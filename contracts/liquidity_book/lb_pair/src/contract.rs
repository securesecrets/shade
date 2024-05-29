use crate::{execute::*, helper::*, prelude::*, query::*, state::*};
use lb_libraries::{
    lb_token::state_structs::LbPair,
    math::{sample_math::OracleSample, tree_math::TreeUint24, u24::U24},
    oracle_helper::Oracle,
    pair_parameter_helper::PairParameters,
};
use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{
        from_binary, shd_entry_point, to_binary, Addr, Binary, ContractInfo, CosmosMsg, Deps,
        DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult, SubMsg, SubMsgResult,
        Uint128, Uint256, WasmMsg,
    },
    contract_interfaces::liquidity_book::{lb_pair::*, lb_staking, lb_token},
    swap::core::{TokenAmount, TokenType, ViewingKey},
};
use std::vec;

/////////////// INSTANTIATE ///////////////
#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response> {
    // Constants
    const EMPTY_ADDR: &str = "";
    const EMPTY_STRING: &str = "";
    const LB_TOKEN_DECIMALS: u8 = 18;
    const START_ORACLE_ID: u16 = 1;
    const START_REWARDS_EPOCH: u64 = 1;
    let tree: TreeUint24 = TreeUint24::new();
    let mut oracle = Oracle(OracleSample::default());

    // Initializing the Token Contract
    let token_x_symbol = match msg.token_x.clone() {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => query_token_symbol(deps.as_ref(), token_code_hash, contract_addr)?,
        TokenType::NativeToken { denom } => denom,
    };

    let token_y_symbol = match msg.token_y.clone() {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => query_token_symbol(deps.as_ref(), token_code_hash, contract_addr)?,
        TokenType::NativeToken { denom } => denom,
    };

    let instantiate_token_msg = lb_token::InstantiateMsg {
        has_admin: false,
        admin: None,
        curators: [env.contract.address.clone()].to_vec(),
        entropy: msg.entropy,
        lb_pair_info: LbPair {
            name: format!(
                "Lb-token-{}-{}-{}",
                token_x_symbol, token_y_symbol, &msg.bin_step
            ),
            symbol: format!("LB-{}-{}-{}", token_x_symbol, token_y_symbol, &msg.bin_step),
            lb_pair_address: env.contract.address.clone(),
            decimals: LB_TOKEN_DECIMALS,
        },
        initial_tokens: Vec::new(),
    };

    let mut response = Response::new();
    response = response.add_submessage(SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: msg.lb_token_implementation.id,
            code_hash: msg.lb_token_implementation.code_hash.clone(),
            msg: to_binary(&instantiate_token_msg)?,
            label: format!(
                "{}-{}-Pair-Token-{}-{}",
                token_x_symbol, token_y_symbol, msg.bin_step, env.block.height
            ),
            funds: vec![],
            admin: None,
        }),
        INSTANTIATE_LP_TOKEN_REPLY_ID,
    ));

    //Initializing PairParameters
    let mut pair_parameters = PairParameters::default();
    pair_parameters.set_static_fee_parameters(
        msg.pair_parameters.base_factor,
        msg.pair_parameters.filter_period,
        msg.pair_parameters.decay_period,
        msg.pair_parameters.reduction_factor,
        msg.pair_parameters.variable_fee_control,
        msg.pair_parameters.protocol_share,
        msg.pair_parameters.max_volatility_accumulator,
    )?;
    pair_parameters.set_active_id(msg.active_id)?;
    pair_parameters.set_oracle_id(START_ORACLE_ID); // Activating the oracle
    pair_parameters.update_id_reference();

    // RegisterReceiving Token
    let mut messages = vec![];
    let viewing_key = ViewingKey::from(msg.viewing_key.as_str());
    for token in [&msg.token_x, &msg.token_y] {
        if let TokenType::CustomToken {
            contract_addr: _,
            token_code_hash: _,
        } = token
        {
            register_pair_token(&env, &mut messages, token, &viewing_key)?;
        }
    }

    if let Some(t_r_b) = msg.total_reward_bins {
        if t_r_b >= U24::MAX {
            return Err(Error::InvalidInput {});
        }
    }

    // State initialization
    let state = State {
        creator: info.sender,
        factory: msg.factory,
        token_x: msg.token_x,
        token_y: msg.token_y,
        bin_step: msg.bin_step,
        pair_parameters,
        reserves: [0u8; 32],
        protocol_fees: [0u8; 32],

        // ContractInfo for lb_token and lb_staking are intentionally kept empty and will be filled in later
        lb_token: ContractInfo {
            address: Addr::unchecked(EMPTY_ADDR.to_string()),
            code_hash: EMPTY_STRING.to_string(),
        },
        lb_staking: ContractInfo {
            address: Addr::unchecked(EMPTY_ADDR.to_string()),
            code_hash: EMPTY_STRING.to_string(),
        },

        viewing_key,
        protocol_fees_recipient: msg.protocol_fee_recipient,
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
        last_swap_timestamp: env.block.time,
        rewards_epoch_index: START_REWARDS_EPOCH,
        base_rewards_bins: msg.total_reward_bins,
        toggle_distributions_algorithm: false,
        max_bins_per_swap: msg.max_bins_per_swap.unwrap_or(DEFAULT_MAX_BINS_PER_SWAP),
    };

    oracle.0 = *oracle.0.set_created_at(env.block.time.seconds());

    STATE.save(deps.storage, &state)?;
    ORACLE.save(deps.storage, pair_parameters.get_oracle_id(), &oracle)?;
    CONTRACT_STATUS.save(deps.storage, &ContractStatus::Active)?;
    BIN_TREE.save(deps.storage, &tree)?;
    FEE_MAP_TREE.save(deps.storage, state.rewards_epoch_index, &tree)?;
    REWARDS_STATS_STORE.save(
        deps.storage,
        state.rewards_epoch_index,
        &RewardDistributionConfig {
            cumulative_value: Uint256::zero(),
            cumulative_value_mul_bin_id: Uint256::zero(),
            rewards_distribution_algorithm: msg.rewards_distribution_algorithm,
        },
    )?;
    EPHEMERAL_STORAGE.save(
        deps.storage,
        &EphemeralStruct {
            lb_token_code_hash: msg.lb_token_implementation.code_hash,
            staking_contract: msg.staking_contract_implementation,
            token_x_symbol,
            token_y_symbol,
            epoch_index: state.rewards_epoch_index,
            epoch_duration: msg.epoch_staking_duration,
            expiry_duration: msg.expiry_staking_duration,
            recover_funds_receiver: msg.recover_staking_funds_receiver,
            query_auth: msg.query_auth,
        },
    )?;

    response = response.add_messages(messages);

    response.data = Some(env.contract.address.as_bytes().into());

    Ok(response)
}

/////////////// EXECUTE ///////////////
#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response> {
    let contract_status = CONTRACT_STATUS.load(deps.storage)?;
    let config = STATE.load(deps.storage)?;

    match contract_status {
        ContractStatus::FreezeAll => match msg {
            ExecuteMsg::AddLiquidity { .. }
            | ExecuteMsg::SwapTokens { .. }
            | ExecuteMsg::RemoveLiquidity { .. }
            | ExecuteMsg::Receive(..) => {
                return Err(Error::TransactionBlock());
            }

            _ => {}
        },
        ContractStatus::LpWithdrawOnly => match msg {
            ExecuteMsg::AddLiquidity { .. } | ExecuteMsg::SwapTokens { .. } => {
                return Err(Error::TransactionBlock());
            }
            _ => {}
        },
        ContractStatus::Active => {}
    }

    match msg {
        ExecuteMsg::Receive(msg) => {
            let checked_addr = deps.api.addr_validate(&msg.from)?;
            receiver_callback(deps, env, info, checked_addr, msg.amount, msg.msg)
        }
        ExecuteMsg::SwapTokens {
            to,
            offer,
            expected_return: _,
            padding: _,
        } => {
            if !offer.token.is_native_token() {
                return Err(Error::UseReceiveInterface);
            }

            offer
                .token
                .assert_sent_native_token_balance(&info, offer.amount)?;

            let checked_to = if let Some(to) = to {
                deps.api.addr_validate(to.as_str())?
            } else {
                info.sender.clone()
            };

            let swap_for_y: bool = offer.token.unique_key() == config.token_x.unique_key();

            try_swap(deps, env, info, swap_for_y, checked_to, offer.amount)
        }
        ExecuteMsg::AddLiquidity {
            liquidity_parameters,
        } => try_add_liquidity(deps, env, info, liquidity_parameters),
        ExecuteMsg::RemoveLiquidity {
            remove_liquidity_params,
        } => try_remove_liquidity(deps, env, info, remove_liquidity_params),
        ExecuteMsg::CollectProtocolFees {} => try_collect_protocol_fees(deps, env, info),
        ExecuteMsg::SetStaticFeeParameters {
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
        } => try_set_static_fee_parameters(
            deps,
            env,
            info,
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
        ),
        ExecuteMsg::ForceDecay {} => try_force_decay(deps, env, info),
        ExecuteMsg::CalculateRewardsDistribution {} => {
            try_calculate_rewards_distribution(deps, env, info)
        }
        ExecuteMsg::ResetRewardsConfig {
            distribution,
            base_rewards_bins,
        } => try_reset_rewards_config(deps, env, info, distribution, base_rewards_bins),

        ExecuteMsg::SetContractStatus { contract_status } => {
            let state = STATE.load(deps.storage)?;
            validate_admin(
                &deps.querier,
                AdminPermissions::ShadeSwapAdmin,
                &info.sender,
                &state.admin_auth,
            )?;

            CONTRACT_STATUS.save(deps.storage, &contract_status)?;
            Ok(Response::default().add_attribute("new_status", contract_status.to_string()))
        }
    }
}

pub fn receiver_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> Result<Response> {
    let msg = msg.ok_or(Error::ReceiverMsgEmpty)?;

    let config = STATE.load(deps.storage)?;

    let response;
    match from_binary(&msg)? {
        InvokeMsg::SwapTokens {
            to,
            expected_return: _,
            padding: _,
        } => {
            // this check needs to be here instead of in execute() because it is impossible to (cleanly) distinguish between swaps and lp withdraws until this point
            // if contract_status is FreezeAll, this fn will never be called, so only need to check LpWithdrawOnly here
            let contract_status = CONTRACT_STATUS.load(deps.storage)?;
            if contract_status == ContractStatus::LpWithdrawOnly {
                return Err(Error::TransactionBlock());
            }

            //validate recipient address
            let checked_to = if let Some(to) = to {
                deps.api.addr_validate(to.as_str())?
            } else {
                from
            };

            if info.sender != config.token_x.unique_key()
                && info.sender != config.token_y.unique_key()
            {
                return Err(Error::NoMatchingTokenInPair);
            }

            let swap_for_y: bool = info.sender == config.token_x.unique_key();

            response = try_swap(deps, env, info, swap_for_y, checked_to, amount)?;
        }
    };
    Ok(response)
}

/////////////// QUERY ///////////////

#[shd_entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::GetPairInfo {} => query_pair_info(deps),
        QueryMsg::GetFactory {} => query_factory(deps),
        QueryMsg::GetTokenX {} => query_token_x(deps),
        QueryMsg::GetTokenY {} => query_token_y(deps),
        QueryMsg::GetBinStep {} => query_bin_step(deps),
        QueryMsg::GetReserves {} => query_reserves(deps),
        QueryMsg::GetActiveId {} => query_active_id(deps),
        QueryMsg::GetBinReserves { id } => query_bin_reserves(deps, id),
        QueryMsg::GetBinsReserves { ids } => query_bins_reserves(deps, ids),
        QueryMsg::GetAllBinsReserves {
            id,
            page,
            page_size,
        } => query_all_bins_reserves(deps, env, page, page_size, id),
        QueryMsg::GetUpdatedBinAtHeight { height } => query_updated_bins_at_height(deps, height),
        QueryMsg::GetUpdatedBinAtMultipleHeights { heights } => {
            query_updated_bins_at_multiple_heights(deps, heights)
        }
        QueryMsg::GetUpdatedBinAfterHeight {
            height,
            page,
            page_size,
        } => query_updated_bins_after_height(deps, env, height, page, page_size),

        QueryMsg::GetBinUpdatingHeights { page, page_size } => {
            query_bins_updating_heights(deps, page, page_size)
        }

        QueryMsg::GetNextNonEmptyBin { swap_for_y, id } => {
            query_next_non_empty_bin(deps, swap_for_y, id)
        }
        QueryMsg::GetProtocolFees {} => query_protocol_fees(deps),
        QueryMsg::GetStaticFeeParameters {} => query_static_fee_params(deps),
        QueryMsg::GetVariableFeeParameters {} => query_variable_fee_params(deps),
        // TODO: do this for all the other query types
        QueryMsg::GetOracleParameters {} => Ok(to_binary(&query_oracle_params(deps)?)?),
        QueryMsg::GetOracleSampleAt { oracle_id } => query_oracle_sample(deps, env, oracle_id),
        QueryMsg::GetOracleSamplesAt { oracle_ids } => query_oracle_samples(deps, env, oracle_ids),
        QueryMsg::GetOracleSamplesAfter {
            oracle_id,
            page_size,
        } => query_oracle_samples_after(deps, env, oracle_id, page_size),
        QueryMsg::GetPriceFromId { id } => query_price_from_id(deps, id),
        QueryMsg::GetIdFromPrice { price } => query_id_from_price(deps, price),
        QueryMsg::GetSwapIn {
            amount_out,
            swap_for_y,
        } => query_swap_in(deps, env, amount_out.u128(), swap_for_y),
        QueryMsg::GetSwapOut {
            amount_in,
            swap_for_y,
        } => query_swap_out(deps, env, amount_in.u128(), swap_for_y),
        QueryMsg::TotalSupply { id } => query_total_supply(deps, id),
        QueryMsg::GetLbToken {} => query_lb_token(deps),
        QueryMsg::GetStakingContract {} => query_staking(deps),
        QueryMsg::GetTokens {} => query_tokens(deps),
        QueryMsg::SwapSimulation { offer, exclude_fee } => {
            query_swap_simulation(deps, env, offer, exclude_fee)
        }
        QueryMsg::GetRewardsDistribution { epoch_id } => query_rewards_distribution(deps, epoch_id),
    }
}

#[shd_entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match (msg.id, msg.result) {
        (INSTANTIATE_LP_TOKEN_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                let contract_address_string = &String::from_utf8(x.to_vec())?;
                let trimmed_str = contract_address_string.trim_matches('\"');
                let contract_address = deps.api.addr_validate(trimmed_str)?;

                // // not the best name but it matches the pair key idea
                let emp_storage = EPHEMERAL_STORAGE.load(deps.storage)?;
                let mut state = STATE.load(deps.storage)?;

                state.lb_token = ContractInfo {
                    address: contract_address,
                    code_hash: emp_storage.lb_token_code_hash,
                };

                STATE.save(deps.storage, &state)?;

                let mut response = Response::new();
                response.data = Some(env.contract.address.to_string().as_bytes().into());

                let instantiate_token_msg = lb_staking::InstantiateMsg {
                    amm_pair: env.contract.address.to_string(),
                    lb_token: state.lb_token.to_owned().into(),
                    admin_auth: state.admin_auth.into(),
                    query_auth: emp_storage.query_auth.into(),
                    epoch_index: emp_storage.epoch_index,
                    epoch_duration: emp_storage.epoch_duration,
                    expiry_duration: emp_storage.expiry_duration,
                    recover_funds_receiver: emp_storage.recover_funds_receiver,
                };

                response = response.add_submessage(SubMsg::reply_on_success(
                    CosmosMsg::Wasm(WasmMsg::Instantiate {
                        code_id: emp_storage.staking_contract.id,
                        code_hash: emp_storage.staking_contract.code_hash.clone(),
                        msg: to_binary(&instantiate_token_msg)?,
                        label: format!(
                            "{}-{}-Staking-Contract-{}-{}",
                            emp_storage.token_x_symbol,
                            emp_storage.token_y_symbol,
                            state.bin_step,
                            env.block.height
                        ),

                        funds: vec![],
                        admin: None,
                    }),
                    INSTANTIATE_STAKING_CONTRACT_REPLY_ID,
                ));

                Ok(response)
            }
            None => Err(StdError::generic_err("Unknown reply id")),
        },
        (INSTANTIATE_STAKING_CONTRACT_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                let contract_address_string = &String::from_utf8(x.to_vec())?;
                let trimmed_str = contract_address_string.trim_matches('\"');
                let contract_address = deps.api.addr_validate(trimmed_str)?;
                // not the best name but it matches the pair key idea
                let emp_storage = EPHEMERAL_STORAGE.load(deps.storage)?;
                let mut state = STATE.load(deps.storage)?;

                state.lb_staking = ContractInfo {
                    address: contract_address,
                    code_hash: emp_storage.staking_contract.code_hash,
                };

                STATE.save(deps.storage, &state)?;

                let mut response = Response::new();
                response.data = Some(env.contract.address.to_string().as_bytes().into());
                Ok(response)
            }
            None => Err(StdError::generic_err("Unknown reply id")),
        },
        _ => Err(StdError::generic_err("Unknown reply id")),
    }
}
