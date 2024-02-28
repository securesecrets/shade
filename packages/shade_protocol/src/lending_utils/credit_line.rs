use std::{iter::Sum, ops::Add};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::c_std::Uint128;

use crate::lending_utils::{coin::Coin, token::Token};

/// The Credit Line response with the common token denom included. Used in the API.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CreditLineResponse {
    /// Total value of C-Tokens in common_token
    pub collateral: Coin,
    /// collateral * collateral_ratio
    pub credit_line: Coin,
    /// credit_line * borrow_limit_ratio
    pub borrow_limit: Coin,
    /// Total value of debt in common_token
    pub debt: Coin,
}

impl CreditLineResponse {
    pub fn validate(
        &self,
        expected_denom: &Token,
    ) -> Result<CreditLineValues, InvalidCommonTokenDenom> {
        for actual in [
            &self.collateral.denom,
            &self.credit_line.denom,
            &self.borrow_limit.denom,
            &self.debt.denom,
        ] {
            if actual != expected_denom {
                return Err(InvalidCommonTokenDenom {
                    expected: expected_denom.clone(),
                    actual: actual.clone(),
                });
            }
        }

        Ok(CreditLineValues {
            collateral: self.collateral.amount,
            credit_line: self.credit_line.amount,
            borrow_limit: self.borrow_limit.amount,
            debt: self.debt.amount,
        })
    }
}

/// The Credit Line with just the values and no denom included, used for internal calculations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreditLineValues {
    /// Total value of C-Tokens in common_token
    pub collateral: Uint128,
    /// collateral * collateral_ratio
    pub credit_line: Uint128,
    /// credit_line * borrow_limit_ratio
    pub borrow_limit: Uint128,
    /// Total value of debt in common_token
    pub debt: Uint128,
}

impl CreditLineValues {
    pub fn zero() -> Self {
        CreditLineValues {
            collateral: Uint128::zero(),
            credit_line: Uint128::zero(),
            borrow_limit: Uint128::zero(),
            debt: Uint128::zero(),
        }
    }

    pub fn new(
        collateral: impl Into<Uint128>,
        credit_line: impl Into<Uint128>,
        borrow_limit: impl Into<Uint128>,
        debt: impl Into<Uint128>,
    ) -> Self {
        CreditLineValues {
            collateral: collateral.into(),
            credit_line: credit_line.into(),
            borrow_limit: borrow_limit.into(),
            debt: debt.into(),
        }
    }

    pub fn make_response(self, denom: Token) -> CreditLineResponse {
        CreditLineResponse {
            collateral: Coin::new(self.collateral.u128(), denom.clone()),
            credit_line: Coin::new(self.credit_line.u128(), denom.clone()),
            borrow_limit: Coin::new(self.borrow_limit.u128(), denom.clone()),
            debt: Coin::new(self.debt.u128(), denom),
        }
    }
}

impl Add for CreditLineValues {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            collateral: self.collateral + rhs.collateral,
            credit_line: self.credit_line + rhs.credit_line,
            borrow_limit: self.borrow_limit + rhs.borrow_limit,
            debt: self.debt + rhs.debt,
        }
    }
}

impl<'a> Sum<&'a Self> for CreditLineValues {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        iter.fold(Self::zero(), |a, b| Self {
            collateral: a.collateral + b.collateral,
            credit_line: a.credit_line + b.credit_line,
            borrow_limit: a.borrow_limit + b.borrow_limit,
            debt: a.debt + b.debt,
        })
    }
}

/// Used for when CreditLineResponse validation fails
#[derive(Debug, Error, Eq, PartialEq)]
#[error("Invalid token denomination: expected {expected:?}, but found {actual:?}")]
pub struct InvalidCommonTokenDenom {
    pub expected: Token,
    pub actual: Token,
}

#[cfg(test)]
mod tests {
    use super::*;

    use shade_protocol::c_std::{Addr, ContractInfo};

    fn new_snip20(address: &str) -> ContractInfo {
        ContractInfo {
            address: Addr::unchecked(address),
            code_hash: "hash".to_owned(),
        }
    }

    #[test]
    fn sum_credit_line_response() {
        let responses = vec![
            CreditLineValues {
                collateral: Uint128::new(500),
                credit_line: Uint128::new(300),
                borrow_limit: Uint128::new(300),
                debt: Uint128::new(200),
            },
            CreditLineValues {
                collateral: Uint128::new(1800),
                credit_line: Uint128::new(200),
                borrow_limit: Uint128::new(200),
                debt: Uint128::new(50),
            },
            CreditLineValues::zero(),
        ];

        let sum: CreditLineValues = responses.iter().sum();
        assert_eq!(
            sum,
            CreditLineValues {
                collateral: Uint128::new(2300),
                credit_line: Uint128::new(500),
                borrow_limit: Uint128::new(500),
                debt: Uint128::new(250),
            },
        );
    }

    #[test]
    fn credit_line_response_validation() {
        let resp = CreditLineResponse {
            collateral: Coin::new_cw20(50, new_snip20("BTC")),
            credit_line: Coin::new_cw20(40, new_snip20("BTC")),
            borrow_limit: Coin::new_cw20(40, new_snip20("BTC")),
            debt: Coin::new_cw20(20, new_snip20("BTC")),
        };
        assert_eq!(
            Ok(CreditLineValues {
                collateral: Uint128::from(50u128),
                credit_line: Uint128::from(40u128),
                borrow_limit: Uint128::from(40u128),
                debt: Uint128::from(20u128)
            }),
            resp.validate(&Token::new_cw20(new_snip20("BTC")))
        );
        assert_eq!(
            Err(InvalidCommonTokenDenom {
                expected: Token::new_cw20(new_snip20("OSMO")),
                actual: Token::new_cw20(new_snip20("BTC"))
            }),
            resp.validate(&Token::new_cw20(new_snip20("OSMO")))
        );
    }

    #[test]
    fn credit_line_inconsistent_response_validation() {
        let resp = CreditLineResponse {
            collateral: Coin::new_cw20(50, new_snip20("BTC")),
            credit_line: Coin::new_cw20(40, new_snip20("OSMO")),
            borrow_limit: Coin::new_cw20(40, new_snip20("OSMO")),
            debt: Coin::new_cw20(20, new_snip20("BTC")),
        };
        assert!(resp.validate(&Token::new_cw20(new_snip20("OSMO"))).is_err());
        assert!(resp.validate(&Token::new_cw20(new_snip20("BTC"))).is_err());
        assert!(resp.validate(&Token::new_cw20(new_snip20("ATOM"))).is_err());
    }
}
