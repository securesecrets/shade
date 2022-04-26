

mod test{
    use std::ops::Add;
    use crate::handle::{calculate_claim_date, calculate_issuance, active};
    use crate::query;
    use cosmwasm_std::{coins, from_binary, testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage}, Extern, StdError, Uint128, HumanAddr};
    use crate::contract;
    use shade_protocol::{bonds::{self, Config, QueryAnswer, QueryMsg, InitMsg, errors::*}, treasury, utils::asset::Contract, airdrop::errors::address_already_in_account};
    use shade_protocol::utils::errors::DetailedError;

    #[test]
    fn test_config(){
        let mut deps = mock_dependencies(20, &coins(0, ""));

        // Initialize oracle contract
        let env = mock_env("creator", &coins(0, ""));
        let bonds_init_msg = bonds::InitMsg{
            admin: HumanAddr::from("configadmin"),
            oracle: Contract{
                address: HumanAddr::from("oracleaddr"),
                code_hash: String::from("oraclehash"),
            },
            treasury: HumanAddr::from("treasuryaddr"),
            limit_admin: HumanAddr::from("limitadminaddr"),
            global_issuance_limit: Uint128(100_000_000_000),
            global_minimum_bonding_period: 7u64,
            global_maximum_discount: Uint128(7_000_000_000_000_000_000),
            issued_asset: Contract{
                address: HumanAddr::from("assetaddr"),
                code_hash: String::from("assethash"),
            },
            activated: true,
            minting_bond: true,
            bond_issuance_limit: Uint128(10_000_000_000),
            bonding_period: 7u64,
            discount: Uint128(7_000_000_000_000_000_000),
        };
        let res = contract::init(&mut deps, env, bonds_init_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let check_state = Config{
            admin: HumanAddr::from("configadmin"),
            oracle: Contract{
                address: HumanAddr::from("oracleaddr"),
                code_hash: String::from("oraclehash"),
            },
            treasury: HumanAddr::from("treasuryaddr"),
            limit_admin: HumanAddr::from("limitadminaddr"),
            global_issuance_limit: Uint128(100_000_000_000),
            global_minimum_bonding_period: 7u64,
            global_maximum_discount: Uint128(7_000_000_000_000_000_000),
            issued_asset: Contract{
                address: HumanAddr::from("assetaddr"),
                code_hash: String::from("assethash"),
            },
            activated: true,
            minting_bond: true,
            bond_issuance_limit: Uint128(10_000_000_000),
            bonding_period: 7u64,
            discount: Uint128(7_000_000_000_000_000_000),
        };
        let query_answer = query::config(&mut deps).unwrap();
        let query_result = match query_answer{
            QueryAnswer::Config{config} => config == check_state,
            _ => false,
        };
        assert_eq!(true, query_result);
    }

    #[test]
    fn checking_limits() {
        
    }

    #[test]
    fn check_active() {
        assert_eq!(active(&true, &Uint128(10), &Uint128(9)), Ok(()));
        assert_eq!(active(&false, &Uint128(10), &Uint128(9)), Err(contract_not_active()));
        assert_eq!(active(&true, &Uint128(10), &Uint128(10)), Err(global_limit_reached(Uint128(10))));
    }

    #[test]
    fn claim_date() {
        assert_eq!(calculate_claim_date(0, 1), 86400);
        assert_eq!(calculate_claim_date(100_000_000, 7), 100_604_800);
    }

    #[test]
    fn calc_mint() {
        let result = calculate_issuance(
            Uint128(7_000_000_000_000_000_000), 
            Uint128(10_000_000), 
            6, 
            Uint128(5_000_000_000_000_000_000), 
            6, 
            Uint128(7_000_000_000_000_000_000));
        assert_eq!(result, Uint128(15_053_763));
        let result2 = calculate_issuance(
            Uint128(10_000_000_000_000_000_000), 
            Uint128(50_000_000), 
            6, 
            Uint128(50_000_000_000_000_000_000), 
            8, 
            Uint128(9_000_000_000_000_000_000),);
        assert_eq!(result2, Uint128(1_098_901_000));
        let result3 = calculate_issuance(
            Uint128(10_000_000_000_000_000_000), 
            Uint128(5_000_000_000), 
            8, 
            Uint128(50_000_000_000_000_000_000), 
            6, 
            Uint128(9_000_000_000_000_000_000),);
        assert_eq!(result3, Uint128(10989010));
    }
}

#[test]
fn create_and_read_opp(){

}
