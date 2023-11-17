use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::c_std::{Coin as StdCoin, ContractInfo, Decimal, OverflowError, Uint128};
use std::{convert::From, ops::Mul};

use crate::token::Token;

/// Universal coin type which is either a native coin, or cw20 coin
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Coin {
    pub denom: Token,
    pub amount: Uint128,
}

impl Coin {
    pub fn new(amount: u128, denom: Token) -> Self {
        Coin {
            amount: Uint128::new(amount),
            denom,
        }
    }

    pub fn new_cw20(amount: u128, info: ContractInfo) -> Self {
        Self::new(amount, Token::new_cw20(info))
    }

    pub fn checked_add(self, rhs: Self) -> Result<Self, CoinError> {
        if self.denom == rhs.denom {
            Ok(Self {
                amount: self.amount.checked_add(rhs.amount).unwrap(),
                denom: self.denom,
            })
        } else {
            Err(CoinError::IncorrectDenoms {
                operation: "addition".to_owned(),
                denom1: self.denom,
                denom2: rhs.denom,
            })
        }
    }

    pub fn checked_sub(self, rhs: Self) -> Result<Self, CoinError> {
        if self.denom == rhs.denom {
            Ok(Self {
                amount: self.amount.checked_sub(rhs.amount).unwrap(),
                denom: self.denom,
            })
        } else {
            Err(CoinError::IncorrectDenoms {
                operation: "subtraction".to_owned(),
                denom1: self.denom,
                denom2: rhs.denom,
            })
        }
    }
}

pub fn coin_cw20(amount: u128, info: ContractInfo) -> Coin {
    Coin::new(amount, Token::new_cw20(info))
}

#[derive(Debug, Eq, PartialEq)]
pub enum CoinError {
    IncorrectDenoms {
        operation: String,
        denom1: Token,
        denom2: Token,
    },
}
