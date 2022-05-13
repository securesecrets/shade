use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::HandleAnswer;
use crate::msg::ResponseStatus::Success;
use crate::state::{get_receiver_hash, Balances};
use crate::state_staking::UserCooldown;
use cosmwasm_std::{
    to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, Querier, StdError,
    StdResult, Storage,
};
use secret_toolkit::utils::HandleCallback;
use cosmwasm_math_compat::Uint128;
use shade_protocol::snip20_staking::stake::VecQueue;
use shade_protocol::utils::storage::default::BucketStorage;

pub fn try_expose_balance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: HumanAddr,
    code_hash: Option<String>,
    msg: Option<Binary>,
    memo: Option<String>,
) -> StdResult<HandleResponse> {
    // Get balance to expose
    let balance = Balances::from_storage(&mut deps.storage)
        .balance(&deps.api.canonical_address(&env.message.sender)?);

    let receiver_hash: String;
    if let Some(code_hash) = code_hash {
        receiver_hash = code_hash;
    } else if let Some(code_hash) = get_receiver_hash(&deps.storage, &recipient) {
        receiver_hash = code_hash?;
    } else {
        return Err(StdError::generic_err("No code hash received"));
    }

    let messages =
        vec![
            Snip20BalanceReceiverMsg::new(env.message.sender, Uint128::new(balance), memo, msg)
                .to_cosmos_msg(receiver_hash, recipient)?,
        ];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExposeBalance { status: Success })?),
    })
}

pub fn try_expose_balance_with_cooldown<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: HumanAddr,
    code_hash: Option<String>,
    msg: Option<Binary>,
    memo: Option<String>,
) -> StdResult<HandleResponse> {
    // Get balance to expose
    let balance = Balances::from_storage(&mut deps.storage)
        .balance(&deps.api.canonical_address(&env.message.sender)?);

    let receiver_hash: String;
    if let Some(code_hash) = code_hash {
        receiver_hash = code_hash;
    } else if let Some(code_hash) = get_receiver_hash(&deps.storage, &recipient) {
        receiver_hash = code_hash?;
    } else {
        return Err(StdError::generic_err("No code hash received"));
    }

    let mut cooldown =
        UserCooldown::may_load(&deps.storage, env.message.sender.to_string().as_bytes())?
            .unwrap_or(UserCooldown {
                total: Uint128::zero(),
                queue: VecQueue(vec![]),
            });
    cooldown.update(env.block.time);
    cooldown.save(&mut deps.storage, env.message.sender.to_string().as_bytes())?;

    let messages = vec![Snip20BalanceReceiverMsg::new(
        env.message.sender,
        Uint128::new(balance).checked_sub(cooldown.total)?,
        memo,
        msg,
    )
    .to_cosmos_msg_cooldown(receiver_hash, recipient)?];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExposeBalance { status: Success })?),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Snip20BalanceReceiverMsg {
    pub sender: HumanAddr,
    pub balance: Uint128,
    pub memo: Option<String>,
    pub msg: Option<Binary>,
}

impl Snip20BalanceReceiverMsg {
    pub fn new(
        sender: HumanAddr,
        balance: Uint128,
        memo: Option<String>,
        msg: Option<Binary>,
    ) -> Self {
        Self {
            sender,
            balance,
            memo,
            msg,
        }
    }

    pub fn to_cosmos_msg(self, code_hash: String, address: HumanAddr) -> StdResult<CosmosMsg> {
        BalanceReceiverHandleMsg::ReceiveBalance(self).to_cosmos_msg(code_hash, address, None)
    }

    pub fn to_cosmos_msg_cooldown(
        self,
        code_hash: String,
        address: HumanAddr,
    ) -> StdResult<CosmosMsg> {
        BalanceReceiverHandleMsg::ReceiveBalanceWithCooldown(self)
            .to_cosmos_msg(code_hash, address, None)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BalanceReceiverHandleMsg {
    ReceiveBalance(Snip20BalanceReceiverMsg),
    ReceiveBalanceWithCooldown(Snip20BalanceReceiverMsg),
}

impl HandleCallback for BalanceReceiverHandleMsg {
    const BLOCK_SIZE: usize = 256;
}
