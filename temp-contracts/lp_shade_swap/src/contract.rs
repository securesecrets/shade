use shade_protocol::c_std::{
    debug_print,
    to_binary,
    Api,
    Binary,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    InitResponse,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
};

use shade_protocol::{
    contract_interfaces::dao::{
        adapter,
        lp_shade_swap::{is_supported_asset, Config, HandleMsg, InitMsg, QueryMsg},
    },
    utils::asset::{set_allowance, Contract},
};

/*
use shadeswap_shared::{
    self as shadeswap,
    msg::amm_pair,
};
*/

use shade_protocol::secret_toolkit::{
    snip20::{register_receive_msg, set_viewing_key_msg},
    utils::Query,
};

use crate::{
    handle,
    query,
    state::{config_w, self_address_w, unbonding_w, viewing_key_r, viewing_key_w},
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    self_address_w(&mut deps.storage).save(&env.contract.address)?;
    viewing_key_w(&mut deps.storage).save(&msg.viewing_key)?;

    let pair_info: amm_pair::QueryMsgResponse::PairInfoResponse =
        match amm_pair::QueryMsg::GetPairInfo.query(
            &deps.querier,
            msg.pair.code_hash.clone(),
            msg.pair.address.clone(),
        ) {
            Ok(info) => info,
            Err(_) => {
                return Err(StdError::generic_err("Failed to query pair"));
            }
        };

    let token_a = match pair_info.pair.token_0 {
        shadeswap::TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => Contract {
            address: contract_addr,
            code_hash: token_code_hash,
        },
        _ => {
            return Err(StdError::generic_err("Unsupported token type"));
        }
    };

    let token_b = match pair_info.pair.token_1 {
        shadeswap::TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => Contract {
            address: contract_addr,
            code_hash: token_code_hash,
        },
        _ => {
            return Err(StdError::generic_err("Unsupported token type"));
        }
    };

    let staking_info: amm_pair::QueryMsgResponse::StakingContractInfo =
        amm_pair::QueryMsg::GetStakingContractInfo.query(
            &deps.querier,
            msg.pair.code_hash.clone(),
            msg.pair.address.clone(),
        )?;

    //TODO need this query
    let reward_token: Contract = Contract {
        address: HumanAddr("".into()),
        code_hash: "".into(),
    };

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
        staking_contract: staking_info.staking_contract.clone(),
        // TODO: query reward token from staking contract
        reward_token: None,
        //TODO: add this
        split: None,
    };
    // TODO verify split contract
    let mut assets = vec![
        token_a.clone(),
        token_b.clone(),
        pair_info.liquidity_token.clone(),
    ];

    if let Some(token) = config.reward_token {
        assets.push(token);
    }

    let mut messages = vec![];

    // Init unbondings & msgs
    for token in assets {
        unbonding_w(&mut deps.storage).save(token.address.as_str().as_bytes(), &Uint128::zero())?;

        messages.append(&mut vec![
            set_viewing_key_msg(
                msg.viewing_key.clone(),
                None,
                1,
                token.code_hash.clone(),
                token.address.clone(),
            )?,
            register_receive_msg(
                env.contract_code_hash.clone(),
                None,
                256,
                token.code_hash.clone(),
                token.address.clone(),
            )?,
        ]);
    }

    // Init approvals to max
    /*
     for token in vec![token_a, token_b] {
         set_allowance(&deps, &env,
                       config.pair.clone(),
                       Uint128(9_000_000_000_000_000_000_000_000),
                       msg.viewing_key.clone(),
                       token.clone(),
                   );
    }
     */

    config_w(&mut deps.storage).save(&config.clone())?;

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
            adapter::SubHandleMsg::Unbond { asset, amount } => {
                handle::unbond(deps, env, asset, amount)
            }
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
            adapter::SubQueryMsg::Unbondable { asset } => {
                to_binary(&query::unbondable(deps, asset)?)
            }
            adapter::SubQueryMsg::Reserves { asset } => to_binary(&query::reserves(deps, asset)?),
        },
    }
}
