use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::serialization::{Bincode2, Serde};
use std::fmt;

//use crate::querier::{query_balance, query_token_balance};
// use cw20::Cw20HandleMsg;
// use terra_cosmwasm::TerraQuerier;

use cosmwasm_std::{
    to_binary, Api, BankMsg, CanonicalAddr, Coin, CosmosMsg, Decimal, Env, Extern, HumanAddr,
    Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};

//Temporary solution
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenContract {
    contract_addr: HumanAddr,
    callback_code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenContractRaw {
    contract_addr: CanonicalAddr,
    callback_code_hash: String,
}

impl TokenContractRaw {
    pub fn as_bytes(&self) -> Vec<u8> {
        Bincode2::serialize(self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Asset {
    pub info: AssetInfo,
    pub amount: Uint128,
}


impl Asset {
    pub fn is_native_token(&self) -> bool {
        self.info.is_native_token()
    }

    pub fn assert_sent_native_token_balance(&self, env: &Env) -> StdResult<()> {
        if let AssetInfo::NativeToken { denom } = &self.info {
            match env.message.sent_funds.iter().find(|x| x.denom == *denom) {
                Some(coin) => {
                    if self.amount == coin.amount {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance missmatch between the argument and the transferred"))
                    }
                }
                None => {
                    if self.amount.is_zero() {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance missmatch between the argument and the transferred"))
                    }
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn to_raw<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>,
    ) -> StdResult<AssetRaw> {
        Ok(AssetRaw {
            info: match &self.info {
                AssetInfo::NativeToken { denom } => AssetInfoRaw::NativeToken {
                    denom: denom.to_string(),
                },
                AssetInfo::Token { contract } => AssetInfoRaw::Token { contract: TokenContractRaw {
                    contract_addr: deps.api.canonical_address(&contract.contract_addr)?,
                    callback_code_hash: contract.callback_code_hash.to_owned(),
                }},
            },
            amount: self.amount,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetInfo {
    Token { contract: TokenContract },
    NativeToken { denom: String },
}

impl AssetInfo {
    pub fn to_raw<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>,
    ) -> StdResult<AssetInfoRaw> {
        match self {
            AssetInfo::NativeToken { denom } => Ok(AssetInfoRaw::NativeToken {
                denom: denom.to_string(),
            }),
            AssetInfo::Token { contract } => Ok(AssetInfoRaw::Token { contract: TokenContractRaw {
                contract_addr: deps.api.canonical_address(&contract.contract_addr)?,
                callback_code_hash: contract.callback_code_hash.to_owned(),
            }}),
        }
    }

    pub fn is_native_token(&self) -> bool {
        match self {
            AssetInfo::NativeToken { .. } => true,
            AssetInfo::Token { .. } => false,
        }
    }

    pub fn equal(&self, asset: &AssetInfo) -> bool {
        match self {
            AssetInfo::Token { contract: self_contract } => {
                //let self_contract_addr = contract.contract_addr;
                match asset {
                    AssetInfo::Token { contract } => self_contract.contract_addr == contract.contract_addr,
                    AssetInfo::NativeToken { .. } => false,
                }
            }
            AssetInfo::NativeToken { denom, .. } => {
                let self_denom = denom;
                match asset {
                    AssetInfo::Token { .. } => false,
                    AssetInfo::NativeToken { denom, .. } => self_denom == denom,
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetRaw {
    pub info: AssetInfoRaw,
    pub amount: Uint128,
}

impl AssetRaw {
    pub fn to_normal<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>,
    ) -> StdResult<Asset> {
        Ok(Asset {
            info: match &self.info {
                AssetInfoRaw::NativeToken { denom } => AssetInfo::NativeToken {
                    denom: denom.to_string(),
                },
                AssetInfoRaw::Token { contract } => AssetInfo::Token { contract: TokenContract {
                    contract_addr: deps.api.human_address(&contract.contract_addr)?,
                    callback_code_hash: contract.callback_code_hash.to_owned()
                }},
            },
            amount: self.amount,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum AssetInfoRaw {
    Token { contract: TokenContractRaw },
    NativeToken { denom: String },
}

impl AssetInfoRaw {
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            //Will need to return vec because of struct ownership issues
            AssetInfoRaw::NativeToken { denom } => denom.as_bytes().to_vec(),
            AssetInfoRaw::Token { contract } => contract.as_bytes()
        }
    }

    pub fn to_normal<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>,
    ) -> StdResult<AssetInfo> {
        match self {
            AssetInfoRaw::NativeToken { denom } => Ok(AssetInfo::NativeToken {
                denom: denom.to_string(),
            }),
            AssetInfoRaw::Token { contract } => Ok(AssetInfo::Token { contract: TokenContract{
                contract_addr: deps.api.human_address(&contract.contract_addr)?,
                callback_code_hash: contract.callback_code_hash.to_owned() }}),
        }
    }

    pub fn equal(&self, asset: &AssetInfoRaw) -> bool {
        match self {
            AssetInfoRaw::Token { contract: self_contract } => {
                //let self_contract_addr = contract.contract_addr;
                match asset {
                    AssetInfoRaw::Token { contract } => {
                        self_contract.contract_addr == contract.contract_addr
                    }
                    AssetInfoRaw::NativeToken { .. } => false,
                }
            }
            AssetInfoRaw::NativeToken { denom, .. } => {
                let self_denom = denom;
                match asset {
                    AssetInfoRaw::Token { .. } => false,
                    AssetInfoRaw::NativeToken { denom, .. } => self_denom == denom,
                }
            }
        }
    }
}