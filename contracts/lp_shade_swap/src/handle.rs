use cosmwasm_std::{
    debug_print, to_binary, Api, BalanceResponse, BankQuery, Binary, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HumanAddr, Querier, StakingMsg, StdError, StdResult, Storage, Uint128,
};

use secret_toolkit::snip20::{balance_query};

use shade_protocol::{
    contract_interfaces::dao::{
        lp_shade_swap::{
            HandleAnswer, Config, SplitMethod,
            is_supported_asset, get_supported_asset,
        },
        treasury::Flag,
        mint,
        adapter,
    },
    utils::{
        generic_response::ResponseStatus,
        asset::{
            Contract,
            scrt_balance,
        },
        wrap::{wrap_and_send, unwrap},
    },
};

use crate::{
    query,
    state::{
        config_r, config_w,
        self_address_r,
        unbonding_w, unbonding_r,
        viewing_key_r,
    },
};

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    _from: HumanAddr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    if !is_supported_asset(&config, &env.message.sender) {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    /* Base tokens in pair
     *
     * max out how much LP you can provide
     * bond LP token into rewards
     */

    let mut desired_token: Contract;

    if env.message.sender == config.token_a.address {
        desired_token = config.token_0;
    }
    else if env.message.sender == config.token_b.address {
        desired_token = config.token_1;
    }
    else if env.message.sender == config.lp_token {
        // TODO: stake lp tokens & exit
    }
    else {
        // TODO: send to treasury, non-pair rewards token
    }

    // get exchange rate & Split tokens
    match config.split {
        Some(split) => {
            match split {
                SplitMethod::Conversion { mint } => {
                    // TODO: get exchange rate
                    mint::QueryMsg::Mint {
                        offer_asset: desired_token.address.clone(),
                        amount: Uint128(1u128.pow(desired_token.decimals)),
                    }.query(
                    );
                },
                //SplitMethod::Market { contract } => { }
                //SplitMethod::Lend { contract } => { }
            }
        }
    }

    let pair_info: amm_pair::QueryMsgResponse::PairInfoResponse = match amm_pair::QueryMsg::GetPairInfo.query(
        &deps.querier,
        msg.pair.code_hash.clone(),
        msg.pair.address.clone(),
    ) {
        Ok(info) => info,
        Err(_) => {
            return Err(StdError::generic_err("Failed to query pair"));
        }
    };

    if desired_token.address == pair_info.token_0.address {
        denominator = pair_info.amount_0;
    }
    else if desired_token.address == pair_info.token_1.address {
        denominator = pair_info.amount_1;
    }
    else {
        return Err(StdError::generic_err(format!(
                    "Asset configuration conflict, pair info missing: {}", 
                    desired_token.address.to_string()
                )));
    }

    let provide_amounts: (Uint128, Uint128);
    // TODO math with exchange_rate & pool ratio & received amount
    
    // Can be added with a trigger if too slow
    let mut messages = vec![];
    messages.append(
        set_allowance(&deps, &env,
                      config.pair.clone(),
                      provide_amounts.0,
                      msg.viewing_key.clone(),
                      config.token_a.clone(),
                  )?);
    messages.append(
        set_allowance(&deps, &env,
                      config.pair.clone(),
                      provide_amounts.0,
                      msg.viewing_key.clone(),
                      config.token_b.clone(),
                  )?);

    /* TODO
     * - add LP to pair 
     * - stake LP tokens in staking_contract (auto complete from pair?)
     */

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
) -> StdResult<HandleResponse> {
    let cur_config = config_r(&deps.storage).load()?;

    if env.message.sender != cur_config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    config_w(&mut deps.storage).save(&config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

/* Claim rewards and restake, hold enough for pending unbondings
 * Send available unbonded funds to treasury
 */
pub fn update<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {

    let mut messages = vec![];

    let config = config_r(&deps.storage).load()?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    /* Claim Rewards
     *
     * If rewards is an LP denom, try to re-add LP based on balances
     * e.g. SILK/SHD w/ SHD rewards
     *      pair/split the new SHD with SILK and provide
     *
     * Else send direct to treasury e.g. sSCRT/sETH w/ SHD rewards
     */
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    //TODO: needs treasury & manager as admin, maybe just manager?
    /*
    if env.message.sender != config.admin && env.message.sender != config.treasury {
        return Err(StdError::Unauthorized { backtrace: None });
    }
    */

    let mut messages = vec![];

    if asset == config.liquidity_token.address {
        /* Pull LP token out of rewards contract
         * Hold for claiming
         */
    }
    else if vec![
        config.token_a.address, 
        config.token_b.address,
    ].contains(&asset) {
        /* Pull LP from rewards
         * Split LP into tokens A & B
         * Mark requested token for claim
         */
    }
    else {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    unbonding_w(&mut deps.storage).update(asset.as_str().as_bytes(), |u| Ok(u.unwrap_or_else(|| Uint128::zero()) + amount))?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: amount,
        })?),
    })
}

pub fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    let asset_contract = get_supported_asset(&config, &asset);

    let mut messages = vec![];

    let balance = balance_query(
        &deps.querier,
        env.contract.address,
        viewing_key_r(&deps.storage).load()?,
        1,
        asset_contract.code_hash.clone(),
        asset_contract.address.clone(),
    )?.amount;

    let mut claim_amount = unbonding_r(&deps.storage).load(asset.as_str().as_bytes())?;

    if balance < claim_amount {
        claim_amount = balance;
    }

    unbonding_w(&mut deps.storage).update(
        asset.as_str().as_bytes(), 
        |u| Ok((u.unwrap() - claim_amount)?)
    )?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount: claim_amount,
        })?),
    })
}
