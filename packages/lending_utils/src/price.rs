use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use shade_protocol::c_std::Decimal;

use crate::{coin::Coin, token::Token};

// Structure containing price ratio for sell market_token / buy common_token
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, JsonSchema, Debug)]
pub struct PriceRate {
    pub buy_denom: Token,
    pub sell_denom: Token,
    pub rate_sell_per_buy: Decimal,
}

/// Helper that multiplies coins amount in sell denom times proper price rate. Allows to obtain the
/// buy token given a certain amount of sell token.
/// Returns error, if Coin.denom != Price.sell_denom
/// Inverted price can't be just returned, because price is a weighted average
pub fn coin_times_price_rate(coin: &Coin, price: &PriceRate) -> Result<Coin, PriceError> {
    if coin.denom == price.sell_denom {
        Ok(price
            .buy_denom
            .amount(coin.amount * price.rate_sell_per_buy))
    } else {
        Err(PriceError::MulPrice {
            incorrect: coin.denom.clone(),
            correct: price.sell_denom.clone(),
        })
    }
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum PriceError {
    #[error("Calculating price failed because incorrect denom was used: {incorrect} instead of {correct}")]
    MulPrice { incorrect: Token, correct: Token },
}

#[cfg(test)]
mod tests {
    use super::*;

    use shade_protocol::c_std::{Addr, ContractInfo};

    use crate::coin::coin_cw20;

    fn new_snip20(address: &str) -> ContractInfo {
        ContractInfo {
            address: Addr::unchecked(address),
            code_hash: "hash".to_owned(),
        }
    }

    #[test]
    fn price_rate_correct_denom() {
        let price_rate = PriceRate {
            buy_denom: Token::new_cw20(new_snip20("USD")),
            sell_denom: Token::new_cw20(new_snip20("EUR")),
            rate_sell_per_buy: Decimal::percent(110),
        };
        let eur_coin = coin_cw20(100, new_snip20("EUR"));
        let usd_coin = coin_times_price_rate(&eur_coin.into(), &price_rate).unwrap();
        assert_eq!(usd_coin, coin_cw20(110, new_snip20("USD")).into());
    }

    #[test]
    fn price_rate_wrong_buy_denom() {
        let price_rate = PriceRate {
            buy_denom: Token::new_cw20(new_snip20("USD")),
            sell_denom: Token::new_cw20(new_snip20("EUR")),
            rate_sell_per_buy: Decimal::percent(110),
        };
        let usd_coin = coin_cw20(100, new_snip20("USD"));
        let err = coin_times_price_rate(&usd_coin.into(), &price_rate).unwrap_err();
        assert_eq!(
            PriceError::MulPrice {
                incorrect: Token::Cw20(new_snip20("USD")),
                correct: Token::Cw20(new_snip20("EUR"))
            },
            err
        );
    }

    #[test]
    fn price_rate_incorrect_denom() {
        let price_rate = PriceRate {
            buy_denom: Token::new_cw20(new_snip20("USD")),
            sell_denom: Token::new_cw20(new_snip20("EUR")),
            rate_sell_per_buy: Decimal::percent(110),
        };
        let pln_coin = coin_cw20(100, new_snip20("PLN"));
        let err = coin_times_price_rate(&pln_coin.into(), &price_rate).unwrap_err();
        assert_eq!(
            PriceError::MulPrice {
                incorrect: Token::Cw20(new_snip20("PLN")),
                correct: Token::Cw20(new_snip20("EUR"))
            },
            err
        );
    }
}
