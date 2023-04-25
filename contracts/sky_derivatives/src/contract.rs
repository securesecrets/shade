use shade_protocol::c_std::{
    shd_entry_point,
    to_binary,
    Binary,
    Decimal,
    Deps,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdError,
    StdResult,
    Uint128,
};

use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    contract_interfaces::{
        dao::adapter,
        sky::sky_derivatives::{
            Config,
            DexPairs,
            ExecuteMsg, 
            ExecuteAnswer,
            InstantiateMsg, 
            QueryMsg,
            TreasuryUnbondings,
            SelfAddr,
        },
    },
    utils::{
        generic_response::ResponseStatus,
        storage::plus::ItemStorage,
    },
};

use crate::{
    execute,
    query,
};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    SelfAddr(env.contract.address.clone()).save(deps.storage)?;
    TreasuryUnbondings(Uint128::zero()).save(deps.storage)?;

    // Validate shade admin works
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin,
        info.sender.to_string(),
        &msg.shade_admin_addr,
    )?;

    // Validate trading fees
    if msg.trading_fees.dex_fee > Decimal::one() || msg.trading_fees.stake_fee > Decimal::one() ||
            msg.trading_fees.unbond_fee > Decimal::one() {
        return Err(StdError::generic_err("Trading fee cannot be over 100%"));
    }

    Config {
        shade_admin_addr: msg.shade_admin_addr,
        treasury: msg.treasury,
        derivative: msg.derivative.clone(),
        trading_fees: msg.trading_fees,
        max_arb_amount: msg.max_arb_amount,
        min_profit_amount: msg.min_profit_amount,
        viewing_key: msg.viewing_key.clone(),
    }.save(deps.storage)?;

    // Validate each dex pair
    let mut new_pairs = vec![];
    for pair in msg.dex_pairs {
        // derivative must be the 2nd entry in the dex_pair
        if !execute::validate_dex_pair(&msg.derivative, &pair) {
            return Err(StdError::generic_err(
                "Invalid pair - original token must be token 0 and derivative must be token 1, decimals must match derivative"
            ));
        }
        new_pairs.push(pair);
    }
    DexPairs(new_pairs).save(deps.storage)?;

    // Viewing keys
    let messages = execute::set_viewing_keys(&msg.derivative, &msg.viewing_key)?;

    Ok(Response::new()
       .set_data(to_binary(&ExecuteAnswer::Init {
           status: ResponseStatus::Success,
       })?)
       .add_messages(messages)
    )
}

#[shd_entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { 
            shade_admin_addr,
            treasury,
            derivative,
            trading_fees,
            max_arb_amount,
            min_profit_amount,
            viewing_key,
        } => execute::try_update_config(
            deps, 
            info,
            shade_admin_addr, 
            treasury,
            derivative, 
            trading_fees,
            max_arb_amount,
            min_profit_amount,
            viewing_key,
        ),
        ExecuteMsg::SetPairs { pairs } => execute::try_set_pairs(deps, info, pairs),
        ExecuteMsg::SetPair { pair, index } => execute::try_set_pair(deps, info, pair, index),
        ExecuteMsg::AddPair { pair } => execute::try_add_pair(deps, info, pair),
        ExecuteMsg::RemovePair { index } => execute::try_remove_pair(deps, info, index),
        ExecuteMsg::Arbitrage { index } => execute::try_arb_pair(deps, info, index),
        ExecuteMsg::ArbAllPairs {} => execute::try_arb_all_pairs(deps, info),
        ExecuteMsg::Adapter(adapter) => match adapter {
            adapter::SubExecuteMsg::Unbond { asset, amount } =>
                execute::try_adapter_unbond(deps, env, info, asset, Uint128::from(amount.u128())),
            adapter::SubExecuteMsg::Claim { asset } => execute::try_adapter_claim(deps, env, info, asset),
            adapter::SubExecuteMsg::Update { asset } => execute::try_adapter_update(deps, env, info, asset),
        },
    }
}

#[shd_entry_point]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::DexPairs {} => to_binary(&query::dex_pairs(deps)?),
        QueryMsg::IsProfitable { index } => {
            match index {
                Some(i) => to_binary(&query::is_profitable(deps, i)?),
                None => to_binary(&query::is_profitable(deps, 0)?),
            }
        },
        QueryMsg::IsAnyPairProfitable { } => 
            to_binary(&query::is_any_pair_profitable(deps)?),
        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } =>
                to_binary(&query::adapter_balance(deps, asset)?),
            adapter::SubQueryMsg::Claimable { asset } =>
                to_binary(&query::adapter_claimable(deps, asset)?),
            adapter::SubQueryMsg::Unbonding { asset } =>
                to_binary(&query::adapter_unbonding(deps, asset)?),
            adapter::SubQueryMsg::Unbondable { asset } =>
                to_binary(&query::adapter_unbondable(deps, asset)?),
            adapter::SubQueryMsg::Reserves { asset } =>
                to_binary(&query::adapter_reserves(deps, asset)?),
        },
    }
}
