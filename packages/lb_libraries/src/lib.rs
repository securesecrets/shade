//! Helper Libraries

use cosmwasm_std::Uint256;

pub mod bin_helper;
pub mod constants;
pub mod error;
pub mod fee_helper;
pub mod lb_token;
pub mod math;
pub mod oracle_helper;
pub mod pair_parameter_helper;
pub mod price_helper;
pub mod types;

pub fn approx_div(a: Uint256, b: Uint256) -> Uint256 {
    if b == Uint256::zero() {
        panic!("Division by zero");
    }
    let div = a / b;
    let rem = a % b;
    if rem >= b / Uint256::from(2u128) {
        // If so, we add one to the division result
        div + Uint256::one()
    } else {
        // If not, we return the division result as it is
        div
    }
}
