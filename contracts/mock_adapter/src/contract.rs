use cosmwasm_std::{
    to_binary,
    Api,
    Binary,
    HumanAddr,
    Uint128,
    Env,
    Extern,
    HandleResponse,
    InitResponse,
    Querier,
    StdError,
    StdResult,
    Storage,
};

use secret_toolkit::snip20::{send_msg, balance_query, set_viewing_key_msg, register_receive_msg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::{
    contract_interfaces::dao::adapter,
    utils::{
        asset::Contract,
        generic_response::ResponseStatus,
    }
};
use secret_storage_plus::Item; 

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: HumanAddr,
    pub unbond_blocks: Uint128,
    pub token: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Adapter(adapter::SubHandleMsg),
}

const viewing_key: String = "jUsTfOrTeStInG".into();

const CONFIG: Item<Config> = Item::new("config");
const ADDRESS: Item<HumanAddr> = Item::new("address");
const BLOCK: Item<u64> = Item::new("block");

// (amount, block)
const UNBONDINGS: Item<Vec<(Uint128, Uint128)>> = Item::new("unbondings");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: Config,
) -> StdResult<InitResponse> {

    CONFIG.save(&mut deps.storage, &msg)?;
    ADDRESS.save(&mut deps.storage, &env.contract.address)?;
    UNBONDINGS.save(&mut deps.storage, vec![])?;
    BLOCK.save(&mut deps.storage, &env.block.height)?;

    Ok(InitResponse {
        messages: vec![
            //TODO
            //set_viewing_key_msg(),
            //register_receive_msg(),
        ],
        log: vec![],
    })
}


pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: adapter::HandleMsg,
) -> StdResult<HandleResponse> {

    let config = CONFIG.load(&deps.storage)?;
    BLOCK.save(&mut deps.storage, &env.block.height)?;

    match msg {
        adapter::HandleMsg::Adapter(adapter) => match adapter {
            adapter::SubHandleMsg::Unbond { asset, amount } => {
                if asset != config.token.address {
                    return Err(StdError::generic_err("Unrecognized Asset"));
                }

                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(&deps.storage)?,
                    viewing_key,
                    1,
                    config.token.code_hash.clone(),
                    config.token.address.clone(),
                )?
                .amount;

                let mut unbondings = UNBONDINGS.load(&deps.storage)?;

                let available = (balance - unbondings.iter().map(|(amount, _)| amount).sum())?;

                if available < amount || amount.is_zero() {
                    return Err(StdError::generic_err(format!("Cannot unbond {}, available is {}", amount, available)));
                }

                let mut messages = vec![];

                if config.unbond_blocks.is_zero() {
                    //TODO messages.push(send_msg()?);
                }
                else {
                    unbondings.push((amount, env.block.height));
                }

                Ok(HandleResponse {
                    messages,
                    log: vec![],
                    data: Some(to_binary(&adapter::HandleAnswer::Unbond {
                        status: ResponseStatus::Success,
                        amount,
                    })?),
                })
            },
            adapter::SubHandleMsg::Claim { asset } => {
                let mut unbonding = UNBONDINGS.load(&deps.storage)?;
                let mut remaining = vec![];
                let mut claimed = Uint128::zero();

                for u in unbonding {
                    if env.block.height - u.1.u128() >= config.unbond_blocks {
                        claimed += u.0;
                    }
                    else {
                        remaining.push(u);
                    }
                }
                let mut messages = vec![
                    //send_msg(),
                ];

                Ok(HandleResponse {
                    messages,
                    log: vec![],
                    data: Some(to_binary(&adapter::HandleAnswer::Claim {
                        status: ResponseStatus::Success,
                        amount: claimed,
                    })?),
                })
            },
            adapter::SubHandleMsg::Update { asset } => {
                Ok(HandleResponse {
                    messages: vec![],
                    log: vec![],
                    data: Some(to_binary(&adapter::HandleAnswer::Update {
                        status: ResponseStatus::Success,
                    })?),
                })
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config,
    Adapter(adapter::SubQueryMsg),
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {

    let config = CONFIG.load(&deps.storage)?;
    to_binary(&match msg {
        QueryMsg::Config => config,
        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => {

                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(&deps.storage)?,
                    viewing_key,
                    1,
                    config.token.code_hash.clone(),
                    config.token.address.clone(),
                )?
                .amount;
                let unbonding = UNBONDINGS.load(&deps.storage)?
                    .iter()
                    .map(|(amount, _)| amount)
                    .sum();

                adapter::QueryAnswer::Balance { amount: (balance - unbonding)? }
            },
            adapter::SubQueryMsg::Unbonding { asset } => {
                adapter::QueryAnswer::Unbonding { 
                    amount: UNBONDINGS.load(&deps.storage)?
                        .iter()
                        .map(|(amount, block)| -> {
                            if BLOCK.load(&deps.storage)? - block >= config.unbond_blocks {
                                Uint128::zero()
                            }
                            else {
                                amount
                            }
                        })
                        .sum(),
                }
            },
            adapter::SubQueryMsg::Claimable { asset } => {
                adapter::QueryAnswer::Claimable { 
                    amount: UNBONDINGS.load(&deps.storage)?
                        .iter()
                        .map(|(amount, block)| -> {
                            if BLOCK.load(&deps.storage)? - block >= config.unbond_blocks {
                                amount
                            }
                            else {
                                Uint128::zero()
                            }
                        })
                        .sum(),
                }
            },
            adapter::SubQueryMsg::Unbondable { asset } => {
                let unbondings = UNBONDINGS.load(&deps.storage)?;
                let sum = unbondings.iter().map(|(amount, _)| amount).sum();
                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(&deps.storage)?,
                    viewing_key,
                    1,
                    config.token.code_hash.clone(),
                    config.token.address.clone(),
                )?
                .amount;

                adapter::QueryAnswer::Unbondable { amount: (balance - sum)? }
            },
            adapter::SubQueryMsg::Reserves { asset } => {
                let mut reserves = Uint128::zero();
                let unbondings = UNBONDINGS.load(&deps.storage)?.iter()
                    .map(|(amount, _)| amount)
                    .sum();

                if config.unbond_blocks.is_zero() {
                    reserves = balance_query(
                        &deps.querier,
                        ADDRESS.load(&deps.storage)?,
                        viewing_key,
                        1,
                        config.token.code_hash.clone(),
                        config.token.address.clone(),
                    )?
                    .amount;
                    reserves = (reserves - unbondings)?;
                }

                adapter::QueryAnswer::Reserves { amount: reserves }
            },
        }
    })
}
