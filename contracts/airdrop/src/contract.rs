use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage, Uint128};
use shade_protocol::{
    airdrop::{
        InitMsg, HandleMsg,
        QueryMsg, Config
    }
};
use crate::state::{config_w, sn_delegators_w};
use shade_protocol::airdrop::StoredDelegator;
use crate::handle::try_update_config;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config{
        owner: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        airdrop_snip20: msg.airdrop_snip20,
        prefered_validator: msg.prefered_validator,
        start_date: match msg.start_date {
            None => env.block.time,
            Some(date) => date
        },
        end_date: msg.end_date
    };

    config_w(&mut deps.storage).save(&config)?;

    let mut delegators = sn_delegators_w(&mut deps.storage);
    // Store the delegators list
    for delegator in msg.sn_snapshot {
        delegators.save(delegator.address.to_string().as_bytes(), &StoredDelegator{
            address: delegator.address,
            delegations: delegator.delegations,
            redeemed: false
        });
    }

    Ok(InitResponse {
        messages: vec![],
        log: vec![]
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateConfig {
            admin, airdrop_snip20, prefered_validator,
            start_date, end_date
        } => try_update_config(deps, env, admin, airdrop_snip20, prefered_validator,
                               start_date, end_date),
        HandleMsg::Redeem { } => try_redeem(deps, env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDates { } =>
        QueryMsg::GetEligibility { address } =>
    }
}