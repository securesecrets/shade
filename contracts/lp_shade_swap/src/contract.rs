use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdResult, StdError,
    Storage, Uint128,
};

use shade_protocol::{
    adapter,
    shadeswap,
    lp_shade_swap::{
        Config, HandleMsg, InitMsg, QueryMsg,
        is_supported_asset,
    },
    utils::asset::Contract,
};

use secret_toolkit::{
    snip20::{register_receive_msg, set_viewing_key_msg},
    utils::Query,
};

use crate::{
    handle, query,
    state::{
        config_w, self_address_w, 
        viewing_key_r, viewing_key_w,
        unbonding_w,
    },
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    self_address_w(&mut deps.storage).save(&env.contract.address)?;
    viewing_key_w(&mut deps.storage).save(&msg.viewing_key)?;

    let pair_info: shadeswap::PairInfoResponse = match shadeswap::PairQuery::PairInfo.query(
        &deps.querier,
        msg.pair.code_hash.clone(),
        msg.pair.address.clone(),
    ) {
        Ok(info) => info,
        Err(_) => {
            return Err(StdError::generic_err("Failed to query pair"));
        }
        /*
        shadeswap::PairInfoResponse {
            liquidity_token, factory, pair,
            amount_0, amount_1,
            total_liquidity, contract_version,
        } => {
        }
        */
    };

    let token_a = match pair_info.pair.0 {
        shadeswap::TokenType::CustomToken {
            contract_addr,
            token_code_hash
        } => Contract { 
            address: contract_addr,
            code_hash: token_code_hash,
        },
        _ => {
            return Err(StdError::generic_err("Unsupported token type"));
        }
    };

    let token_b = match pair_info.pair.1 {
        shadeswap::TokenType::CustomToken {
            contract_addr,
            token_code_hash
        } => Contract { 
            address: contract_addr,
            code_hash: token_code_hash,
        },
        _ => {
            return Err(StdError::generic_err("Unsupported token type"));
        }
    };

    //let reward_token = TODO: query for reward token

    let config = Config {
        admin: match msg.admin {
            None => env.message.sender.clone(),
            Some(admin) => admin,
        },
        treasury: msg.treasury,
        pair: msg.pair.clone(),
        token_a: token_a.clone(),
        token_b: token_b.clone(),
        liquidity_token: pair_info.liquidity_token.clone(),
        rewards_contract: msg.rewards_contract.clone(),
        // TODO: query reward token from rewards contract
        reward_token: None, //msg.reward_token,
    };

    // Init unbondings to 0
    for asset in vec![
            token_a.clone(),
            token_b.clone(),
            pair_info.liquidity_token.clone(),
        ] {
        unbonding_w(&mut deps.storage).save(
            asset.address.as_str().as_bytes(),
            &Uint128::zero(),
        )?;
    }

    config_w(&mut deps.storage).save(&config.clone())?;

    let mut messages = vec![
        set_viewing_key_msg(
            msg.viewing_key.clone(),
            None,
            1,
            config.token_a.code_hash.clone(),
            config.token_a.address.clone(),
        )?,
        register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            256,
            config.token_a.code_hash.clone(),
            config.token_a.address.clone(),
        )?,
        set_viewing_key_msg(
            msg.viewing_key.clone(),
            None,
            1,
            config.token_b.code_hash.clone(),
            config.token_b.address.clone(),
        )?,
        register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            256,
            config.token_b.code_hash.clone(),
            config.token_b.address.clone(),
        )?,
        set_viewing_key_msg(
            msg.viewing_key.clone(),
            None,
            1,
            pair_info.liquidity_token.code_hash.clone(),
            pair_info.liquidity_token.address.clone(),
        )?,
        register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            256,
            pair_info.liquidity_token.code_hash.clone(),
            pair_info.liquidity_token.address.clone(),
        )?,
    ];

    if let Some(ref reward_token) = config.reward_token {

        if !is_supported_asset(&config.clone(), &reward_token.address) {
            messages.append(&mut vec![
                set_viewing_key_msg(
                    msg.viewing_key.clone(),
                    None,
                    1,
                    reward_token.code_hash.clone(),
                    reward_token.address.clone(),
                )?,
                register_receive_msg(
                    env.contract_code_hash.clone(),
                    None,
                    256,
                    reward_token.code_hash.clone(),
                    reward_token.address.clone(),
                )?,
            ]);
        }
    }

    Ok(InitResponse {
        messages,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => handle::receive(deps, env, sender, from, amount, msg),
        HandleMsg::UpdateConfig { config } => handle::try_update_config(deps, env, config),
        HandleMsg::Adapter(adapter) => match adapter {
            adapter::SubHandleMsg::Unbond { asset, amount } => handle::unbond(deps, env, asset, amount),
            adapter::SubHandleMsg::Claim { asset } => handle::claim(deps, env, asset),
            adapter::SubHandleMsg::Update { asset } => handle::update(deps, env, asset),
        },
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => to_binary(&query::balance(deps, asset)?),
            adapter::SubQueryMsg::Claimable { asset } => to_binary(&query::claimable(deps, asset)?),
            adapter::SubQueryMsg::Unbonding { asset } => to_binary(&query::unbonding(deps, asset)?),
            adapter::SubQueryMsg::Unbondable { asset } => to_binary(&query::unbondable(deps, asset)?),
        }
    }
}
