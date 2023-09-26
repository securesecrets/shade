use cosmwasm_std::{Coin as StdCoin, Decimal, OverflowError, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{convert::From, ops::Mul};
use thiserror::Error;
use wyndex::asset::{Asset, AssetInfo};

use crate::token::Token;

/// Universal coin type which is either a native coin, or cw20 coin
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord)]
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

    pub fn new_native(amount: u128, denom: &str) -> Self {
        Self::new(amount, Token::new_native(denom))
    }

    pub fn new_cw20(amount: u128, addr: &str) -> Self {
        Self::new(amount, Token::new_cw20(addr))
    }

    pub fn checked_add(self, rhs: Self) -> Result<Self, CoinError> {
        if self.denom == rhs.denom {
            Ok(Self {
                amount: self.amount.checked_add(rhs.amount)?,
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
                amount: self.amount.checked_sub(rhs.amount)?,
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

pub fn coin_native(amount: u128, denom: impl Into<String>) -> Coin {
    Coin::new(amount, Token::new_native(&denom.into()))
}

pub fn coin_cw20(amount: u128, denom: impl Into<String>) -> Coin {
    Coin::new(amount, Token::new_cw20(&denom.into()))
}

impl From<StdCoin> for Coin {
    fn from(c: StdCoin) -> Self {
        Coin {
            amount: c.amount,
            denom: Token::Native(c.denom),
        }
    }
}

impl From<Coin> for Asset {
    fn from(c: Coin) -> Self {
        let info: AssetInfo = match c.denom {
            Token::Native(denom) => AssetInfo::Native(denom),
            Token::Cw20(address) => AssetInfo::Token(address),
        };

        Asset {
            info,
            amount: c.amount,
        }
    }
}

impl TryFrom<Coin> for StdCoin {
    type Error = ();

    fn try_from(c: Coin) -> Result<Self, Self::Error> {
        match c.denom {
            Token::Native(denom) => Ok(StdCoin {
                amount: c.amount,
                denom,
            }),
            Token::Cw20(_) => Err(()),
        }
    }
}

impl Mul<Decimal> for Coin {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self::Output {
        Self {
            denom: self.denom,
            amount: self.amount * rhs,
        }
    }
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum CoinError {
    #[error(
        "Operation {operation} is not allowed, because denoms does not match: {denom1:?} {denom2:?}"
    )]
    IncorrectDenoms {
        operation: String,
        denom1: Token,
        denom2: Token,
    },

    #[error("{0}")]
    Overflow(#[from] OverflowError),
}

