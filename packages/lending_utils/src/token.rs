use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::{
    c_std::{
        coin, to_binary, Addr, BankMsg, Coin as StdCoin, ContractInfo, CosmosMsg, CustomQuery,
        Decimal, Deps, StdError, StdResult, Uint128, WasmMsg,
    },
    secret_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey},
    contract_interfaces::snip20
};

use crate::coin::{self, Coin};

use std::fmt;

/// Universal token type which is either a native token, or cw20 token
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord, Hash,
)]
pub enum Token {
    /// Snip20 token with its snip20 contract address
    Cw20(ContractInfo),
}

impl Token {
    pub fn new_cw20(info: ContractInfo) -> Self {
        Self::Cw20(ContractInfo)
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
            Token::Cw20(addr) => Some(addr),
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
    pub fn query_balance<T: CustomQuery>(
        &self,
        deps: Deps<'_, T>,
        contract_info: impl Into<ContractInfo>,
        viewing_key: String,
    ) -> StdResult<u128> {
        Ok(match self {
            Self::Cw20(info) =>
                snip20::QueryMsg::Balance {
                    address: contract_info.into().address.to_string(),
                    viewing_key
                }.query::<snip20::QueryAnswer::Balance>(&deps.querier, &config.sscrt_token.clone())?;
                // deps
                // .querier
                // .query_wasm_smart::<cw20::BalanceResponse>(
                //     cw20_token,
                //     &snip20::QueryMsg::Balance {
                //         address: address.into(),
                //     },
                // )?
                .balance
                .into(),
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
        to_address: impl Into<String>,
        amount: impl Into<Uint128>,
    ) -> StdResult<CosmosMsg<CoreumMsg>> {
        Ok(match self {
            Self::Cw20(address) => snip20::helpers::send_msg(
                to_address.into(),
                amount.into(),
                None,
                None,
                None,
                address.to_string(),
            )
            // CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            //     contract_addr: address.to_owned(),
            //     msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
            //         recipient: to_address.into(),
            //         amount: amount.into(),
            //     })?,
            //     funds: vec![],
            // }),
        })
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Cw20(s) => write!(f, "{}", s),
        }
    }
}
