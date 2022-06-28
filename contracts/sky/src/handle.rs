use crate::query::{conversion_mint_profitability, cycle_profitability};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{
    to_binary,
    Api,
    Binary,
    CosmosMsg,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage,
    WasmMsg,
};
use fadroma::scrt::to_cosmos_msg;
use secret_toolkit::{
    snip20::{send_msg, set_viewing_key_msg},
    utils::Query,
};
use shade_protocol::{
    contract_interfaces::{
        dex::{self, shadeswap::SwapTokens},
        mint::mint::{self, HandleMsg::Receive, QueryAnswer, QueryAnswer::Mint, QueryMsg},
        sky::sky::{self, Config, Cycle, Cycles, HandleAnswer, ViewingKeys},
        snip20::helpers::Snip20Asset,
    },
    utils::{asset::Contract, storage::plus::ItemStorage},
};

/// ## Markdown

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
) -> StdResult<HandleResponse> {
    if env.message.sender != Config::load(&deps.storage)?.admin {
        return Err(StdError::unauthorized());
    }
    config.save(&mut deps.storage)?;
    let view_key = ViewingKeys::load(&deps.storage)?.0;
    let mut messages = vec![
        set_viewing_key_msg(
            view_key.clone(),
            None,
            1,
            config.shd_token_contract.code_hash.clone(),
            config.shd_token_contract.address.clone(),
        )?,
        set_viewing_key_msg(
            view_key.clone(),
            None,
            1,
            config.silk_token_contract.code_hash.clone(),
            config.silk_token_contract.address.clone(),
        )?,
    ];
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig { status: true })?),
    })
}

pub fn try_set_cycles<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cycles_to_set: Vec<Cycle>,
) -> StdResult<HandleResponse> {
    if env.message.sender != Config::load(&deps.storage)?.admin {
        return Err(StdError::unauthorized());
    }

    let new_cycles = Cycles(cycles_to_set);
    new_cycles.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetCycles { status: true })?),
    })
}

pub fn try_append_cycle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cycles_to_add: Vec<Cycle>,
) -> StdResult<HandleResponse> {
    if env.message.sender != Config::load(&deps.storage)?.admin {
        return Err(StdError::unauthorized());
    }

    let mut cycles = Cycles::load(&deps.storage)?;

    cycles.0.append(&mut cycles_to_add.clone());

    cycles.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AppendCycles { status: true })?),
    })
}

pub fn try_execute<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config: Config = Config::load(&deps.storage)?;

    let res = conversion_mint_profitability(deps, amount)?;

    let mut profitable = false;
    let mut is_mint_first = false;
    let mut first_swap_expected = Uint128::zero();
    match res {
        sky::QueryAnswer::ArbPegProfitability {
            is_profitable,
            mint_first,
            first_swap_result,
        } => {
            profitable = is_profitable;
            is_mint_first = mint_first;
            first_swap_expected = first_swap_result;
        }
        _ => {}
    }

    if !profitable {
        return Err(StdError::GenericErr {
            msg: String::from("Trade not profitable"),
            backtrace: None,
        });
    }

    let mut messages = vec![];

    if is_mint_first {
        messages.push(send_msg(
            config.mint_contract_silk.address,
            cosmwasm_std::Uint128(amount.clone().u128()),
            Some(to_binary(&mint::MintMsgHook {
                minimum_expected_amount: Uint128::zero(),
            })?),
            None,
            None,
            256,
            config.shd_token_contract.code_hash,
            config.shd_token_contract.address,
        )?);

        messages.push(send_msg(
            config.market_swap_contract.address.clone(),
            cosmwasm_std::Uint128(first_swap_expected.clone().u128()),
            Some(to_binary(&SwapTokens {
                expected_return: Some(amount.clone()),
                to: None,
                router_link: None,
                callback_signature: None,
            })?),
            None,
            None,
            256,
            config.silk_token_contract.code_hash.clone(),
            config.silk_token_contract.address.clone(),
        )?);
    } else {
        messages.push(send_msg(
            config.market_swap_contract.address.clone(),
            cosmwasm_std::Uint128(amount.u128()),
            Some(to_binary(&SwapTokens {
                expected_return: Some(Uint128::zero()),
                to: None,
                router_link: None,
                callback_signature: None,
            })?),
            None,
            None,
            256,
            config.shd_token_contract.code_hash.clone(),
            config.shd_token_contract.address.clone(),
        )?);

        messages.push(send_msg(
            config.mint_contract_shd.address.clone(),
            cosmwasm_std::Uint128(first_swap_expected.clone().u128()),
            Some(to_binary(&mint::MintMsgHook {
                minimum_expected_amount: amount.clone(),
            })?),
            None,
            None,
            256,
            config.silk_token_contract.code_hash.clone(),
            config.silk_token_contract.address.clone(),
        )?);
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExecuteArb { status: true })?),
    })
}

pub fn try_arb_cycle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    index: Uint128,
) -> StdResult<HandleResponse> {
    let mut messages = vec![];
    let cycles = Cycles::load(&deps.storage)?.0;

    let res = cycle_profitability(deps, amount, index)?;
    match res {
        sky::QueryAnswer::IsCycleProfitable {
            is_profitable,
            direction,
            swap_amounts,
        } => {
            let mut cur_asset = Contract {
                address: direction.start_addr.clone(),
                code_hash: "".to_string(),
            };
            if direction.pair_addrs[0]
                .token0_contract
                .address
                .eq(&direction.start_addr.clone())
            {
                cur_asset = direction.pair_addrs[0].token0_contract.clone();
            } else {
                cur_asset = direction.pair_addrs[0].token1_contract.clone();
            }
            if !is_profitable {
                return Err(StdError::GenericErr {
                    msg: "bad".to_string(),
                    backtrace: None,
                });
            }
            for (i, arb_pair) in direction.pair_addrs.clone().iter().enumerate() {
                let mut msg;
                if arb_pair.eq(&direction.pair_addrs[direction.pair_addrs.len() - 1]) {
                    msg = Some(to_binary(&SwapTokens {
                        expected_return: Some(amount),
                        to: None,
                        router_link: None,
                        callback_signature: None,
                    })?);
                } else {
                    msg = Some(to_binary(&SwapTokens {
                        expected_return: Some(Uint128::zero()),
                        to: None,
                        router_link: None,
                        callback_signature: None,
                    })?);
                }
                messages.push(send_msg(
                    arb_pair.pair_contract.address.clone(),
                    cosmwasm_std::Uint128::from(swap_amounts[i].u128()),
                    msg,
                    None,
                    None,
                    256,
                    cur_asset.code_hash.clone(),
                    cur_asset.address.clone(),
                )?);
                if cur_asset.eq(&arb_pair.token0_contract.clone()) {
                    cur_asset = arb_pair.token1_contract.clone();
                } else {
                    cur_asset = arb_pair.token0_contract.clone();
                }
            }
        }
        _ => {}
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExecuteArbCycle { status: true })?),
    })
}

pub fn try_arb_all_cycles<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let mut messages = vec![];
    let cycles = Cycles::load(&deps.storage)?.0;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig { status: true })?),
    })
}

pub fn constant_product(
    swap_amount: Uint128,
    pool_buy: Uint128,
    pool_sell: Uint128,
) -> StdResult<Uint128> {
    //let cp = pool_buy.u128().clone() * pool_sell.u128().clone();
    //let lpb = pool_sell.u128().clone() + swap_amount.u128().clone();
    //let ncp = div(Uint128::new(cp.clone()), Uint128::new(lpb.clone()))?;
    //let result = pool_buy.u128().clone() - ncp.u128().clone();
    let cp = pool_buy.checked_mul(pool_sell)?;
    let lpb = pool_sell.checked_add(swap_amount)?;
    let ncp = cp.checked_div(lpb)?;
    let result = pool_buy.checked_sub(ncp)?;

    Ok(result)
}
