use crate::query::cycle_profitability;
use shade_admin::admin::{self, ValidateAdminPermissionResponse};
use shade_protocol::{
    c_std::{
        self,
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
    },
    contract_interfaces::{
        dao::adapter,
        sky::{self, Config, Cycle, Cycles, HandleAnswer, Offer, SelfAddr, ViewingKeys},
    },
    math_compat::{Decimal, Uint128},
    secret_toolkit::{
        snip20::{send_msg, set_viewing_key_msg},
        utils::Query,
    },
    utils::{asset::Contract, generic_response::ResponseStatus, storage::plus::ItemStorage},
};

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    shade_admin: Option<Contract>,
    shd_token_contract: Option<Contract>,
    silk_token_contract: Option<Contract>,
    sscrt_token_contract: Option<Contract>,
    treasury: Option<Contract>,
    payback_rate: Option<Decimal>,
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
    if let Some(sscrt_token_contract) = sscrt_token_contract {
        config.sscrt_token_contract = sscrt_token_contract;
        messages.push(set_viewing_key_msg(
            ViewingKeys::load(&deps.storage)?.0,
            None,
            1,
            config.sscrt_token_contract.code_hash.clone(),
            config.sscrt_token_contract.address.clone(),
        )?);
    }
    if let Some(treasury) = treasury {
        config.treasury = treasury;
    }
    if let Some(payback_rate) = payback_rate {
        config.payback_rate = payback_rate;
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

    for cycle in cycles_to_set.clone() {
        cycle.validate_cycle()?;
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

    for cycle in cycles_to_add.clone() {
        cycle.validate_cycle()?;
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

pub fn try_update_cycle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cycle: Cycle,
    index: Uint128,
) -> StdResult<HandleResponse> {
    // Admin-only
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

    cycle.validate_cycle()?;
    let mut cycles = Cycles::load(&deps.storage)?;
    cycles.0[index.u128() as usize] = cycle;
    cycles.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateCycle { status: true })?),
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

pub fn try_arb_cycle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
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
    let mut payback_amount = Uint128::zero();
    let res = cycle_profitability(deps, amount, index)?; // get profitability data from query
    match res {
        sky::QueryAnswer::IsCycleProfitable {
            is_profitable,
            direction,
            swap_amounts,
            profit,
        } => {
            return_swap_amounts = swap_amounts.clone();
            if direction.pair_addrs[0] // test to see which of the token attributes are the proposed starting addr
                .token0_contract
                == direction.start_addr.clone()
            {
                cur_asset = direction.pair_addrs[0].token0_contract.clone();
            } else {
                cur_asset = direction.pair_addrs[0].token1_contract.clone();
            }
            // if tx is unprofitable, err out
            if !is_profitable {
                return Err(StdError::generic_err("Unprofitable"));
            }
            //loop through the pairs in the cycle
            for (i, arb_pair) in direction.pair_addrs.clone().iter().enumerate() {
                if direction.pair_addrs.len() == i {
                    messages.push(arb_pair.to_cosmos_msg(
                        Offer {
                            asset: cur_asset.clone(),
                            amount: swap_amounts[i],
                        },
                        amount,
                    )?);
                } else {
                    messages.push(arb_pair.to_cosmos_msg(
                        Offer {
                            asset: cur_asset.clone(),
                            amount: swap_amounts[i],
                        },
                        Uint128::zero(),
                    )?);
                }
                // reset cur asset to the other asset held in the struct
                if cur_asset == arb_pair.token0_contract.clone() {
                    cur_asset = arb_pair.token1_contract.clone();
                } else {
                    cur_asset = arb_pair.token0_contract.clone();
                }
            }
            // calculate payback amount
            let payback_percent = Config::load(&deps.storage)?.payback_rate;
            if payback_percent > Decimal::zero() {
                payback_amount = profit * payback_percent;
            }
        }
        _ => {}
    }

    // the final cur_asset should be the same as the start_addr
    assert!(cur_asset.clone() == Cycles::load(&deps.storage)?.0[index.u128() as usize].start_addr);
    messages.push(send_msg(
        env.message.sender,
        c_std::Uint128(payback_amount.u128()),
        None,
        None,
        None,
        1,
        cur_asset.code_hash.clone(),
        cur_asset.address.clone(),
    )?);

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExecuteArbCycle {
            status: true,
            swap_amounts: return_swap_amounts,
            payback_amount,
        })?),
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
        return Err(StdError::unauthorized());
    }
    if !(config.shd_token_contract.address == asset
        || config.silk_token_contract.address == asset
        || config.sscrt_token_contract.address == asset)
    {
        return Err(StdError::GenericErr {
            msg: String::from("Unrecognized asset"),
            backtrace: None,
        });
    }
    let contract;
    if config.shd_token_contract.address == asset {
        contract = config.shd_token_contract;
    } else if config.silk_token_contract.address == asset {
        contract = config.silk_token_contract;
    } else {
        contract = config.sscrt_token_contract;
    }
    let messages = vec![send_msg(
        config.treasury.address,
        c_std::Uint128::from(amount.u128()),
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
            amount: c_std::Uint128::from(amount.u128()),
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
            amount: c_std::Uint128::zero(),
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
