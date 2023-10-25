use crate::{
    c_std::{Deps, StdResult, Uint128},
    swap::core::{TokenPairAmount, TokenType},
    Contract,
};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, JsonSchema)]
pub struct TokenPair(pub TokenType, pub TokenType, pub bool);

pub struct TokenPairIterator<'a> {
    pair: &'a TokenPair,
    index: u8,
}

impl TokenPair {
    /// Returns `true` if one of the token types in the pair is the same as the argument.
    pub fn contains(&self, token: &TokenType) -> bool {
        self.0 == *token || self.1 == *token
    }

    /// Returns the index of the stored token type (0 or 1) that matches the argument.
    /// Returns `None` if there are no matches.
    pub fn get_token_index(&self, token: &TokenType) -> Option<usize> {
        if self.0 == *token {
            return Some(0);
        } else if self.1 == *token {
            return Some(1);
        }

        None
    }

    pub fn get_token(&self, index: usize) -> Option<&TokenType> {
        match index {
            0 => Some(&self.0),
            1 => Some(&self.1),
            _ => None,
        }
    }

    pub fn new_from_custom(token_a: Contract, token_b: Contract, is_stable: bool) -> Self {
        Self(
            TokenType::CustomToken {
                contract_addr: token_a.address.to_owned(),
                token_code_hash: token_a.code_hash,
            },
            TokenType::CustomToken {
                contract_addr: token_b.address.to_owned(),
                token_code_hash: token_b.code_hash,
            },
            is_stable,
        )
    }

    pub fn new_amount(
        &self,
        amount_a: impl Into<Uint128> + Copy,
        amount_b: impl Into<Uint128> + Copy,
    ) -> TokenPairAmount {
        TokenPairAmount {
            pair: self.clone(),
            amount_0: amount_a.into(),
            amount_1: amount_b.into(),
        }
    }
}

impl TokenPair {
    /// Returns the balance for each token in the pair. The order of the balances in returned array
    /// correspond to the token order in the pair i.e `[ self.0 balance, self.1 balance ]`.
    pub fn query_balances(
        &self,
        deps: Deps,
        exchange_addr: String,
        viewing_key: String,
    ) -> StdResult<[Uint128; 2]> {
        let amount_0 = self
            .0
            .query_balance(deps, exchange_addr.clone(), viewing_key.clone())?;
        let amount_1 = self.1.query_balance(deps, exchange_addr, viewing_key)?;

        // order is important
        Ok([amount_0, amount_1])
    }

    // pub fn query_decimals(&self, deps: &Deps) -> StdResult<[u8; 2]> {
    //     let decimal_0 = self.0.query_decimals(deps)?;
    //     let decimal_1 = self.1.query_decimals(deps)?;
    //     Ok([decimal_0, decimal_1])
    // }
}

impl PartialEq for TokenPair {
    fn eq(&self, other: &TokenPair) -> bool {
        (self.0 == other.0 && self.1 == other.1 && self.2 == other.2)
            || (self.0 == other.1 && self.1 == other.0 && self.2 == other.2)
    }
}

impl<'a> IntoIterator for &'a TokenPair {
    type IntoIter = TokenPairIterator<'a>;
    type Item = &'a TokenType;

    fn into_iter(self) -> Self::IntoIter {
        TokenPairIterator {
            pair: self,
            index: 0,
        }
    }
}

impl<'a> Iterator for TokenPairIterator<'a> {
    type Item = &'a TokenType;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(&self.pair.0),
            1 => Some(&self.pair.1),
            _ => None,
        };

        self.index += 1;

        result
    }
}

impl Serialize for TokenPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (self.0.clone(), self.1.clone(), self.2.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TokenPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
            .map(|(token_0, token_1, is_stable)| TokenPair(token_0, token_1, is_stable))
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;

    use super::*;

    #[test]
    fn token_pair_equality() {
        let pair: TokenPair = TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked("address"),
                token_code_hash: "hash".into(),
            },
            TokenType::NativeToken {
                denom: "denom".into(),
            },
            false,
        );

        let pair2 = TokenPair(pair.1.clone(), pair.0.clone(), false);

        assert_eq!(pair, pair);
        assert_eq!(pair2, pair2);
        assert_eq!(pair, pair2);

        let pair2 = TokenPair(pair.1.clone(), pair.1.clone(), false);

        assert_eq!(pair2, pair2);
        assert_ne!(pair, pair2);

        let pair2 = TokenPair(
            pair.1.clone(),
            TokenType::CustomToken {
                contract_addr: Addr::unchecked("address2"),
                token_code_hash: "hash2".into(),
            },
            false,
        );

        assert_eq!(pair, pair);
        assert_eq!(pair2, pair2);
        assert_ne!(pair, pair2);

        let pair2_reversed = TokenPair(pair2.1.clone(), pair2.0.clone(), false);

        assert_eq!(pair2_reversed, pair2);
        assert_ne!(pair, pair2);
    }
}
