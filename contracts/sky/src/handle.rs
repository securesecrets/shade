use crate::query::{conversion_mint_profitability, cycle_profitability};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{
    to_binary,
    Api,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use secret_toolkit::{
    snip20::{send_msg, set_viewing_key_msg},
    utils::Query,
};
use shade_admin::admin::{self, ValidateAdminPermissionResponse};
use shade_protocol::{
    contract_interfaces::{
        dao::adapter,
        dex::shadeswap::SwapTokens,
        mint::mint,
        sky::{self, Config, Cycle, Cycles, HandleAnswer, Minted, SelfAddr, ViewingKeys},
    },
    utils::{asset::Contract, generic_response::ResponseStatus, storage::plus::ItemStorage},
};

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    shade_admin: Option<Contract>,
    mint_contract_shd: Option<Contract>,
    mint_contract_silk: Option<Contract>,
    market_swap_contract: Option<Contract>,
    shd_token_contract: Option<Contract>,
    silk_token_contract: Option<Contract>,
    treasury: Option<Contract>,
) -> StdResult<HandleResponse> {
    //Admin-only
    let mut config = Config::load(&mut deps.storage)?;
    let admin_response: ValidateAdminPermissionResponse =
        admin::QueryMsg::ValidateAdminPermission {
            contract_address: SelfAddr::load(&mut deps.storage)?.0.to_string(),
            admin_address: env.message.sender.to_string(),
        }
        .query(
            &deps.querier,
            config.shade_admin.code_hash.clone(),
            config.shade_admin.address.clone(),
        )?;

    if admin_response.error_msg.is_some() {
        return Err(StdError::unauthorized());
    }

    let mut messages = vec![];

    if let Some(shade_admin) = shade_admin {
        config.shade_admin = shade_admin;
    }
    if let Some(mint_contract_shd) = mint_contract_shd {
        config.mint_contract_shd = mint_contract_shd;
    }
    if let Some(mint_contract_silk) = mint_contract_silk {
        config.mint_contract_silk = mint_contract_silk;
    }
    if let Some(market_swap_contract) = market_swap_contract {
        config.market_swap_contract = market_swap_contract;
    }
    if let Some(shd_token_contract) = shd_token_contract {
        config.shd_token_contract = shd_token_contract;
        messages.push(set_viewing_key_msg(
            ViewingKeys::load(&deps.storage)?.0,
            None,
            1,
            config.shd_token_contract.code_hash.clone(),
            config.shd_token_contract.address.clone(),
        )?);
    }
    if let Some(silk_token_contract) = silk_token_contract {
        config.silk_token_contract = silk_token_contract;
        messages.push(set_viewing_key_msg(
            ViewingKeys::load(&deps.storage)?.0,
            None,
            1,
            config.silk_token_contract.code_hash.clone(),
            config.silk_token_contract.address.clone(),
        )?);
    }
    if let Some(treasury) = treasury {
        config.treasury = treasury;
    }
    config.save(&mut deps.storage)?;
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
    //Admin-only
    let shade_admin = Config::load(&mut deps.storage)?.shade_admin;
    let admin_response: ValidateAdminPermissionResponse =
        admin::QueryMsg::ValidateAdminPermission {
            contract_address: SelfAddr::load(&mut deps.storage)?.0.to_string(),
            admin_address: env.message.sender.to_string(),
        }
        .query(&deps.querier, shade_admin.code_hash, shade_admin.address)?;

    if admin_response.error_msg.is_some() {
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
    //Admin-only
    let shade_admin = Config::load(&mut deps.storage)?.shade_admin;
    let admin_response: ValidateAdminPermissionResponse =
        admin::QueryMsg::ValidateAdminPermission {
            contract_address: SelfAddr::load(&mut deps.storage)?.0.to_string(),
            admin_address: env.message.sender.to_string(),
        }
        .query(&deps.querier, shade_admin.code_hash, shade_admin.address)?;

    if admin_response.error_msg.is_some() {
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

pub fn try_remove_cycle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    index: Uint128,
) -> StdResult<HandleResponse> {
    //Admin-only
    let shade_admin = Config::load(&mut deps.storage)?.shade_admin;
    let admin_response: ValidateAdminPermissionResponse =
        admin::QueryMsg::ValidateAdminPermission {
            contract_address: SelfAddr::load(&mut deps.storage)?.0.to_string(),
            admin_address: env.message.sender.to_string(),
        }
        .query(&deps.querier, shade_admin.code_hash, shade_admin.address)?;

    if admin_response.error_msg.is_some() {
        return Err(StdError::unauthorized());
    }

    // I'm pissed I couldn't do this in one line
    let mut cycles = Cycles::load(&deps.storage)?.0;
    cycles.remove(index.u128() as usize);
    Cycles(cycles).save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveCycle { status: true })?),
    })
}

pub fn try_execute<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config: Config = Config::load(&deps.storage)?;

    //grab profitability data from query
    let res = conversion_mint_profitability(deps, amount)?;

    let mut profitable = false;
    let mut is_mint_first = false;
    let mut first_swap_expected = Uint128::zero();
    let mut profit = Uint128::zero();
    match res {
        sky::QueryAnswer::ArbPegProfitability {
            is_profitable,
            mint_first,
            first_swap_result,
            profit: new_profit,
        } => {
            profitable = is_profitable;
            is_mint_first = mint_first;
            first_swap_expected = first_swap_result;
            profit = new_profit;
        }
        _ => {}
    }

    //if tx is not profitable, err out
    if !profitable {
        return Err(StdError::GenericErr {
            msg: String::from("Trade not profitable"),
            backtrace: None,
        });
    }

    let mut messages = vec![];
    let mut minted = Minted::load(&deps.storage)?;

    if is_mint_first {
        //if true mint silk from shd then sell the silk on the market
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

        minted.1 = minted.1.clone().checked_add(first_swap_expected)?;

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
        // if false, buy silk with shd then mint shd with the silk
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

        minted.0 = minted.0.clone().checked_add(amount.checked_add(profit)?)?;
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExecuteArb {
            status: true,
            amount,
            after_first_swap: first_swap_expected,
            final_amount: amount.checked_add(profit)?,
        })?),
    })
}

pub fn try_arb_cycle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    amount: Uint128,
    index: Uint128,
) -> StdResult<HandleResponse> {
    let mut messages = vec![];
    let mut return_swap_amounts = vec![];
    // cur_asset will keep track of the asset that we have swapped into
    let mut cur_asset = Contract {
        address: HumanAddr::default(),
        code_hash: "".to_string(),
    };
    let res = cycle_profitability(deps, amount, index)?; // get profitability data from query
    match res {
        sky::QueryAnswer::IsCycleProfitable {
            is_profitable,
            direction,
            swap_amounts,
            ..
        } => {
            return_swap_amounts = swap_amounts.clone();
            if direction.pair_addrs[0] // test to see which of the token attributes are the proposed starting addr
                .token0_contract
                .address
                == direction.start_addr.clone()
            {
                cur_asset = direction.pair_addrs[0].token0_contract.clone();
            } else {
                cur_asset = direction.pair_addrs[0].token1_contract.clone();
            }
            // if tx is unprofitable, err out
            if !is_profitable {
                return Err(StdError::GenericErr {
                    msg: "bad".to_string(),
                    backtrace: None,
                });
            }
            //loop through the pairs in the cycle
            for (i, arb_pair) in direction.pair_addrs.clone().iter().enumerate() {
                let msg;
                // if we're on the last iteration, we set our
                // minmum expected return to the swap amount to ensure the tx is profitable
                if direction.pair_addrs.len() == i {
                    msg = Some(to_binary(&SwapTokens {
                        expected_return: Some(amount),
                        to: None,
                        router_link: None,
                        callback_signature: None,
                    })?);
                } else {
                    // otherwise set expected return to zero bc we only care about the final trade
                    msg = Some(to_binary(&SwapTokens {
                        expected_return: Some(Uint128::zero()),
                        to: None,
                        router_link: None,
                        callback_signature: None,
                    })?);
                }
                //push each msg
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
                // reset cur asset to the other asset held in the struct
                if cur_asset == arb_pair.token0_contract.clone() {
                    cur_asset = arb_pair.token1_contract.clone();
                } else {
                    cur_asset = arb_pair.token0_contract.clone();
                }
            }
        }
        _ => {}
    }

    // the final cur_asset should be the same as the start_addr
    assert!(
        cur_asset.address.clone()
            == Cycles::load(&deps.storage)?.0[index.u128() as usize].start_addr
    );

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExecuteArbCycle {
            status: true,
            swap_amounts: return_swap_amounts,
        })?),
    })
}

pub fn try_minted_reset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    shd: Option<Uint128>,
    silk: Option<Uint128>,
) -> StdResult<HandleResponse> {
    //Admin-only
    let shade_admin = Config::load(&mut deps.storage)?.shade_admin;
    let admin_response: ValidateAdminPermissionResponse =
        admin::QueryMsg::ValidateAdminPermission {
            contract_address: SelfAddr::load(&mut deps.storage)?.0.to_string(),
            admin_address: env.message.sender.to_string(),
        }
        .query(&deps.querier, shade_admin.code_hash, shade_admin.address)?;

    if admin_response.error_msg.is_some() {
        return Err(StdError::unauthorized());
    }

    let mut minted = Minted::load(&mut deps.storage)?;

    if let Some(shd) = shd {
        minted.0 = shd;
    } else {
        minted.0 = Uint128::zero();
    }
    if let Some(silk) = silk {
        minted.1 = silk;
    } else {
        minted.0 = Uint128::zero();
    }
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::MintedReset { status: true })?),
    })
}

pub fn try_adapter_unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config = Config::load(&deps.storage)?;
    if !(env.message.sender == config.treasury.address) {
        return Err(StdError::Unauthorized { backtrace: None });
    }
    if !(config.shd_token_contract.address == asset)
        && !(config.silk_token_contract.address == asset)
    {
        return Err(StdError::GenericErr {
            msg: String::from("Unrecognized asset"),
            backtrace: None,
        });
    }
    let contract;
    if config.shd_token_contract.address == asset {
        contract = config.shd_token_contract;
    } else {
        contract = config.silk_token_contract;
    }
    let messages = vec![send_msg(
        config.treasury.address,
        cosmwasm_std::Uint128::from(amount.u128()),
        None,
        None,
        None,
        256,
        contract.code_hash,
        contract.address,
    )?];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: cosmwasm_std::Uint128::from(amount.u128()),
        })?),
    })
}

pub fn try_adapter_claim<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _asset: HumanAddr,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount: cosmwasm_std::Uint128::zero(),
        })?),
    })
}

pub fn try_adapter_update<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _asset: HumanAddr,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?),
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
