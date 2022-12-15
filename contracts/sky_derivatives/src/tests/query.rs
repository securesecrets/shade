use cosmwasm_floating_point::float::Float;

use crate::query;
use shade_protocol::{
    c_std::Uint128,
    contract_interfaces::sky::sky_derivatives::{
        Direction,
        QueryAnswer,
    },
};

#[test]
fn optimization_math() {
    assert_eq!(
        query::optimization_math(
            (Float::from(1_070_000u32), Float::from(1_000_000u32)),
            Float::from_float(1.081),
            Float::from_float(0.9995),
            Float::from_float(0.998),
            Float::from_float(0.997),
            None,
        ).unwrap(),
        QueryAnswer::IsProfitable {
            is_profitable: true,
            swap_amounts: Some((
                    Uint128::from(3602u32),
                    Uint128::from(3345u32),
                    Uint128::from(3615u32))),
            direction: Some(Direction::Unbond),
        },
    );

    assert_eq!(
        query::optimization_math(
            (Float::from(1_070_000_000_000u128), Float::from(1_000_000_000_000u128)),
            Float::from_float(1.081),
            Float::from_float(0.9995),
            Float::from_float(0.998),
            Float::from_float(0.997),
            None,
        ).unwrap(),
        QueryAnswer::IsProfitable {
            is_profitable: true,
            swap_amounts: Some((
                    Uint128::from(3_602_994_595u32),
                    Uint128::from(3_345_916_162u32),
                    Uint128::from(3_615_126_904u32))),
            direction: Some(Direction::Unbond),
        },
    );

    assert_eq!(
        query::optimization_math(
            (Float::from(1_087_500_000_000u128), Float::from(1_000_000_000_000u128)),
            Float::from_float(1.081),
            Float::from_float(0.9995),
            Float::from_float(0.998),
            Float::from_float(0.997),
            None,
        ).unwrap(),
        QueryAnswer::IsProfitable {
            is_profitable: true,
            swap_amounts: Some((
                    Uint128::from(535_451_320u32),
                    Uint128::from(494_338_961u32),
                    Uint128::from(535_716_014u32))),
            direction: Some(Direction::Stake),
        },
    );


}
