use crate::helper::*;
use crate::{prelude::*, state::*};
use ethnum::U256;
use lb_libraries::{
    approx_div,
    bin_helper::BinHelper,
    constants::{BASIS_POINT_MAX, MAX_FEE, PRECISION, SCALE_OFFSET},
    lb_token::state_structs::{TokenAmount, TokenIdBalance},
    math::{
        liquidity_configurations::LiquidityConfigurations,
        packed_u128_math::PackedUint128Math,
        tree_math::TreeUint24,
        u24::U24,
        u256x256_math::U256x256Math,
        uint256_to_u256::{ConvertU256, ConvertUint256},
    },
    oracle_helper::Oracle,
    pair_parameter_helper::PairParameters,
    price_helper::PriceHelper,
    types::Bytes32,
};
use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{
        to_binary, Addr, Attribute, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError,
        StdResult, Storage, Timestamp, Uint128, Uint256,
    },
    contract_interfaces::{
        liquidity_book::{lb_pair::*, lb_staking, lb_token},
        swap::router::ExecuteMsgResponse,
    },
    swap::core::TokenType,
};

#[derive(Clone, Debug)]
pub struct MintArrays {
    pub ids: Vec<U256>,
    pub amounts: Vec<Bytes32>,
    pub liquidity_minted: Vec<U256>,
}

/// Swap tokens iterating over the bins until the entire amount is swapped.
///
/// Token X will be swapped for token Y if `swap_for_y` is true, and token Y for token X if `swap_for_y` is false.
///
/// This function will not transfer the tokens from the caller, it is expected that the tokens have already been
/// transferred to this contract through another contract, most likely the router.
/// That is why this function shouldn't be called directly, but only through one of the swap functions of a router
/// that will also perform safety checks, such as minimum amounts and slippage.
///
/// The variable fee is updated throughout the swap, it increases with the number of bins crossed.
/// The oracle is updated at the end of the swap.
///
/// # Arguments
///
/// * `swap_for_y` - Whether you're swapping token X for token Y (true) or token Y for token X (false)
/// * `to` - The address to send the tokens to
///
/// # Returns
///
/// * `amounts_out` - The encoded amounts of token X and token Y sent to `to`
pub fn try_swap(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    swap_for_y: bool,
    to: Addr,
    amounts_received: Uint128, //Will get this parameter from router contract
) -> Result<Response> {
    let state = STATE.load(deps.storage)?;
    let tree = BIN_TREE.load(deps.storage)?;
    let token_x = &state.token_x;
    let token_y = &state.token_y;
    let reserves = state.reserves;
    let mut protocol_fees = state.protocol_fees;

    let mut ids = Vec::new();

    // Logging the swap activity
    let mut total_fees: [u8; 32] = [0; 32];
    let mut lp_fees: [u8; 32] = [0; 32];
    let mut shade_dao_fees: [u8; 32] = [0; 32];

    let mut amounts_out: [u8; 32] = [0; 32];
    let mut amounts_left: [u8; 32] = if swap_for_y {
        BinHelper::received_x(amounts_received)
    } else {
        BinHelper::received_y(amounts_received)
    };
    if amounts_left == [0; 32] {
        return Err(Error::InsufficientAmountIn);
    };

    let mut volume_tracker = amounts_left;
    let mut reserves = reserves.add(amounts_left);
    let mut params = state.pair_parameters;
    let mut reward_dis_config =
        REWARDS_STATS_STORE.load(deps.storage, state.rewards_epoch_index)?;
    let bin_step = state.bin_step;
    let mut active_id = params.get_active_id();

    // updating the volatility
    params.update_references(&env.block.time)?;

    if reward_dis_config.rewards_distribution_algorithm
        == RewardsDistributionAlgorithm::TimeBasedRewards
    {
        // updating for the data required for rewards distribution
        update_reward_dis_config(
            env.block.time.seconds(),
            &state,
            &mut reward_dis_config,
            active_id,
        )?
    }

    // Allowing only a limited max number of bins crossed per swap
    for _ in 0..state.max_bins_per_swap {
        let bin_reserves = BIN_MAP
            .load(deps.storage, active_id)
            .map_err(|_| Error::ZeroBinReserve { active_id })?;

        if !BinHelper::is_empty(bin_reserves, !swap_for_y) {
            let price = PriceHelper::get_price_from_id(active_id, bin_step)?;
            params.update_volatility_accumulator(active_id)?;
            let (mut amounts_in_with_fees, amounts_out_of_bin, fees) = BinHelper::get_amounts(
                bin_reserves,
                params,
                bin_step,
                swap_for_y,
                amounts_left,
                price,
            )?;

            if U256::from_le_bytes(amounts_in_with_fees) > U256::ZERO {
                if reward_dis_config.rewards_distribution_algorithm
                    == RewardsDistributionAlgorithm::VolumeBasedRewards
                {
                    // Logging fee for volume-based rewards
                    update_fee_map_tree(
                        deps.storage,
                        active_id,
                        fees,
                        swap_for_y,
                        price,
                        &state,
                        &mut reward_dis_config,
                    )?;
                }

                amounts_left = amounts_left.sub(amounts_in_with_fees);
                amounts_out = amounts_out.add(amounts_out_of_bin);

                let p_fees =
                    fees.scalar_mul_div_basis_point_round_down(params.get_protocol_share().into())?;
                total_fees = total_fees.add(fees);
                lp_fees = lp_fees.add(fees.sub(p_fees));
                shade_dao_fees = shade_dao_fees.add(p_fees);

                if U256::from_le_bytes(p_fees) > U256::ZERO {
                    protocol_fees = protocol_fees.add(p_fees);
                    amounts_in_with_fees = amounts_in_with_fees.sub(p_fees);
                }

                BIN_MAP.save(
                    deps.storage,
                    active_id,
                    &bin_reserves
                        .add(amounts_in_with_fees) // actually amount in wihtout fees
                        .sub(amounts_out_of_bin),
                )?;
                ids.push(active_id);
            }
        }

        if amounts_left == [0; 32] {
            break;
        } else {
            let next_id = _get_next_non_empty_bin(&tree, swap_for_y, active_id);
            if next_id == 0 || next_id == (U24::MAX) {
                return Err(Error::OutOfLiquidity);
            }
            active_id = next_id;
        }
    }

    REWARDS_STATS_STORE.save(deps.storage, state.rewards_epoch_index, &reward_dis_config)?;

    if amounts_out == [0; 32] {
        return Err(Error::InsufficientAmountOut);
    }

    reserves = reserves.sub(amounts_out);
    volume_tracker = volume_tracker.add(amounts_out);

    //updating the oracle for volume and fee analysis
    updating_oracles_for_vol_analysis(
        deps.storage,
        &env,
        &mut params,
        active_id,
        volume_tracker,
        total_fees,
    )?;

    STATE.update(deps.storage, |mut state| {
        state.last_swap_timestamp = env.block.time;
        state.protocol_fees = protocol_fees;
        params
            .set_active_id(active_id)
            .map_err(|err| StdError::generic_err(err.to_string()))?;
        state.pair_parameters = params;
        state.reserves = reserves;
        Ok::<State, StdError>(state)
    })?;

    let mut messages: Vec<CosmosMsg> = Vec::new();
    // Determine the output amount and the corresponding transfer message based on swap_for_y
    let amount_out = if swap_for_y {
        amounts_out.decode_y()
    } else {
        amounts_out.decode_x()
    };
    let msg = if swap_for_y {
        // BinHelper::transfer_y(amounts_out, token_y.clone(), to)
        todo!()
    } else {
        // BinHelper::transfer_x(amounts_out, token_x.clone(), to)
        todo!()
    };
    // Add the message to messages if it exists
    if let Some(message) = msg {
        messages.push(message);
    }

    // logging the bins changed
    update_bin_reserves(deps.storage, &env, ids)?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![
            Attribute::new("amount_in", amounts_received),
            Attribute::new("amount_out", amount_out.to_string()),
            Attribute::new("lp_fee_amount", lp_fees.decode_alt(swap_for_y).to_string()),
            Attribute::new(
                "total_fee_amount",
                total_fees.decode_alt(swap_for_y).to_string(),
            ),
            Attribute::new(
                "shade_dao_fee_amount",
                shade_dao_fees.decode_alt(swap_for_y).to_string(),
            ),
            Attribute::new("token_in_key", token_x.unique_key()),
            Attribute::new("token_out_key", token_y.unique_key()),
        ])
        .set_data(to_binary(&ExecuteMsgResponse::SwapResult {
            amount_in: amounts_received,
            amount_out: Uint128::from(amount_out),
        })?))
}

fn update_reward_dis_config(
    time_now: u64,
    state: &State,
    reward_dis_config: &mut RewardDistributionConfig,
    active_id: u32,
) -> Result<()> {
    let time_difference = Uint256::from(time_now - state.last_swap_timestamp.seconds());

    reward_dis_config.cumulative_value += time_difference;
    reward_dis_config.cumulative_value_mul_bin_id += time_difference * (Uint256::from(active_id));
    Ok(())
}

fn update_fee_map_tree(
    storage: &mut dyn Storage,
    active_id: u32,
    fees: Bytes32,
    swap_for_y: bool,
    price: U256,
    state: &State,
    reward_stats: &mut RewardDistributionConfig,
) -> Result<()> {
    let feeu128: u128 = fees.decode_alt(swap_for_y);
    let swap_value_uint256 = match swap_for_y {
        true => U256x256Math::mul_shift_round_up(U256::from(feeu128), price, SCALE_OFFSET)?
            .u256_to_uint256(),
        false => Uint256::from(feeu128),
    };

    reward_stats.cumulative_value += swap_value_uint256;
    FEE_MAP_TREE.update(
        storage,
        state.rewards_epoch_index,
        |fee_tree| -> Result<_> {
            Ok(match fee_tree {
                Some(mut t) => {
                    t.add(active_id);
                    t
                }
                None => panic!("Fee tree not initialized"), // TODO: remove this panic and include a custom error
            })
        },
    )?;

    FEE_MAP.update(storage, active_id, |cumm_fee| -> Result<_> {
        let updated_cumm_fee = match cumm_fee {
            Some(f) => f + swap_value_uint256,
            None => swap_value_uint256,
        };
        Ok(updated_cumm_fee)
    })?;

    Ok(())
}

fn updating_oracles_for_vol_analysis(
    storage: &mut dyn Storage,
    env: &Env,
    params: &mut PairParameters,
    active_id: u32,
    vol: Bytes32,
    fees: Bytes32,
) -> Result<()> {
    //updating oracles
    let oracle_id = params.get_oracle_id();
    let mut oracle = ORACLE.load(storage, oracle_id)?;

    let updated_sample;
    (*params, updated_sample) = oracle.update(
        &env.block.time,
        *params,
        active_id,
        Some(vol),
        Some(fees),
        DEFAULT_ORACLE_LENGTH,
    )?;

    if let Some(n_s) = updated_sample {
        ORACLE.save(storage, params.get_oracle_id(), &Oracle(n_s))?;
    }
    Ok(())
}

fn update_bin_reserves(storage: &mut dyn Storage, env: &Env, ids: Vec<u32>) -> Result<()> {
    //updating oracles
    BIN_RESERVES_UPDATED.update(storage, env.block.height, |x| -> StdResult<Vec<u32>> {
        if let Some(mut y) = x {
            y.extend(ids);
            Ok(y)
        } else {
            Ok(ids)
        }
    })?;
    BIN_RESERVES_UPDATED_LOG.push(storage, &env.block.height)?;
    Ok(())
}

pub fn try_add_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    liquidity_parameters: LiquidityParameters,
) -> Result<Response> {
    // Add liquidity while performing safety checks
    // transfering funds and checking one's already send
    // Main function -> add_liquidity_internal
    // Preparing txn output

    // 1- Add liquidity while performing safety checks
    // 1.1- Proceed only if deadline has not exceeded
    if env.block.time.seconds() > liquidity_parameters.deadline {
        return Err(Error::DeadlineExceeded {
            deadline: liquidity_parameters.deadline,
            current_timestamp: env.block.time.seconds(),
        });
    }
    let config = STATE.load(deps.storage)?;
    let response = Response::new();
    // 1.2- Checking token order
    if liquidity_parameters.token_x != config.token_x
        || liquidity_parameters.token_y != config.token_y
        || liquidity_parameters.bin_step != config.bin_step
    {
        return Err(Error::WrongPair);
    }

    // response = response.add_messages(transfer_messages);

    //3- Main function -> add_liquidity_internal
    let response =
        add_liquidity_internal(deps, env, info, &config, &liquidity_parameters, response)?;

    Ok(response)
}

fn add_liquidity_internal(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &State,
    liquidity_parameters: &LiquidityParameters,
    response: Response,
) -> Result<Response> {
    match_lengths(liquidity_parameters)?;
    check_ids_bounds(liquidity_parameters)?;

    let state = STATE.load(deps.storage)?;

    let mut liquidity_configs = vec![
        LiquidityConfigurations {
            distribution_x: 0,
            distribution_y: 0,
            id: 0
        };
        liquidity_parameters.delta_ids.len()
    ];
    let mut deposit_ids = Vec::with_capacity(liquidity_parameters.delta_ids.len());

    let active_id = state.pair_parameters.get_active_id();
    check_active_id_slippage(liquidity_parameters, active_id)?;

    let mut distribution_sum_x = 0u64;
    let mut distribution_sum_y = 0u64;
    let precison: u64 = PRECISION as u64;

    for i in 0..liquidity_configs.len() {
        let id = calculate_id(liquidity_parameters, active_id, i)?;
        deposit_ids.push(id);

        distribution_sum_x += liquidity_parameters.distribution_x[i];
        distribution_sum_y += liquidity_parameters.distribution_y[i];

        if liquidity_parameters.distribution_x[i] > precison
            || liquidity_parameters.distribution_y[i] > precison
            || distribution_sum_x > precison
            || distribution_sum_y > precison
        {
            return Err(Error::DistrubtionError);
        }

        liquidity_configs[i] = LiquidityConfigurations {
            distribution_x: liquidity_parameters.distribution_x[i],
            distribution_y: liquidity_parameters.distribution_y[i],
            id,
        };
    }

    let (amounts_deposited, amounts_left, _liquidity_minted, response) = mint(
        &mut deps,
        &env,
        info.clone(),
        config,
        info.sender.clone(),
        liquidity_configs,
        info.sender,
        liquidity_parameters.amount_x,
        liquidity_parameters.amount_y,
        response,
    )?;

    //4- Preparing txn output logs
    let amount_x_added = Uint128::from(amounts_deposited.decode_x());
    let amount_y_added = Uint128::from(amounts_deposited.decode_y());
    let amount_x_min = liquidity_parameters.amount_x_min;
    let amount_y_min = liquidity_parameters.amount_y_min;

    if amount_x_added < amount_x_min || amount_y_added < amount_y_min {
        return Err(Error::AmountSlippageCaught {
            amount_x_min,
            amount_x: amount_x_added,
            amount_y_min,
            amount_y: amount_y_added,
        });
    }
    let _amount_x_left = Uint128::from(amounts_left.decode_x());
    let _amount_y_left = Uint128::from(amounts_left.decode_y());

    // let liq_minted: Vec<Uint256> = liquidity_minted
    //     .iter()
    //     .map(|&liq| liq.u256_to_uint256())
    //     .collect();

    // let _deposit_ids_string = serialize_or_err(&deposit_ids)?;
    BIN_RESERVES_UPDATED.update(deps.storage, env.block.height, |x| -> StdResult<Vec<u32>> {
        if let Some(mut y) = x {
            y.extend(deposit_ids);
            Ok(y)
        } else {
            Ok(deposit_ids)
        }
    })?;
    BIN_RESERVES_UPDATED_LOG.push(deps.storage, &env.block.height)?;

    // let _liquidity_minted_string = serialize_or_err(&liq_minted)?;

    // response = response
    //     .add_attribute("amount_x_added", amount_x_added)
    //     .add_attribute("amount_y_added", amount_y_added)
    //     .add_attribute("amount_x_left", amount_x_left)
    //     .add_attribute("amount_y_left", amount_y_left)
    //     .add_attribute("liquidity_minted", liquidity_minted_string)
    //     .add_attribute("deposit_ids", deposit_ids_string);

    Ok(response)
}

/// Mint liquidity tokens by depositing tokens into the pool.
///
/// It will mint Liquidity Book (LB) tokens for each bin where the user adds liquidity.
/// This function will not transfer the tokens from the caller, it is expected that the tokens have already been
/// transferred to this contract through another contract, most likely the router.
/// That is why this function shouldn't be called directly, but through one of the add liquidity functions of a
/// router that will also perform safety checks.
///
/// Any excess amount of token will be sent to the `refund_to` address.
///
/// # Arguments
///
/// * `to` - The address that will receive the LB tokens
/// * `liquidity_configs` - The encoded liquidity configurations, each one containing the id of the bin and the
/// percentage of token X and token Y to add to the bin.
/// * `refund_to` - The address that will receive the excess amount of tokens
///
/// # Returns
///
/// * `amounts_received` - The amounts of token X and token Y received by the pool
/// * `amounts_left` - The amounts of token X and token Y that were not added to the pool and were sent to to
/// * `liquidity_minted` - The amounts of LB tokens minted for each bin
#[allow(clippy::too_many_arguments)]
fn mint(
    mut deps: &mut DepsMut,
    env: &Env,
    info: MessageInfo,
    config: &State,
    to: Addr,
    liquidity_configs: Vec<LiquidityConfigurations>,
    _refund_to: Addr,
    amount_received_x: Uint128,
    amount_received_y: Uint128,
    mut response: Response,
) -> Result<(Bytes32, Bytes32, Vec<U256>, Response)> {
    let state = STATE.load(deps.storage)?;

    let _token_x = state.token_x;
    let _token_y = state.token_y;

    if liquidity_configs.is_empty() {
        return Err(Error::EmptyMarketConfigs);
    }

    let mut mint_arrays = MintArrays {
        ids: (vec![U256::MIN; liquidity_configs.len()]),
        amounts: (vec![[0u8; 32]; liquidity_configs.len()]),
        liquidity_minted: (vec![U256::MIN; liquidity_configs.len()]),
    };

    let amounts_received = BinHelper::received(amount_received_x, amount_received_y);
    let mut messages: Vec<CosmosMsg> = Vec::new();

    let amounts_left = mint_bins(
        &mut deps,
        &env.block.time,
        state.bin_step,
        state.pair_parameters,
        liquidity_configs,
        amounts_received,
        to,
        &mut mint_arrays,
        &mut messages,
    )?;

    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.reserves = state.reserves.add(amounts_received.sub(amounts_left)); //Total liquidity of pool
        Ok(state)
    })?;

    let (amount_left_x, amount_left_y) = amounts_left.decode();

    let mut transfer_messages = Vec::new();
    // 2- tokens checking and transfer
    for (token, amount) in [
        (
            config.token_x.clone(),
            amount_received_x - Uint128::from(amount_left_x),
        ),
        (
            config.token_y.clone(),
            amount_received_y - Uint128::from(amount_left_y),
        ),
    ]
    .iter()
    {
        match token {
            TokenType::CustomToken {
                contract_addr: _,
                token_code_hash: _,
            } => {
                let msg =
                    token.transfer_from(*amount, info.sender.clone(), env.contract.address.clone());

                if let Some(m) = msg {
                    transfer_messages.push(m);
                }
            }
            TokenType::NativeToken { .. } => {
                token.assert_sent_native_token_balance(&info, *amount)?;
            }
        }
    }

    response = response
        .add_messages(messages)
        .add_messages(transfer_messages);

    Ok((
        amounts_received,
        amounts_left,
        mint_arrays.liquidity_minted,
        response,
    ))
}

/// Helper function to mint liquidity in each bin in the liquidity configurations.
///
/// # Arguments
///
/// * `liquidity_configs` - The liquidity configurations.
/// * `amounts_received` - The amounts received.
/// * `to` - The address to mint the liquidity to.
/// * `arrays` - The arrays to store the results.
///
/// # Returns
///
/// * `amounts_left` - The amounts left.
fn mint_bins(
    deps: &mut DepsMut,
    time: &Timestamp,
    bin_step: u16,
    pair_parameters: PairParameters,
    liquidity_configs: Vec<LiquidityConfigurations>,
    amounts_received: Bytes32,
    to: Addr,
    mint_arrays: &mut MintArrays,
    messages: &mut Vec<CosmosMsg>,
) -> Result<Bytes32> {
    let config = STATE.load(deps.storage)?;
    let active_id = pair_parameters.get_active_id();

    let mut amounts_left = amounts_received;

    //Minting tokens

    let mut mint_tokens: Vec<TokenAmount> = Vec::new();

    for (index, liq_conf) in liquidity_configs.iter().enumerate() {
        let (max_amounts_in_to_bin, id) = liq_conf.get_amounts_and_id(amounts_received)?;

        let (shares, amounts_in, amounts_in_to_bin) = update_bin(
            deps,
            time,
            bin_step,
            active_id,
            id,
            max_amounts_in_to_bin,
            pair_parameters,
        )?;

        amounts_left = amounts_left.sub(amounts_in);

        mint_arrays.ids[index] = id.into();
        mint_arrays.amounts[index] = amounts_in_to_bin;
        mint_arrays.liquidity_minted[index] = shares;

        let amount = shares.u256_to_uint256();

        //Minting tokens
        mint_tokens.push(TokenAmount {
            token_id: id.to_string(),
            balances: vec![TokenIdBalance {
                address: to.clone(),
                amount,
            }],
        });
    }
    let msg = lb_token::ExecuteMsg::MintTokens {
        mint_tokens,
        memo: None,
        padding: None,
    }
    .to_cosmos_msg(
        config.lb_token.code_hash.clone(),
        config.lb_token.address.to_string(),
        None,
    )?;

    messages.push(msg);
    Ok(amounts_left)
}

/// Helper function to update a bin during minting.
///
/// # Arguments
///
/// * `bin_step` - The bin step of the pair
/// * `active_id` - The id of the active bin
/// * `id` - The id of the bin
/// * `max_amounts_in_to_bin` - The maximum amounts in to the bin
/// * `parameters` - The parameters of the pair
///
/// # Returns
///
/// * `shares` - The amount of shares minted
/// * `amounts_in` - The amounts in
/// * `amounts_in_to_bin` - The amounts in to the bin
fn update_bin(
    deps: &mut DepsMut,
    time: &Timestamp,
    bin_step: u16,
    active_id: u32,
    id: u32,
    amounts_in: Bytes32,
    mut parameters: PairParameters,
) -> Result<(U256, Bytes32, Bytes32)> {
    let bin_reserves = BIN_MAP.load(deps.storage, id).unwrap_or([0u8; 32]);
    let config = STATE.load(deps.storage)?;
    let price = PriceHelper::get_price_from_id(id, bin_step)?;
    let total_supply = _query_total_supply(
        deps.as_ref(),
        id,
        config.lb_token.code_hash,
        config.lb_token.address,
    )?;
    let (shares, amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
        bin_reserves,
        amounts_in,
        price,
        total_supply,
    )?;

    let amounts_in_to_bin = amounts_in;

    if id == active_id {
        parameters.update_volatility_parameters(id, time)?;

        // Helps calculate fee if there's an implict swap.
        let fees = BinHelper::get_composition_fees(
            bin_reserves,
            parameters,
            bin_step,
            amounts_in,
            total_supply,
            shares,
        )?;

        if fees != [0u8; 32] {
            let user_liquidity = BinHelper::get_liquidity(amounts_in.sub(fees), price)?;
            let bin_liquidity = BinHelper::get_liquidity(bin_reserves, price)?;

            let _shares =
                U256x256Math::mul_div_round_down(user_liquidity, total_supply, bin_liquidity)?;
            let protocol_c_fees =
                fees.scalar_mul_div_basis_point_round_down(parameters.get_protocol_share().into())?;

            if protocol_c_fees != [0u8; 32] {
                let _amounts_in_to_bin = amounts_in_to_bin.sub(protocol_c_fees);
                STATE.update(deps.storage, |mut state| -> StdResult<_> {
                    state.protocol_fees = state.protocol_fees.add(protocol_c_fees);
                    Ok(state)
                })?;
            }

            let oracle_id = parameters.get_oracle_id();

            let mut oracle = ORACLE.load(deps.storage, oracle_id)?;
            let new_sample;
            (parameters, new_sample) =
                oracle.update(time, parameters, id, None, None, DEFAULT_ORACLE_LENGTH)?;
            if let Some(n_s) = new_sample {
                ORACLE.save(deps.storage, parameters.get_oracle_id(), &Oracle(n_s))?;
            }

            STATE.update(deps.storage, |mut state| -> StdResult<_> {
                state.pair_parameters = parameters;
                Ok(state)
            })?;
        }
    } else {
        BinHelper::verify_amounts(amounts_in, active_id, id)?;
    }

    if shares == 0 || amounts_in_to_bin == [0u8; 32] {
        return Err(Error::ZeroAmount { id });
    }

    if total_supply == 0 {
        BIN_TREE.update(deps.storage, |mut tree| -> StdResult<_> {
            tree.add(id);
            Ok(tree)
        })?;
    }

    BIN_MAP.save(deps.storage, id, &bin_reserves.add(amounts_in_to_bin))?;

    Ok((shares, amounts_in, amounts_in_to_bin))
}

pub fn try_remove_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    remove_liquidity_params: RemoveLiquidity,
) -> Result<Response> {
    let config = STATE.load(deps.storage)?;

    let is_wrong_order = config.token_x != remove_liquidity_params.token_x;

    let (amount_x_min, amount_y_min) = if is_wrong_order {
        if remove_liquidity_params.token_x != config.token_y
            || remove_liquidity_params.token_y != config.token_x
            || remove_liquidity_params.bin_step != config.bin_step
        {
            return Err(Error::WrongPair);
        }
        (
            remove_liquidity_params.amount_y_min,
            remove_liquidity_params.amount_x_min,
        )
    } else {
        if remove_liquidity_params.token_x != config.token_x
            || remove_liquidity_params.token_y != config.token_y
            || remove_liquidity_params.bin_step != config.bin_step
        {
            return Err(Error::WrongPair);
        }
        (
            remove_liquidity_params.amount_x_min,
            remove_liquidity_params.amount_y_min,
        )
    };

    let (_amount_x, _amount_y, response) = remove_liquidity(
        deps,
        env,
        info.clone(),
        info.sender,
        amount_x_min,
        amount_y_min,
        remove_liquidity_params.ids,
        remove_liquidity_params.amounts,
    )?;

    Ok(response)
}

fn remove_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _to: Addr,
    amount_x_min: Uint128,
    amount_y_min: Uint128,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
) -> Result<(Uint128, Uint128, Response)> {
    let (amounts_burned, response) = burn(deps, env, info, ids, amounts)?;
    let mut amount_x: Uint128 = Uint128::zero();
    let mut amount_y: Uint128 = Uint128::zero();
    for amount_burned in amounts_burned {
        amount_x += Uint128::from(amount_burned.decode_x());
        amount_y += Uint128::from(amount_burned.decode_y());
    }

    if amount_x < amount_x_min || amount_y < amount_y_min {
        return Err(Error::AmountSlippageCaught {
            amount_x_min,
            amount_x,
            amount_y_min,
            amount_y,
        });
    }

    Ok((amount_x, amount_y, response))
}

/// Burn Liquidity Book (LB) tokens and withdraw tokens from the pool.
///
/// This function will burn the tokens directly from the caller.
///
/// # Arguments
///
/// * `from` - The address that will burn the LB tokens
/// * `to` - The address that will receive the tokens
/// * `ids` - The ids of the bins from which to withdraw
/// * `amounts_to_burn` - The amounts of LB tokens to burn for each bin
///
/// # Returns
///
/// * `amounts` - The amounts of token X and token Y received by the user
fn burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ids: Vec<u32>,
    amounts_to_burn: Vec<Uint256>,
) -> Result<(Vec<[u8; 32]>, Response)> {
    let mut config = STATE.load(deps.storage)?;

    let token_x = config.token_x;
    let token_y = config.token_y;

    if ids.is_empty() || ids.len() != amounts_to_burn.len() {
        return Err(Error::InvalidInput);
    }

    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut burn_tokens: Vec<TokenAmount> = Vec::new();

    let mut amounts = vec![[0u8; 32]; ids.len()];
    let mut amounts_out = [0u8; 32];

    for i in 0..ids.len() {
        let id = ids[i];
        let amount_to_burn = amounts_to_burn[i];

        if amount_to_burn.is_zero() {
            return Err(Error::ZeroShares { id });
        }

        let bin_reserves = BIN_MAP
            .load(deps.storage, id)
            .map_err(|_| Error::ZeroBinReserve {
                active_id: i as u32,
            })?;
        let total_supply = _query_total_supply(
            deps.as_ref(),
            id,
            config.lb_token.code_hash.clone(),
            config.lb_token.address.clone(),
        )?;

        burn_tokens.push(TokenAmount {
            token_id: id.to_string(),
            balances: vec![TokenIdBalance {
                address: info.sender.clone(),
                amount: amount_to_burn,
            }],
        });

        let amount_to_burn_u256 = amount_to_burn.uint256_to_u256();

        let amounts_out_from_bin_vals =
            BinHelper::get_amount_out_of_bin(bin_reserves, amount_to_burn_u256, total_supply)?;
        let amounts_out_from_bin: Bytes32 =
            Bytes32::encode(amounts_out_from_bin_vals.0, amounts_out_from_bin_vals.1);

        if amounts_out_from_bin.iter().all(|&x| x == 0) {
            return Err(Error::ZeroAmountsOut {
                id,
                amount_to_burn: amount_to_burn_u256.u256_to_uint256(),
                total_supply: total_supply.u256_to_uint256(),
            });
        }

        let bin_reserves = bin_reserves.sub(amounts_out_from_bin);

        if total_supply == amount_to_burn_u256 {
            BIN_MAP.remove(deps.storage, id);
            BIN_TREE.update(deps.storage, |mut tree| -> StdResult<_> {
                tree.remove(id);
                Ok(tree)
            })?;
        } else {
            BIN_MAP.save(deps.storage, id, &bin_reserves)?;
        }

        amounts[i] = amounts_out_from_bin;
        amounts_out = amounts_out.add(amounts_out_from_bin);
    }

    let msg = lb_token::ExecuteMsg::BurnTokens {
        burn_tokens,
        memo: None,
        padding: None,
    }
    .to_cosmos_msg(
        config.lb_token.code_hash,
        config.lb_token.address.to_string(),
        None,
    )?;

    messages.push(msg);

    config.reserves = config.reserves.sub(amounts_out);

    let raw_msgs: Option<Vec<CosmosMsg>> = todo!();
    // let raw_msgs = BinHelper::transfer(amounts_out, token_x, token_y, info.sender);

    STATE.update(deps.storage, |mut state| -> StdResult<State> {
        state.reserves = state.reserves.sub(amounts_out);
        Ok(state)
    })?;

    BIN_RESERVES_UPDATED.update(deps.storage, env.block.height, |x| -> StdResult<Vec<u32>> {
        if let Some(mut y) = x {
            y.extend(ids);
            Ok(y)
        } else {
            Ok(ids)
        }
    })?;
    BIN_RESERVES_UPDATED_LOG.push(deps.storage, &env.block.height)?;

    if let Some(msgs) = raw_msgs {
        messages.extend(msgs)
    }

    Ok((amounts, Response::default().add_messages(messages)))
}

// Administrative functions

/// Collect the protocol fees from the pool.
pub fn try_collect_protocol_fees(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response> {
    let state = STATE.load(deps.storage)?;
    // only_protocol_fee_recipient(&info.sender, &state.factory.address)?;

    let token_x = state.token_x;
    let token_y = state.token_y;

    let mut messages: Vec<CosmosMsg> = Vec::new();

    let protocol_fees = state.protocol_fees;

    let (x, y) = protocol_fees.decode();
    let ones = Bytes32::encode(if x > 0 { 1 } else { 0 }, if y > 0 { 1 } else { 0 });

    //The purpose of subtracting ones from the protocolFees is to leave a small amount (1 unit of each token) in the protocol fees.
    //This is done to avoid completely draining the fees and possibly causing any issues with calculations that depend on non-zero values
    let collected_protocol_fees = protocol_fees.sub(ones);

    if U256::from_le_bytes(collected_protocol_fees) != U256::ZERO {
        // This is setting the protocol fees to the smallest possible values
        STATE.update(deps.storage, |mut state| -> StdResult<State> {
            state.protocol_fees = ones;
            state.reserves = state.reserves.sub(collected_protocol_fees);
            Ok(state)
        })?;

        if collected_protocol_fees.iter().any(|&x| x != 0) {
            // if let Some(msgs) = BinHelper::transfer(
            //     collected_protocol_fees,
            //     token_x.clone(),
            //     token_y.clone(),
            //     state.protocol_fees_recipient,
            // ) {
            //     messages.extend(msgs);
            // };
        }

        Ok(Response::default()
            .add_attribute(
                format!("Collected Protocol Fees for token {}", token_x.unique_key()),
                collected_protocol_fees.decode_x().to_string(),
            )
            .add_attribute(
                format!("Collected Protocol Fees for token {}", token_y.unique_key()),
                collected_protocol_fees.decode_y().to_string(),
            )
            .add_attribute("Action performed by", info.sender.to_string())
            .add_messages(messages))
    } else {
        Err(Error::NotEnoughFunds)
    }
}

/// Sets the static fee parameters of the pool.
///
/// Can only be called by the factory.
///
/// # Arguments
///
/// * `base_factor` - The base factor of the static fee
/// * `filter_period` - The filter period of the static fee
/// * `decay_period` - The decay period of the static fee
/// * `reduction_factor` - The reduction factor of the static fee
/// * `variable_fee_control` - The variable fee control of the static fee
/// * `protocol_share` - The protocol share of the static fee
/// * `max_volatility_accumulator` - The max volatility accumulator of the static fee
pub fn try_set_static_fee_parameters(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    base_factor: u16,
    filter_period: u16,
    decay_period: u16,
    reduction_factor: u16,
    variable_fee_control: u32,
    protocol_share: u16,
    max_volatility_accumulator: u32,
) -> Result<Response> {
    let state = STATE.load(deps.storage)?;
    only_factory(&info.sender, &state.factory.address)?;

    let mut params = state.pair_parameters;

    params.set_static_fee_parameters(
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    )?;

    let total_fee = params.get_base_fee(state.bin_step) + params.get_variable_fee(state.bin_step);
    if total_fee > MAX_FEE {
        return Err(Error::MaxTotalFeeExceeded {});
    }

    STATE.update(deps.storage, |mut state| -> StdResult<State> {
        state.pair_parameters = params;
        Ok(state)
    })?;

    Ok(Response::default().add_attribute("status", "ok"))
}

/// Forces the decay of the volatility reference variables.
///
/// Can only be called by the factory.
pub fn try_force_decay(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response> {
    let state = STATE.load(deps.storage)?;
    only_factory(&info.sender, &state.factory.address)?;

    let mut params = state.pair_parameters;
    params.update_id_reference();
    params.update_volatility_reference()?;

    STATE.update(deps.storage, |mut state| -> StdResult<State> {
        state.pair_parameters = params;
        Ok(state)
    })?;

    Ok(Response::default())
}

pub fn try_calculate_rewards_distribution(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response> {
    let mut state = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::LiquidityBookAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;

    // save the results in temporary storage

    let reward_stats = REWARDS_STATS_STORE.load(deps.storage, state.rewards_epoch_index)?;
    let distribution = if !reward_stats.cumulative_value.is_zero() {
        match reward_stats.rewards_distribution_algorithm {
            RewardsDistributionAlgorithm::TimeBasedRewards => {
                calculate_time_based_rewards_distribution(&env, &state, &reward_stats)?
            }
            RewardsDistributionAlgorithm::VolumeBasedRewards => {
                calculate_volume_based_rewards_distribution(deps.as_ref(), &state, &reward_stats)?
            }
        }
    } else {
        let rewards_bins = match state.base_rewards_bins {
            Some(r_b) => r_b,
            None => DEFAULT_REWARDS_BINS,
        };
        calculate_default_distribution(rewards_bins, state.pair_parameters.get_active_id())?
    };

    REWARDS_DISTRIBUTION.save(deps.storage, state.rewards_epoch_index, &distribution)?;

    //distribution algorithm
    let res = lb_staking::ExecuteMsg::EndEpoch {
        rewards_distribution: distribution,
        epoch_index: state.rewards_epoch_index,
    }
    .to_cosmos_msg(
        state.lb_staking.code_hash.to_owned(),
        state.lb_staking.address.to_string(),
        None,
    )?;

    state.rewards_epoch_index += 1;
    let toggle = state.toggle_distributions_algorithm;
    state.last_swap_timestamp = env.block.time;
    state.toggle_distributions_algorithm = false;
    STATE.save(deps.storage, &state)?;

    let mut distribution_algorithm = &reward_stats.rewards_distribution_algorithm;
    if toggle {
        distribution_algorithm = match reward_stats.rewards_distribution_algorithm {
            RewardsDistributionAlgorithm::TimeBasedRewards => {
                &RewardsDistributionAlgorithm::VolumeBasedRewards
            }
            RewardsDistributionAlgorithm::VolumeBasedRewards => {
                &RewardsDistributionAlgorithm::TimeBasedRewards
            }
        };
    }

    REWARDS_STATS_STORE.save(
        deps.storage,
        state.rewards_epoch_index,
        &RewardDistributionConfig {
            cumulative_value: Uint256::zero(),
            cumulative_value_mul_bin_id: Uint256::zero(),
            rewards_distribution_algorithm: distribution_algorithm.clone(),
        },
    )?;

    if distribution_algorithm == &RewardsDistributionAlgorithm::VolumeBasedRewards {
        let tree: TreeUint24 = TreeUint24::new();
        FEE_MAP_TREE.save(deps.storage, state.rewards_epoch_index, &tree)?;
    }

    Ok(Response::default().add_message(res))
}

fn calculate_default_distribution(rewards_bins: u32, avg_bin: u32) -> Result<RewardsDistribution> {
    let half_total = rewards_bins / 2;
    let min_bin = avg_bin.saturating_sub(half_total) + 1;
    let max_bin = avg_bin.saturating_add(half_total);

    let difference = max_bin - min_bin + 1;

    let ids: Vec<u32> = (min_bin..=max_bin).collect();
    let weightages = vec![BASIS_POINT_MAX as u16 / difference as u16; difference as usize];

    Ok(RewardsDistribution {
        ids,
        weightages,
        denominator: BASIS_POINT_MAX as u16,
    })
}

fn calculate_time_based_rewards_distribution(
    env: &Env,
    state: &State,
    reward_stats: &RewardDistributionConfig,
) -> Result<RewardsDistribution> {
    let mut cumm_value_mul_bin = reward_stats.cumulative_value_mul_bin_id;
    let mut cumm_value = reward_stats.cumulative_value;

    let active_id = state.pair_parameters.get_active_id();

    let time_difference =
        Uint256::from(env.block.time.seconds() - state.last_swap_timestamp.seconds());

    cumm_value += time_difference;
    cumm_value_mul_bin += time_difference * (Uint256::from(active_id));

    let avg_bin = approx_div(cumm_value_mul_bin, cumm_value)
        .uint256_to_u256()
        .as_u32();

    let rewards_bins = match state.base_rewards_bins {
        Some(r_b) => r_b,
        None => DEFAULT_REWARDS_BINS,
    };

    calculate_default_distribution(rewards_bins, avg_bin)
}

fn calculate_volume_based_rewards_distribution(
    deps: Deps,
    state: &State,
    reward_stats: &RewardDistributionConfig,
) -> Result<RewardsDistribution> {
    let cum_fee = reward_stats.cumulative_value;
    let mut ids: Vec<u32> = Vec::new();
    let mut weightages: Vec<u16> = Vec::new();

    let fee_tree: TreeUint24 = FEE_MAP_TREE.load(deps.storage, state.rewards_epoch_index)?;
    let mut id: u32 = 0;
    let basis_point_max: Uint256 = Uint256::from(BASIS_POINT_MAX);
    let mut total_weight = 0;

    for _ in 0..U24::MAX {
        id = fee_tree.find_first_left(id);
        if id == U24::MAX || id == 0 {
            break;
        }

        let fee: Uint256 = FEE_MAP.load(deps.storage, id)?;
        ids.push(id);
        let weightage: u16 = fee
            .multiply_ratio(basis_point_max, cum_fee)
            .uint256_to_u256()
            .as_u16();
        weightages.push(weightage);
        total_weight += weightage;
    }

    let reminder = BASIS_POINT_MAX as u16 - total_weight;

    if reminder > 0 {
        let len = weightages.len() - 1;
        weightages[len] += reminder;
    }

    let distribution = RewardsDistribution {
        ids,
        weightages,
        denominator: BASIS_POINT_MAX as u16,
    };

    Ok(distribution)
}

//Can only change the distribution algorithm at the start of next epoch
//Eventhough the distribution was changes mid epoch the effects of change will occur after the epoch.
pub fn try_reset_rewards_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    rewards_distribution_algorithm: Option<RewardsDistributionAlgorithm>,
    base_reward_bins: Option<u32>,
) -> Result<Response> {
    let mut state = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::LiquidityBookAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    let reward_stats = REWARDS_STATS_STORE.load(deps.storage, state.rewards_epoch_index)?;

    //Eventhough the distribution was changes mid epoch the effects of change will occur after the epoch.
    match rewards_distribution_algorithm {
        Some(distribution) => {
            if reward_stats.rewards_distribution_algorithm != distribution {
                state.toggle_distributions_algorithm = true;
            }
        }
        None => {}
    };

    match base_reward_bins {
        Some(b_r_b) => {
            if b_r_b > U24::MAX {
                return Err(Error::U24Overflow);
            }
            state.base_rewards_bins = Some(b_r_b)
        }
        None => {}
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}
