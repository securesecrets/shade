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
use serde::{Deserialize, Serialize};
use shade_protocol::{
    contract_interfaces::dao::adapter,
    utils::{
        asset::Contract,
        generic_response::ResponseStatus,
    },
    schemars::JsonSchema,
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
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    //RegisterAsset { token: Contract },
    //CompleteUnbond { token: HumanAddr, amount: Uint128 },
    Adapter(adapter::SubHandleMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config,
    Adapter(adapter::SubQueryMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    Adapter(adapter::SubQueryMsg),
}

const viewing_key: &str = "jUsTfOrTeStInG";

const CONFIG: Item<Config> = Item::new("config");
const ADDRESS: Item<HumanAddr> = Item::new("address");
const BLOCK: Item<Uint128> = Item::new("block");
const REWARDS: Item<Uint128> = Item::new("rewards");

// (amount, block)
const UNBONDINGS: Item<Vec<(Uint128, Uint128)>> = Item::new("unbondings");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: Config,
) -> StdResult<InitResponse> {

    CONFIG.save(&mut deps.storage, &msg)?;
    ADDRESS.save(&mut deps.storage, &env.contract.address)?;
    UNBONDINGS.save(&mut deps.storage, &Vec::new())?;
    BLOCK.save(&mut deps.storage, &Uint128(env.block.height as u128))?;

    Ok(InitResponse {
        messages: vec![
            //TODO
            set_viewing_key_msg(
                viewing_key.to_string(),
                None,
                256,
                msg.token.code_hash.clone(),
                msg.token.address.clone(),
            )?,
            register_receive_msg(
                env.contract_code_hash.clone(),
                None,
                256,
                msg.token.code_hash.clone(),
                msg.token.address.clone(),
            )?,
        ],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {

    let config = CONFIG.load(&deps.storage)?;
    BLOCK.save(&mut deps.storage, &Uint128(env.block.height as u128))?;

    match msg {
        HandleMsg::Receive {
            sender, from, amount,
            memo, msg,
        } => {
            if env.message.sender != config.token.address {
                return Err(StdError::generic_err("Unrecognized Asset"));
            }

            /*
            match from_binary(&msg)? {
                // add rewards
            }
            */
            Ok(HandleResponse {
                messages: vec![],
                log: vec![],
                data: None,
                /*
                Some(to_binary(&adapter::HandleAnswer::Unbond {
                    status: ResponseStatus::Success,
                    amount,
                })?),
                */
            })
        },
        HandleMsg::Adapter(adapter) => match adapter {
            adapter::SubHandleMsg::Unbond { asset, amount } => {
                if asset != config.token.address {
                    return Err(StdError::generic_err("Unrecognized Asset"));
                }

                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(&deps.storage)?,
                    viewing_key.to_string(),
                    1,
                    config.token.code_hash.clone(),
                    config.token.address.clone(),
                )?
                .amount;

                let mut unbondings = UNBONDINGS.load(&deps.storage)?;
                let total_unbondings = Uint128(unbondings.iter().map(|(amount, _)| amount.u128()).sum());

                let available = (balance - total_unbondings)?;

                if available < amount || amount.is_zero() {
                    return Err(StdError::generic_err(format!("Cannot unbond {}, available is {}", amount, available)));
                }

                let mut messages = vec![];

                if config.unbond_blocks.is_zero() {
                    messages.push(send_msg(
                        config.owner.clone(),
                        amount,
                        None,
                        None,
                        None,
                        1,
                        config.token.code_hash.clone(),
                        config.token.address.clone(),
                    )?);
                }
                else {
                    unbondings.push((amount, Uint128(env.block.height as u128)));
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
                    if env.block.height as u128 - u.1.u128() >= config.unbond_blocks.u128() {
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


pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {

    let config = CONFIG.load(&deps.storage)?;

    match msg {
        QueryMsg::Config => to_binary(&QueryAnswer::Config { config }),
        QueryMsg::Adapter(adapter) => to_binary(&match adapter {
            adapter::SubQueryMsg::Balance { asset } => {

                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(&deps.storage)?,
                    viewing_key.to_string(),
                    1,
                    config.token.code_hash.clone(),
                    config.token.address.clone(),
                )?
                .amount;
                let unbonding = Uint128(UNBONDINGS.load(&deps.storage)?
                    .iter()
                    .map(|(amount, _)| amount.u128())
                    .sum());

                adapter::QueryAnswer::Balance { amount: (balance - unbonding)? }
            },
            adapter::SubQueryMsg::Unbonding { asset } => {
                let last_block = BLOCK.load(&deps.storage)?;
                adapter::QueryAnswer::Unbonding { 
                    amount: Uint128(UNBONDINGS.load(&deps.storage)?
                        .into_iter()
                        .map(|(amount, block)| {
                            if Uint128(last_block.u128() - block.u128()) >= config.unbond_blocks {
                                0u128
                            }
                            else {
                                amount.u128()
                            }
                        })
                        .sum()),
                }
            },
            adapter::SubQueryMsg::Claimable { asset } => {
                let last_block = BLOCK.load(&deps.storage)?;
                adapter::QueryAnswer::Claimable { 
                    amount: Uint128(UNBONDINGS.load(&deps.storage)?
                        .into_iter()
                        .map(|(amount, block)| {
                            if Uint128(last_block.u128() - block.u128()) >= config.unbond_blocks {
                                amount.u128()
                            }
                            else {
                                0u128
                            }
                        })
                        .sum()),
                }
            },
            adapter::SubQueryMsg::Unbondable { asset } => {
                let unbondings = UNBONDINGS.load(&deps.storage)?;
                let sum = Uint128(unbondings.iter().map(|(amount, _)| amount.u128()).sum());
                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(&deps.storage)?,
                    viewing_key.to_string(),
                    1,
                    config.token.code_hash.clone(),
                    config.token.address.clone(),
                )?
                .amount;

                adapter::QueryAnswer::Unbondable { amount: (balance - sum)? }
            },
            adapter::SubQueryMsg::Reserves { asset } => {
                let mut reserves = Uint128::zero();
                let unbondings = Uint128(UNBONDINGS.load(&deps.storage)?.iter()
                    .map(|(amount, _)| amount.u128())
                    .sum());

                if config.unbond_blocks.is_zero() {
                    reserves = balance_query(
                        &deps.querier,
                        ADDRESS.load(&deps.storage)?,
                        viewing_key.to_string(),
                        1,
                        config.token.code_hash.clone(),
                        config.token.address.clone(),
                    )?
                    .amount;
                    reserves = (reserves - unbondings)?;
                }

                adapter::QueryAnswer::Reserves { amount: reserves }
            },
        })
    }
}
