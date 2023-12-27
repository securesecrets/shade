use cosmwasm_std::{
    to_binary,
    Addr,
    BankMsg,
    Coin,
    ContractInfo,
    CosmosMsg,
    Deps,
    MessageInfo,
    StdError,
    StdResult,
    Uint128,
    Uint256,
    WasmMsg,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
// use shade_oracles::querier::{query_price, query_prices};
use crate::{
    snip20::{
        helpers::{balance_query, token_info},
        ExecuteMsg::{Send, TransferFrom},
    },
    swap::amm_pair,
    utils::ExecuteCallback,
    Contract,
};

use super::TokenAmount;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
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

// impl StableTokenType {
//     pub fn query_price(&self, deps: Deps, oracle: Contract) -> StdResult<Uint256> {
//         self.stable_token_data.query_price(deps, oracle)
//     }
// }

// pub fn query_two_prices(
//     deps: Deps,
//     oracle: Contract,
//     oracle_key0: String,
//     oracle_key1: String,
// ) -> StdResult<[Uint256; 2]> {
//     let shade_oracle_contract: shade_protocol::Contract = shade_protocol::Contract {
//         address: oracle.address,
//         code_hash: oracle.code_hash,
//     };
//     let res = query_prices(
//         &shade_oracle_contract,
//         &deps.querier,
//         &[oracle_key0, oracle_key1],
//     )?;
//     Ok([res[0].data.rate, res[1].data.rate])
// }
//
// impl StableTokenData {
//     pub fn query_price(&self, deps: Deps, oracle: Contract) -> StdResult<Uint256> {
//         let shade_oracle_contract: shade_protocol::Contract = shade_protocol::Contract {
//             address: oracle.address,
//             code_hash: oracle.code_hash,
//         };
//         let res = query_price(&shade_oracle_contract, &deps.querier, &self.oracle_key)?;
//         Ok(res.data.rate)
//     }
// }

impl TokenType {
    pub fn query_decimals(&self, deps: &Deps) -> StdResult<u8> {
        match self {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
                ..
            } => Ok(token_info(&deps.querier, &Contract {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            })?
            .decimals),
            TokenType::NativeToken { denom } => match denom.as_str() {
                "uscrt" => Ok(6),
                _ => Err(StdError::generic_err(
                    "Cannot retrieve decimals for native token",
                )),
            },
        }
    }

    // pub fn load_stable_data(&self, stable_token_data: StableTokenData) -> StableTokenType {
    //     StableTokenType {
    //         token: self.clone(),
    //         stable_token_data,
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

// New Methods from LB
impl TokenType {
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

    pub fn transfer(&self, amount: Uint128, recipient: Addr) -> Option<CosmosMsg> {
        if amount.gt(&Uint128::zero()) {
            match &self {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } => {
                    let msg = Send {
                        recipient: recipient.to_string(),
                        amount,
                        padding: None,
                        msg: None,
                        recipient_code_hash: None,
                        memo: None,
                    };
                    let contract: ContractInfo = ContractInfo {
                        address: self.address(),
                        code_hash: self.code_hash(),
                    };
                    let cosmos_msg = msg.to_cosmos_msg(&contract, vec![]).unwrap();

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
                    let msg = TransferFrom {
                        owner: owner.to_string(),
                        recipient: recipient.to_string(),
                        amount,
                        padding: None,
                        memo: None,
                    };
                    let contract: ContractInfo = ContractInfo {
                        address: self.address(),
                        code_hash: self.code_hash(),
                    };
                    let cosmos_msg = msg.to_cosmos_msg(&contract, vec![]).unwrap();

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
