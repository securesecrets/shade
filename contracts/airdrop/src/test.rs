#[cfg(test)]
pub mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::Uint128;
    use shade_protocol::airdrop::claim_info::RequiredTask;
    use shade_protocol::airdrop::InitMsg;
    use shade_protocol::asset::Contract;
    use crate::contract::init;
    use crate::handle::calculate_decay_factor;


    #[test]
    fn decay_factor() {
        assert_eq!(Uint128(50), Uint128(100) * calculate_decay_factor(100, 200, 300));

        assert_eq!(Uint128(25), Uint128(100) * calculate_decay_factor(0, 75, 100));
    }
}