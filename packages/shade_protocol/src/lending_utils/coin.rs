use crate::c_std::{Coin as StdCoin, ContractInfo, Decimal, OverflowError, Uint128};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::{cmp::Ordering, convert::From, ops::Mul};

use crate::lending_utils::token::Token;

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

impl PartialOrd for Coin {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.amount.cmp(&other.amount))
    }
}

impl Ord for Coin {
    fn cmp(&self, other: &Self) -> Ordering {
        self.amount.cmp(&other.amount)
    }
}

pub fn coin_cw20(amount: u128, info: ContractInfo) -> Coin {
    Coin::new(amount, Token::new_cw20(info))
}

#[derive(Debug, Error, PartialEq)]
pub enum CoinError {
    #[error(
       "Operation {operation} is not allowed, because denoms does not match: {denom1:?} {denom2:?}"
    )]
    IncorrectDenoms {
        operation: String,
        denom1: Token,
        denom2: Token,
    },
}
