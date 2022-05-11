mod math;

pub(crate) mod errors;

mod compat {
    impl From<crate::Uint128> for cosmwasm_std::Uint128 {
        fn from(x: crate::Uint128) -> Self {
            cosmwasm_std::Uint128(x.u128())
        }
    }

    impl From<cosmwasm_std::Uint128> for crate::Uint128 {
        fn from(x: cosmwasm_std::Uint128) -> Self {
            x.0.into()
        }
    }
}

pub use crate::math::{
    Decimal,
    Decimal256,
    Decimal256RangeExceeded,
    DecimalRangeExceeded,
    Fraction,
    Isqrt,
    Uint128,
    Uint256,
    Uint512,
    Uint64,
};
