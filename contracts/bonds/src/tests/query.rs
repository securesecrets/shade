use fadroma::ensemble::{ContractEnsemble};
use cosmwasm_std::{HumanAddr};
use fadroma_platform_scrt::ContractLink;
use shade_protocol::contract_interfaces::{
    bonds, 
    snip20::{helpers::Snip20Asset}, 
    query_auth,
};

use cosmwasm_math_compat::Uint128;

pub fn query_no_opps(
    chain: &mut ContractEnsemble,
    bonds: &ContractLink<HumanAddr>,
) -> () {
    let msg = bonds::QueryMsg::BondOpportunities { };

    let query: bonds::QueryAnswer = chain.query(
        bonds.address.clone(), 
        &msg,
    ).unwrap();

    match query{
        bonds::QueryAnswer::BondOpportunities { bond_opportunities } => {
            assert_eq!(bond_opportunities, vec![]);
        }
        _ => assert!(false)
    }
}

pub fn query_opp_parameters(
    chain: &mut ContractEnsemble,
    bonds: &ContractLink<HumanAddr>,
    issuance_limit: Option<Uint128>,
    amount_issued: Option<Uint128>,
    deposit_denom: Option<Snip20Asset>,
    start_time: Option<u64>,
    end_time: Option<u64>,
    bonding_period: Option<u64>,
    discount: Option<Uint128>,
    max_accepted_collateral_price: Option<Uint128>,
    err_collateral_price: Option<Uint128>,
    minting_bond: Option<bool>
) -> () {
    let query: bonds::QueryAnswer = chain.query(
        bonds.address.clone(),
        &bonds::QueryMsg::BondOpportunities {  }
    ).unwrap();

    match query {
        bonds::QueryAnswer::BondOpportunities { bond_opportunities, .. } => {
            if issuance_limit.is_some() {
                assert_eq!(bond_opportunities[0].issuance_limit, issuance_limit.unwrap())
            }
            if amount_issued.is_some() {
                assert_eq!(bond_opportunities[0].amount_issued, amount_issued.unwrap())
            }
            if deposit_denom.is_some() {
                assert_eq!(bond_opportunities[0].deposit_denom, deposit_denom.unwrap())
            }
            if start_time.is_some() {
                assert_eq!(bond_opportunities[0].start_time, start_time.unwrap())
            }
            if end_time.is_some() {
                assert_eq!(bond_opportunities[0].end_time, end_time.unwrap())
            }
            if bonding_period.is_some() {
                assert_eq!(bond_opportunities[0].bonding_period, bonding_period.unwrap())
            }
            if discount.is_some() {
                assert_eq!(bond_opportunities[0].discount, discount.unwrap())
            }
            if max_accepted_collateral_price.is_some() {
                assert_eq!(bond_opportunities[0].max_accepted_collateral_price, max_accepted_collateral_price.unwrap())
            }
            if err_collateral_price.is_some() {
                assert_eq!(bond_opportunities[0].err_collateral_price, err_collateral_price.unwrap())
            }
            if minting_bond.is_some() {
                assert_eq!(bond_opportunities[0].minting_bond, minting_bond.unwrap())
            }
        }
        _ => assert!(false)
    };
}
