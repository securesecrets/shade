mod test {
    use crate::handle::{active, calculate_claim_date, calculate_issuance};
    use shade_protocol::c_std::Uint128;
    use shade_protocol::{
        contract_interfaces::{
            bonds::{errors::*},
        },
    };
    
    #[test]
    fn checking_limits() {}

    #[test]
    fn check_active() {
        assert_eq!(active(&true, &Uint128::new(10), &Uint128::new(9)), Ok(()));
        assert_eq!(
            active(&false, &Uint128::new(10), &Uint128::new(9)),
            Err(contract_not_active())
        );
        assert_eq!(
            active(&true, &Uint128::new(10), &Uint128::new(10)),
            Err(global_limit_reached(Uint128::new(10)))
        );
    }

    #[test]
    fn claim_date() {
        assert_eq!(calculate_claim_date(0, 1), 1);
        assert_eq!(calculate_claim_date(100_000_000, 7), 100_000_007);
    }

    #[test]
    fn calc_mint() {
        let result = calculate_issuance(
            Uint128::new(7_000_000_000_000_000_000),
            Uint128::new(10_000_000),
            6,
            Uint128::new(5_000_000_000_000_000_000),
            6,
            Uint128::new(7_000),
            Uint128::new(0),
        );
        assert_eq!(result.0, Uint128::new(15_053_763));
        let result2 = calculate_issuance(
            Uint128::new(10_000_000_000_000_000_000),
            Uint128::new(50_000_000),
            6,
            Uint128::new(50_000_000_000_000_000_000),
            8,
            Uint128::new(9_000),
            Uint128::new(0),
        );
        assert_eq!(result2.0, Uint128::new(1_098_901_000));
        let result3 = calculate_issuance(
            Uint128::new(10_000_000_000_000_000_000),
            Uint128::new(5_000_000_000),
            8,
            Uint128::new(50_000_000_000_000_000_000),
            6,
            Uint128::new(9_000),
            Uint128::new(0),
        );
        assert_eq!(result3.0, Uint128::new(10989010));
    }
}
