use crate::c_std::{Uint256, StdResult};

// For generic purpose math formulas
pub fn sqrt(value: Uint256) -> StdResult<Uint256> {
    let mut z = Uint256::zero();

    if value.gt(&Uint256::from(3u128)) {
        z = value;
        let mut x = value
            .checked_div(Uint256::from(2u128))?
            .checked_add(Uint256::from(1u128))?;

        while x.lt(&z) {
            z = x;
            x = value
                .checked_div(x)?
                .checked_add(x)?
                .checked_div(Uint256::from(2u128))?;
        }
    } else if !value.is_zero() {
        z = Uint256::from(1u128);
    }

    Ok(z)
}