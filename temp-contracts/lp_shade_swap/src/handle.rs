use shade_protocol::c_std::{
    to_binary, Api, BalanceResponse, BankQuery, Binary, Coin, CosmosMsg, Env, DepsMut,
    Response, Addr, Querier, StakingMsg, StdError, StdResult, Storage, Uint128,
};

use shade_protocol::snip20::helpers::{balance_query};

use shade_protocol::{
    contract_interfaces::{
        dao::{
            lp_shade_swap::{
                HandleAnswer, Config, SplitMethod,
                is_supported_asset, get_supported_asset,
            },
            adapter,
        },
        mint,
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

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    _from: Addr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<Response> {

    let config = config_r(deps.storage).load()?;

    if !is_supported_asset(&config, &info.sender) {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    /* Base tokens in pair
     *
     * max out how much LP you can provide
     * bond LP token into rewards
     */

    let mut desired_token: Contract;

    if env.message.sender == config.token_a.address {
        desired_token = config.token_a;
    }
    else if env.message.sender == config.token_b.address {
        desired_token = config.token_b;
    }
    else if env.message.sender == config.liquidity_token.address {
        // TODO: stake lp tokens & exit
    }
    else {
        // TODO: send to treasury, non-pair rewards token
    }

    // get exchange rate &dyn Storageplit tokens
    match config.split {
        Some(split) => {
            match split {
                /*
                SplitMethod::Conversion { mint } => {
                    // TODO: get exchange rate
                    mint::QueryMsg::Mint {
                        offer_asset: desired_token.address.clone(),
                        amount: Uint128(1u128.pow(desired_token.decimals)),
                    }.query(
                    );
                },
                */
                //SplitMethod::Market { contract } => { }
                //SplitMethod::Lend { contract } => { }
            }
        }
    }

    /*
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
    */

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

    Ok(Response::new().set_data(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?))
}


pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: Config,
) -> StdResult<Response> {
    let cur_config = config_r(deps.storage).load()?;

    if info.sender != cur_config.admin {
        return Err(StdError::generic_err("unauthorized"));
    }

    // Save new info
    config_w(deps.storage).save(&config)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?))
}

/* Claim rewards and restake, hold enough for pending unbondings
 * Send available unbonded funds to treasury
 */
pub fn update(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: Addr,
) -> StdResult<Response> {

    let mut messages = vec![];

    let config = config_r(deps.storage).load()?;

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
    Ok(Response::new().set_data(to_binary(&adapter::HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?))
}

pub fn unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: Addr,
    amount: Uint128,
) -> StdResult<Response> {

    let config = config_r(deps.storage).load()?;

    //TODO: needs treasury & manager as admin, maybe just manager?
    /*
    if info.sender != config.admin && info.sender != config.treasury {
        return Err(StdError::generic_err("unauthorized"));
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

    unbonding_w(deps.storage).update(asset.as_str().as_bytes(), |u| Ok(u.unwrap_or_else(|| Uint128::zero()) + amount))?;

    Ok(Response::new().set_data(to_binary(&adapter::HandleAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: amount,
        })?))
}

pub fn claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: Addr,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    let asset_contract = get_supported_asset(&config, &asset);

    let mut messages = vec![];

    let balance = balance_query(
        &deps.querier,
        env.contract.address,
        viewing_key_r(deps.storage).load()?,
        1,
        asset_contract.code_hash.clone(),
        asset_contract.address.clone(),
    )?.amount;

    let mut claim_amount = unbonding_r(deps.storage).load(asset.as_str().as_bytes())?;

    if balance < claim_amount {
        claim_amount = balance;
    }

    unbonding_w(&mut deps.storage).update(
        asset.as_str().as_bytes(),
        |u| Ok((u.unwrap() - claim_amount)?)
    )?;

    Ok(Response::new().set_data(to_binary(&adapter::HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount: claim_amount,
        })?))
}
