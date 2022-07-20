use std::ops::Div;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint256;
use cosmwasm_std::StdResult;

#[cw_serde]
/// Version capable of doing math
pub struct Rebase {
    pub elastic: Uint256,
    pub base: Uint256,
}

impl Default for Rebase {
    fn default() -> Self {
        Self::new()
    }
}

impl Rebase {
    pub fn new() -> Self {
        Rebase {
            elastic: Uint256::zero(),
            base: Uint256::zero(),
        }
    }

    /// Calculates the base value in relationship to `elastic` and self
    pub fn to_base(&self, elastic: Uint256, round_up: bool) -> StdResult<Uint256> {
        let mut base: Uint256;
        if self.elastic.is_zero() {
            base = elastic;
        } else {
            base = elastic.multiply_ratio(self.base, self.elastic);
            if round_up && base.multiply_ratio(self.elastic, self.base) < elastic {
                base = base.checked_add(Uint256::from(1u128))?;
            }
        }
        Ok(base)
    }

    /// Calculates the elastic value in relationship to `base` and self
    pub fn to_elastic(&self, base: Uint256, round_up: bool) -> StdResult<Uint256> {
        let mut elastic: Uint256;
        if self.base.is_zero() {
            elastic = base;
        } else {
            elastic = base.multiply_ratio(self.elastic, self.base);
            if round_up && elastic.multiply_ratio(self.base, self.elastic) < base {
                elastic = elastic.checked_add(Uint256::from(1u128))?;
            }
        }
        Ok(elastic)
    }

    /// Add `elastic` to `self` and update `total.base`
    pub fn add(&mut self, elastic: Uint256, round_up: bool) -> StdResult<(&mut Self, Uint256)> {
        let base = self.to_base(elastic, round_up)?;
        self.elastic = self.elastic.checked_add(elastic)?;
        self.base = self.base.checked_add(base)?;
        Ok((self, base))
    }

    /// Sub `base` from `total` and update `self.elastic`
    pub fn sub(&mut self, base: Uint256, round_up: bool) -> StdResult<(&mut Self, Uint256)> {
        let elastic = self.to_elastic(base, round_up)?;
        self.elastic = self.elastic.checked_sub(elastic)?;
        self.base = self.base.checked_sub(base)?;
        Ok((self, elastic))
    }

    /// Add `elastic` and `base` to self.
    pub fn add_self(&mut self, elastic: Uint256, base: Uint256) -> StdResult<&mut Self> {
        self.elastic = self.elastic.checked_add(elastic)?;
        self.base = self.base.checked_add(base)?;
        Ok(self)
    }

    /// Subtract `elastic` and `base` from self.
    pub fn sub_self(&mut self, elastic: Uint256, base: Uint256) -> StdResult<&mut Self> {
        self.elastic = self.elastic.checked_sub(elastic)?;
        self.base = self.base.checked_sub(base)?;
        Ok(self)
    }
}

#[test]
fn test_rebase_math() {
    let mut total_borrowed = Rebase::new();
    let value = Uint256::from(100u128);
    total_borrowed.add(value, false).unwrap();
    assert_eq!(value, total_borrowed.elastic);
    assert_eq!(value, total_borrowed.base);
}

#[test]
fn test_vault_rebase_math() {
    let mut total_borrowed = Rebase::new();
    total_borrowed.add(Uint256::from(320u128), false).unwrap();
    assert_eq!(
        Uint256::from(1u128),
        total_borrowed.elastic.div(total_borrowed.base)
    );
    total_borrowed.elastic = total_borrowed
        .elastic
        .checked_add(Uint256::from(160u128))
        .unwrap();
    assert_eq!(
        Uint256::from(30u128),
        total_borrowed
            .to_elastic(Uint256::from(20u128), true)
            .unwrap()
    );
}
