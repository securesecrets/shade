#[warn(unused_imports)]
use crate::c_std::{MessageInfo, StdError, StdResult, Uint128};
use crate::swap::core::TokenType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::swap::core::TokenPair;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenPairAmount {
    pub pair: TokenPair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
}

impl TokenPairAmount {
    pub fn assert_sent_native_token_balance(&self, info: &MessageInfo) -> StdResult<()> {
        self.pair
            .0
            .assert_sent_native_token_balance(info, self.amount_0)?;
        self.pair
            .1
            .assert_sent_native_token_balance(info, self.amount_1)?;

        Ok(())
    }

    /// reorders the TokenPairAmount so that it's tokens and amounts are in the same order as the given 'pair_to_match'
    pub fn create_new_pair_amount_to_match_order_of(
        self,
        pair_to_match: &TokenPair,
    ) -> StdResult<Self> {
        if !self.pair.contains(&pair_to_match.0) || !self.pair.contains(&pair_to_match.1) {
            return Err(StdError::generic_err(
                "Pair to match does not contain same tokens as current TokenPairAmount",
            ));
        }

        //at this point we know that the 'pair to match' contains the same tokens as the current pair

        if self.pair.0 == pair_to_match.0 {
            //order is already correct
            Ok(self)
        } else {
            //order is wrong

            Ok(TokenPairAmount {
                pair: TokenPair(self.pair.1, self.pair.0, self.pair.2),
                amount_0: self.amount_1,
                amount_1: self.amount_0,
            })
        }
    }
}

impl<'a> IntoIterator for &'a TokenPairAmount {
    type IntoIter = TokenPairAmountIterator<'a>;
    type Item = (Uint128, &'a TokenType);

    fn into_iter(self) -> Self::IntoIter {
        TokenPairAmountIterator {
            pair: self,
            index: 0,
        }
    }
}

pub struct TokenPairAmountIterator<'a> {
    pair: &'a TokenPairAmount,
    index: u8,
}

impl<'a> Iterator for TokenPairAmountIterator<'a> {
    type Item = (Uint128, &'a TokenType);

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some((self.pair.amount_0, &self.pair.pair.0)),
            1 => Some((self.pair.amount_1, &self.pair.pair.1)),
            _ => None,
        };
        self.index += 1;
        result
    }
}

pub mod tests {
    use super::*;
    use crate::c_std::Addr;

    #[test]
    pub fn test_rearrange() {
        let token0 = TokenType::CustomToken {
            contract_addr: Addr::unchecked("token0".to_string()),
            token_code_hash: "token0".to_string(),
        };
        let token1 = TokenType::CustomToken {
            contract_addr: Addr::unchecked("token1".to_string()),
            token_code_hash: "token1".to_string(),
        };

        let pair_correct_order = TokenPair(token0.clone(), token1.clone(), false);

        // test reversing a pair amount
        let pair_param_2 = true;
        let reverse_amount = TokenPairAmount {
            pair: TokenPair(token1.clone(), token0.clone(), pair_param_2.clone()),
            amount_0: Uint128::one(),
            amount_1: Uint128::zero(),
        };

        let fixed = reverse_amount
            .clone()
            .create_new_pair_amount_to_match_order_of(&pair_correct_order)
            .unwrap();
        assert_eq!(fixed.pair.0, pair_correct_order.0);
        assert_eq!(fixed.pair.1, pair_correct_order.1);
        assert_eq!(fixed.pair.2, pair_param_2);
        assert_eq!(fixed.amount_0, Uint128::zero());
        assert_eq!(fixed.amount_1, Uint128::one());

        //test leaving a pair amount how it is
        let pair_param_2 = false;
        let amount = TokenPairAmount {
            pair: TokenPair(token0.clone(), token1.clone(), pair_param_2.clone()),
            amount_0: Uint128::zero(),
            amount_1: Uint128::one(),
        };

        let fixed = amount
            .create_new_pair_amount_to_match_order_of(&pair_correct_order)
            .unwrap();
        assert_eq!(fixed.pair.0, pair_correct_order.0);
        assert_eq!(fixed.pair.1, pair_correct_order.1);
        assert_eq!(fixed.pair.2, pair_param_2);
        assert_eq!(fixed.amount_0, Uint128::zero());
        assert_eq!(fixed.amount_1, Uint128::one());

        //test a non-matching pair
        let broken_pair = TokenPair(
            token0.clone(),
            TokenType::CustomToken {
                contract_addr: Addr::unchecked("token3".to_string()),
                token_code_hash: "a".to_string(),
            },
            false,
        );
        assert!(
            reverse_amount
                .create_new_pair_amount_to_match_order_of(&broken_pair)
                .is_err()
        );
    }
}
