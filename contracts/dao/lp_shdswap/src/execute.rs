use crate::storage::*;
use shade_protocol::{
    c_std::{
        to_binary,
        Addr,
        Binary,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Uint128,
    },
    contract_interfaces::dao::{
        adapter,
        lp_shdswap::{get_supported_asset, is_supported_asset, Config, ExecuteAnswer, SplitMethod},
    },
    snip20::helpers::balance_query,
    utils::{asset::Contract, generic_response::ResponseStatus},
};

pub fn receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _sender: Addr,
    _from: Addr,
    _amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    if !is_supported_asset(&config, &info.sender) {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    /* Base tokens in pair
     *
     * max out how much LP you can provide
     * bond LP token into rewards
     */

    let desired_token: Contract;

    if info.sender == config.token_a.address {
        desired_token = config.token_a;
        println!("{}", desired_token.address);
    } else if info.sender == config.token_b.address {
        desired_token = config.token_b;
        println!("{}", desired_token.address);
    } else if info.sender == config.liquidity_token.address {
        // TODO: stake lp tokens & exit
    } else {
        // TODO: send to treasury, non-pair rewards token
    }

    // get exchange rate &dyn Storageplit tokens
    match config.split {
        Some(split) => {
            match split {
                SplitMethod::Conversion { contract: _ } => {} /*
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
        None => {}
    }

    /*let pair_info: shadeswap::PairInfoResponse =
        match (shadeswap::PairQuery::GetPairInfo {}.query(&deps.querier, &msg.pair)) {
            Ok(info) => info,
            Err(_) => {
                return Err(StdError::generic_err("Failed to query pair"));
            }
        };

    if desired_token.address == pair_info.token_0.address {
        denominator = pair_info.amount_0;
    } else if desired_token.address == pair_info.token_1.address {
        denominator = pair_info.amount_1;
    } else {
        return Err(StdError::generic_err(format!(
            "Asset configuration conflict, pair info missing: {}",
            desired_token.address.to_string()
        )));
    }*/

    let _provide_amounts: (Uint128, Uint128);
    // TODO math with exchange_rate & pool ratio & received amount

    // Can be added with a trigger if too slow
    //let mut messages = vec![];
    /*messages.append(set_allowance(
        &deps,
        &env,
        config.pair.clone(),
        provide_amounts.0,
        msg.viewing_key.clone(),
        config.token_a.clone(),
    )?);
    messages.append(set_allowance(
        &deps,
        &env,
        config.pair.clone(),
        provide_amounts.0,
        msg.viewing_key.clone(),
        config.token_b.clone(),
    )?);*/

    /* TODO
     * - add LP to pair
     * - stake LP tokens in staking_contract (auto complete from pair?)
     */

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Receive {
        status: ResponseStatus::Success,
    })?))
}

pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    config: Config,
) -> StdResult<Response> {
    let cur_config = CONFIG.load(deps.storage)?;

    if info.sender != cur_config.admin {
        return Err(StdError::generic_err("unauthorized"));
    }

    // Save new info
    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::UpdateConfig {
            status: ResponseStatus::Success,
            config,
        })?),
    )
}

pub fn refesh_allowances(_deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
    Ok(Response::new())
}

/* Claim rewards and restake, hold enough for pending unbondings
 * Send available unbonded funds to treasury
 */
pub fn update(deps: DepsMut, _env: Env, _info: MessageInfo, asset: Addr) -> StdResult<Response> {
    //let mut messages = vec![];

    let config = CONFIG.load(deps.storage)?;

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
    Ok(
        Response::new().set_data(to_binary(&adapter::ExecuteAnswer::Update {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn unbond(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    asset: Addr,
    amount: Uint128,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    //TODO: needs treasury & manager as admin, maybe just manager?
    /*
    if info.sender != config.admin && info.sender != config.treasury {
        return Err(StdError::generic_err("unauthorized"));
    }
    */

    //let mut messages = vec![];

    if asset == config.liquidity_token.address {
        /* Pull LP token out of rewards contract
         * Hold for claiming
         */
    } else if vec![config.token_a.address, config.token_b.address].contains(&asset) {
        /* Pull LP from rewards
         * Split LP into tokens A & B
         * Mark requested token for claim
         */
    } else {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    UNBONDING.update(deps.storage, asset.clone(), |u| -> StdResult<Uint128> {
        Ok(u.unwrap_or_else(|| Uint128::zero()) + amount)
    })?;

    Ok(
        Response::new().set_data(to_binary(&adapter::ExecuteAnswer::Unbond {
            status: ResponseStatus::Success,
            amount,
        })?),
    )
}

pub fn claim(deps: DepsMut, env: Env, _info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    let asset_contract = get_supported_asset(&config, &asset);

    //let mut messages = vec![];

    let balance = balance_query(
        &deps.querier,
        env.contract.address,
        VIEWING_KEY.load(deps.storage)?,
        &asset_contract,
    )?;

    let mut claim_amount = UNBONDING.load(deps.storage, asset.clone())?;

    if balance < claim_amount {
        claim_amount = balance;
    }

    UNBONDING.update(deps.storage, asset.clone(), |u| -> StdResult<Uint128> {
        Ok(u.unwrap_or_else(|| Uint128::zero()) - claim_amount)
    })?;

    Ok(
        Response::new().set_data(to_binary(&adapter::ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
            amount: claim_amount,
        })?),
    )
}
