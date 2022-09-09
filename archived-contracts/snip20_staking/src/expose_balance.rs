
use shade_protocol::cosmwasm_schema::cw_serde;

use crate::{
    msg::{HandleAnswer, ResponseStatus::Success},
    state::{get_receiver_hash, Balances},
    state_staking::UserCooldown,
};
use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{
    to_binary,
    Api,
    Binary,
    CosmosMsg,
    Env,
    DepsMut,
    Response,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::{
    contract_interfaces::staking::snip20_staking::stake::VecQueue,
    utils::storage::default::BucketStorage,
};

pub fn try_expose_balance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    code_hash: Option<String>,
    msg: Option<Binary>,
    memo: Option<String>,
) -> StdResult<Response> {
    // Get balance to expose
    let balance = Balances::from_storage(deps.storage)
        .balance(&deps.api.canonical_address(&info.sender)?);

    let receiver_hash: String;
    if let Some(code_hash) = code_hash {
        receiver_hash = code_hash;
    } else if let Some(code_hash) = get_receiver_hash(deps.storage, &recipient) {
        receiver_hash = code_hash?;
    } else {
        return Err(StdError::generic_err("No code hash received"));
    }

    let messages = vec![
        Snip20BalanceReceiverMsg::new(info.sender, Uint128::new(balance), memo, msg)
            .to_cosmos_msg(receiver_hash, recipient)?,
    ];

    Ok(Response::new().set_data(to_binary(&HandleAnswer::ExposeBalance { status: Success })?))
}

pub fn try_expose_balance_with_cooldown(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    code_hash: Option<String>,
    msg: Option<Binary>,
    memo: Option<String>,
) -> StdResult<Response> {
    // Get balance to expose
    let balance = Balances::from_storage(deps.storage)
        .balance(&deps.api.canonical_address(&info.sender)?);

    let receiver_hash: String;
    if let Some(code_hash) = code_hash {
        receiver_hash = code_hash;
    } else if let Some(code_hash) = get_receiver_hash(deps.storage, &recipient) {
        receiver_hash = code_hash?;
    } else {
        return Err(StdError::generic_err("No code hash received"));
    }

    let mut cooldown =
        UserCooldown::may_load(deps.storage, info.sender.to_string().as_bytes())?
            .unwrap_or(UserCooldown {
                total: Uint128::zero(),
                queue: VecQueue(vec![]),
            });
    cooldown.update(env.block.time.seconds());
    cooldown.save(deps.storage, info.sender.to_string().as_bytes())?;

    let messages = vec![
        Snip20BalanceReceiverMsg::new(
            info.sender,
            Uint128::new(balance).checked_sub(cooldown.total)?,
            memo,
            msg,
        )
        .to_cosmos_msg_cooldown(receiver_hash, recipient)?,
    ];

    Ok(Response::new().set_data(to_binary(&HandleAnswer::ExposeBalance { status: Success })?))
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Snip20BalanceReceiverMsg {
    pub sender: Addr,
    pub balance: Uint128,
    pub memo: Option<String>,
    pub msg: Option<Binary>,
}

impl Snip20BalanceReceiverMsg {
    pub fn new(
        sender: Addr,
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

    pub fn to_cosmos_msg(self, code_hash: String, address: Addr) -> StdResult<CosmosMsg> {
        BalanceReceiverHandleMsg::ReceiveBalance(self).to_cosmos_msg(code_hash, address, None)
    }

    pub fn to_cosmos_msg_cooldown(
        self,
        code_hash: String,
        address: Addr,
    ) -> StdResult<CosmosMsg> {
        BalanceReceiverHandleMsg::ReceiveBalanceWithCooldown(self)
            .to_cosmos_msg(code_hash, address, None)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BalanceReceiverHandleMsg {
    ReceiveBalance(Snip20BalanceReceiverMsg),
    ReceiveBalanceWithCooldown(Snip20BalanceReceiverMsg),
}

impl ExecuteCallback for BalanceReceiverHandleMsg {
    const BLOCK_SIZE: usize = 256;
}
