#![allow(unused)] // For beginning only.

use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, ContractInfo, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Reply, Response, StdError, StdResult, Storage, SubMsg, SubMsgResult, Timestamp, Uint256,
    WasmMsg,
};
use ethnum::U256;
use shade_protocol::{
    admin::helpers::{admin_is_valid, validate_admin, AdminPermissions},
    lb_libraries::{math, pair_parameter_helper, price_helper, tokens, types, viewing_keys},
    liquidity_book::{
        lb_factory::*,
        lb_pair::ExecuteMsg::{ForceDecay as LbPairForceDecay, SetStaticFeeParameters},
    },
};

use math::encoded_sample::EncodedSample;
use pair_parameter_helper::PairParameters;
use price_helper::PriceHelper;
use tokens::TokenType;
use types::{Bytes32, ContractInstantiationInfo, StaticFeeParameters};

use crate::{
    prelude::*,
    state::*,
    types::{LBPair, LBPairInformation, NextPairKey},
};

pub static _OFFSET_IS_PRESET_OPEN: u8 = 255;
pub static _MIN_BIN_STEP: u8 = 1; // 0.001%
pub static _MAX_FLASHLOAN_FEE: u8 = 10 ^ 17; // 10%

pub const INSTANTIATE_REPLY_ID: u64 = 1u64;

/////////////// INSTANTIATE ///////////////

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response> {
    if msg.flash_loan_fee > _MAX_FLASHLOAN_FEE {
        return Err(Error::FlashLoanFeeAboveMax {
            fee: msg.flash_loan_fee,
            max_fee: _MAX_FLASHLOAN_FEE,
        });
    }

    let state = State {
        contract_info: ContractInfo {
            address: env.contract.address,
            code_hash: env.contract.code_hash,
        },
        owner: msg.owner.unwrap_or_else(|| info.sender.clone()),
        fee_recipient: msg.fee_recipient,
        flash_loan_fee: msg.flash_loan_fee,
        lb_pair_implementation: ContractInstantiationInfo::default(),
        lb_token_implementation: ContractInstantiationInfo::default(),
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
    };

    CONFIG.save(deps.storage, &state)?;

    // TODO: decide on response output and format
    Ok(Response::default())
}

/////////////// EXECUTE ///////////////

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response> {
    match msg {
        ExecuteMsg::SetLBPairImplementation {
            lb_pair_implementation,
        } => try_set_lb_pair_implementation(deps, env, info, lb_pair_implementation),
        ExecuteMsg::SetLBTokenImplementation {
            lb_token_implementation,
        } => try_set_lb_token_implementation(deps, env, info, lb_token_implementation),
        ExecuteMsg::CreateLBPair {
            token_x,
            token_y,
            active_id,
            bin_step,
            viewing_key,
        } => try_create_lb_pair(
            deps,
            env,
            info,
            token_x,
            token_y,
            active_id,
            bin_step,
            viewing_key,
        ),
        // ExecuteMsg::SetLBPairIgnored {
        //     token_x,
        //     token_y,
        //     bin_step,
        //     ignored,
        // } => try_set_lb_pair_ignored(deps, env, info, token_x, token_y, bin_step, ignored),
        ExecuteMsg::SetPairPreset {
            bin_step,
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
            is_open,
        } => try_set_pair_preset(
            deps,
            env,
            info,
            bin_step,
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
            is_open,
        ),
        ExecuteMsg::SetPresetOpenState { bin_step, is_open } => {
            try_set_preset_open_state(deps, env, info, bin_step, is_open)
        }
        ExecuteMsg::RemovePreset { bin_step } => try_remove_preset(deps, env, info, bin_step),
        ExecuteMsg::SetFeeParametersOnPair {
            token_x,
            token_y,
            bin_step,
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
        } => try_set_fee_parameters_on_pair(
            deps,
            env,
            info,
            token_x,
            token_y,
            bin_step,
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
        ),
        ExecuteMsg::SetFeeRecipient { fee_recipient } => {
            try_set_fee_recipient(deps, env, info, fee_recipient)
        }
        ExecuteMsg::SetFlashLoanFee { flash_loan_fee } => {
            try_set_flash_loan_fee(deps, env, info, flash_loan_fee)
        }
        ExecuteMsg::AddQuoteAsset { asset } => try_add_quote_asset(deps, env, info, asset),
        ExecuteMsg::RemoveQuoteAsset { asset } => try_remove_quote_asset(deps, env, info, asset),
        ExecuteMsg::ForceDecay { pair } => try_force_decay(deps, env, info, pair),
    }
}

/// Sets the LBPair implementation details.
///
/// # Arguments
///
/// * `new_lb_pair_implementation` - The code ID and code hash of the implementation.
fn try_set_lb_pair_implementation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_lb_pair_implementation: ContractInstantiationInfo,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;

    // TODO: query the LBPair contract to check that the factory address is correct
    // if ILBPair(new_lb_pair_implementation).getFactory() != env.contract.address {
    //     return Err(Error::LBPairSafetyCheckFailed(new_lb_pair_implementation.address))
    // }

    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;

    let old_lb_pair_implementation = state.lb_pair_implementation;
    if (old_lb_pair_implementation == new_lb_pair_implementation) {
        return Err(Error::SameImplementation {
            lb_implementation: old_lb_pair_implementation.id,
        });
    }

    CONFIG.update(deps.storage, |mut state| -> StdResult<_> {
        state.lb_pair_implementation = new_lb_pair_implementation;
        Ok(state)
    })?;

    Ok(Response::default())
}

/// Sets the LBToken implementation details.
///
/// # Arguments
///
/// * `new_lb_token_implementation` - The code ID and code hash of the implementation.
fn try_set_lb_token_implementation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_lb_token_implementation: ContractInstantiationInfo,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    // TODO: query the LBToken contract to check that the factory address is correct
    // if ILBToken(new_lb_token_implementation).getFactory() != env.contract.address {
    //     return Err(Error::LBTokenSafetyCheckFailed(new_lb_token_implementation.address))
    // }

    let old_lb_token_implementation = state.lb_token_implementation;
    if (old_lb_token_implementation == new_lb_token_implementation) {
        return Err(Error::SameImplementation {
            lb_implementation: old_lb_token_implementation.id,
        });
    }

    CONFIG.update(deps.storage, |mut state| -> StdResult<_> {
        state.lb_token_implementation = new_lb_token_implementation;
        Ok(state)
    })?;

    Ok(Response::default())
}

/// Creates a liquidity bin LBPair for token_x and token_y.
///
/// # Arguments
///
/// * `token_x` - The address of the first token.
/// * `token_y` - The address of the second token.
/// * `active_id` - The active id of the pair.
/// * `bin_step` - The bin step in basis point, used to calculate log(1 + binStep / 10_000).
///
/// # Returns
///
/// * `pair` - The address of the newly created LBPair.
fn try_create_lb_pair(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_x: TokenType,
    token_y: TokenType,
    active_id: u32,
    bin_step: u16,
    viewing_key: String,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;

    if !PRESETS.has(deps.storage, bin_step) {
        return Err(Error::BinStepHasNoPreset { bin_step });
    }

    let preset = PRESETS
        .load(deps.storage, bin_step)
        .map_err(|_| Error::BinStepHasNoPreset { bin_step })?;
    let is_owner = info.sender == state.owner;

    if !_is_preset_open(preset.0 .0) && !is_owner {
        return Err(Error::PresetIsLockedForUsers {
            user: info.sender,
            bin_step,
        });
    }

    if !QUOTE_ASSET_WHITELIST
        .iter(deps.storage)?
        .any(|result| match result {
            Ok(t) => t.eq(&token_y),
            Err(_) => false, // Handle the error case as needed
        })
    {
        return Err(Error::QuoteAssetNotWhitelisted {
            quote_asset: token_y.unique_key().clone(),
        });
    }

    if token_x == token_y {
        return Err(Error::IdenticalAddresses {
            token: token_x.unique_key().clone(),
        });
    }

    // safety check, making sure that the price can be calculated
    PriceHelper::get_price_from_id(active_id, bin_step);

    let (token_a, token_b) = _sort_tokens(token_x.clone(), token_y.clone());

    // TODO: error if address doesn't exist on chain
    // if (address(tokenA) == address(0)) revert LBFactory__AddressZero();

    if LB_PAIRS_INFO
        .load(
            deps.storage,
            (
                token_a.unique_key().clone(),
                token_b.unique_key().clone(),
                bin_step,
            ),
        )
        .is_ok()
    {
        return Err(Error::LBPairAlreadyExists {
            token_x: token_x.unique_key(),
            token_y: token_y.unique_key(),
            bin_step,
        });
    }

    if state.lb_pair_implementation.id == 0 {
        return Err(Error::ImplementationNotSet);
    }

    let mut messages = vec![];

    messages.push(SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: state.lb_pair_implementation.id,
            label: format!(
                "{}-{}-{}-pair-{}-{}",
                token_x.unique_key(),
                token_y.unique_key(),
                bin_step,
                env.contract.address,
                state.lb_pair_implementation.id
            ),
            msg: to_binary(&LBPairInstantiateMsg {
                factory: env.contract,
                token_x,
                token_y,
                bin_step,
                pair_parameters: StaticFeeParameters {
                    base_factor: preset.get_base_factor(),
                    filter_period: preset.get_filter_period(),
                    decay_period: preset.get_decay_period(),
                    reduction_factor: preset.get_reduction_factor(),
                    variable_fee_control: preset.get_variable_fee_control(),
                    protocol_share: preset.get_protocol_share(),
                    max_volatility_accumulator: preset.get_max_volatility_accumulator(),
                },
                active_id,
                lb_token_implementation: state.lb_token_implementation,
                //TODO add viewing key
                viewing_key,
                //TODO add pair_name
                pair_name: String::new(),
                entropy: String::new(),
                protocol_fee_recipient: state.fee_recipient,
                admin_auth: state.admin_auth.into(),
            })?,
            code_hash: state.lb_pair_implementation.code_hash.clone(),
            funds: vec![],
            admin: None,
        }),
        INSTANTIATE_REPLY_ID,
    ));

    ephemeral_storage_w(deps.storage).save(&NextPairKey {
        token_a: token_a.clone(),
        token_b: token_b.clone(),
        bin_step,
        code_hash: state.lb_pair_implementation.code_hash,
        is_open: is_owner,
    })?;

    Ok(Response::new().add_submessages(messages))

    // emit LBPairCreated(tokenX, tokenY, binStep, pair, _allLBPairs.length - 1);
}

// /// Sets whether the pair is ignored or not for routing, it will make the pair unusable by the router.
// ///
// /// # Arguments
// ///
// /// * `token_x` - The address of the first token of the pair.
// /// * `token_y` - The address of the second token of the pair.
// /// * `bin_step` - The bin step in basis point of the pair.
// /// * `ignored` - Whether to ignore (true) or not (false) the pair for routing.
// fn try_set_lb_pair_ignored(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     token_a: TokenType,
//     token_b: TokenType,
//     bin_step: u16,
//     ignored: bool,
// ) -> Result<Response> {
//     let state = CONFIG.load(deps.storage)?;
//     only_owner(&info.sender, &state.owner)?;

//     let (token_a, token_b) = _sort_tokens(token_a, token_b);

//     let mut pair_information = LB_PAIRS_INFO
//         .load(
//             deps.storage,
//             (
//                 token_a.unique_key().clone(),
//                 token_b.unique_key().clone(),
//                 bin_step,
//             ),
//         )
//         .unwrap();

//     if pair_information
//         .lb_pair
//         .contract
//         .address
//         .as_str()
//         .is_empty()
//     {
//         return Err(Error::LBPairDoesNotExist {
//             token_x: token_a.unique_key().clone(),
//             token_y: token_b.unique_key().clone(),
//             bin_step,
//         });
//     }

//     if pair_information.ignored_for_routing == ignored {
//         return Err(Error::LBPairIgnoredIsAlreadyInTheSameState);
//     }

//     pair_information.ignored_for_routing = ignored;

//     LB_PAIRS_INFO.save(
//         deps.storage,
//         (
//             token_a.unique_key().clone(),
//             token_b.unique_key().clone(),
//             bin_step,
//         ),
//         &pair_information,
//     )?;

//     // emit LBPairIgnoredStateChanged(pairInformation.LBPair, ignored);

//     // TODO: be more specific about which pair changed
//     Ok(Response::default()
//         .add_attribute_plaintext("LBPair ignored state changed", format!("{}", ignored)))
// }

/// Sets the preset parameters of a bin step
///
/// # Arguments
///
/// * `bin_step` - The bin step in basis point, used to calculate the price
/// * `base_factor` - The base factor, used to calculate the base fee, baseFee = baseFactor * binStep
/// * `filter_period` - The period where the accumulator value is untouched, prevent spam
/// * `decay_period` - The period where the accumulator value is decayed, by the reduction factor
/// * `reduction_factor` - The reduction factor, used to calculate the reduction of the accumulator
/// * `variable_fee_control` - The variable fee control, used to control the variable fee, can be 0 to disable it
/// * `protocol_share` - The share of the fees received by the protocol
/// * `max_volatility_accumulator` - The max value of the volatility accumulator
/// * `is_open` - Whether the preset is open or not to be used by users
fn try_set_pair_preset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bin_step: u16,
    base_factor: u16,
    filter_period: u16,
    decay_period: u16,
    reduction_factor: u16,
    variable_fee_control: u32,
    protocol_share: u16,
    max_volatility_accumulator: u32,
    is_open: bool,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    if bin_step < _MIN_BIN_STEP as u16 {
        return Err(Error::BinStepTooLow { bin_step });
    }

    let mut preset = PairParameters(EncodedSample([0u8; 32])).set_static_fee_parameters(
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    )?;

    if is_open {
        preset = PairParameters(preset.0.set_bool(true, _OFFSET_IS_PRESET_OPEN));
    }

    PRESETS.save(deps.storage, bin_step, &preset)?;

    Ok(Response::default().add_attribute_plaintext("set preset", bin_step.to_string()))
}

/// Sets if the preset is open or not to be used by users
///
/// # Arguments
///
/// * `bin_step` - The bin step in basis point, used to calculate the price
/// * `is_open` - Whether the preset is open or not
fn try_set_preset_open_state(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bin_step: u16,
    is_open: bool,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    if !PRESETS.has(deps.storage, bin_step) {
        return Err(Error::BinStepHasNoPreset { bin_step });
    }

    let preset = PRESETS.load(deps.storage, bin_step).unwrap();

    if preset.0.decode_bool(_OFFSET_IS_PRESET_OPEN) == is_open {
        return Err(Error::PresetOpenStateIsAlreadyInTheSameState);
    }

    PRESETS.save(
        deps.storage,
        bin_step,
        &PairParameters(preset.0.set_bool(is_open, _OFFSET_IS_PRESET_OPEN)),
    )?;

    Ok(Response::default().add_attribute_plaintext(
        format!("bin step: {}", bin_step),
        format!("is_open: {}", is_open),
    ))
}

/// Remove the preset linked to a bin_step
///
/// # Arguments
///
/// * `bin_step` - The bin step to remove
fn try_remove_preset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bin_step: u16,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    if !PRESETS.has(deps.storage, bin_step) {
        return Err(Error::BinStepHasNoPreset { bin_step });
    }

    PRESETS.remove(deps.storage, bin_step);

    Ok(Response::default().add_attribute_plaintext("preset removed", bin_step.to_string()))
}

/// Function to set the fee parameters of a LBPair
///
/// # Arguments
///
/// * `token_x` - The address of the first token
/// * `token_y` - The address of the second token
/// * `bin_step` - The bin step in basis point, used to calculate the price
/// * `base_factor` - The base factor, used to calculate the base fee, baseFee = baseFactor * binStep
/// * `filter_period` - The period where the accumulator value is untouched, prevent spam
/// * `decay_period` - The period where the accumulator value is decayed, by the reduction factor
/// * `reduction_factor` - The reduction factor, used to calculate the reduction of the accumulator
/// * `variable_fee_control` - The variable fee control, used to control the variable fee, can be 0 to disable it
/// * `protocol_share` - The share of the fees received by the protocol
/// * `max_volatility_accumulator` - The max value of volatility accumulator
fn try_set_fee_parameters_on_pair(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_x: TokenType,
    token_y: TokenType,
    bin_step: u16,
    base_factor: u16,
    filter_period: u16,
    decay_period: u16,
    reduction_factor: u16,
    variable_fee_control: u32,
    protocol_share: u16,
    max_volatility_accumulator: u32,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    let (token_a, token_b) = _sort_tokens(token_x, token_y);
    let mut lb_pair = LB_PAIRS_INFO
        .load(
            deps.storage,
            (
                token_a.unique_key().clone(),
                token_b.unique_key().clone(),
                bin_step,
            ),
        )
        .map_err(|_| Error::LBPairNotCreated {
            token_x: token_a.unique_key(),
            token_y: token_b.unique_key(),
            bin_step,
        })?
        .lb_pair;

    let mut response = Response::new();

    let msg: CosmosMsg = SetStaticFeeParameters {
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    }
    .to_cosmos_msg(
        lb_pair.contract.code_hash,
        lb_pair.contract.address.to_string(),
        None,
    )?;

    response = response.add_message(msg);
    Ok(response)
}

/// Function to set the recipient of the fees. This address needs to be able to receive SNIP20s.
///
/// # Arguments
///
/// * `fee_recipient` - The address of the recipient
fn try_set_fee_recipient(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    fee_recipient: Addr,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    // TODO: Is there way to check that the address exists / is not zero?

    let old_fee_recipient = state.fee_recipient;
    if old_fee_recipient == fee_recipient {
        return Err(Error::SameFeeRecipient {
            fee_recipient: old_fee_recipient.clone(),
        });
    }

    CONFIG.update(deps.storage, |mut state| -> StdResult<_> {
        state.fee_recipient = fee_recipient.clone();
        Ok(state)
    })?;

    Ok(Response::default()
        .add_attribute_plaintext("old fee recipient", old_fee_recipient.as_str())
        .add_attribute_plaintext("new fee recipient", fee_recipient.as_str()))
}

/// Function to set the flash loan fee
///
/// # Arguments
///
/// * `flash_loan_fee` - The value of the fee for flash loan
fn try_set_flash_loan_fee(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    flash_loan_fee: u8,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    let old_flash_loan_fee = state.flash_loan_fee;

    if old_flash_loan_fee == flash_loan_fee {
        return Err(Error::SameFlashLoanFee {
            fee: flash_loan_fee,
        });
    }
    if flash_loan_fee > _MAX_FLASHLOAN_FEE {
        return Err(Error::FlashLoanFeeAboveMax {
            fee: flash_loan_fee,
            max_fee: _MAX_FLASHLOAN_FEE,
        });
    }

    CONFIG.update(deps.storage, |mut state| -> StdResult<_> {
        state.flash_loan_fee = flash_loan_fee;
        Ok(state)
    })?;

    Ok(Response::default()
        .add_attribute_plaintext("old flash loan fee", old_flash_loan_fee.to_string())
        .add_attribute_plaintext("new flash loan fee", flash_loan_fee.to_string()))
}

/// Function to add an asset to the whitelist of quote assets
///
/// # Arguments
///
/// * `quote_asset` - The quote asset (e.g: NATIVE, USDC...)
fn try_add_quote_asset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    quote_asset: TokenType,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    if QUOTE_ASSET_WHITELIST
        .iter(deps.storage)?
        .any(|result| match result {
            Ok(t) => t.eq(&quote_asset),
            Err(_) => false, // Handle the error case as needed
        })
    {
        return Err(Error::QuoteAssetAlreadyWhitelisted {
            quote_asset: quote_asset.unique_key().clone(),
        });
    }

    QUOTE_ASSET_WHITELIST.push(deps.storage, &quote_asset);

    Ok(Response::default()
        .add_attribute_plaintext("quote asset added", quote_asset.unique_key().as_str()))
}

/// Function to remove an asset from the whitelist of quote assets
///
/// # Arguments
///
/// * `quote_asset` - The quote asset (e.g: NATIVE, USDC...)
fn try_remove_quote_asset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: TokenType,
) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    // Enumerate the iterator and use `find` to locate the asset
    let found_asset = QUOTE_ASSET_WHITELIST
        .iter(deps.storage)?
        .enumerate()
        .find(|(_, result)| {
            // Assuming the iterator contains Result, we'll filter only Ok values that match the asset
            result.as_ref().ok().map_or(false, |t| t.eq(&asset))
        });

    match found_asset {
        Some((index, Ok(_))) => {
            // Asset was found at the given index
            QUOTE_ASSET_WHITELIST.remove(deps.storage, index.try_into().unwrap())?;
        }
        _ => {
            // Asset was not found
            return Err(Error::QuoteAssetNotWhitelisted {
                quote_asset: asset.unique_key().clone(),
            });
        }
    }

    Ok(Response::default()
        .add_attribute_plaintext("quote asset removed", asset.unique_key().as_str()))
}

fn try_force_decay(deps: DepsMut, env: Env, info: MessageInfo, pair: LBPair) -> Result<Response> {
    let state = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    // TODO: I think this needs to send a message to the LBPair contract to execute the force decay.
    // pair.forceDecay();
    let (token_a, token_b) = _sort_tokens(pair.token_x, pair.token_y);
    let mut lb_pair = LB_PAIRS_INFO
        .load(
            deps.storage,
            (
                token_a.unique_key().clone(),
                token_b.unique_key().clone(),
                pair.bin_step,
            ),
        )
        .map_err(|_| Error::LBPairNotCreated {
            token_x: token_a.unique_key(),
            token_y: token_b.unique_key(),
            bin_step: pair.bin_step,
        })?
        .lb_pair;

    let mut response = Response::new();

    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lb_pair.contract.address.to_string(),
        code_hash: lb_pair.contract.code_hash,
        msg: to_binary(&LbPairForceDecay {})?,
        funds: vec![],
    }));

    Ok(response)
}

fn only_owner(sender: &Addr, owner: &Addr) -> Result<()> {
    if sender != owner {
        return Err(Error::OnlyOwner);
    }
    Ok(())
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::GetMinBinStep {} => query_min_bin_step(deps),
        QueryMsg::GetFeeRecipient {} => query_fee_recipient(deps),
        QueryMsg::GetMaxFlashLoanFee {} => query_max_flash_loan_fee(deps),
        QueryMsg::GetFlashLoanFee {} => query_flash_loan_fee(deps),
        QueryMsg::GetLBPairImplementation {} => query_lb_pair_implementation(deps),
        QueryMsg::GetLBTokenImplementation {} => query_lb_token_implementation(deps),
        QueryMsg::GetNumberOfLBPairs {} => query_number_of_lb_pairs(deps),
        QueryMsg::GetLBPairAtIndex { index } => query_lb_pair_at_index(deps, index),
        QueryMsg::GetNumberOfQuoteAssets {} => query_number_of_quote_assets(deps),
        QueryMsg::GetQuoteAssetAtIndex { index } => query_quote_asset_at_index(deps, index),
        QueryMsg::IsQuoteAsset { token } => query_is_quote_asset(deps, token),
        QueryMsg::GetLBPairInformation {
            token_x,
            token_y,
            bin_step,
        } => query_lb_pair_information(deps, token_x, token_y, bin_step),
        QueryMsg::GetPreset { bin_step } => query_preset(deps, bin_step),
        QueryMsg::GetAllBinSteps {} => query_all_bin_steps(deps),
        QueryMsg::GetOpenBinSteps {} => query_open_bin_steps(deps),
        QueryMsg::GetAllLBPairs { token_x, token_y } => query_all_lb_pairs(deps, token_x, token_y),
    }
}

/// Returns the minimum bin step a pair can have.
///
/// # Returns
///
/// * `min_bin_step` - The minimum bin step of the pair.
fn query_min_bin_step(deps: Deps) -> Result<Binary> {
    let response = MinBinStepResponse {
        min_bin_step: _MIN_BIN_STEP,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the protocol fee recipient.
///
/// # Returns
///
/// * `fee_recipient` - The address of the fee recipient.
fn query_fee_recipient(deps: Deps) -> Result<Binary> {
    let state = CONFIG.load(deps.storage)?;
    let response = FeeRecipientResponse {
        fee_recipient: state.fee_recipient,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the maximum fee percentage for flash loans.
///
/// # Returns
///
/// * `max_fee` - The maximum fee percentage for flash loans.
fn query_max_flash_loan_fee(deps: Deps) -> Result<Binary> {
    let response = MaxFlashLoanFeeResponse {
        max_fee: _MAX_FLASHLOAN_FEE,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the fee percentage for flash loans.
///
/// # Returns
///
/// * `flash_loan_fee` - The fee percentage for flash loans.
fn query_flash_loan_fee(deps: Deps) -> Result<Binary> {
    let state = CONFIG.load(deps.storage)?;
    let response = FlashLoanFeeResponse {
        flash_loan_fee: state.flash_loan_fee,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the code ID and hash of the LBPair implementation.
///
/// # Returns
///
/// * `lb_pair_implementation` - The code ID and hash of the LBPair implementation.
fn query_lb_pair_implementation(deps: Deps) -> Result<Binary> {
    let state = CONFIG.load(deps.storage)?;
    let response = LBPairImplementationResponse {
        lb_pair_implementation: state.lb_pair_implementation,
    };
    to_binary(&response).map_err(Error::CwErr)
}

// Returns the code ID and hash of the LBToken implementation.
///
/// # Returns
///
/// * `lb_token_implementation` - The code ID and hash of the LBToken implementation.
fn query_lb_token_implementation(deps: Deps) -> Result<Binary> {
    let state = CONFIG.load(deps.storage)?;
    let response = LBTokenImplementationResponse {
        lb_token_implementation: state.lb_token_implementation,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the number of LBPairs created.
///
/// # Returns
///
/// * `lb_pair_number` - The number of LBPairs created.
fn query_number_of_lb_pairs(deps: Deps) -> Result<Binary> {
    let lb_pair_number = ALL_LB_PAIRS.get_len(deps.storage)?;

    let response = NumberOfLBPairsResponse { lb_pair_number };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the LBPair created at index `index`.
///
/// # Arguments
///
/// * `index` - The index of the LBPair.
///
/// # Returns
///
/// * lb_pair - The address of the LBPair at index `index`.
// TODO: Unsure if this function is necessary. Not sure how to index the Keyset.
fn query_lb_pair_at_index(deps: Deps, index: u32) -> Result<Binary> {
    let lb_pair = todo!();

    let response = LBPairAtIndexResponse { lb_pair };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the number of quote assets whitelisted.
///
/// # Returns
///
/// * `number_of_quote_assets` - The number of quote assets.
fn query_number_of_quote_assets(deps: Deps) -> Result<Binary> {
    let number_of_quote_assets = QUOTE_ASSET_WHITELIST.get_len(deps.storage)?;

    let response = NumberOfQuoteAssetsResponse {
        number_of_quote_assets,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the quote asset whitelisted at index `index`.
///
/// # Arguments
///
/// * `index` - The index of the quote asset.
///
/// # Returns
///
/// * `asset` - The address of the quote asset at index `index`.
// TODO: Unsure if this function is necessary. Not sure how to index the Keyset.
fn query_quote_asset_at_index(deps: Deps, index: u32) -> Result<Binary> {
    let asset = QUOTE_ASSET_WHITELIST.get_at(deps.storage, index)?;

    let response = QuoteAssetAtIndexResponse { asset };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns whether a token is a quote asset (true) or not (false).
///
/// # Arguments
///
/// * `token` - The address of the asset.
fn query_is_quote_asset(deps: Deps, token: TokenType) -> Result<Binary> {
    let is_quote = QUOTE_ASSET_WHITELIST
        .iter(deps.storage)?
        .any(|result| match result {
            Ok(t) => t.eq(&token),
            Err(_) => false,
        });

    let response = IsQuoteAssetResponse { is_quote };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the LBPairInformation if it exists, if not, then the address 0 is returned.
///
/// # Arguments
///
/// * `token_a` - The address of the first token of the pair.
/// * `token_b` - The address of the second token of the pair.
/// * `bin_step` - The bin step of the LBPair.
///
/// # Returns
///
/// * `lb_pair_information` - The LBPairInformation.
fn query_lb_pair_information(
    deps: Deps,
    token_a: TokenType,
    token_b: TokenType,
    bin_step: u16,
) -> Result<Binary> {
    let lb_pair_information = _get_lb_pair_information(deps, token_a, token_b, bin_step)?;

    let response = LBPairInformationResponse {
        lb_pair_information,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the LBPairInformation if it exists, if not, then the address 0 is returned. The order doesn't matter
///
/// # Arguments
///
/// * `token_a` - The address of the first token of the pair
/// * `token_b` - The address of the second token of the pair
/// * `bin_step` - The bin step of the LBPair
///
/// # Returns
///
/// * The LBPairInformation
fn _get_lb_pair_information(
    deps: Deps,
    token_a: TokenType,
    token_b: TokenType,
    bin_step: u16,
) -> Result<LBPairInformation> {
    let (token_a, token_b) = _sort_tokens(token_a, token_b);
    let info = LB_PAIRS_INFO
        .load(
            deps.storage,
            (token_a.unique_key(), token_b.unique_key(), bin_step),
        )
        .unwrap();

    Ok(info)
}

/// Function to sort 2 tokens in ascending order.
///
/// # Arguments
///
/// * `token_a` - The first token
/// * `token_b` - The second token
///
/// # Returns
///
/// * The sorted first token
/// * The sorted second token
fn _sort_tokens(token_a: TokenType, token_b: TokenType) -> (TokenType, TokenType) {
    if token_a.unique_key() < token_b.unique_key() {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    }
}

/// Returns the different parameters of the preset.
///
/// # Arguments
///
/// * `bin_step` - The bin step of the preset.
///
/// # Returns
///
/// * `base_factor` - The base factor of the preset.
/// * `filter_period` - The filter period of the preset.
/// * `decay_period` - The decay period of the preset.
/// * `reduction_factor` - The reduction factor of the preset.
/// * `variable_fee_control` - The variable fee control of the preset.
/// * `protocol_share` - The protocol share of the preset.
/// * `max_volatility_accumulator` - The max volatility accumulator of the preset.
/// * `is_open` - Whether the preset is open or not.
fn query_preset(deps: Deps, bin_step: u16) -> Result<Binary> {
    if !PRESETS.has(deps.storage, bin_step) {
        return Err(Error::BinStepHasNoPreset { bin_step });
    }

    // NOTE: each preset is an encoded Bytes32.
    // The EncodedSample wrapper provides methods to decode.
    // The PairParameters wrapper provides methods to decode specific values.
    let preset = PRESETS.load(deps.storage, bin_step).unwrap();

    let base_factor = preset.get_base_factor();
    let filter_period = preset.get_filter_period();
    let decay_period = preset.get_decay_period();
    let reduction_factor = preset.get_reduction_factor();
    let variable_fee_control = preset.get_variable_fee_control();
    let protocol_share = preset.get_protocol_share();
    let max_volatility_accumulator = preset.get_max_volatility_accumulator();

    let is_open = preset.0.decode_bool(_OFFSET_IS_PRESET_OPEN);

    let response = PresetResponse {
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
        is_open,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the list of available bin steps with a preset.
///
/// # Returns
///
/// * `bin_step_with_preset` - The list of bin steps.
fn query_all_bin_steps(deps: Deps) -> Result<Binary> {
    // NOTE: iterating over the keys of the PRESETS Keymap will return all available bin_steps
    // not too confident with this implementation...

    let mut bin_step_with_preset = Vec::<u16>::new();

    let iterator = PRESETS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .into_iter();

    for result in iterator {
        let (bin_step, preset) = result.map_err(Error::CwErr)?;
        bin_step_with_preset.push(bin_step)
    }

    let response = AllBinStepsResponse {
        bin_step_with_preset,
    };
    to_binary(&response).map_err(Error::CwErr)
}

// this does the same thing as `query_all_bin_steps` but returns only the ones where `is_open` is true
/// Returns the list of open bin steps.
///
/// # Returns
///
/// * `open_bin_step` - The list of open bin steps.
fn query_open_bin_steps(deps: Deps) -> Result<Binary> {
    // this way is harder to ready, but maybe more efficient?
    // let open_bin_steps = PRESETS
    //     .map(|result| {
    //         result
    //             .map_err(Error::CwErr)
    //             .map(|(bin_step, preset)| {
    //                 if _is_preset_open(preset.0 .0) {
    //                     Some(bin_step)
    //                 } else {
    //                     None
    //                 }
    //             })
    //             .transpose()
    //     })
    //     .collect::<Result<Vec<u16>>>()?;

    let mut open_bin_steps = Vec::<u16>::new();

    let iterator = PRESETS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .into_iter();

    for result in iterator {
        let (bin_step, preset) = result.map_err(Error::CwErr)?;
        if _is_preset_open(preset.0 .0) {
            open_bin_steps.push(bin_step)
        }
    }

    let response = OpenBinStepsResponse { open_bin_steps };
    to_binary(&response).map_err(Error::CwErr)
}

fn _is_preset_open(preset: Bytes32) -> bool {
    return EncodedSample(preset).decode_bool(_OFFSET_IS_PRESET_OPEN);
}

/// Returns all the LBPair of a pair of tokens.
///
/// # Arguments
///
/// * `token_x` - The first token of the pair.
/// * `token_y` - The second token of the pair.
///
/// # Returns
///
/// * `lb_pairs_available` - The list of available LBPairs.
fn query_all_lb_pairs(deps: Deps, token_x: TokenType, token_y: TokenType) -> Result<Binary> {
    let (token_a, token_b) = _sort_tokens(token_x, token_y);

    // Create a Vec of available bin steps for this pair
    let bin_steps: Vec<u16> = AVAILABLE_LB_PAIR_BIN_STEPS
        .load(
            deps.storage,
            (token_a.unique_key().clone(), token_b.unique_key().clone()),
        )
        .map_err(|_| Error::Generic("This token pair is not in the map".to_string()))?;

    // Not sure if this condition is possible, but just in case.
    if bin_steps.len() == 0 {
        return Err(Error::Generic("No available bin_steps".to_string()));
    }

    // Collect LBPairInformation values into a vector
    let lb_pairs_available: Result<Vec<LBPairInformation>> = bin_steps
        .into_iter()
        .map(|bin_step| {
            LB_PAIRS_INFO
                .load(
                    deps.storage,
                    (
                        token_a.unique_key().clone(),
                        token_b.unique_key().clone(),
                        bin_step,
                    ),
                )
                .map_err(|_| Error::Generic("Error retrieving LBPairInformation".to_string()))
        })
        .collect();

    let response = AllLBPairsResponse {
        lb_pairs_available: lb_pairs_available?,
    };
    to_binary(&response).map_err(Error::CwErr)
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match (msg.id, msg.result) {
        (INSTANTIATE_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                let contract_address = deps.api.addr_validate(&String::from_utf8(x.to_vec())?)?;
                let lb_pair_key = ephemeral_storage_r(deps.storage).load()?;

                let token_a = lb_pair_key.token_a;
                let token_b = lb_pair_key.token_b;
                let bin_step = lb_pair_key.bin_step;
                let code_hash = lb_pair_key.code_hash;

                let lb_pair = LBPair {
                    token_x: token_a.clone(),
                    token_y: token_b.clone(),
                    bin_step,
                    contract: ContractInfo {
                        address: contract_address,
                        code_hash,
                    },
                };
                LB_PAIRS_INFO.save(
                    deps.storage,
                    (
                        token_a.unique_key().clone(),
                        token_b.unique_key().clone(),
                        bin_step,
                    ),
                    &LBPairInformation {
                        bin_step: lb_pair_key.bin_step,
                        lb_pair: lb_pair.clone(),
                        // TODO: get 'is_owner' from the create_lb_pair function
                        created_by_owner: lb_pair_key.is_open,
                        ignored_for_routing: false,
                    },
                )?;

                ALL_LB_PAIRS.push(deps.storage, &lb_pair)?;

                // load the different bin_step LBPairs that exist for this pair of tokens, then add the new one
                let mut bin_step_list = AVAILABLE_LB_PAIR_BIN_STEPS
                    .load(
                        deps.storage,
                        (token_a.unique_key().clone(), token_b.unique_key().clone()),
                    )
                    .unwrap_or(Vec::<u16>::new());
                bin_step_list.push(bin_step);
                AVAILABLE_LB_PAIR_BIN_STEPS.save(
                    deps.storage,
                    (token_a.unique_key(), token_b.unique_key()),
                    &bin_step_list,
                )?;

                ephemeral_storage_w(deps.storage).remove();
                Ok(Response::default())
            }
            None => Err(StdError::generic_err(format!("Expecting contract id"))),
        },
        _ => Err(StdError::generic_err(format!("Unknown reply id"))),
    }
}
