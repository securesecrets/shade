use cosmwasm_schema::cw_serde;
use shade_protocol::{
    c_std::{
        shd_entry_point,
        to_binary,
        Addr,
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
    contract_interfaces::dao::adapter,
    snip20::helpers::{balance_query, register_receive, send_msg, set_viewing_key_msg},
    utils::{
        asset::Contract,
        generic_response::ResponseStatus,
        storage::plus::Item,
        ExecuteCallback,
        InstantiateCallback,
        Query,
    },
};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub instant: bool,
    pub token: Contract,
}

impl InstantiateCallback for Config {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    GiveMeMoney {
        amount: Uint128,
    },
    CompleteUnbonding {},
    Adapter(adapter::SubExecuteMsg),
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryMsg {
    Config,
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
    Adapter(adapter::SubQueryMsg),
}

const VIEWING_KEY: &str = "jUsTfOrTeStInG";

const CONFIG: Item<Config> = Item::new("config");
const ADDRESS: Item<Addr> = Item::new("address");
const REWARDS: Item<Uint128> = Item::new("rewards");

const UNBONDING: Item<Uint128> = Item::new("unbonding");
const CLAIMABLE: Item<Uint128> = Item::new("claimable");

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: Config,
) -> StdResult<Response> {
    CONFIG.save(deps.storage, &msg)?;
    ADDRESS.save(deps.storage, &env.contract.address)?;
    //BLOCK.save(deps.storage, &Uint128::new(env.block.height as u128))?;

    UNBONDING.save(deps.storage, &Uint128::zero())?;
    CLAIMABLE.save(deps.storage, &Uint128::zero())?;
    REWARDS.save(deps.storage, &Uint128::zero())?;

    Ok(Response::new().add_messages(vec![
        set_viewing_key_msg(VIEWING_KEY.to_string(), None, &msg.token.clone())?,
        register_receive(env.contract.code_hash.clone(), None, &msg.token.clone())?,
    ]))
}

#[shd_entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    //BLOCK.save(deps.storage, &Uint128::new(env.block.height as u128))?;

    match msg {
        ExecuteMsg::Receive {
            sender: _,
            from,
            amount,
            memo: _,
            msg: _,
        } => {
            if info.sender != config.token.address {
                return Err(StdError::generic_err("Unrecognized Asset"));
            }

            // If sender is not manager, consider rewards
            if from != config.owner {
                let rew = REWARDS.load(deps.storage)?;
                REWARDS.save(deps.storage, &(rew + amount))?;
            }

            Ok(Response::new())
        }
        ExecuteMsg::GiveMeMoney { amount } => Ok(Response::new().add_message(send_msg(
            info.sender,
            amount,
            None,
            None,
            None,
            &config.token,
        )?)),
        ExecuteMsg::CompleteUnbonding {} => {
            let unbonding = UNBONDING.load(deps.storage)?;
            let claimable = CLAIMABLE.load(deps.storage)?;

            UNBONDING.save(deps.storage, &Uint128::zero())?;
            CLAIMABLE.save(deps.storage, &(claimable + unbonding))?;
            Ok(Response::new())
        }
        ExecuteMsg::Adapter(adapter) => match adapter {
            adapter::SubExecuteMsg::Unbond { asset, amount } => {
                if asset != config.token.address {
                    return Err(StdError::generic_err("Unrecognized Asset"));
                }

                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(deps.storage)?,
                    VIEWING_KEY.to_string(),
                    &config.token.clone(),
                )?;

                //let rewards = REWARDS.load(deps.storage)?;

                let unbonding = UNBONDING.load(deps.storage)?;
                let claimable = CLAIMABLE.load(deps.storage)?;
                let rewards = REWARDS.load(deps.storage)?;

                let available = (balance + rewards) - (unbonding + claimable);

                if available < amount {
                    return Err(StdError::generic_err(format!(
                        "Cannot unbond {}, {} available",
                        amount, available
                    )));
                }

                let mut messages = vec![];

                if config.instant {
                    messages.push(send_msg(
                        config.owner.clone(),
                        amount,
                        None,
                        None,
                        None,
                        &config.token.clone(),
                    )?);
                } else {
                    UNBONDING.save(deps.storage, &(unbonding + amount))?;
                }

                Ok(Response::new().add_messages(messages).set_data(to_binary(
                    &adapter::ExecuteAnswer::Unbond {
                        status: ResponseStatus::Success,
                        amount,
                    },
                )?))
            }
            adapter::SubExecuteMsg::Claim { asset } => {
                if asset != config.token.address {
                    return Err(StdError::generic_err("Unrecognized Asset"));
                }
                let claimable = CLAIMABLE.load(deps.storage)?;
                CLAIMABLE.save(deps.storage, &Uint128::zero())?;
                REWARDS.save(deps.storage, &Uint128::zero())?;

                Ok(Response::new()
                    .add_message(send_msg(
                        config.owner.clone(),
                        claimable,
                        None,
                        None,
                        None,
                        &config.token.clone(),
                    )?)
                    .set_data(to_binary(&adapter::ExecuteAnswer::Claim {
                        status: ResponseStatus::Success,
                        amount: claimable,
                    })?))
            }
            adapter::SubExecuteMsg::Update { asset } => {
                if asset != config.token.address {
                    return Err(StdError::generic_err("Unrecognized Asset"));
                }
                // 'claim & restake' rewards
                REWARDS.save(deps.storage, &Uint128::zero())?;

                Ok(
                    Response::new().set_data(to_binary(&adapter::ExecuteAnswer::Update {
                        status: ResponseStatus::Success,
                    })?),
                )
            }
        },
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    match msg {
        QueryMsg::Config => to_binary(&QueryAnswer::Config { config }),
        QueryMsg::Adapter(adapter) => to_binary(&match adapter {
            adapter::SubQueryMsg::Balance { asset } => {
                if asset != config.token.address {
                    return Err(StdError::generic_err("Unrecognized Asset"));
                }
                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(deps.storage)?,
                    VIEWING_KEY.to_string(),
                    &config.token.clone(),
                )?;

                adapter::QueryAnswer::Balance { amount: balance }
            }
            adapter::SubQueryMsg::Unbonding { asset } => {
                if asset != config.token.address {
                    return Err(StdError::generic_err("Unrecognized Asset"));
                }
                adapter::QueryAnswer::Unbonding {
                    amount: UNBONDING.load(deps.storage)?,
                }
            }
            adapter::SubQueryMsg::Claimable { asset } => {
                if asset != config.token.address {
                    return Err(StdError::generic_err("Unrecognized Asset"));
                }

                let _c = CLAIMABLE.load(deps.storage)?;

                adapter::QueryAnswer::Claimable {
                    amount: CLAIMABLE.load(deps.storage)?,
                }
            }
            adapter::SubQueryMsg::Unbondable { asset } => {
                if asset != config.token.address {
                    return Err(StdError::generic_err("Unrecognized Asset"));
                }
                let unbonding = UNBONDING.load(deps.storage)?;
                let claimable = CLAIMABLE.load(deps.storage)?;
                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(deps.storage)?,
                    VIEWING_KEY.to_string(),
                    &config.token.clone(),
                )?;

                adapter::QueryAnswer::Unbondable {
                    amount: balance - (unbonding + claimable),
                }
            }
            adapter::SubQueryMsg::Reserves { asset } => {
                if asset != config.token.address {
                    return Err(StdError::generic_err("Unrecognized Asset"));
                }

                let reserves = match config.instant {
                    true => {
                        balance_query(
                            &deps.querier,
                            ADDRESS.load(deps.storage)?,
                            VIEWING_KEY.to_string(),
                            &config.token.clone(),
                        )? - (UNBONDING.load(deps.storage)? + CLAIMABLE.load(deps.storage)?)
                    }
                    false => {
                        let rewards = REWARDS.load(deps.storage)?;
                        let unbonding = UNBONDING.load(deps.storage)?;
                        if rewards > unbonding {
                            rewards - unbonding
                        } else {
                            Uint128::zero()
                        }
                    }
                };

                adapter::QueryAnswer::Reserves { amount: reserves }
            }
        }),
    }
}
