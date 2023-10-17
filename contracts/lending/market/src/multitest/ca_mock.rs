// Credit Agency's contract mock
// Created to avoid circular dependency between market and CA contracts.
// Contains additional ExecuteMsg::SetCreditLine functionality, which sets
// response to TotalCreditLine query.

use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult,
};
use cw_multi_test::{Contract, ContractWrapper};
use cw_storage_plus::Map;
use utils::credit_line::CreditLineResponse;

pub const CLR: Map<&Addr, CreditLineResponse> = Map::new("clr");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantiateMsg {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Sets CreditLineResponse for address taken from info.sender
    SetCreditLine { credit_line: CreditLineResponse },
    /// Stud
    EnterMarket { account: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    TotalCreditLine { account: String },
}

fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, StdError> {
    Ok(Response::default())
}

fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    match msg {
        ExecuteMsg::SetCreditLine { credit_line } => {
            CLR.update(deps.storage, &info.sender, |_| -> StdResult<_> {
                Ok(credit_line)
            })?;
        }
        ExecuteMsg::EnterMarket { .. } => {}
    }

    Ok(Response::new())
}

fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, StdError> {
    match msg {
        QueryMsg::TotalCreditLine { account } => {
            to_binary(&CLR.load(deps.storage, &Addr::unchecked(account))?)
        }
    }
}

pub fn contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}
