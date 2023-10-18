//! ### Liquidity Book token Helper Library
//! Author: Haseeb
//!

use crate::utils::liquidity_book::transfer::{self, HandleMsg, QueryAnswer, QueryMsg};

use std::fmt::{Display, Formatter, Result};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary,
    Addr,
    BankMsg,
    Coin,
    ContractInfo,
    CosmosMsg,
    Deps,
    MessageInfo,
    QuerierWrapper,
    QueryRequest,
    StdError,
    StdResult,
    Uint128,
    WasmMsg,
    WasmQuery,
};

#[cw_serde]
pub struct SwapTokenAmount {
    pub token: TokenType,
    pub amount: Uint128,
}

#[cw_serde]
pub enum TokenType {
    CustomToken {
        contract_addr: Addr,
        token_code_hash: String,
    },
    NativeToken {
        denom: String,
    },
}

impl TokenType {
    // pub fn query_decimals(&self, deps: &Deps) -> StdResult<u8> {
    //     match self {
    //         TokenType::CustomToken {
    //             contract_addr,
    //             token_code_hash,
    //             ..
    //         } => Ok(token_info(&deps.querier, &Contract {
    //             address: contract_addr.clone(),
    //             code_hash: token_code_hash.clone(),
    //         })?
    //         .decimals),
    //         TokenType::NativeToken { denom } => match denom.as_str() {
    //             "uscrt" => Ok(6),
    //             _ => Err(StdError::generic_err(
    //                 "Cannot retrieve decimals for native token",
    //             )),
    //         },
    //     }
    // }

    pub fn is_native_token(&self) -> bool {
        match self {
            TokenType::NativeToken { .. } => true,
            TokenType::CustomToken { .. } => false,
        }
    }

    pub fn unique_key(&self) -> String {
        match self {
            TokenType::NativeToken { denom } => denom.to_string(),
            TokenType::CustomToken {
                contract_addr,
                token_code_hash: _,
            } => contract_addr.to_string(),
        }
    }

    pub fn address(&self) -> Addr {
        match self {
            TokenType::NativeToken { .. } => panic!("Doesn't work for native tokens"),
            TokenType::CustomToken {
                contract_addr,
                token_code_hash: _,
            } => contract_addr.clone(),
        }
    }

    pub fn code_hash(&self) -> String {
        match self {
            TokenType::NativeToken { .. } => panic!("Doesn't work for native tokens"),
            TokenType::CustomToken {
                contract_addr: _,
                token_code_hash,
            } => token_code_hash.to_string(),
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
        if let TokenType::NativeToken { denom } = &self {
            return match info.funds.iter().find(|x| x.denom == *denom) {
                Some(coin) => {
                    if amount == coin.amount {
                        Ok(())
                    } else {
                        Err(StdError::generic_err(
                            "Native token balance mismatch between the argument and the transferred",
                        ))
                    }
                }
                None => {
                    if amount.is_zero() {
                        Ok(())
                    } else {
                        Err(StdError::generic_err(
                            "Native token balance mismatch between the argument and the transferred",
                        ))
                    }
                }
            };
        }

        Ok(())
    }

    pub fn transfer(&self, amount: Uint128, recipient: Addr) -> Option<CosmosMsg> {
        if amount.gt(&Uint128::zero()) {
            match &self {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } => {
                    let msg = HandleMsg::Send {
                        recipient: recipient.to_string(),
                        amount,
                        padding: None,
                        msg: None,
                        recipient_code_hash: None,
                        memo: None,
                    };
                    // //TODO add token hash
                    let cosmos_msg = msg
                        .to_cosmos_msg(token_code_hash.to_string(), contract_addr.to_string(), None)
                        .unwrap();

                    Some(cosmos_msg)
                }

                TokenType::NativeToken { denom } => Some(CosmosMsg::Bank(BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![Coin {
                        denom: denom.clone(),
                        amount,
                    }],
                })),
            }
        } else {
            None
        }
    }

    pub fn transfer_from(
        &self,
        amount: Uint128,
        owner: Addr,
        recipient: Addr,
    ) -> Option<CosmosMsg> {
        if amount.gt(&Uint128::zero()) {
            match &self {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } => {
                    let msg = HandleMsg::TransferFrom {
                        owner: owner.to_string(),
                        recipient: recipient.to_string(),
                        amount,
                        padding: None,
                        memo: None,
                    };

                    // //TODO add token hash
                    let cosmos_msg = msg
                        .to_cosmos_msg(token_code_hash.to_string(), contract_addr.to_string(), None)
                        .unwrap();

                    Some(cosmos_msg)
                }

                TokenType::NativeToken { denom } => Some(CosmosMsg::Bank(BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![Coin {
                        denom: denom.clone(),
                        amount,
                    }],
                })),
            }
        } else {
            None
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
            TokenType::NativeToken { denom } => {
                let result = deps.querier.query_balance(exchange_addr, denom)?;
                Ok(result.amount)
            }
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => balance_query(
                &deps.querier,
                deps.api.addr_validate(&exchange_addr)?,
                viewing_key,
                &ContractInfo {
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
            } => CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.clone().into_string(),
                code_hash: token_code_hash.to_string(),
                msg: to_binary(&transfer::HandleMsg::Send {
                    recipient,
                    amount,
                    padding: None,
                    msg: None,
                    recipient_code_hash: None,
                    memo: None,
                })?,
                funds: vec![],
            }),
            TokenType::NativeToken { denom } => CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient,
                amount: vec![Coin {
                    denom: denom.clone(),
                    amount,
                }],
            }),
        };
        Ok(msg)
    }
}

/// Returns a StdResult<Uint128> from performing a Balance query
pub fn balance_query(
    querier: &QuerierWrapper,
    address: Addr,
    key: String,
    contract: &ContractInfo,
) -> StdResult<Uint128> {
    let msg: QueryMsg = QueryMsg::Balance {
        address: address.to_string(),
        key,
    };

    let result: crate::utils::liquidity_book::transfer::QueryAnswer =
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.address.to_string(),
            code_hash: contract.code_hash.clone(),
            msg: to_binary(&msg)?,
        }))?;

    let QueryAnswer::Balance { amount, .. } = result;
    Ok(amount)
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            TokenType::NativeToken { denom, .. } => write!(f, "{}", denom),
            TokenType::CustomToken { contract_addr, .. } => write!(f, "{}", contract_addr),
        }
    }
}

#[cw_serde]
pub struct TokenAmount {
    pub token: TokenType,
    pub amount: Uint128,
}

impl TokenAmount {
    pub fn assert_sent_native_token_balance(&self, info: &MessageInfo) -> StdResult<()> {
        self.token
            .assert_sent_native_token_balance(info, self.amount)
    }
}
