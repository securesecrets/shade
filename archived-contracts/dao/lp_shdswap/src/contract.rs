use shade_protocol::{
    c_std::{
        entry_point,
        to_binary,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Uint128,
    },
    contract_interfaces::{
        dao::{
            adapter,
            lp_shdswap::{Config, ExecuteMsg, InstantiateMsg, QueryMsg},
        },
        dex::shadeswap,
    },
    snip20::helpers::{register_receive, set_viewing_key_msg},
    utils::{asset::Contract, Query},
};

use crate::{execute, query, storage::*};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    SELF_ADDRESS.save(deps.storage, &env.contract.address)?;
    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;

    let pair_info: shadeswap::PairInfoResponse =
        match (shadeswap::PairQuery::GetPairInfo {}.query(&deps.querier, &msg.pair)) {
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

    /*let staking_info: amm_pair::QueryMsgResponse::StakingContractInfo =
        amm_pair::QueryMsg::GetStakingContractInfo.query(
            &deps.querier,
            msg.pair.code_hash.clone(),
            msg.pair.address.clone(),
        )?;

    //TODO need this query
    let reward_token: Contract = Contract {
        address: Addr::unchecked(""),
        code_hash: "".into(),
    };*/

    let config = Config {
        admin: match msg.admin {
            None => info.sender.clone(),
            Some(admin) => admin,
        },
        treasury: msg.treasury,
        pair: msg.pair.clone(),
        token_a: token_a.clone(),
        token_b: token_b.clone(),
        liquidity_token: pair_info.liquidity_token.clone(),
        staking_contract: Some(Contract::default()),
        //staking_info.staking_contract.clone(),
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

    if let Some(token) = config.reward_token.clone() {
        assets.push(token);
    }

    let mut messages = vec![];

    // Init unbondings & msgs
    for token in assets {
        UNBONDING.save(deps.storage, token.address.clone(), &Uint128::zero())?;

        messages.append(&mut vec![
            set_viewing_key_msg(msg.viewing_key.clone(), None, &token)?,
            register_receive(env.contract.code_hash.clone(), None, &token)?,
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

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => execute::receive(deps, env, info, sender, from, amount, msg),
        ExecuteMsg::UpdateConfig { config } => execute::try_update_config(deps, env, info, config),
        ExecuteMsg::RefreshApprovals => execute::refesh_allowances(deps, env, info),
        ExecuteMsg::Adapter(adapter) => match adapter {
            adapter::SubExecuteMsg::Unbond { asset, amount } => {
                let asset = deps.api.addr_validate(&asset)?;
                execute::unbond(deps, env, info, asset, amount)
            }
            adapter::SubExecuteMsg::Claim { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                execute::claim(deps, env, info, asset)
            }
            adapter::SubExecuteMsg::Update { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                execute::update(deps, env, info, asset)
            }
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                to_binary(&query::balance(deps, asset)?)
            }
            adapter::SubQueryMsg::Claimable { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                to_binary(&query::claimable(deps, asset)?)
            }
            adapter::SubQueryMsg::Unbonding { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                to_binary(&query::unbonding(deps, asset)?)
            }
            adapter::SubQueryMsg::Unbondable { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                to_binary(&query::unbondable(deps, asset)?)
            }
            adapter::SubQueryMsg::Reserves { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                to_binary(&query::reserves(deps, asset)?)
            }
        },
    }
}
