use cosmwasm_schema::cw_serde;
use shade_protocol::{
    c_std::{
        entry_point,
        to_binary,
        Addr,
        Api,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Querier,
        Response,
        StdError,
        StdResult,
        Storage,
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
    pub unbond_blocks: Uint128,
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
    //RegisterAsset { token: Contract },
    //CompleteUnbond { token: Addr, amount: Uint128 },
    Adapter(adapter::SubExecuteMsg),
}

#[cw_serde]
pub enum QueryMsg {
    Config,
    Adapter(adapter::SubQueryMsg),
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
    Adapter(adapter::SubQueryMsg),
}

const viewing_key: &str = "jUsTfOrTeStInG";

const CONFIG: Item<Config> = Item::new("config");
const ADDRESS: Item<Addr> = Item::new("address");
const BLOCK: Item<Uint128> = Item::new("block");
const REWARDS: Item<Uint128> = Item::new("rewards");

// (amount, block)
const UNBONDINGS: Item<Vec<(Uint128, Uint128)>> = Item::new("unbondings");

#[entry_point]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: Config) -> StdResult<Response> {
    CONFIG.save(deps.storage, &msg)?;
    ADDRESS.save(deps.storage, &env.contract.address)?;
    UNBONDINGS.save(deps.storage, &Vec::new())?;
    BLOCK.save(deps.storage, &Uint128::new(env.block.height as u128))?;

    Ok(Response::new().add_messages(vec![
        set_viewing_key_msg(viewing_key.to_string(), None, &msg.token.clone())?,
        register_receive(env.contract.code_hash.clone(), None, &msg.token.clone())?,
    ]))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    BLOCK.save(deps.storage, &Uint128::new(env.block.height as u128))?;

    match msg {
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            memo,
            msg,
        } => {
            if info.sender != config.token.address {
                return Err(StdError::generic_err("Unrecognized Asset"));
            }

            /*
            match from_binary(&msg)? {
                // add rewards
            }
            */
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
        ExecuteMsg::Adapter(adapter) => match adapter {
            adapter::SubExecuteMsg::Unbond { asset, amount } => {
                if asset != config.token.address {
                    return Err(StdError::generic_err("Unrecognized Asset"));
                }

                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(deps.storage)?,
                    viewing_key.to_string(),
                    &config.token.clone(),
                )?;

                let mut unbondings = UNBONDINGS.load(deps.storage)?;
                let total_unbondings =
                    Uint128::new(unbondings.iter().map(|(amount, _)| amount.u128()).sum());

                let available = balance - total_unbondings;

                if available < amount || amount.is_zero() {
                    return Err(StdError::generic_err(format!(
                        "Cannot unbond {}, available is {}",
                        amount, available
                    )));
                }

                let mut messages = vec![];

                if config.unbond_blocks.is_zero() {
                    messages.push(send_msg(
                        config.owner.clone(),
                        amount,
                        None,
                        None,
                        None,
                        &config.token.clone(),
                    )?);
                } else {
                    unbondings.push((amount, Uint128::new(env.block.height as u128)));
                }

                Ok(Response::new().add_messages(messages).set_data(to_binary(
                    &adapter::ExecuteAnswer::Unbond {
                        status: ResponseStatus::Success,
                        amount,
                    },
                )?))
            }
            adapter::SubExecuteMsg::Claim { asset } => {
                let mut unbonding = UNBONDINGS.load(deps.storage)?;
                let mut remaining = vec![];
                let mut claimed = Uint128::zero();

                for u in unbonding {
                    if env.block.height as u128 - u.1.u128() >= config.unbond_blocks.u128() {
                        claimed += u.0;
                    } else {
                        remaining.push(u);
                    }
                }
                let mut messages = vec![send_msg(
                    config.owner.clone(),
                    claimed,
                    None,
                    None,
                    None,
                    &config.token.clone(),
                )?];

                Ok(Response::new().add_messages(messages).set_data(to_binary(
                    &adapter::ExecuteAnswer::Claim {
                        status: ResponseStatus::Success,
                        amount: claimed,
                    },
                )?))
            }
            adapter::SubExecuteMsg::Update { asset } => Ok(Response::new().set_data(to_binary(
                &adapter::ExecuteAnswer::Update {
                    status: ResponseStatus::Success,
                },
            )?)),
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    match msg {
        QueryMsg::Config => to_binary(&QueryAnswer::Config { config }),
        QueryMsg::Adapter(adapter) => to_binary(&match adapter {
            adapter::SubQueryMsg::Balance { asset } => {
                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(deps.storage)?,
                    viewing_key.to_string(),
                    &config.token.clone(),
                )?;
                let unbonding = Uint128::new(
                    UNBONDINGS
                        .load(deps.storage)?
                        .iter()
                        .map(|(amount, _)| amount.u128())
                        .sum(),
                );

                adapter::QueryAnswer::Balance {
                    amount: balance - unbonding,
                }
            }
            adapter::SubQueryMsg::Unbonding { asset } => {
                let last_block = BLOCK.load(deps.storage)?;
                adapter::QueryAnswer::Unbonding {
                    amount: Uint128::new(
                        UNBONDINGS
                            .load(deps.storage)?
                            .into_iter()
                            .map(|(amount, block)| {
                                if Uint128::new(last_block.u128() - block.u128())
                                    >= config.unbond_blocks
                                {
                                    0u128
                                } else {
                                    amount.u128()
                                }
                            })
                            .sum(),
                    ),
                }
            }
            adapter::SubQueryMsg::Claimable { asset } => {
                let last_block = BLOCK.load(deps.storage)?;
                adapter::QueryAnswer::Claimable {
                    amount: Uint128::new(
                        UNBONDINGS
                            .load(deps.storage)?
                            .into_iter()
                            .map(|(amount, block)| {
                                if Uint128::new(last_block.u128() - block.u128())
                                    >= config.unbond_blocks
                                {
                                    amount.u128()
                                } else {
                                    0u128
                                }
                            })
                            .sum(),
                    ),
                }
            }
            adapter::SubQueryMsg::Unbondable { asset } => {
                let unbondings = UNBONDINGS.load(deps.storage)?;
                let sum = Uint128::new(unbondings.iter().map(|(amount, _)| amount.u128()).sum());
                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(deps.storage)?,
                    viewing_key.to_string(),
                    &config.token.clone(),
                )?;

                adapter::QueryAnswer::Unbondable {
                    amount: balance - sum,
                }
            }
            adapter::SubQueryMsg::Reserves { asset } => {
                let mut reserves = Uint128::zero();
                let unbondings = Uint128::new(
                    UNBONDINGS
                        .load(deps.storage)?
                        .iter()
                        .map(|(amount, _)| amount.u128())
                        .sum(),
                );

                if config.unbond_blocks.is_zero() {
                    reserves = balance_query(
                        &deps.querier,
                        ADDRESS.load(deps.storage)?,
                        viewing_key.to_string(),
                        &config.token.clone(),
                    )?;
                    reserves = reserves - unbondings;
                }

                adapter::QueryAnswer::Reserves { amount: reserves }
            }
        }),
    }
}
