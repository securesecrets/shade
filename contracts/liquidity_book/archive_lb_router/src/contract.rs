#![allow(unused)] // For beginning only.

use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Reply, Response, StdError, StdResult, SubMsg, SubMsgResult, Timestamp, Uint256, WasmMsg,
};
use cosmwasm_std::{ContractInfo, Uint128};
use ethnum::U256;

use libraries::bin_helper::BinHelper;
use libraries::math::encoded_sample::EncodedSample;
use libraries::math::packed_u128_math::Decode;
use libraries::math::u24::U24;
use libraries::tokens::TokenType;
use libraries::types::{Bytes32, LBPairInformation, LiquidityConfigurations};

use crate::msg::*;
use crate::operations::swap_tokens_for_exact_tokens;
use crate::prelude::*;
use crate::state::*;

/////////////// INSTANTIATE ///////////////
const BLOCK_SIZE: usize = 256;
pub const SHADE_ROUTER_KEY: &str = "SHADE_ROUTER_KEY";
pub const MINT_REPLY_ID: u64 = 1u64;
pub const SWAP_REPLY_ID: u64 = 1u64;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // TODO: Only the factory should be allowed to instantiate this contract
    // I think you can restrict that on code upload
    let mut config = Config {
        factory: msg.factory,
        admins: Vec::new(),
        viewing_key: SHADE_ROUTER_KEY.to_string(),
    };

    if let Some(admins) = msg.admins {
        for admin in admins {
            config
                .admins
                .push(deps.api.addr_canonicalize(admin.as_str())?);
        }
    } else {
        config
            .admins
            .push(deps.api.addr_canonicalize(info.sender.as_str())?);
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

/////////////// EXECUTE ///////////////

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response> {
    match msg {
        ExecuteMsg::CreateLBPair {
            token_x,
            token_y,
            active_id,
            bin_step,
        } => create_lb_pair(deps, env, token_x, token_y, active_id, bin_step),
        ExecuteMsg::SwapTokensForExact {
            offer,
            expected_return,
            path,
            recipient,
        } => {
            if !offer.token.is_native_token() {
                return Err(Error::NonNativeTokenErr);
            }
            offer.assert_sent_native_token_balance(&info)?;
            let sender = info.sender.clone();
            let checked_address = match recipient {
                Some(x) => Some(deps.api.addr_validate(&x)?),
                None => None,
            };
            let response = Response::new();
            Ok(swap_tokens_for_exact_tokens(
                deps,
                env,
                offer,
                expected_return,
                &path,
                sender,
                checked_address,
                response,
            )?)
        }
        _ => todo!(),
    }
}

// #[entry_point]
// pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response> {
//     match (msg.id, msg.result) {
//         (MINT_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
//             Some(x) => {
//                 let data: interfaces::ILBPair::MintResponse = from_binary(&x)?;

//                 let amount_x_added = Uint128::from(data.amounts_received.decode_x());
//                 let amount_y_added = Uint128::from(data.amounts_received.decode_y());

//                 let amount_x_left = Uint128::from(data.amounts_left.decode_x());
//                 let amount_y_left = Uint128::from(data.amounts_left.decode_y());
//                 let deposit_ids = serde_json_wasm::to_string(&data.deposit_ids);
//                 let liquidity_minted = serde_json_wasm::to_string(&data.liquidity_minted);

//                 Ok(Response::new()
//                     .add_attribute("amount_x_added", amount_x_added)
//                     .add_attribute("amount_y_added", amount_y_added)
//                     .add_attribute("amount_x_left", amount_x_left)
//                     .add_attribute("amount_y_left", amount_y_left)
//                     .add_attribute("liquidity_minted", liquidity_minted.unwrap())
//                     .add_attribute("deposit_ids", deposit_ids.unwrap()))
//             }
//             None => Err(Error::UnknownReplyId { id: msg.id }),
//         },
//         _ => Err(Error::UnknownReplyId { id: msg.id }),
//     }
// }

// fn complete_liquidity(deps:DepsMut, env:Env,)

pub fn create_lb_pair(
    deps: DepsMut,
    env: Env,
    token_x: TokenType,
    token_y: TokenType,
    active_id: u32,
    bin_step: u16,
) -> Result<Response> {
    let config = CONFIG.load(deps.storage)?;
    let factory = config.factory;

    let msg = interfaces::ILBFactory::ExecuteMsg::CreateLBPair {
        token_x,
        token_y,
        active_id,
        bin_step,
    };

    let msg: CosmosMsg = msg.to_cosmos_msg(factory.code_hash, factory.address.to_string(), None)?;

    Ok(Response::new().add_message(msg))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::GetFactory {} => {
            to_binary(&query_factory(deps)?).map_err(|err| Error::CwErr(err))
        }
        QueryMsg::GetIdFromPrice { lb_pair, price } => {
            to_binary(&query_id_from_price(deps, lb_pair, price)?).map_err(|err| Error::CwErr(err))
        }
        QueryMsg::GetPriceFromId { lb_pair, id } => {
            to_binary(&query_price_from_id(deps, lb_pair, id)?).map_err(|err| Error::CwErr(err))
        }
        QueryMsg::GetSwapIn {
            lb_pair,
            amount_out,
            swap_for_y,
        } => to_binary(&query_swap_in(deps, lb_pair, amount_out, swap_for_y)?)
            .map_err(|err| Error::CwErr(err)),
        QueryMsg::GetSwapOut {
            lb_pair,
            amount_in,
            swap_for_y,
        } => to_binary(&query_swap_out(deps, lb_pair, amount_in, swap_for_y)?)
            .map_err(|err| Error::CwErr(err)),
    }
}

fn query_factory(deps: Deps) -> Result<FactoryResponse> {
    let state = CONFIG.load(deps.storage)?;
    Ok(FactoryResponse {
        factory: state.factory.address,
    })
}

fn query_id_from_price(
    deps: Deps,
    lb_pair: ContractInfo,
    price: Uint256,
) -> Result<IdFromPriceResponse> {
    let msg = interfaces::ILBPair::QueryMsg::GetIdFromPrice { price };
    let interfaces::ILBPair::IdFromPriceResponse { id } =
        deps.querier
            .query_wasm_smart::<interfaces::ILBPair::IdFromPriceResponse>(
                lb_pair.code_hash,
                lb_pair.address.to_string(),
                &(&msg),
            )?;

    Ok(IdFromPriceResponse { id })
}

fn query_price_from_id(deps: Deps, lb_pair: ContractInfo, id: u32) -> Result<PriceFromIdResponse> {
    let msg = interfaces::ILBPair::QueryMsg::GetPriceFromId { id };
    let interfaces::ILBPair::PriceFromIdResponse { price } =
        deps.querier
            .query_wasm_smart::<interfaces::ILBPair::PriceFromIdResponse>(
                lb_pair.code_hash,
                lb_pair.address.to_string(),
                &(&msg),
            )?;

    Ok(PriceFromIdResponse { price })
}

fn query_swap_in(
    deps: Deps,
    lb_pair: ContractInfo,
    amount_out: Uint128,
    swap_for_y: bool,
) -> Result<SwapInResponse> {
    let msg = interfaces::ILBPair::QueryMsg::GetSwapIn {
        amount_out,
        swap_for_y,
    };
    let interfaces::ILBPair::SwapInResponse {
        amount_in,
        amount_out_left,
        fee,
    } = deps
        .querier
        .query_wasm_smart::<interfaces::ILBPair::SwapInResponse>(
            lb_pair.code_hash,
            lb_pair.address.to_string(),
            &(&msg),
        )?;

    Ok(SwapInResponse {
        amount_in,
        amount_out_left,
        fee,
    })
}

fn query_swap_out(
    deps: Deps,
    lb_pair: ContractInfo,
    amount_in: Uint128,
    swap_for_y: bool,
) -> Result<SwapOutResponse> {
    let msg = interfaces::ILBPair::QueryMsg::GetSwapOut {
        amount_in,
        swap_for_y,
    };
    let interfaces::ILBPair::SwapOutResponse {
        amount_in_left,
        amount_out,
        fee,
    } = deps
        .querier
        .query_wasm_smart::<interfaces::ILBPair::SwapOutResponse>(
            lb_pair.code_hash,
            lb_pair.address.to_string(),
            &(&msg),
        )?;

    Ok(SwapOutResponse {
        amount_in_left,
        amount_out,
        fee,
    })
}
