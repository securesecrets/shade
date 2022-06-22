use crate::tests::init_contracts;
use fadroma::ensemble::{MockEnv, ContractEnsemble};
use cosmwasm_std::HumanAddr;
use shade_protocol::contract_interfaces::{bonds, snip20, oracles::{band, oracle}};
use shade_admin::admin;
use cosmwasm_math_compat::Uint128;
use shade_protocol::utils::asset::Contract;

#[test]
pub fn set_admin() {
    let (mut chain, 
        bonds, 
        issu, 
        coll, 
        band, 
        oracle,
        query_auth,
        shade_admins
    ) = init_contracts().unwrap();

    let msg = admin::HandleMsg::AddContract { contract_address: bonds.address.clone().to_string() };

    assert!(chain.execute(&msg, MockEnv::new("admin", shade_admins.clone())).is_ok());

    let query: bonds::QueryAnswer = chain.query(
        bonds.address,
        &bonds::QueryMsg::Config {  }
    ).unwrap();

    // match query {
    //     bonds::QueryAnswer::Config { config, .. } => {
    //         assert_eq!(config.admin, vec![HumanAddr::from("admin"), HumanAddr::from("new_admin")]);
    //     }
    //     _ => assert!(false)
    // };
}

#[test]
pub fn purchase_opportunity() {
    let (mut chain, 
        bonds, 
        issu, 
        coll, 
        band, 
        oracle,
        query_auth,
        shade_admins
    ) = init_contracts().unwrap();

    // let msg = band

    let msg = admin::HandleMsg::AddContract { contract_address: bonds.address.clone().to_string() };

    assert!(chain.execute(&msg, MockEnv::new("admin", shade_admins.clone())).is_ok());

    let msg = snip20::HandleMsg::IncreaseAllowance { spender: bonds.address.clone(), amount: Uint128::new(9999999999), expiration: None, padding: None };

    assert!(chain.execute(&msg, MockEnv::new("admin", issu.clone())).is_ok());

    let msg = bonds::HandleMsg::OpenBond { 
        collateral_asset: Contract {
            address: coll.address,
            code_hash: coll.code_hash
        }, 
        start_time: chain.block().time, 
        end_time: (chain.block().time + 2), 
        bond_issuance_limit: Some(Uint128::new(1000000)), 
        bonding_period: Some(1), 
        discount: Some(Uint128::new(1000)), 
        max_accepted_collateral_price: Uint128::new(10000000000000000000000000), 
        err_collateral_price: Uint128::new(10000000000000000000000000), 
        minting_bond: false, 
        padding: None
    };

    assert!(chain.execute(&msg, MockEnv::new("admin", bonds.clone())).is_ok());

    let query: bonds::QueryAnswer = chain.query(
        bonds.address,
        &bonds::QueryMsg::BondOpportunities {  }
    ).unwrap();

    match query {
        bonds::QueryAnswer::BondOpportunities { bond_opportunities, .. } => {
            assert_eq!(bond_opportunities[0].discount, Uint128::new(10000))
        }
        _ => assert!(false)
    };

    // let msg = bonds::
}