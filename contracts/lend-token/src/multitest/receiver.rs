use anyhow::{anyhow, bail, Result as AnyResult};
use serde::{Deserialize, Serialize};
use shade_protocol::{
    c_std::{
        from_binary, from_slice, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
        Response, StdResult, Uint128,
    },
    contract_interfaces::snip20::Snip20ReceiveMsg,
    multi_test::Contract,
    secret_storage_plus::Item,
};

/// Cw20 Execute message
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Cw20ExecMsg {
    Valid,
    Invalid,
}

/// Execute message
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecMsg {
    Receive(Cw20ReceiveMsg),
}

/// Response for any query message
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub struct QueryResp {
    pub counter: Uint128,
}

/// Contract receiving Cw20Receive messages and counting them. Querying the contract with
/// anything gives back amount of valid messages received.
pub struct Receiver {
    /// Proper messages counter
    counter: Item<'static, Uint128>,
}

impl Receiver {
    pub fn new() -> Self {
        Self {
            counter: Item::new("counter"),
        }
    }
}

impl Contract<Empty> for Receiver {
    fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Vec<u8>,
    ) -> anyhow::Result<Response<Empty>> {
        self.counter.save(deps.storage, &Uint128::zero())?;
        Ok(Response::default())
    }

    fn execute(
        &self,
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<Response<Empty>> {
        use Cw20ExecMsg::*;
        use ExecMsg::*;

        let msg: ExecMsg = from_slice(&msg)?;
        let msg: Cw20ExecMsg = match msg {
            Receive(msg) => from_binary(&msg.msg)?,
        };

        match msg {
            Valid {} => self.counter.update(deps.storage, |cnt| -> StdResult<_> {
                Ok(cnt + Uint128::new(1))
            })?,
            Invalid {} => bail!("Invalid message on receiver"),
        };

        Ok(Response::new())
    }

    fn query(&self, deps: Deps, _env: Env, _msg: Vec<u8>) -> anyhow::Result<Binary> {
        to_binary(&QueryResp {
            counter: self.counter.load(deps.storage)?,
        })
        .map_err(Into::into)
    }

    fn sudo(&self, _deps: DepsMut, _env: Env, _msg: Vec<u8>) -> anyhow::Result<Response<Empty>> {
        Err(anyhow!("Controller stub sudo"))
    }

    fn migrate(&self, _deps: DepsMut, _env: Env, _msg: Vec<u8>) -> AnyResult<Response<Empty>> {
        Err(anyhow!("Controller stub migrate"))
    }

    fn reply(&self, _deps: DepsMut, _env: Env, _msg: Reply) -> anyhow::Result<Response<Empty>> {
        Err(anyhow!("Controller stub reply"))
    }
}
