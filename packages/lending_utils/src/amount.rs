use cosmwasm_std::{Decimal, Fraction, Uint128, Uint256};

/// Converts the given amount of base tokens to the equivalent amount of cTokens
pub fn base_to_token(amount: Uint128, multiplier: Decimal) -> Uint128 {
    // amount / multiplier
    let result256 =
        amount.full_mul(multiplier.denominator()) / Uint256::from(multiplier.numerator());

    result256.try_into().unwrap()
}

/// Converts the given amount of cTokens to the equivalent amount of base tokens
pub fn token_to_base(amount: Uint128, multiplier: Decimal) -> Uint128 {
    amount * multiplier
}

