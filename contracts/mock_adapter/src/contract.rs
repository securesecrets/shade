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
    storage::plus::Item,
};

use secret_toolkit::snip20::{send_msg, balance_query, set_viewing_key_msg, register_receive_msg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::{
    contract_interfaces::dao::adapter,
    utils::asset::Contract,
};

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
    Adapter(adapter::SubHandleMsg
}

const viewing_key: String = "jUsTfOrTeStInG";

const CONFIG: Item<Config> = Item::new("config");
const UNBONDINGS: Item<Vec<(HumanAddr, Uint128)>> = Item::new("unbondings");
const CLAIMABLE: Item<Vec<(HumanAddr, Uint128)>> = Item::new("claimable");
const ADDRESS: Item<HumanAddr> = Item::new("address");

pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: Config,
) -> StdResult<InitResponse> {

    CONFIG.save(&mut deps.storage, &msg)?;
    ADDRESS.save(&mut deps.storage, &env.contract.address)?;

    Ok(InitResponse {
        messages: vec![
            set_viewing_key_msg(),
            register_receive_msg(),
        ],
        log: vec![],
    })
}


pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: adapter::HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        adapter::HandleMsg::Adapter(adapter) => match adapter {
            adapter::SubHandleMsg::Unbond { asset, amount } => {},
            adapter::SubHandleMsg::Claim { asset } => {},
            adapter::SubHandleMsg::Update { asset } => {},
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
    match msg {
        QueryMsg::Config => Ok(config),
        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => {
                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(&deps.storage)?,
                    VIEWING_KEY.load(&deps.storage)?,
                    1,
                    config.token.code_hash.clone(),
                    config.token.address.clone(),
                )?
                .amount;
                let unbonding = UNBONDING.load(&deps.storage)?
                    .iter()
                    .map(|(_, amount)| amount)
                    .sum();
                let claimable = CLAIMABLE.load(&deps.storage)?
                    .iter()
                    .map(|(_, amount)| amount)
                    .sum();

                Ok(adapter::QueryAnswer::Balance { amount: (balance - (unbonding + claimable))? })
            },
            adapter::SubQueryMsg::Unbonding { asset } => {
                Ok(adapter::QueryAnswer::Unbonding { 
                    amount: UNBONDING.load(&deps.storage)?
                        .iter()
                        .map(|(_, amount)| amount)
                        .sum();
                })
            },
            adapter::SubQueryMsg::Claimable { asset } => {
                Ok(adapter::QueryAnswer::Claimable { 
                    amount: CLAIMABLE.load(&deps.storage)?
                        .iter()
                        .map(|(_, amount)| amount)
                        .sum();
                })
            },
            adapter::SubQueryMsg::Unbondable { asset } => {
                let unbondings = UNBONDINGS.load(&deps.storage)?;
                let sum = unbondings.iter().map(|(_, amount)| amount).sum();
                let balance = balance_query(
                    &deps.querier,
                    ADDRESS.load(&deps.storage)?,
                    VIEWING_KEY.load(&deps.storage)?,
                    1,
                    config.token.code_hash.clone(),
                    config.token.address.clone(),
                )?
                .amount;

                Ok(adapter::QueryAnswer::Unbondable { amount: (balance - sum)? })
            },
            adapter::SubQueryMsg::Reserves { asset } => {
                let mut reserves = Uint128::zero();

                if config.unbond_blocks.is_zero() {
                    reserves = balance_query(
                        &deps.querier,
                        ADDRESS.load(&deps.storage)?,
                        VIEWING_KEY.load(&deps.storage)?,
                        1,
                        config.token.code_hash.clone(),
                        config.token.address.clone(),
                    )?
                    .amount;
                }

                Ok(adapter::QueryAnswer::Reserves { amount: reserves })
            },
        }
    }

}
