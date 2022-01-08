use cosmwasm_std::{StdError, StdResult, Uint128};

// Non permanent solutions to Uint128 issues

pub fn mult(a: Uint128, b: Uint128) -> Uint128 {
    if a.is_zero() || b.is_zero() {
        return Uint128::zero();
    }

    Uint128(a.u128() * b.u128())
}

pub fn div(nom: Uint128, den: Uint128) -> StdResult<Uint128> {
    if den == Uint128::zero() {
        return Err(StdError::generic_err("Division by 0"))
    }

    Ok(Uint128(nom.u128() / den.u128()))
}

#[cfg(test)]
pub mod tests {
    use cosmwasm_std::Uint128;
    use crate::math::{div, mult};

    #[test]
    fn multiply() {
        assert_eq!(Uint128(10), mult(Uint128(5), Uint128(2)))
    }

    #[test]
    fn divide() {
        assert_eq!(Uint128(5), div(Uint128(10), Uint128(2)).unwrap())
    }

    #[test]
    fn divide_by_zero() {
        assert!(div(Uint128(10), Uint128(0)).is_err())
    }
}