use crate::prelude::*;
use ethnum::U256;
use lb_libraries::math::{tree_math::TreeUint24, u24::U24, uint256_to_u256::ConvertUint256};
use shade_protocol::{
    c_std::{Addr, CosmosMsg, Deps, Env, StdResult},
    contract_interfaces::liquidity_book::{lb_pair::*, lb_token},
    // TODO: sort out viewing key strategy
    s_toolkit::snip20::{register_receive_msg, set_viewing_key_msg},
    snip20,
    swap::core::{TokenType, ViewingKey},
};

pub const INSTANTIATE_LP_TOKEN_REPLY_ID: u64 = 1u64;
pub const INSTANTIATE_STAKING_CONTRACT_REPLY_ID: u64 = 2u64;
pub const MINT_REPLY_ID: u64 = 1u64;
pub const DEFAULT_REWARDS_BINS: u32 = 100;
pub const DEFAULT_MAX_BINS_PER_SWAP: u32 = 100;
pub const DEFAULT_ORACLE_LENGTH: u16 = u16::MAX;

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
            viewing_key.to_string(),
            None,
            256,
            contract_addr.clone().to_string(),
            token_code_hash.to_string(),
        )?);
        messages.push(register_receive_msg(
            env.contract.code_hash.clone(),
            None,
            256,
            contract_addr.to_string(),
            token_code_hash.to_string(),
        )?);
    }

    Ok(())
}

pub fn match_lengths(liquidity_parameters: &LiquidityParameters) -> Result<()> {
    if liquidity_parameters.delta_ids.len() != liquidity_parameters.distribution_x.len()
        || liquidity_parameters.delta_ids.len() != liquidity_parameters.distribution_y.len()
    {
        return Err(Error::LengthsMismatch);
    }
    Ok(())
}

pub fn check_ids_bounds(liquidity_parameters: &LiquidityParameters) -> Result<()> {
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

pub fn check_active_id_slippage(
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
pub fn calculate_id(
    liquidity_parameters: &LiquidityParameters,
    active_id: u32,
    i: usize,
) -> Result<u32> {
    // let id: u32;

    let id: i64 = active_id as i64 + liquidity_parameters.delta_ids[i];

    if id < 0 || id as u32 > U24::MAX {
        return Err(Error::DeltaIdOverflows {
            delta_id: liquidity_parameters.delta_ids[i],
        });
    }

    Ok(id as u32)
}

pub fn _query_total_supply(deps: Deps, id: u32, code_hash: String, address: Addr) -> Result<U256> {
    let msg = lb_token::QueryMsg::IdTotalBalance { id: id.to_string() };

    let res = deps.querier.query_wasm_smart::<lb_token::QueryAnswer>(
        code_hash,
        address.to_string(),
        &msg,
    )?;

    let total_supply_uint256 = match res {
        lb_token::QueryAnswer::IdTotalBalance { amount } => amount,
        _ => panic!("{}", format!("Wrong response for lb_token")),
    };

    Ok(total_supply_uint256.uint256_to_u256())
}

pub fn query_token_symbol(deps: Deps, code_hash: String, address: Addr) -> Result<String> {
    let msg = snip20::QueryMsg::TokenInfo {};

    let res = deps.querier.query_wasm_smart::<snip20::QueryAnswer>(
        code_hash,
        address.to_string(),
        &(&msg),
    )?;

    let symbol = match res {
        snip20::QueryAnswer::TokenInfo { symbol, .. } => symbol,
        _ => panic!("{}", format!("Token {} not valid", address)),
    };

    Ok(symbol)
}

/// Returns id of the next non-empty bin.
///
/// # Arguments
/// * `swap_for_y Whether the swap is for Y
/// * `id` - The id of the bin
pub fn _get_next_non_empty_bin(tree: &TreeUint24, swap_for_y: bool, id: u32) -> u32 {
    if swap_for_y {
        tree.find_first_right(id)
    } else {
        tree.find_first_left(id)
    }
}

pub fn only_factory(sender: &Addr, factory: &Addr) -> Result<()> {
    if sender != factory {
        return Err(Error::OnlyFactory);
    }
    Ok(())
}
