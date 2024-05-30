use crate::{helper::*, prelude::*, state::*};
use ethnum::U256;
use lb_libraries::{
    bin_helper::BinHelper,
    constants::SCALE_OFFSET,
    fee_helper::FeeHelper,
    math::{
        packed_u128_math::PackedUint128Math,
        u24::U24,
        u256x256_math::U256x256Math,
        uint256_to_u256::{ConvertU256, ConvertUint256},
    },
    oracle_helper::MAX_SAMPLE_LIFETIME,
    price_helper::PriceHelper,
    types::Bytes32,
};
use shade_protocol::{
    c_std::{
        from_binary, to_binary, Addr, Binary, Decimal, Deps, Env, StdResult, Uint128, Uint256,
    },
    contract_interfaces::{
        liquidity_book::lb_pair::*,
        swap::{
            amm_pair::{
                FeeInfo,
                QueryMsgResponse::{GetPairInfo, SwapSimulation},
            },
            core::{Fee, TokenAmount, TokenPair, TokenType},
        },
    },
    Contract,
};
use std::collections::HashSet;

// TODO - Revisit if this function is necessary. It seems like something that might belong in the
//        lb-factory contract. It should at least have it's own interface and not use amm_pair's.
pub fn query_pair_info(deps: Deps) -> Result<GetPairInfoResponse> {
    unimplemented!()

    // let state = STATE.load(deps.storage)?;
    //
    // let (reserve_x, reserve_y) = state.reserves.decode();
    //
    // let response = GetPairInfo {
    //     liquidity_token: Contract {
    //         address: state.lb_token.address,
    //         code_hash: state.lb_token.code_hash,
    //     },
    //     factory: Some(Contract {
    //         address: state.factory.address,
    //         code_hash: state.factory.code_hash,
    //     }),
    //     pair: TokenPair(state.token_x, state.token_y, false),
    //     amount_0: Uint128::from(reserve_x),
    //     amount_1: Uint128::from(reserve_y),
    //     total_liquidity: Uint128::default(), // no global liquidity, liquidity is calculated on per bin basis
    //     contract_version: 1, // TODO set this like const AMM_PAIR_CONTRACT_VERSION: u32 = 1;
    //     fee_info: FeeInfo {
    //         shade_dao_address: Addr::unchecked(""), // TODO set shade dao address
    //         lp_fee: Fee {
    //             // TODO set this
    //             nom: state.pair_parameters.get_base_fee(state.bin_step) as u64,
    //             denom: 1_000_000_000_000_000_000,
    //         },
    //         shade_dao_fee: Fee {
    //             nom: state.pair_parameters.get_base_fee(state.bin_step) as u64,
    //             denom: 1_000_000_000_000_000_000,
    //         },
    //         stable_lp_fee: Fee {
    //             nom: state.pair_parameters.get_base_fee(state.bin_step) as u64,
    //             denom: 1_000_000_000_000_000_000,
    //         },
    //         stable_shade_dao_fee: Fee {
    //             nom: state.pair_parameters.get_base_fee(state.bin_step) as u64,
    //             denom: 1_000_000_000_000_000_000,
    //         },
    //     },
    //     stable_info: None,
    // };
    //
    // to_binary(&response).map_err(Error::CwErr)
}

// TODO - Revisit if this function is necessary. It seems like something that might belong in the
//        lb-router contract. It should at least have it's own interface and not use amm_pair's.
pub fn query_swap_simulation(
    deps: Deps,
    env: Env,
    offer: TokenAmount,
    _exclude_fee: Option<bool>,
) -> Result<SwapOutResponse> {
    unimplemented!()

    // let state = STATE.load(deps.storage)?;
    //
    // let mut swap_for_y = false;
    // match offer.token {
    //     token if token == state.token_x => swap_for_y = true,
    //     token if token == state.token_y => {}
    //     _ => panic!("No such token"),
    // };
    //
    // let res = query_swap_out(deps, env, offer.amount.into(), swap_for_y)?;
    //
    // let res = from_binary::<SwapOutResponse>(&res)?;
    //
    // if res.amount_in_left.u128() > 0u128 {
    //     return Err(Error::AmountInLeft {
    //         amount_left_in: res.amount_in_left,
    //         total_amount: offer.amount,
    //         swapped_amount: res.amount_out,
    //     });
    // }
    //
    // let price = Decimal::from_ratio(res.amount_out, offer.amount).to_string();
    //
    // let response = SwapSimulation {
    //     total_fee_amount: res.total_fees,
    //     lp_fee_amount: res.lp_fees,               //TODO lpfee
    //     shade_dao_fee_amount: res.shade_dao_fees, // dao fee
    //     result: SwapResult {
    //         return_amount: res.amount_out,
    //     },
    //     price,
    // };
    //
    // to_binary(&response).map_err(Error::CwErr)
}

/// Returns the Liquidity Book Factory.
///
/// # Returns
///
/// * `factory` - The Liquidity Book Factory
pub fn query_factory(deps: Deps) -> Result<FactoryResponse> {
    let state = STATE.load(deps.storage)?;
    let factory = state.factory.address;

    let response = FactoryResponse { factory };
    Ok(response)
}

/// Returns the Liquidity Book Factory.
///
/// # Returns
///
/// * `factory` - The Liquidity Book Factory
pub fn query_lb_token(deps: Deps) -> Result<LbTokenResponse> {
    let state = STATE.load(deps.storage)?;
    let lb_token = state.lb_token;

    let response = LbTokenResponse { contract: lb_token };
    Ok(response)
}

/// Returns the Liquidity Book Factory.
///
/// # Returns
///
/// * `factory` - The Liquidity Book Factory
pub fn query_staking(deps: Deps) -> Result<StakingResponse> {
    let state = STATE.load(deps.storage)?;
    let staking_contract = state.lb_staking;

    let response = StakingResponse {
        contract: staking_contract,
    };
    Ok(response)
}

/// Returns the token X and Y of the Liquidity Book Pair.
///
/// # Returns
///
/// * `token_x` - The address of the token X
pub fn query_tokens(deps: Deps) -> Result<TokensResponse> {
    let state = STATE.load(deps.storage)?;

    let response = TokensResponse {
        token_x: state.token_x,
        token_y: state.token_y,
    };
    Ok(response)
}

/// Returns the token X of the Liquidity Book Pair.
///
/// # Returns
///
/// * `token_x` - The address of the token X
pub fn query_token_x(deps: Deps) -> Result<TokenXResponse> {
    let state = STATE.load(deps.storage)?;
    let token_x = state.token_x;

    let response = TokenXResponse { token_x };
    Ok(response)
}

/// Returns the token Y of the Liquidity Book Pair.
///
/// # Returns
///
/// * `token_y` - The address of the token Y
pub fn query_token_y(deps: Deps) -> Result<TokenYResponse> {
    let state = STATE.load(deps.storage)?;
    let token_y = state.token_y;

    let response = TokenYResponse { token_y };
    Ok(response)
}

/// Returns the bin_step of the Liquidity Book Pair.
///
/// The bin step is the increase in price between two consecutive bins, in basis points.
/// For example, a bin step of 1 means that the price of the next bin is 0.01% higher than the price of the previous bin.
///
/// # Returns
///
/// * `bin_step` - The bin step of the Liquidity Book Pair, in 10_000th
pub fn query_bin_step(deps: Deps) -> Result<BinStepResponse> {
    let state = STATE.load(deps.storage)?;
    let bin_step = state.bin_step;

    let response = BinStepResponse { bin_step };
    Ok(response)
}

/// Returns the reserves of the Liquidity Book Pair.
///
/// This is the sum of the reserves of all bins, minus the protocol fees.
///
/// # Returns
///
/// * `reserve_x` - The reserve of token X
/// * `reserve_y` - The reserve of token Y
pub fn query_reserves(deps: Deps) -> Result<ReservesResponse> {
    let state = STATE.load(deps.storage)?;
    let (mut reserve_x, mut reserve_y) = state.reserves.decode();
    let (protocol_fee_x, protocol_fee_y) = state.protocol_fees.decode();

    reserve_x -= protocol_fee_x;
    reserve_y -= protocol_fee_y;

    let response = ReservesResponse {
        reserve_x,
        reserve_y,
    };
    Ok(response)
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
pub fn query_active_id(deps: Deps) -> Result<ActiveIdResponse> {
    let state = STATE.load(deps.storage)?;
    let active_id = state.pair_parameters.get_active_id();

    let response = ActiveIdResponse { active_id };
    Ok(response)
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
pub fn query_all_bins_reserves(
    deps: Deps,
    env: Env,
    page: Option<u32>,
    page_size: Option<u32>,
    id: Option<u32>,
) -> Result<AllBinsResponse> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(10);

    let mut id = id.unwrap_or(0u32);
    let mut bin_responses = Vec::new();
    let tree = BIN_TREE.load(deps.storage)?;
    let total = if page > 0 {
        page * page_size
    } else {
        page_size
    };

    let state = STATE.load(deps.storage)?;
    let mut counter: u32 = 0;

    for _ in 0..state.max_bins_per_swap {
        let next_id = tree.find_first_left(id);
        id = next_id;

        if next_id == 0 || next_id == U24::MAX {
            break;
        }

        let (bin_reserve_x, bin_reserve_y) =
            BIN_MAP.load(deps.storage, id).unwrap_or_default().decode();
        bin_responses.push(BinResponse {
            bin_reserve_x,
            bin_reserve_y,
            bin_id: id,
        });
        counter += 1;

        if counter == total {
            break;
        }
    }
    let response = AllBinsResponse {
        reserves: bin_responses,
        last_id: id,
        current_block_height: env.block.height,
    };
    Ok(response)
}

/// Returns the reserves of many bins.
///
/// # Arguments
///
/// * `id` - The id of the bin
///
/// # Returns
///
/// * `bin_reserve_x` - The reserve of token X in the bin
/// * `bin_reserve_y` - The reserve of token Y in the bin
// TODO: Check type names for consistency. This one, for example, should be BinsReservesResponse.
pub fn query_bins_reserves(deps: Deps, ids: Vec<u32>) -> Result<BinsResponse> {
    let mut bin_responses = Vec::new();
    for id in ids {
        let bin: Bytes32 = BIN_MAP.load(deps.storage, id).unwrap_or([0u8; 32]);
        let (bin_reserve_x, bin_reserve_y) = bin.decode();
        bin_responses.push(BinResponse {
            bin_reserve_x,
            bin_reserve_y,
            bin_id: id,
        });
    }

    let response: BinsResponse = BinsResponse(bin_responses);
    Ok(response)
}

pub fn query_updated_bins_at_height(
    deps: Deps,
    height: u64,
) -> Result<UpdatedBinsAtHeightResponse> {
    let ids = BIN_RESERVES_UPDATED.load(deps.storage, height)?;

    let mut bin_responses = Vec::new();

    for id in ids {
        let bin: Bytes32 = BIN_MAP.load(deps.storage, id).unwrap_or([0u8; 32]);
        let (bin_reserve_x, bin_reserve_y) = bin.decode();
        bin_responses.push(BinResponse {
            bin_reserve_x,
            bin_reserve_y,
            bin_id: id,
        });
    }

    let response: UpdatedBinsAtHeightResponse = UpdatedBinsAtHeightResponse(bin_responses);

    Ok(response)
}

pub fn query_updated_bins_at_multiple_heights(
    deps: Deps,
    heights: Vec<u64>,
) -> Result<UpdatedBinsAtMultipleHeightResponse> {
    let mut bin_responses = Vec::new();
    let mut processed_ids = HashSet::new();

    for height in heights {
        let ids = BIN_RESERVES_UPDATED.load(deps.storage, height)?;

        for id in ids {
            // Check if the id has already been processed
            if processed_ids.insert(id) {
                let bin: Bytes32 = BIN_MAP.load(deps.storage, id).unwrap_or([0u8; 32]);
                let (bin_reserve_x, bin_reserve_y) = bin.decode();
                bin_responses.push(BinResponse {
                    bin_reserve_x,
                    bin_reserve_y,
                    bin_id: id,
                });
            }
        }
    }

    let response: UpdatedBinsAtMultipleHeightResponse =
        UpdatedBinsAtMultipleHeightResponse(bin_responses);

    Ok(response)
}

pub fn query_updated_bins_after_height(
    deps: Deps,
    env: Env,
    height: u64,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<UpdatedBinsAfterHeightResponse> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(10);
    let mut processed_ids = HashSet::new();

    let heights: StdResult<Vec<u64>> = BIN_RESERVES_UPDATED_LOG
        .iter(deps.storage)?
        .rev()
        .skip((page * page_size) as usize)
        .take_while(|result| match result {
            Ok(h) => {
                if &height >= h {
                    false
                } else {
                    true
                }
            }
            Err(_) => todo!(),
        })
        .take(page_size as usize)
        .collect();

    let mut bin_responses = Vec::new();

    for height in heights? {
        let ids = BIN_RESERVES_UPDATED.load(deps.storage, height)?;

        for id in ids {
            if processed_ids.insert(id) {
                let bin: Bytes32 = BIN_MAP.load(deps.storage, id).unwrap_or([0u8; 32]);
                let (bin_reserve_x, bin_reserve_y) = bin.decode();
                bin_responses.push(BinResponse {
                    bin_reserve_x,
                    bin_reserve_y,
                    bin_id: id,
                });
            }
        }
    }

    let response = UpdatedBinsAfterHeightResponse {
        bins: bin_responses,
        current_block_height: env.block.height,
    };

    Ok(response)
}

pub fn query_bins_updating_heights(
    deps: Deps,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<BinUpdatingHeightsResponse> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(10);
    let txs: StdResult<Vec<u64>> = BIN_RESERVES_UPDATED_LOG
        .iter(deps.storage)?
        .rev()
        .skip((page * page_size) as usize)
        .take(page_size as usize)
        .collect();

    let response = BinUpdatingHeightsResponse(txs?);

    Ok(response)
}

/// Returns the bins changed after that block height
///
/// # Arguments
///
/// * `id` - The id of the bin
///
/// # Returns
///
/// * `bin_reserve_x` - The reserve of token X in the bin
/// * `bin_reserve_y` - The reserve of token Y in the bin

pub fn query_bin_reserves(deps: Deps, id: u32) -> Result<BinResponse> {
    let bin: Bytes32 = BIN_MAP.load(deps.storage, id).unwrap_or([0u8; 32]);
    let (bin_reserve_x, bin_reserve_y) = bin.decode();

    let response = BinResponse {
        bin_reserve_x,
        bin_reserve_y,
        bin_id: id,
    };
    Ok(response)
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
pub fn query_next_non_empty_bin(
    deps: Deps,
    swap_for_y: bool,
    id: u32,
) -> Result<NextNonEmptyBinResponse> {
    let tree = BIN_TREE.load(deps.storage)?;
    let next_id = _get_next_non_empty_bin(&tree, swap_for_y, id);

    let response = NextNonEmptyBinResponse { next_id };
    Ok(response)
}

/// Returns the protocol fees of the Liquidity Book Pair.
///
/// # Returns
///
/// * `protocol_fee_x` - The protocol fees of token X
/// * `protocol_fee_y` - The protocol fees of token Y
pub fn query_protocol_fees(deps: Deps) -> Result<ProtocolFeesResponse> {
    let state = STATE.load(deps.storage)?;
    let (protocol_fee_x, protocol_fee_y) = state.protocol_fees.decode();

    let response = ProtocolFeesResponse {
        protocol_fee_x,
        protocol_fee_y,
    };
    Ok(response)
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
pub fn query_static_fee_params(deps: Deps) -> Result<StaticFeeParametersResponse> {
    let state = STATE.load(deps.storage)?;
    let params = state.pair_parameters;

    let base_factor = params.get_base_factor();
    let filter_period = params.get_filter_period();
    let decay_period = params.get_decay_period();
    let reduction_factor = params.get_reduction_factor();
    let variable_fee_control = params.get_variable_fee_control();
    let protocol_share = params.get_protocol_share();
    let max_volatility_accumulator = params.get_max_volatility_accumulator();

    let response = StaticFeeParametersResponse {
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    };
    Ok(response)
}

/// Returns the variable fee parameters of the Liquidity Book Pair.
///
/// # Returns
///
/// * `volatility_accumulator` - The volatility accumulator for the variable fee
/// * `volatility_reference` - The volatility reference for the variable fee
/// * `id_reference` - The id reference for the variable fee
/// * `time_of_last_update` - The time of last update for the variable fee
pub fn query_variable_fee_params(deps: Deps) -> Result<VariableFeeParametersResponse> {
    let state = STATE.load(deps.storage)?;
    let params = state.pair_parameters;

    let volatility_accumulator = params.get_volatility_accumulator();
    let volatility_reference = params.get_volatility_reference();
    let id_reference = params.get_id_reference();
    let time_of_last_update = params.get_time_of_last_update();

    let response = VariableFeeParametersResponse {
        volatility_accumulator,
        volatility_reference,
        id_reference,
        time_of_last_update,
    };

    Ok(response)
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
pub fn query_oracle_params(deps: Deps) -> Result<OracleParametersResponse> {
    let state = STATE.load(deps.storage)?;
    let params = state.pair_parameters;

    let sample_lifetime = MAX_SAMPLE_LIFETIME;
    let oracle_id = params.get_oracle_id();
    let oracle = ORACLE.load(deps.storage, oracle_id)?;
    let size = DEFAULT_ORACLE_LENGTH;

    if oracle_id > 0 {
        let sample = oracle.0;

        let last_updated = sample.get_sample_last_update();
        let first_timestamp = sample.get_sample_last_update();

        let response = OracleParametersResponse {
            sample_lifetime,
            size,
            last_updated,
            first_timestamp,
        };

        Ok(response)
    } else {
        // This happens if the oracle hasn't been used yet.
        let response = OracleParametersResponse {
            sample_lifetime,
            size,
            last_updated: 0,
            first_timestamp: 0,
        };

        Ok(response)
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
pub fn query_oracle_sample(
    deps: Deps,
    _env: Env,
    oracle_id: u16,
) -> Result<OracleSampleAtResponse> {
    if oracle_id == 0 {
        return Err(Error::OracleNotActive);
    }

    let oracle = ORACLE.load(deps.storage, oracle_id)?;

    let response = OracleSampleAtResponse {
        sample: OracleSampleResponse {
            cumulative_id: oracle.0.get_cumulative_id(),
            cumulative_txns: oracle.0.get_cumulative_txns(),
            cumulative_volatility: oracle.0.get_cumulative_volatility(),
            cumulative_bin_crossed: oracle.0.get_cumulative_bin_crossed(),
            cumulative_volume_x: oracle.0.get_vol_token_x(),
            cumulative_volume_y: oracle.0.get_vol_token_y(),
            cumulative_fee_x: oracle.0.get_fee_token_x(),
            cumulative_fee_y: oracle.0.get_fee_token_y(),
            oracle_id,
            lifetime: oracle.0.get_sample_lifetime(),
            created_at: oracle.0.get_sample_creation(),
        },
    };

    Ok(response)
}

pub fn query_oracle_samples(
    deps: Deps,
    _env: Env,
    oracle_ids: Vec<u16>,
) -> Result<OracleSamplesAtResponse> {
    let mut samples = Vec::new();

    for oracle_id in oracle_ids {
        if oracle_id == 0 {
            return Err(Error::OracleNotActive);
        }

        // Initialize response with default values
        let mut sample = OracleSampleResponse {
            cumulative_id: 0,
            cumulative_txns: 0,
            cumulative_volatility: 0,
            cumulative_bin_crossed: 0,
            cumulative_volume_x: 0,
            cumulative_volume_y: 0,
            cumulative_fee_x: 0,
            cumulative_fee_y: 0,
            oracle_id,
            lifetime: 0,
            created_at: 0,
        };

        // Update response if oracle data is successfully loaded
        if let Ok(oracle) = ORACLE.load(deps.storage, oracle_id) {
            sample.cumulative_id = oracle.0.get_cumulative_id();
            sample.cumulative_txns = oracle.0.get_cumulative_txns();
            sample.cumulative_volatility = oracle.0.get_cumulative_volatility();
            sample.cumulative_bin_crossed = oracle.0.get_cumulative_bin_crossed();
            sample.cumulative_volume_x = oracle.0.get_vol_token_x();
            sample.cumulative_volume_y = oracle.0.get_vol_token_y();
            sample.cumulative_fee_x = oracle.0.get_fee_token_x();
            sample.cumulative_fee_y = oracle.0.get_fee_token_y();
            sample.lifetime = oracle.0.get_sample_lifetime();
            sample.created_at = oracle.0.get_sample_creation();
        }

        samples.push(sample);
    }

    let response = OracleSamplesAtResponse { samples };

    Ok(response)
}

pub fn query_oracle_samples_after(
    deps: Deps,
    _env: Env,
    start_oracle_id: u16,
    page_size: Option<u16>,
) -> Result<OracleSamplesAfterResponse> {
    let mut samples = Vec::new();

    let state = STATE.load(deps.storage)?;
    let active_oracle_id = state.pair_parameters.get_oracle_id();
    let length = DEFAULT_ORACLE_LENGTH;

    // Convert to larger integer type for calculations
    let active_oracle_id_u32 = u32::from(active_oracle_id);
    let start_oracle_id_u32 = u32::from(start_oracle_id);
    let length_u32 = u32::from(length);

    // Perform calculation in larger integer type
    let num_iterations_u32 = if active_oracle_id_u32 == start_oracle_id_u32 {
        1
    } else {
        (active_oracle_id_u32 + length_u32 - start_oracle_id_u32) % length_u32
    };

    // Convert the result back to u16 for iteration
    let num_iterations = num_iterations_u32 as u16;

    for i in 0..num_iterations {
        // Calculate the current oracle_id considering the circular nature

        let current_oracle_id_u32 = (start_oracle_id_u32 + (i as u32)) % length_u32;
        let current_oracle_id = current_oracle_id_u32 as u16;

        if current_oracle_id == 0 {
            continue;
        }

        let sample = match ORACLE.load(deps.storage, current_oracle_id) {
            Ok(oracle) => OracleSampleResponse {
                cumulative_id: oracle.0.get_cumulative_id(),
                cumulative_txns: oracle.0.get_cumulative_txns(),
                cumulative_volatility: oracle.0.get_cumulative_volatility(),
                cumulative_bin_crossed: oracle.0.get_cumulative_bin_crossed(),
                cumulative_volume_x: oracle.0.get_vol_token_x(),
                cumulative_volume_y: oracle.0.get_vol_token_y(),
                cumulative_fee_x: oracle.0.get_fee_token_x(),
                cumulative_fee_y: oracle.0.get_fee_token_y(),
                oracle_id: current_oracle_id,
                lifetime: oracle.0.get_sample_lifetime(),
                created_at: oracle.0.get_sample_creation(),
            },
            // TODO: should this return an Error instead of all zeroes?
            Err(_) => OracleSampleResponse {
                cumulative_id: 0,
                cumulative_txns: 0,
                cumulative_volatility: 0,
                cumulative_bin_crossed: 0,
                cumulative_volume_x: 0,
                cumulative_volume_y: 0,
                cumulative_fee_x: 0,
                cumulative_fee_y: 0,
                oracle_id: current_oracle_id,
                lifetime: 0,
                created_at: 0,
            },
        };
        samples.push(sample);
    }

    let response = OracleSamplesAfterResponse { samples };

    Ok(response)
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
pub fn query_price_from_id(deps: Deps, id: u32) -> Result<PriceFromIdResponse> {
    let state = STATE.load(deps.storage)?;
    let price = PriceHelper::get_price_from_id(id, state.bin_step)?.u256_to_uint256();

    let response = PriceFromIdResponse { price };
    Ok(response)
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
pub fn query_id_from_price(deps: Deps, price: Uint256) -> Result<IdFromPriceResponse> {
    let state = STATE.load(deps.storage)?;
    let price = price.uint256_to_u256();
    let id = PriceHelper::get_id_from_price(price, state.bin_step)?;

    let response = IdFromPriceResponse { id };
    Ok(response)
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
pub fn query_swap_in(
    deps: Deps,
    env: Env,
    amount_out: u128,
    swap_for_y: bool,
) -> Result<SwapInResponse> {
    let state = STATE.load(deps.storage)?;
    let tree = BIN_TREE.load(deps.storage)?;

    let mut amount_in = 0u128;
    let mut amount_out_left = amount_out;
    let mut fee = 0u128;

    let mut params = state.pair_parameters;
    let bin_step = state.bin_step;

    let mut id = params.get_active_id();

    params.update_references(&env.block.time)?;

    for _ in 0..state.max_bins_per_swap {
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

            params.update_volatility_accumulator(id)?;

            let amount_in_without_fee = if swap_for_y {
                U256x256Math::shift_div_round_up(amount_out_of_bin.into(), SCALE_OFFSET, price)?
            } else {
                U256x256Math::mul_shift_round_up(amount_out_of_bin.into(), price, SCALE_OFFSET)?
            }
            .as_u128();

            let total_fee = params.get_total_fee(bin_step)?;
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

    let response = SwapInResponse {
        amount_in: Uint128::from(amount_in),
        amount_out_left: Uint128::from(amount_out_left),
        fee: Uint128::from(fee),
    };
    Ok(response)
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
pub fn query_swap_out(
    deps: Deps,
    env: Env,
    amount_in: u128,
    swap_for_y: bool,
) -> Result<SwapOutResponse> {
    let state = STATE.load(deps.storage)?;
    let tree = BIN_TREE.load(deps.storage)?;

    let mut amounts_in_left = Bytes32::encode_alt(amount_in, swap_for_y);
    let mut amounts_out = [0u8; 32];
    let _fee = 0u128;
    let mut _fee = 0u128;
    let mut total_fees: [u8; 32] = [0; 32];
    let mut lp_fees: [u8; 32] = [0; 32];
    let mut shade_dao_fees: [u8; 32] = [0; 32];

    let mut params = state.pair_parameters;
    let bin_step = state.bin_step;

    let mut id = params.get_active_id();

    params.update_references(&env.block.time)?;

    for _ in 0..state.max_bins_per_swap {
        let bin_reserves = BIN_MAP.load(deps.storage, id).unwrap_or_default();
        if !BinHelper::is_empty(bin_reserves, !swap_for_y) {
            let price = PriceHelper::get_price_from_id(id, bin_step)?;

            params = *params.update_volatility_accumulator(id)?;

            let (amounts_in_with_fees, amounts_out_of_bin, fees) = BinHelper::get_amounts(
                bin_reserves,
                params,
                bin_step,
                swap_for_y,
                amounts_in_left,
                price,
            )?;

            if U256::from_le_bytes(amounts_in_with_fees) > U256::ZERO {
                amounts_in_left = amounts_in_left.sub(amounts_in_with_fees);
                amounts_out = amounts_out.add(amounts_out_of_bin);

                let p_fees =
                    fees.scalar_mul_div_basis_point_round_down(params.get_protocol_share().into())?;
                total_fees = total_fees.add(fees);
                lp_fees = lp_fees.add(fees.sub(p_fees));
                shade_dao_fees = shade_dao_fees.add(p_fees);
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

    let response = SwapOutResponse {
        amount_in_left: Uint128::from(amount_in_left),
        amount_out: Uint128::from(amounts_out.decode_alt(!swap_for_y)),
        total_fees: Uint128::from(total_fees.decode_alt(swap_for_y)),
        shade_dao_fees: Uint128::from(shade_dao_fees.decode_alt(swap_for_y)),
        lp_fees: Uint128::from(lp_fees.decode_alt(swap_for_y)),
    };
    Ok(response)
}

/// Returns the Liquidity Book Factory.
///
/// # Returns
///
/// * `factory` - The Liquidity Book Factory
pub fn query_total_supply(deps: Deps, id: u32) -> Result<TotalSupplyResponse> {
    let state = STATE.load(deps.storage)?;
    let _factory = state.factory.address;

    let total_supply =
        _query_total_supply(deps, id, state.lb_token.code_hash, state.lb_token.address)?
            .u256_to_uint256();

    let response = TotalSupplyResponse { total_supply };

    Ok(response)
}

pub fn query_rewards_distribution(
    deps: Deps,
    epoch_id: Option<u64>,
) -> Result<RewardsDistributionResponse> {
    let epoch_id = match epoch_id {
        Some(id) => id,
        None => STATE.load(deps.storage)?.rewards_epoch_index - 1,
    };

    let response = RewardsDistributionResponse {
        distribution: REWARDS_DISTRIBUTION.load(deps.storage, epoch_id)?,
    };

    Ok(response)
}
