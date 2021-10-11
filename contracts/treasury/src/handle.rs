use cosmwasm_std::{
    debug_print, to_binary, Api, Binary,
    Env, Extern, HandleResponse,
    Querier, StdError, StdResult, Storage, 
    CosmosMsg, HumanAddr, Uint128
};
use secret_toolkit::{
    snip20::{
        token_info_query,
        register_receive_msg, 
        set_viewing_key_msg,
    },
};

use shade_protocol::{
    treasury::{
        HandleAnswer, 
        Snip20Asset
    },
    sscrt_staking::{
    }
    asset::Contract,
    generic_response::ResponseStatus,
};

use crate::state::{
    config_w, config_r, 
    assets_r, assets_w,
    viewing_key_r,
};

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    _from: HumanAddr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    let assets = assets_r(&deps.storage);

    let asset: Snip20Asset = assets.load(env.message.sender.to_string().as_bytes())?;
    debug_print!("Treasured {} u{}", amount, asset.token_info.symbol);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Receive {
            status: ResponseStatus::Success,
        } )? ),
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(owner) = owner {
            state.owner = owner;
        }
        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_register_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: &Contract,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut messages = vec![];
    let token_info = token_info_query(&deps.querier, 1,
                                      contract.code_hash.clone(),
                                      contract.address.clone())?;

    assets_w(&mut deps.storage).save(contract.address.to_string().as_bytes(), &Snip20Asset {
        contract: contract.clone(),
        token_info,
    })?;

    // Register contract in asset
    messages.push(register_receive(&env, &contract)?);

    // Set viewing key
    messages.push(set_viewing_key_msg(
                    viewing_key_r(&deps.storage).load()?,
                    None,
                    1,
                    contract.code_hash.clone(),
                    contract.address.clone())?);


    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( 
            &HandleAnswer::RegisterAsset {
                status: ResponseStatus::Success } 
            )? 
        )
    })
}

pub fn register_receive (
    env: &Env,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    let cosmos_msg = register_receive_msg(
        env.contract_code_hash.clone(),
        None,
        256,
        contract.code_hash.clone(),
        contract.address.clone(),
    );

    cosmos_msg
}

pub fn refresh_stake<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {

    let mut messages: Vec<CosmosMsg> = vec![];

    let balance = (querier.query_balance(address.clone(), &"uscrt".to_string())?).amount;
    let current_stake = Uint128(0);

    let delegations = delegations_r(&deps.storage).load()?;

    for delegation in delegations {
        current_stake += delegation.amount;
    }
    let config = config_r(&deps.storage).load()?;

    let desired_stake = calculate_stake(current_stake + balance, config.scrt_stake);

    if current_stake < desired_stake {
        messages.push(CosmosMsg::Staking(StakingMsg::Delegate {
            validator: HumanAddr(validator.to_string()),
            amount: Coin {
                denom: "uscrt".to_string(),
                amount: desired_stake - current_stake,
            },
        }))
    } else if desired_stake < current_stake {
        messages.push(
        )
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Receive {
            status: ResponseStatus::Success,
        } )? ),
    })

}

// unstakes amount
pub fn unstake(amount: Uint128) {
}

// Converts amount sscrt -> scrt and stakes it
pub fn stake(amount: Uint128) {
    CosmosMsg::Staking(StakingMsg::Undelegate {
        validator: HumanAddr(validator.to_string()),
        amount: Coin {
            denom: "uscrt".to_string(),
            amount: Uint128(amount),
        },
    })
}

pub fn calculate_stake(
    total_balance: Uint128, stake: Uint128
) -> Uint128 {
    total_balance.multiply_ratio(stake, 10000u128)
}

pub fn choose_validator<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {
}

pub fn claim_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {

}
