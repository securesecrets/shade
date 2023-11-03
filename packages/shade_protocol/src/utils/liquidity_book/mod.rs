//! Helper Libraries

use cosmwasm_std::Uint256;

pub mod bin_helper;
pub mod constants;
pub mod fee_helper;
pub mod lb_token;
pub mod math;
pub mod oracle_helper;
pub mod pair_parameter_helper;
pub mod price_helper;
pub mod tokens;
pub mod transfer;
pub mod types;
pub mod viewing_keys;

pub fn ceil_div(a: Uint256, b: Uint256) -> Uint256 {
    if b == Uint256::zero() {
        panic!("Division by zero");
    }
    let div = a / b;
    let rem = a % b;
    if rem == Uint256::zero() {
        div
    } else {
        div + Uint256::one()
    }
}
