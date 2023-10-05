use crate::amm_pair;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, ContractInfo, CosmosMsg, Deps, MessageInfo, StdError,
    StdResult, Uint128, Uint256, WasmMsg,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::snip20::{
    helpers::{balance_query, token_info},
    ExecuteMsg::Send,
};
use shade_protocol::Contract;

use super::TokenAmount;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    CustomToken {
        contract_addr: Addr,
        token_code_hash: String,
    },
    NativeToken {
        denom: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StableTokenData {
    pub oracle_key: String,
    pub decimals: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StableTokenType {
    pub token: TokenType,
    pub stable_token_data: StableTokenData,
}

impl TokenType {
    pub fn query_decimals(&self, deps: &Deps) -> StdResult<u8> {
        match self {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
                ..
            } => Ok(token_info(
                &deps.querier,
                &Contract {
                    address: contract_addr.clone(),
                    code_hash: token_code_hash.clone(),
                },
            )?
            .decimals),
            TokenType::NativeToken { denom } => match denom.as_str() {
                "uscrt" => Ok(6),
                _ => Err(StdError::generic_err(
                    "Cannot retrieve decimals for native token",
                )),
            },
        }
    }

    pub fn load_stable_data(&self, stable_token_data: StableTokenData) -> StableTokenType {
        StableTokenType {
            token: self.clone(),
            stable_token_data,
        }
    }

    pub fn is_native_token(&self) -> bool {
        match self {
            TokenType::NativeToken { .. } => true,
            TokenType::CustomToken { .. } => false,
        }
    }
    pub fn unique_key(&self) -> String {
        match self {
            TokenType::NativeToken { denom, .. } => denom.to_string(),
            TokenType::CustomToken { contract_addr, .. } => contract_addr.to_string(),
        }
    }
    pub fn is_custom_token(&self) -> bool {
        match self {
            TokenType::NativeToken { .. } => false,
            TokenType::CustomToken { .. } => true,
        }
    }

    pub fn assert_sent_native_token_balance(
        &self,
        info: &MessageInfo,
        amount: Uint128,
    ) -> StdResult<()> {
        if let TokenType::NativeToken { denom, .. } = &self {
            return match info.funds.iter().find(|x| x.denom == *denom) {
                Some(coin) => {
                    if amount == coin.amount {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance mismatch between the argument and the transferred"))
                    }
                }
                None => {
                    if amount.is_zero() {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance mismatch between the argument and the transferred"))
                    }
                }
            };
        }

        Ok(())
    }

    pub fn new_amount(&self, amount: impl Into<Uint128> + Copy) -> TokenAmount {
        TokenAmount {
            token: self.clone(),
            amount: amount.into(),
        }
    }
}

impl From<Contract> for TokenType {
    fn from(value: Contract) -> Self {
        Self::CustomToken {
            contract_addr: value.address,
            token_code_hash: value.code_hash,
        }
    }
}

impl From<ContractInfo> for TokenType {
    fn from(value: ContractInfo) -> Self {
        Self::CustomToken {
            contract_addr: value.address,
            token_code_hash: value.code_hash,
        }
    }
}

impl TokenType {
    pub fn query_balance(
        &self,
        deps: Deps,
        exchange_addr: String,
        viewing_key: String,
    ) -> StdResult<Uint128> {
        match self {
            TokenType::NativeToken { denom, .. } => {
                let result = deps.querier.query_balance(exchange_addr, denom)?;
                Ok(result.amount)
            }
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
                ..
            } => balance_query(
                &deps.querier,
                deps.api.addr_validate(&exchange_addr)?,
                viewing_key,
                &Contract {
                    address: contract_addr.clone(),
                    code_hash: token_code_hash.clone(),
                },
            ),
        }
    }

    pub fn create_send_msg(&self, recipient: String, amount: Uint128) -> StdResult<CosmosMsg> {
        let msg = match self {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
                ..
            } => CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.clone().into_string(),
                code_hash: token_code_hash.to_string(),
                msg: to_binary(&Send {
                    recipient,
                    amount,
                    padding: None,
                    msg: None,
                    recipient_code_hash: None,
                    memo: None,
                })?,
                funds: vec![],
            }),
            TokenType::NativeToken { denom, .. } => CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient,
                amount: vec![Coin {
                    denom: denom.clone(),
                    amount,
                }],
            }),
        };
        Ok(msg)
    }

    pub fn into_contract_info(&self) -> Option<ContractInfo> {
        match self {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
                ..
            } => Some(ContractInfo {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            }),
            TokenType::NativeToken { .. } => None,
        }
    }
}
