#![allow(unused)] // For beginning only.

use crate::{prelude::*, state::*};
use core::panic;
use cosmwasm_std::{
    from_binary, to_binary, Addr, BankMsg, Binary, Coin, ContractInfo, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, Response, StdError, StdResult, SubMsgResult, Timestamp, Uint128, Uint256,
    WasmMsg,
};
use ethnum::U256;
use serde::Serialize;
use shade_protocol::{
    c_std::{shd_entry_point, Reply, SubMsg},
    contract_interfaces::liquidity_book::{
        lb_pair::*, lb_token, lb_token::InstantiateMsg as LBTokenInstantiateMsg,
    },
    lb_libraries::{
        bin_helper::{self, BinHelper},
        constants::{self, MAX_FEE, SCALE_OFFSET},
        fee_helper::{self, FeeHelper},
        lb_token::state_structs::{LbPair, TokenAmount, TokenIdBalance},
        math::{
            encoded_sample::EncodedSample,
            liquidity_configurations::LiquidityConfigurations,
            packed_u128_math::{Decode, Encode, PackedMath},
            sample_math::OracleSample,
            tree_math::TreeUint24,
            u24::U24,
            u256x256_math::U256x256Math,
            uint256_to_u256::{self, ConvertU256, ConvertUint256},
        },
        oracle_helper::{Oracle, OracleError, MAX_SAMPLE_LIFETIME},
        pair_parameter_helper::PairParameters,
        price_helper::PriceHelper,
        tokens, types,
        viewing_keys::{register_receive, set_viewing_key_msg, ViewingKey},
    },
    snip20,
    utils::pad_handle_result,
    BLOCK_SIZE,
};
use std::{collections::HashMap, ops::Sub};
use tokens::TokenType;
use types::{Bytes32, LBPairInformation, MintArrays};

pub const INSTANTIATE_LP_TOKEN_REPLY_ID: u64 = 1u64;
pub const MINT_REPLY_ID: u64 = 1u64;

/////////////// INSTANTIATE ///////////////

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response> {
    //Initializing the Token Contract

    // TODO: Only the factory should be allowed to instantiate this contract
    // I think you can restrict that on code upload
    // Proposed solution -> Haseeb, literally hardcore the factory_address
    // let factory_address = Addr::unchecked("factory_contract_address");
    // And factory is only used at the time of instantiation.

    // if info.sender != factory_address {
    //     return Err(Error::OnlyFactory);
    // }

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

    let instantiate_token_msg = LBTokenInstantiateMsg {
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
            decimals: 18,
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
                "{}-{}-Pair-Token-{}",
                token_x_symbol, token_y_symbol, msg.bin_step
            ),
            funds: vec![],
        }),
        INSTANTIATE_LP_TOKEN_REPLY_ID,
    ));

    //Initializing PairParameters
    let pair_parameters = PairParameters(EncodedSample([0u8; 32]));
    let pair_parameters = pair_parameters
        .set_static_fee_parameters(
            msg.pair_parameters.base_factor,
            msg.pair_parameters.filter_period,
            msg.pair_parameters.decay_period,
            msg.pair_parameters.reduction_factor,
            msg.pair_parameters.variable_fee_control,
            msg.pair_parameters.protocol_share,
            msg.pair_parameters.max_volatility_accumulator,
        )
        .unwrap();
    let pair_parameters = pair_parameters
        .set_active_id(msg.active_id)?
        .update_id_reference();

    //RegisterReceiving Token
    let mut messages = vec![];
    let viewing_key = ViewingKey::from(msg.viewing_key.as_str());
    for token in [&msg.token_x, &msg.token_y] {
        if let TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } = token
        {
            register_pair_token(&env, &mut messages, &token, &viewing_key);
        }
    }

    let state = State {
        creator: info.sender.clone(),
        factory: msg.factory,
        token_x: msg.token_x,
        token_y: msg.token_y,
        bin_step: msg.bin_step,
        pair_parameters,
        reserves: [0u8; 32],
        protocol_fees: [0u8; 32],
        lb_token: ContractInfo {
            address: Addr::unchecked("".to_string()),
            code_hash: "".to_string(),
        }, // intentionally keeping this empty will be filled in reply
        viewing_key,
        protocol_fees_recipient: msg.protocol_fee_recipient,
    };

    // deps.api
    //     .debug(format!("Contract was initialized by {}", info.sender).as_str());

    let tree = TreeUint24::new();
    let oracle = Oracle {
        samples: HashMap::<u16, OracleSample>::new(),
    };

    CONFIG.save(deps.storage, &state)?;
    ORACLE.save(deps.storage, &oracle)?;

    BIN_TREE.save(deps.storage, &tree)?;

    ephemeral_storage_w(deps.storage).save(&NextTokenKey {
        code_hash: msg.lb_token_implementation.code_hash,
    })?;

    response = response.add_messages(messages);

    response.data = Some(env.contract.address.as_bytes().into());

    Ok(response)
}

/////////////// EXECUTE ///////////////
//TODO: add contract status
#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => {
            let checked_addr = deps.api.addr_validate(&msg.from)?;
            receiver_callback(deps, env, info, checked_addr, msg.amount, msg.msg)
        }
        ExecuteMsg::SwapTokens {
            to,
            offer,
            expected_return,
            padding,
        } => {
            let config = CONFIG.load(deps.storage)?;
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

            let swap_for_y: bool;
            if info.sender == config.token_x.unique_key() {
                swap_for_y = true;
            } else {
                swap_for_y = false;
            }

            try_swap(deps, env, info, swap_for_y, checked_to, offer.amount)
        }
        //TODO: Flash loan
        ExecuteMsg::FlashLoan {} => todo!(),
        ExecuteMsg::AddLiquidity {
            liquidity_parameters,
        } => try_add_liquidity(deps, env, info, liquidity_parameters),
        ExecuteMsg::RemoveLiquidity {
            remove_liquidity_params,
        } => try_remove_liquidity(deps, env, info, remove_liquidity_params),
        ExecuteMsg::CollectProtocolFees {} => try_collect_protocol_fees(deps, env, info),
        ExecuteMsg::IncreaseOracleLength { new_length } => {
            try_increase_oracle_length(deps, env, info, new_length)
        }
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
    }
}

pub fn register_pair_token(
    env: &Env,
    messages: &mut Vec<CosmosMsg>,
    token: &TokenType,
    viewing_key: &ViewingKey,
) -> StdResult<()> {
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = token
    {
        messages.push(set_viewing_key_msg(
            viewing_key.0.clone(),
            None,
            &ContractInfo {
                address: contract_addr.clone(),
                code_hash: token_code_hash.to_string(),
            },
        )?);
        messages.push(register_receive(
            env.contract.code_hash.clone(),
            None,
            &ContractInfo {
                address: contract_addr.clone(),
                code_hash: token_code_hash.to_string(),
            },
        )?);
    }

    Ok(())
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
fn try_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    swap_for_y: bool,
    to: Addr,
    amounts_received: Uint128, //Will get this parameter from router contract
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    let tree = BIN_TREE.load(deps.storage)?;
    let token_x = state.token_x;
    let token_y = state.token_y;

    let reserves = state.reserves;
    let mut protocol_fees = state.protocol_fees;

    let mut amounts_out = [0u8; 32];
    let mut amounts_left = if swap_for_y {
        BinHelper::received_x(amounts_received)
    } else {
        BinHelper::received_y(amounts_received)
    };
    if amounts_left == [0u8; 32] {
        return Err(Error::InsufficientAmountIn);
    };

    let mut reserves = reserves.add(amounts_left);

    let mut params = state.pair_parameters;
    let bin_step = state.bin_step;

    let mut active_id = params.get_active_id();

    params = params.update_references(&env.block.time)?;

    loop {
        let bin_reserves = BIN_MAP
            .load(deps.storage, active_id)
            .map_err(|_| Error::ZeroBinReserve { active_id })?;

        if !BinHelper::is_empty(bin_reserves, !swap_for_y) {
            params = params.update_volatility_accumulator(active_id)?;

            let (mut amounts_in_with_fees, amounts_out_of_bin, total_fees) =
                BinHelper::get_amounts(
                    bin_reserves,
                    params,
                    bin_step,
                    swap_for_y,
                    active_id,
                    amounts_left,
                )?;

            if U256::from_le_bytes(amounts_in_with_fees) > U256::ZERO {
                amounts_left = amounts_left.sub(amounts_in_with_fees);
                amounts_out = amounts_out.add(amounts_out_of_bin);

                let p_fees = total_fees
                    .scalar_mul_div_basis_point_round_down(params.get_protocol_share().into())?;

                if U256::from_le_bytes(p_fees) > U256::ZERO {
                    protocol_fees = protocol_fees.add(p_fees);
                    amounts_in_with_fees = amounts_in_with_fees.sub(p_fees);
                }

                BIN_MAP.save(
                    deps.storage,
                    active_id,
                    &bin_reserves
                        .add(amounts_in_with_fees)
                        .sub(amounts_out_of_bin),
                )?;
            }
        }

        if amounts_left == [0u8; 32] {
            break;
        } else {
            let next_id = _get_next_non_empty_bin(&tree, swap_for_y, active_id);

            if next_id == 0 || next_id == (U24::MAX) {
                return Err(Error::OutOfLiquidity);
            }
            active_id = next_id;
        }
    }

    if amounts_out == [0u8; 32] {
        return Err(Error::InsufficientAmountOut);
    }

    reserves = reserves.sub(amounts_out);

    let mut oracle = ORACLE.load(deps.storage)?;
    params = oracle.update(&env.block.time, params, active_id)?;

    CONFIG.update(deps.storage, |mut state| -> Result<_> {
        state.protocol_fees = protocol_fees;
        state.pair_parameters = PairParameters::set_active_id(params, active_id)?;
        state.reserves = reserves;
        Ok(state)
    })?;

    let mut messages: Vec<CosmosMsg> = Vec::new();
    if swap_for_y {
        let msg = BinHelper::transfer_y(amounts_out, token_y, to);

        if let Some(message) = msg {
            messages.push(message);
        }
    } else {
        let msg = BinHelper::transfer_x(amounts_out, token_x, to);

        if let Some(message) = msg {
            messages.push(message);
        }
    }

    Ok(Response::default().add_messages(messages))
}

/// Flash loan tokens from the pool to a receiver contract and execute a callback function.
///
/// The receiver contract is expected to return the tokens plus a fee to this contract.
/// The fee is calculated as a percentage of the amount borrowed, and is the same for both tokens.
///
/// # Arguments
///
/// * `receiver` - The contract that will receive the tokens and execute the callback function
/// * `amounts` - The encoded amounts of token X and token Y to flash loan
/// * `data` - Any data that will be passed to the callback function
///
/// # Requirements
///
/// * `receiver` must implement the ILBFlashLoanCallback interface
fn try_flash_loan(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receiver: ContractInfo,
    amounts: Bytes32,
    data: Binary,
) -> Result<Response> {
    todo!()
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
    let config = CONFIG.load(deps.storage)?;
    let mut response = Response::new();
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

pub fn add_liquidity_internal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &State,
    liquidity_parameters: &LiquidityParameters,
    mut response: Response,
) -> Result<Response> {
    match_lengths(&liquidity_parameters)?;
    check_ids_bounds(&liquidity_parameters)?;

    let state = CONFIG.load(deps.storage)?;

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
    check_active_id_slippage(&liquidity_parameters, active_id)?;

    for i in 0..liquidity_configs.len() {
        let id = calculate_id(&liquidity_parameters, active_id, i)?;
        deposit_ids.push(id);
        liquidity_configs[i] = LiquidityConfigurations {
            distribution_x: liquidity_parameters.distribution_x[i],
            distribution_y: liquidity_parameters.distribution_y[i],
            id,
        };
    }

    let (amounts_deposited, amounts_left, liquidity_minted, mut response) = mint(
        deps,
        env,
        info.clone(),
        &config,
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
    let amount_x_left = Uint128::from(amounts_left.decode_x());
    let amount_y_left = Uint128::from(amounts_left.decode_y());

    let liq_minted: Vec<Uint256> = liquidity_minted
        .iter()
        .map(|&liq| liq.u256_to_uint256())
        .collect();

    let deposit_ids_string = serialize_or_err(&deposit_ids)?;
    let liquidity_minted_string = serialize_or_err(&liq_minted)?;

    // response = response
    //     .add_attribute("amount_x_added", amount_x_added)
    //     .add_attribute("amount_y_added", amount_y_added)
    //     .add_attribute("amount_x_left", amount_x_left)
    //     .add_attribute("amount_y_left", amount_y_left)
    //     .add_attribute("liquidity_minted", liquidity_minted_string)
    //     .add_attribute("deposit_ids", deposit_ids_string);

    Ok((response))
}

fn match_lengths(liquidity_parameters: &LiquidityParameters) -> Result<()> {
    if liquidity_parameters.delta_ids.len() != liquidity_parameters.distribution_x.len()
        || liquidity_parameters.delta_ids.len() != liquidity_parameters.distribution_y.len()
    {
        return Err(Error::LengthsMismatch);
    }
    Ok(())
}

fn check_ids_bounds(liquidity_parameters: &LiquidityParameters) -> Result<()> {
    if liquidity_parameters.active_id_desired > U24::MAX
        || liquidity_parameters.id_slippage > U24::MAX
    {
        return Err(Error::IdDesiredOverflows {
            id_desired: liquidity_parameters.active_id_desired,
            id_slippage: liquidity_parameters.id_slippage,
        });
    }
    Ok(())
}

fn check_active_id_slippage(
    liquidity_parameters: &LiquidityParameters,
    active_id: u32,
) -> Result<()> {
    if liquidity_parameters.active_id_desired + liquidity_parameters.id_slippage < active_id
        || active_id + liquidity_parameters.id_slippage < liquidity_parameters.active_id_desired
    {
        return Err(Error::IdSlippageCaught {
            active_id_desired: liquidity_parameters.active_id_desired,
            id_slippage: liquidity_parameters.id_slippage,
            active_id,
        });
    }
    Ok(())
}

//function won't distinguish between overflow and underflow errors; it'll throw the same DeltaIdOverflows
fn calculate_id(
    liquidity_parameters: &LiquidityParameters,
    active_id: u32,
    i: usize,
) -> Result<u32> {
    // let id: u32;

    let id: i64 = active_id as i64 + liquidity_parameters.delta_ids[i];

    if (id < 0 || id as u32 > U24::MAX) {
        return Err(Error::DeltaIdOverflows {
            delta_id: liquidity_parameters.delta_ids[i],
        });
    }

    Ok(id as u32)
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
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &State,
    to: Addr,
    liquidity_configs: Vec<LiquidityConfigurations>,
    refund_to: Addr,
    amount_received_x: Uint128,
    amount_received_y: Uint128,
    mut response: Response,
) -> Result<(Bytes32, Bytes32, Vec<U256>, Response)> {
    let state = CONFIG.load(deps.storage)?;

    let token_x = state.token_x;
    let token_y = state.token_y;

    // TODO: add a check that the "to" address is not zero or this contract's address
    // equivalent to notAddressZeroOrThis(to)

    if liquidity_configs.is_empty() {
        return Err(Error::EmptyMarketConfigs);
    }

    let mut mint_arrays = MintArrays {
        ids: (vec![U256::MIN; liquidity_configs.len()]),
        amounts: (vec![[0u8; 32]; liquidity_configs.len()]),
        liquidity_minted: (vec![U256::MIN; liquidity_configs.len()]),
    };

    //TODO: make things more efficient maybe avoid this encoding
    let amounts_received = BinHelper::received(amount_received_x, amount_received_y);
    let mut messages: Vec<CosmosMsg> = Vec::new();

    let (amounts_left) = _mint_bins(
        &mut deps,
        &env.block.time,
        state.bin_step,
        state.pair_parameters,
        liquidity_configs,
        amounts_received,
        to.clone(),
        &mut mint_arrays,
        &mut messages,
    )?;

    CONFIG.update(deps.storage, |mut state| -> StdResult<_> {
        state.reserves = state.reserves.add(amounts_received.sub(amounts_left)); //Total liquidity of pool
        Ok(state)
    })?;
    // if amounts_left.iter().any(|&x| x != 0) {
    //     if let Some(msgs) = BinHelper::transfer(amounts_left, token_x, token_y, refund_to) {
    //         messages.extend(msgs);
    //     };
    // }

    let (amount_left_x, amount_left_y) = amounts_left.decode();

    let mut transfer_messages = Vec::new();
    // 2- tokens checking and transfer
    for (token, amount) in [
        (
            config.token_x.clone(),
            amount_received_x.sub(Uint128::from(amount_left_x)),
        ),
        (
            config.token_y.clone(),
            amount_received_y.sub(Uint128::from(amount_left_y)),
        ),
    ]
    .iter()
    {
        match token {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
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
fn _mint_bins(
    deps: &mut DepsMut,
    time: &Timestamp,
    bin_step: u16,
    pair_parameters: PairParameters,
    liquidity_configs: Vec<LiquidityConfigurations>,
    amounts_received: Bytes32,
    to: Addr,
    mint_arrays: &mut MintArrays,
    messages: &mut Vec<CosmosMsg>,
) -> Result<(Bytes32)> {
    let config = CONFIG.load(deps.storage)?;
    let active_id = pair_parameters.get_active_id();

    let mut amounts_left = amounts_received;

    //Minting tokens

    let mut mint_tokens: Vec<TokenAmount> = Vec::new();

    for (index, liq_conf) in liquidity_configs.iter().enumerate() {
        let (max_amounts_in_to_bin, id) = liq_conf.get_amounts_and_id(amounts_received)?;

        let (shares, amounts_in, amounts_in_to_bin) = _update_bin(
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
fn _update_bin(
    deps: &mut DepsMut,
    time: &Timestamp,
    bin_step: u16,
    active_id: u32,
    id: u32,
    max_amounts_in_to_bin: Bytes32,
    parameters: PairParameters,
) -> Result<(U256, Bytes32, Bytes32)> {
    let bin_reserves = BIN_MAP.load(deps.storage, id).unwrap_or([0u8; 32]);
    let config = CONFIG.load(deps.storage)?;
    let price = PriceHelper::get_price_from_id(id, bin_step)?;

    let total_supply = _query_total_supply(
        deps.as_ref(),
        id,
        config.lb_token.code_hash,
        config.lb_token.address,
    )?;

    let (shares, amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
        bin_reserves,
        max_amounts_in_to_bin,
        price,
        total_supply,
    )?;

    let amounts_in_to_bin = amounts_in;

    if id == active_id {
        let mut parameters = parameters.update_volatility_parameters(id, time)?;

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

            let shares =
                U256x256Math::mul_div_round_down(user_liquidity, total_supply, bin_liquidity)?;
            let protocol_c_fees =
                fees.scalar_mul_div_basis_point_round_down(parameters.get_protocol_share().into())?;

            if protocol_c_fees != [0u8; 32] {
                let amounts_in_to_bin = amounts_in_to_bin.sub(protocol_c_fees);
                CONFIG.update(deps.storage, |mut state| -> StdResult<_> {
                    state.protocol_fees = state.protocol_fees.add(protocol_c_fees);
                    Ok(state)
                })?;
            }

            let mut oracle = ORACLE.load(deps.storage)?;
            parameters = oracle.update(time, parameters, id)?;
            CONFIG.update(deps.storage, |mut state| -> StdResult<_> {
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

fn _query_total_supply(deps: Deps, id: u32, code_hash: String, address: Addr) -> Result<U256> {
    let msg = lb_token::QueryMsg::IdTotalBalance { id: id.to_string() };

    let res = deps.querier.query_wasm_smart::<lb_token::QueryAnswer>(
        code_hash,
        address.to_string(),
        &(&msg),
    )?;
    let mut total_supply_uint256 = Uint256::zero();
    match res {
        lb_token::QueryAnswer::IdTotalBalance { amount } => total_supply_uint256 = amount,
        _ => (),
    };

    Ok(total_supply_uint256.uint256_to_u256())
}

fn query_token_symbol(deps: Deps, code_hash: String, address: Addr) -> Result<String> {
    let msg = snip20::QueryMsg::TokenInfo {};

    let res = deps.querier.query_wasm_smart::<snip20::QueryAnswer>(
        code_hash,
        address.to_string(),
        &(&msg),
    )?;

    let symbol = match res {
        snip20::QueryAnswer::TokenInfo {
            name,
            symbol,
            decimals,
            total_supply,
        } => (symbol),
        _ => panic!("{}", format!("Token {} not valid", address)),
    };

    Ok(symbol)
}

pub fn try_remove_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    remove_liquidity_params: RemoveLiquidity,
) -> Result<Response> {
    let config = CONFIG.load(deps.storage)?;

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

    let (amount_x, amount_y, mut response) = remove_liquidity(
        deps,
        env.clone(),
        info.clone(),
        info.sender.clone(),
        amount_x_min,
        amount_y_min,
        remove_liquidity_params.ids,
        remove_liquidity_params.amounts,
    )?;

    Ok(response)
}

pub fn remove_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to: Addr,
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
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ids: Vec<u32>,
    amounts_to_burn: Vec<Uint256>,
) -> Result<(Vec<[u8; 32]>, Response)> {
    let mut config = CONFIG.load(deps.storage)?;

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
            Encode::encode(amounts_out_from_bin_vals.0, amounts_out_from_bin_vals.1);

        if amounts_out_from_bin.iter().all(|&x| x == 0) {
            return Err(Error::ZeroAmountsOut {
                id,
                // bin_reserves,
                amount_to_burn: amount_to_burn_u256,
                total_supply,
                // amounts_out_from_bin,
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

    let raw_msgs = BinHelper::transfer(amounts_out, token_x, token_y, info.sender.clone());

    CONFIG.update(deps.storage, |mut state| -> StdResult<State> {
        state.reserves = state.reserves.sub(amounts_out);
        Ok(state)
    })?;

    if let Some(msgs) = raw_msgs {
        messages.extend(msgs)
    }

    Ok((amounts, Response::default().add_messages(messages)))
}

/// Collect the protocol fees from the pool.
fn try_collect_protocol_fees(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
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
        CONFIG.update(deps.storage, |mut state| -> StdResult<State> {
            state.protocol_fees = ones;
            state.reserves = state.reserves.sub(collected_protocol_fees);
            Ok(state)
        })?;

        if collected_protocol_fees.iter().any(|&x| x != 0) {
            if let Some(msgs) = BinHelper::transfer(
                collected_protocol_fees,
                token_x.clone(),
                token_y.clone(),
                state.protocol_fees_recipient,
            ) {
                messages.extend(msgs);
            };
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

/// Increase the length of the oracle used by the pool.
///
/// # Arguments
///
/// * `new_length` - The new length of the oracle
fn try_increase_oracle_length(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_length: u16,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    let mut params = state.pair_parameters;

    let mut oracle_id = params.get_oracle_id();

    // activate the oracle if it is not active yet
    if oracle_id == 0 {
        oracle_id = 1;
        params = PairParameters::set_oracle_id(params, oracle_id);
    }

    ORACLE.update(deps.storage, |mut oracle| {
        oracle
            .increase_length(oracle_id, new_length)
            .map_err(|err| StdError::generic_err(err.to_string()))
    })?;

    Ok(Response::default().add_attribute("Oracle Length Increased to", new_length.to_string()))
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
fn try_set_static_fee_parameters(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    base_factor: u16,
    filter_period: u16,
    decay_period: u16,
    reduction_factor: u16,
    variable_fee_control: u32,
    protocol_share: u16,
    max_volatility_accumulator: u32,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    only_factory(&info.sender, &state.factory.address)?;

    let mut params = state.pair_parameters;

    params = PairParameters::set_static_fee_parameters(
        params,
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    )?;

    {
        let total_fee =
            params.get_base_fee(state.bin_step) + params.get_variable_fee(state.bin_step);
        if total_fee > MAX_FEE {
            return Err(Error::MaxTotalFeeExceeded {});
        }
    }

    CONFIG.update(deps.storage, |mut state| -> StdResult<State> {
        state.pair_parameters = params;
        Ok(state)
    })?;

    Ok(Response::default().add_attribute("status", "ok"))
}

/// Forces the decay of the volatility reference variables.
///
/// Can only be called by the factory.
fn try_force_decay(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    only_factory(&info.sender, &state.factory.address)?;

    let mut params = state.pair_parameters;
    params = PairParameters::update_id_reference(params);
    params = PairParameters::update_volatility_reference(params)?;

    CONFIG.update(deps.storage, |mut state| -> StdResult<State> {
        state.pair_parameters = params;
        Ok(state)
    })?;

    Ok(Response::default()
        .add_attribute_plaintext("Id_reference", params.get_id_reference().to_string())
        .add_attribute_plaintext(
            "Volatility_reference",
            params.get_volatility_reference().to_string(),
        ))
}

fn only_factory(sender: &Addr, factory: &Addr) -> Result<()> {
    if sender != factory {
        return Err(Error::OnlyFactory);
    }
    Ok(())
}

fn serialize_or_err<T: Serialize>(data: &T) -> Result<String> {
    serde_json_wasm::to_string(data).map_err(|_| Error::SerializationError)
}

fn receiver_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> Result<Response> {
    let msg = msg.ok_or_else(|| Error::ReceiverMsgEmpty)?;

    let config = CONFIG.load(deps.storage)?;

    let mut response = Response::new();
    match from_binary(&msg)? {
        InvokeMsg::SwapTokens {
            to,
            expected_return,
            padding: _,
        } => {
            // this check needs to be here instead of in execute() because it is impossible to (cleanly) distinguish between swaps and lp withdraws until this point
            // if contract_status is FreezeAll, this fn will never be called, so only need to check LpWithdrawOnly here
            //TODO: add status: check
            // if contract_status == ContractStatus::LpWithdrawOnly {
            //     return Err(StdError::generic_err(
            //         "Transaction is blocked by contract status",
            //     ));
            // }

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
            let swap_for_y: bool;
            if info.sender == config.token_x.unique_key() {
                swap_for_y = true;
            } else {
                swap_for_y = false;
            }

            response = try_swap(deps, env, info, swap_for_y, checked_to, amount)?;
        }
    };
    Ok(response)
}

/////////////// QUERY ///////////////

#[shd_entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::GetFactory {} => to_binary(&query_factory(deps)?).map_err(Error::CwErr),
        QueryMsg::GetTokenX {} => to_binary(&query_token_x(deps)?).map_err(Error::CwErr),
        QueryMsg::GetTokenY {} => to_binary(&query_token_y(deps)?).map_err(Error::CwErr),
        QueryMsg::GetBinStep {} => to_binary(&query_bin_step(deps)?).map_err(Error::CwErr),
        QueryMsg::GetReserves {} => to_binary(&query_reserves(deps)?).map_err(Error::CwErr),
        QueryMsg::GetActiveId {} => to_binary(&query_active_id(deps)?).map_err(Error::CwErr),
        QueryMsg::GetBin { id } => to_binary(&query_bin(deps, id)?).map_err(Error::CwErr),
        QueryMsg::GetNextNonEmptyBin { swap_for_y, id } => {
            to_binary(&query_next_non_empty_bin(deps, swap_for_y, id)?).map_err(Error::CwErr)
        }
        QueryMsg::GetProtocolFees {} => {
            to_binary(&query_protocol_fees(deps)?).map_err(Error::CwErr)
        }
        QueryMsg::GetStaticFeeParameters {} => {
            to_binary(&query_static_fee_params(deps)?).map_err(Error::CwErr)
        }
        QueryMsg::GetVariableFeeParameters {} => {
            to_binary(&query_variable_fee_params(deps)?).map_err(Error::CwErr)
        }
        QueryMsg::GetOracleParameters {} => {
            to_binary(&query_oracle_params(deps)?).map_err(Error::CwErr)
        }
        QueryMsg::GetOracleSampleAt { look_up_timestamp } => {
            to_binary(&query_oracle_sample_at(deps, env, look_up_timestamp)?).map_err(Error::CwErr)
        }
        QueryMsg::GetPriceFromId { id } => {
            to_binary(&query_price_from_id(deps, id)?).map_err(Error::CwErr)
        }
        QueryMsg::GetIdFromPrice { price } => {
            to_binary(&query_id_from_price(deps, price)?).map_err(Error::CwErr)
        }
        QueryMsg::GetSwapIn {
            amount_out,
            swap_for_y,
        } => to_binary(&query_swap_in(deps, env, amount_out.u128(), swap_for_y)?)
            .map_err(Error::CwErr),
        QueryMsg::GetSwapOut {
            amount_in,
            swap_for_y,
        } => to_binary(&query_swap_out(deps, env, amount_in.u128(), swap_for_y)?)
            .map_err(Error::CwErr),
        QueryMsg::TotalSupply { id } => {
            to_binary(&query_total_supply(deps, id)?).map_err(Error::CwErr)
        }
        QueryMsg::GetLbToken {} => to_binary(&query_lb_token(deps)?).map_err(Error::CwErr),
        QueryMsg::GetTokens {} => to_binary(&query_tokens(deps)?).map_err(Error::CwErr),
    }
}

/// Returns the Liquidity Book Factory.
///
/// # Returns
///
/// * `factory` - The Liquidity Book Factory
fn query_factory(deps: Deps) -> Result<FactoryResponse> {
    let state = CONFIG.load(deps.storage)?;
    let factory = state.factory.address;
    Ok(FactoryResponse { factory })
}

/// Returns the Liquidity Book Factory.
///
/// # Returns
///
/// * `factory` - The Liquidity Book Factory
fn query_lb_token(deps: Deps) -> Result<LbTokenResponse> {
    let state = CONFIG.load(deps.storage)?;
    let lb_token = state.lb_token;
    Ok(LbTokenResponse { lb_token })
}

/// Returns the token X and Y of the Liquidity Book Pair.
///
/// # Returns
///
/// * `token_x` - The address of the token X
fn query_tokens(deps: Deps) -> Result<TokensResponse> {
    let state = CONFIG.load(deps.storage)?;
    Ok(TokensResponse {
        token_x: state.token_x,
        token_y: state.token_y,
    })
}

/// Returns the token X of the Liquidity Book Pair.
///
/// # Returns
///
/// * `token_x` - The address of the token X
fn query_token_x(deps: Deps) -> Result<TokenXResponse> {
    let state = CONFIG.load(deps.storage)?;
    let token_x = state.token_x;
    Ok(TokenXResponse { token_x })
}

/// Returns the token Y of the Liquidity Book Pair.
///
/// # Returns
///
/// * `token_y` - The address of the token Y
fn query_token_y(deps: Deps) -> Result<TokenYResponse> {
    let state = CONFIG.load(deps.storage)?;
    let token_y = state.token_y;
    Ok(TokenYResponse { token_y })
}

/// Returns the bin_step of the Liquidity Book Pair.
///
/// The bin step is the increase in price between two consecutive bins, in basis points.
/// For example, a bin step of 1 means that the price of the next bin is 0.01% higher than the price of the previous bin.
///
/// # Returns
///
/// * `bin_step` - The bin step of the Liquidity Book Pair, in 10_000th
fn query_bin_step(deps: Deps) -> Result<BinStepResponse> {
    let state = CONFIG.load(deps.storage)?;
    let bin_step = state.bin_step;
    Ok(BinStepResponse { bin_step })
}

/// Returns the reserves of the Liquidity Book Pair.
///
/// This is the sum of the reserves of all bins, minus the protocol fees.
///
/// # Returns
///
/// * `reserve_x` - The reserve of token X
/// * `reserve_y` - The reserve of token Y
fn query_reserves(deps: Deps) -> Result<ReservesResponse> {
    let state = CONFIG.load(deps.storage)?;
    let (mut reserve_x, mut reserve_y) = state.reserves.decode();
    let (protocol_fee_x, protocol_fee_y) = state.protocol_fees.decode();

    reserve_x -= protocol_fee_x;
    reserve_y -= protocol_fee_y;

    Ok(ReservesResponse {
        reserve_x,
        reserve_y,
    })
}

/// Returns the active id of the Liquidity Book Pair.
///
/// The active id is the id of the bin that is currently being used for swaps.
/// The price of the active bin is the price of the Liquidity Book Pair and can be calculated as follows:
/// `price = (1 + binStep / 10_000) ^ (activeId - 2^23)`
///
/// # Returns
///
/// * `active_id` - The active id of the Liquidity Book Pair
fn query_active_id(deps: Deps) -> Result<ActiveIdResponse> {
    let state = CONFIG.load(deps.storage)?;
    let active_id = state.pair_parameters.get_active_id();

    Ok(ActiveIdResponse { active_id })
}

/// Returns the reserves of a bin.
///
/// # Arguments
///
/// * `id` - The id of the bin
///
/// # Returns
///
/// * `bin_reserve_x` - The reserve of token X in the bin
/// * `bin_reserve_y` - The reserve of token Y in the bin
fn query_bin(deps: Deps, id: u32) -> Result<BinResponse> {
    let bin: Bytes32 = BIN_MAP.load(deps.storage, id).unwrap_or([0u8; 32]);

    let (bin_reserve_x, bin_reserve_y) = bin.decode();
    Ok(BinResponse {
        bin_reserve_x,
        bin_reserve_y,
    })
}

/// Returns the next non-empty bin.
///
/// The next non-empty bin is the bin with a higher (if swap_for_y is true) or lower (if swap_for_y is false)
/// id that has a non-zero reserve of token X or Y.
///
/// # Arguments
///
/// * `swap_for_y` - Whether the swap is for token Y (true) or token X (false
/// * `id` - The id of the bin
///
/// # Returns
///
/// * `next_id` - The id of the next non-empty bin
fn query_next_non_empty_bin(
    deps: Deps,
    swap_for_y: bool,
    id: u32,
) -> Result<NextNonEmptyBinResponse> {
    let tree = BIN_TREE.load(deps.storage)?;
    let next_id = _get_next_non_empty_bin(&tree, swap_for_y, id);

    Ok(NextNonEmptyBinResponse { next_id })
}

/// Returns id of the next non-empty bin.
///
/// # Arguments
/// * `swap_for_y Whether the swap is for Y
/// * `id` - The id of the bin
fn _get_next_non_empty_bin(tree: &TreeUint24, swap_for_y: bool, id: u32) -> u32 {
    if swap_for_y {
        tree.find_first_right(id)
    } else {
        tree.find_first_left(id)
    }
}

/// Returns the protocol fees of the Liquidity Book Pair.
///
/// # Returns
///
/// * `protocol_fee_x` - The protocol fees of token X
/// * `protocol_fee_y` - The protocol fees of token Y
fn query_protocol_fees(deps: Deps) -> Result<ProtocolFeesResponse> {
    let state = CONFIG.load(deps.storage)?;
    let (protocol_fee_x, protocol_fee_y) = state.protocol_fees.decode();

    Ok(ProtocolFeesResponse {
        protocol_fee_x,
        protocol_fee_y,
    })
}

/// Returns the static fee parameters of the Liquidity Book Pair.
///
/// # Returns
///
/// * `base_factor` - The base factor for the static fee
/// * `filter_period` - The filter period for the static fee
/// * `decay_period` - The decay period for the static fee
/// * `reduction_factor` - The reduction factor for the static fee
/// * `variable_fee_control` - The variable fee control for the static fee
/// * `protocol_share` - The protocol share for the static fee
/// * `max_volatility_accumulator` - The maximum volatility accumulator for the static fee
fn query_static_fee_params(deps: Deps) -> Result<StaticFeeParametersResponse> {
    let state = CONFIG.load(deps.storage)?;
    let params = state.pair_parameters;

    let base_factor = params.get_base_factor();
    let filter_period = params.get_filter_period();
    let decay_period = params.get_decay_period();
    let reduction_factor = params.get_reduction_factor();
    let variable_fee_control = params.get_variable_fee_control();
    let protocol_share = params.get_protocol_share();
    let max_volatility_accumulator = params.get_max_volatility_accumulator();

    Ok(StaticFeeParametersResponse {
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    })
}

/// Returns the variable fee parameters of the Liquidity Book Pair.
///
/// # Returns
///
/// * `volatility_accumulator` - The volatility accumulator for the variable fee
/// * `volatility_reference` - The volatility reference for the variable fee
/// * `id_reference` - The id reference for the variable fee
/// * `time_of_last_update` - The time of last update for the variable fee
fn query_variable_fee_params(deps: Deps) -> Result<VariableFeeParametersResponse> {
    let state = CONFIG.load(deps.storage)?;
    let params = state.pair_parameters;

    let volatility_accumulator = params.get_volatility_accumulator();
    let volatility_reference = params.get_volatility_reference();
    let id_reference = params.get_id_reference();
    let time_of_last_update = params.get_time_of_last_update();

    Ok(VariableFeeParametersResponse {
        volatility_accumulator,
        volatility_reference,
        id_reference,
        time_of_last_update,
    })
}

/// Returns the oracle parameters of the Liquidity Book Pair.
///
/// # Returns
///
/// * `sample_lifetime` - The sample lifetime for the oracle
/// * `size` - The size of the oracle
/// * `active_size` - The active size of the oracle
/// * `last_updated` - The last updated timestamp of the oracle
/// * `first_timestamp` - The first timestamp of the oracle, i.e. the timestamp of the oldest sample
fn query_oracle_params(deps: Deps) -> Result<OracleParametersResponse> {
    let state = CONFIG.load(deps.storage)?;
    let oracle = ORACLE.load(deps.storage)?;
    let params = state.pair_parameters;

    let sample_lifetime = MAX_SAMPLE_LIFETIME;
    let oracle_id = params.get_oracle_id();

    if oracle_id > 0 {
        let (mut sample, mut active_size) = oracle.get_active_sample_and_size(oracle_id)?;
        let size = sample.get_oracle_length();
        let last_updated = sample.get_sample_last_update();

        if last_updated == 0 {
            active_size = 0;
        }

        if active_size > 0 {
            sample = oracle.get_sample(1 + (oracle_id % active_size))?;
        }
        let first_timestamp = sample.get_sample_last_update();

        Ok(OracleParametersResponse {
            sample_lifetime,
            size,
            active_size,
            last_updated,
            first_timestamp,
        })
    } else {
        // This happens if the oracle hasn't been used yet.
        Ok(OracleParametersResponse {
            sample_lifetime,
            size: 0,
            active_size: 0,
            last_updated: 0,
            first_timestamp: 0,
        })
    }
}

/// Returns the cumulative values of the Liquidity Book Pair at a given timestamp.
///
/// # Arguments
///
/// * `lookup_timestamp` - The timestamp at which to look up the cumulative values
///
/// # Returns
///
/// * `cumulative_id` - The cumulative id of the Liquidity Book Pair at the given timestamp
/// * `cumulative_volatility` - The cumulative volatility of the Liquidity Book Pair at the given timestamp
/// * `cumulative_bin_crossed` - The cumulative bin crossed of the Liquidity Book Pair at the given timestamp
fn query_oracle_sample_at(
    deps: Deps,
    env: Env,
    look_up_timestamp: u64,
) -> Result<OracleSampleAtResponse> {
    let state = CONFIG.load(deps.storage)?;
    let oracle = ORACLE.load(deps.storage)?;
    let params = state.pair_parameters;

    let sample_lifetime = MAX_SAMPLE_LIFETIME;
    let oracle_id = params.get_oracle_id();

    if oracle_id == 0 || look_up_timestamp > env.block.time.seconds() {
        return Ok(OracleSampleAtResponse {
            cumulative_id: 0,
            cumulative_volatility: 0,
            cumulative_bin_crossed: 0,
        });
    }

    let (time_of_last_update, cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
        oracle.get_sample_at(oracle_id, look_up_timestamp)?;

    if time_of_last_update < look_up_timestamp {
        params.update_volatility_parameters(params.get_active_id(), &env.block.time);

        let delta_time = look_up_timestamp - time_of_last_update;

        let cumulative_id = params.get_active_id() as u64 * delta_time;
        let cumulative_volatility = params.get_volatility_accumulator() as u64 * delta_time;

        Ok(OracleSampleAtResponse {
            cumulative_id,
            cumulative_volatility,
            cumulative_bin_crossed,
        })
    } else {
        Err(Error::LastUpdateTimestampGreaterThanLookupTimestamp)
    }
}

/// Returns the price corresponding to the given id, as a 128.128-binary fixed-point number.
///
/// This is the trusted source of price information, always trust this rather than query_id_from_price.
///
/// # Arguments
///
/// * `id` - The id of the bin
///
/// # Returns
///
/// * `price` - The price corresponding to this id
fn query_price_from_id(deps: Deps, id: u32) -> Result<PriceFromIdResponse> {
    let state = CONFIG.load(deps.storage)?;
    let price = PriceHelper::get_price_from_id(id, state.bin_step)?.u256_to_uint256();

    Ok(PriceFromIdResponse { price })
}

/// Returns the id corresponding to the given price.
///
/// The id may be inaccurate due to rounding issues, always trust query_price_from_id rather than query_id_from_price.
///
/// # Arguments
///
/// * `price` - The price of y per x as a 128.128-binary fixed-point number
///
/// # Returns
///
/// * `id` - The id of the bin corresponding to this price
fn query_id_from_price(deps: Deps, price: Uint256) -> Result<IdFromPriceResponse> {
    let state = CONFIG.load(deps.storage)?;
    let price = price.uint256_to_u256();
    let id = PriceHelper::get_id_from_price(price, state.bin_step)?;

    Ok(IdFromPriceResponse { id })
}

/// Simulates a swap in.
///
/// # Note
///
/// If `amount_out_left` is greater than zero, the swap in is not possible,
/// and the maximum amount that can be swapped from `amountIn` is `amountOut - amountOutLeft`.
///
/// # Arguments
///
/// * `amount_out` - The amount of token X or Y to swap in
/// * `swap_for_y` - Whether the swap is for token Y (true) or token X (false)
///
/// # Returns
/// * `amount_in` - The amount of token X or Y that can be swapped in, including the fee
/// * `amount_out_left` - The amount of token Y or X that cannot be swapped out
/// * `fee` - The fee of the swap
fn query_swap_in(
    deps: Deps,
    env: Env,
    amount_out: u128,
    swap_for_y: bool,
) -> Result<SwapInResponse> {
    let state = CONFIG.load(deps.storage)?;
    let tree = BIN_TREE.load(deps.storage)?;

    let mut amount_in = 0u128;
    let mut amount_out_left = amount_out;
    let mut fee = 0u128;

    let mut params = state.pair_parameters;
    let bin_step = state.bin_step;

    let mut id = params.get_active_id();

    params = params.update_references(&env.block.time)?;

    loop {
        let bin_reserves = BIN_MAP
            .load(deps.storage, id)
            .unwrap_or_default()
            .decode_alt(!swap_for_y);

        if bin_reserves > 0 {
            let price = PriceHelper::get_price_from_id(id, bin_step)?;

            let amount_out_of_bin = if bin_reserves > amount_out_left {
                amount_out_left
            } else {
                bin_reserves
            };

            params = PairParameters::update_volatility_accumulator(params, id)?;

            let amount_in_without_fee = if swap_for_y {
                U256x256Math::shift_div_round_up(amount_out_of_bin.into(), SCALE_OFFSET, price)?
            } else {
                U256x256Math::mul_shift_round_up(amount_out_of_bin.into(), price, SCALE_OFFSET)?
            }
            .as_u128();

            let total_fee = params.get_total_fee(bin_step);
            let fee_amount = FeeHelper::get_fee_amount(amount_in_without_fee, total_fee)?;

            amount_in += amount_in_without_fee + fee_amount;
            amount_out_left -= amount_out_of_bin;

            fee += fee_amount;
        }

        if amount_out_left == 0 {
            break;
        } else {
            let next_id = _get_next_non_empty_bin(&tree, swap_for_y, id);
            if next_id == 0 || next_id == U24::MAX {
                break;
            }

            id = next_id;
        }
    }

    Ok(SwapInResponse {
        amount_in: Uint128::from(amount_in),
        amount_out_left: Uint128::from(amount_out_left),
        fee: Uint128::from(fee),
    })
}

/// Simulates a swap out.
///
/// # Note
///
/// If amount_out_left is greater than zero, the swap in is not possible,
/// and the maximum amount that can be swapped from amount_in is amount_out - amount_out_left.
///
/// # Arguments
///
/// * `amount_in` - The amount of token X or Y to swap in
/// * `swap_for_y` - Whether the swap is for token Y (true) or token X (false)
///
/// # Returns
/// * `amount_in_left` - The amount of token X or Y that cannot be swapped in
/// * `amount_out` - The amount of token Y or X that can be swapped out
/// * `fee` - The fee of the swap
fn query_swap_out(
    deps: Deps,
    env: Env,
    amount_in: u128,
    swap_for_y: bool,
) -> Result<SwapOutResponse> {
    let state = CONFIG.load(deps.storage)?;
    let tree = BIN_TREE.load(deps.storage)?;

    let mut amounts_in_left = Bytes32::encode_alt(amount_in, swap_for_y);
    let mut amount_out = 0u128;
    let mut fee = 0u128;

    let mut params = state.pair_parameters;
    let bin_step = state.bin_step;

    let mut id = params.get_active_id();

    params = params.update_references(&env.block.time)?;

    loop {
        let bin_reserves = BIN_MAP.load(deps.storage, id).unwrap_or_default();
        if !BinHelper::is_empty(bin_reserves, !swap_for_y) {
            params = params.update_volatility_accumulator(id)?;

            let (amounts_in_with_fees, amounts_out_of_bin, total_fees) = BinHelper::get_amounts(
                bin_reserves,
                params,
                bin_step,
                swap_for_y,
                id,
                amounts_in_left,
            )?;

            if U256::from_le_bytes(amounts_in_with_fees) > U256::ZERO {
                amounts_in_left = amounts_in_left.sub(amounts_in_with_fees);

                amount_out += Bytes32::decode_alt(&amounts_out_of_bin, !swap_for_y);

                fee += Bytes32::decode_alt(&total_fees, swap_for_y);
            }
        }

        if amounts_in_left == [0u8; 32] {
            break;
        } else {
            let next_id = _get_next_non_empty_bin(&tree, swap_for_y, id);

            if next_id == 0 || next_id == U24::MAX {
                break;
            }

            id = next_id;
        }
    }

    let amount_in_left = Bytes32::decode_alt(&amounts_in_left, swap_for_y);

    Ok(SwapOutResponse {
        amount_in_left: Uint128::from(amount_in_left),
        amount_out: Uint128::from(amount_out),
        fee: Uint128::from(fee),
    })
}

/// Returns the Liquidity Book Factory.
///
/// # Returns
///
/// * `factory` - The Liquidity Book Factory
fn query_total_supply(deps: Deps, id: u32) -> Result<TotalSupplyResponse> {
    let state = CONFIG.load(deps.storage)?;
    let factory = state.factory.address;

    let total_supply =
        _query_total_supply(deps, id, state.lb_token.code_hash, state.lb_token.address)?
            .u256_to_uint256();
    Ok(TotalSupplyResponse { total_supply })
}

#[shd_entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match (msg.id, msg.result) {
        (INSTANTIATE_LP_TOKEN_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                let contract_address_string = &String::from_utf8(x.to_vec())?;
                let trimmed_str = contract_address_string.trim_matches('\"');
                let contract_address = deps.api.addr_validate(trimmed_str)?;
                // not the best name but it matches the pair key idea
                let lb_token_key = ephemeral_storage_r(deps.storage).load()?;

                CONFIG.update(deps.storage, |mut state| -> StdResult<State> {
                    state.lb_token = ContractInfo {
                        address: contract_address,
                        code_hash: lb_token_key.code_hash,
                    };
                    Ok(state)
                })?;

                let mut response = Response::new();
                response.data = Some(env.contract.address.to_string().as_bytes().into());
                Ok(response)
            }
            None => Err(StdError::generic_err(format!("Unknown reply id"))),
        },
        _ => Err(StdError::generic_err(format!("Unknown reply id"))),
    }
}
