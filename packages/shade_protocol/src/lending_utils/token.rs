use crate::{
    c_std::{
        coin, to_binary, Addr, BankMsg, Coin as StdCoin, ContractInfo, CosmosMsg, CustomQuery,
        Decimal, Deps, StdError, StdResult, Uint128, WasmMsg,
    },
    contract_interfaces::snip20,
    secret_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey},
    utils::{asset::Contract, Query},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::lending_utils::coin::{self, Coin};

use std::fmt;

/// Universal token type which is either a native token, or cw20 token
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum Token {
    /// Snip20 token with its snip20 contract address
    Cw20(ContractInfo),
}

impl Token {
    pub fn new_cw20(info: ContractInfo) -> Self {
        Self::Cw20(info)
    }

    /// Returns cw20 token address or `None`
    pub fn cw20(self) -> Option<String> {
        match self {
            Token::Cw20(info) => Some(info.address.to_string()),
            _ => None,
        }
    }

    /// Returns cw20 token address or `None`
    pub fn as_cw20(&self) -> Option<&str> {
        match self {
            Token::Cw20(info) => Some(info.address.as_str()),
            _ => None,
        }
    }

    pub fn as_contract_info(&self) -> Option<ContractInfo> {
        match self {
            Token::Cw20(info) => Some(info.clone()),
            _ => None,
        }
    }

    /// Checks i token is cw20
    pub fn is_cw20(&self) -> bool {
        matches!(self, Token::Cw20(_))
    }

    /// Helper function to return the Address of the CW20 token or the denom of the native one.
    pub fn denom(&self) -> String {
        use Token::*;
        match self {
            Cw20(info) => info.address.to_string(),
        }
    }

    /// Queries the balance of the given address
    pub fn query_balance(
        &self,
        deps: Deps,
        address: impl Into<String>,
        viewing_key: String,
    ) -> StdResult<u128> {
        Ok(match self {
            Self::Cw20(info) => {
                let balance_query = snip20::QueryMsg::Balance {
                    address: address.into(),
                    key: viewing_key,
                };
                let contract_type: Contract = (info.clone()).into(); // Explicitly specify the type
                match balance_query.query::<snip20::QueryAnswer>(&deps.querier, &contract_type) {
                    Ok(snip20::QueryAnswer::Balance { amount }) => amount.u128(),
                    Err(e) => return Err(e), // Handle error properly
                    _ => panic!("Unexpected result from query"),
                }
            }
        })
    }

    pub fn amount(&self, amount: impl Into<Uint128>) -> Coin {
        Coin {
            amount: amount.into(),
            denom: self.clone(),
        }
    }

    /// Helper function to create a custom `utils::coin::Coin` from a `Token`.
    pub fn into_coin(self, amount: impl Into<Uint128>) -> Coin {
        Coin {
            amount: amount.into(),
            denom: self,
        }
    }

    /// Creates a send message for this token to send the given amount from this contract to the given address
    pub fn send_msg(
        &self,
        to_address: impl Into<Addr>,
        amount: impl Into<Uint128>,
    ) -> StdResult<CosmosMsg> {
        match self {
            Self::Cw20(info) => {
                // well, great code to work with...
                let contract_type: Contract = (info.clone()).into(); // Explicitly specify the type
                snip20::helpers::send_msg(
                    to_address.into(),
                    amount.into(),
                    None,
                    None,
                    None,
                    &contract_type,
                )
            }
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Cw20(s) => write!(f, "{}", s.address.to_string()),
        }
    }
}

impl KeyDeserialize for &Token {
    type Output = Token;

    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        let (asset_type, address, code_hash) = <(u8, Addr, String)>::from_vec(value)?;

        match asset_type {
            1 => Ok(Token::Cw20(ContractInfo { address, code_hash })),
            _ => Err(StdError::generic_err("Invalid Token key, invalid type")),
        }
    }
}

impl<'a> Prefixer<'a> for &Token {
    fn prefix(&self) -> Vec<Key> {
        self.key()
    }
}

// Allow using `AssetInfoValidated` as a key in a `Map`
impl<'a> PrimaryKey<'a> for &Token {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        match self {
            Token::Cw20(contract_info) => vec![
                Key::Val8([1]),
                Key::Ref(contract_info.address.as_bytes()),
                Key::Ref(contract_info.code_hash.as_bytes()),
            ],
        }
    }
}

// use std::hash::{Hash, Hasher};
//
// impl Hash for Token {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         match self {
//             Token::Cw20(contract_info) => {
//                 contract_info.address.hash(state);
//                 // Hash other fields or other variants as needed
//             },
//             // Handle other Token variants here if there are any
//         }
//     }
// }
