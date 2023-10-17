use std::collections::HashMap;

use anyhow::{anyhow, Result as AnyResult};
use cosmwasm_std::{
    from_slice, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, Uint128,
};
use cw_multi_test::Contract;

use crate::msg::{ControllerQuery, TransferableAmountResp};

/// Controller contract stub allowing to easy testing the transfer without actual controller
/// contract
#[derive(Default)]
pub struct Controller {
    /// Mapping for what can be transferred. Map key is an account, the value is how much amount
    /// can be transferred from this account.
    allowances: HashMap<String, Uint128>,
}

impl Controller {
    pub fn new(allowances: impl Into<HashMap<String, Uint128>>) -> Self {
        Self {
            allowances: allowances.into(),
        }
    }

    fn transferable(&self, account: &str) -> TransferableAmountResp {
        TransferableAmountResp {
            transferable: self.allowances.get(account).cloned().unwrap_or_default(),
        }
    }
}

impl Contract<Empty> for Controller {
    fn instantiate(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Vec<u8>,
    ) -> anyhow::Result<Response<Empty>> {
        Ok(Response::default())
    }

    fn execute(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Vec<u8>,
    ) -> AnyResult<Response<Empty>> {
        Err(anyhow!("Controller stub execution"))
    }

    fn query(&self, _deps: Deps, _env: Env, msg: Vec<u8>) -> anyhow::Result<Binary> {
        use ControllerQuery::*;

        let msg: ControllerQuery = from_slice(&msg)?;

        match msg {
            TransferableAmount { account, .. } => {
                to_binary(&self.transferable(&account)).map_err(Into::into)
            }
        }
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
