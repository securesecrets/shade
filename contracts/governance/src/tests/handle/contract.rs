use cosmwasm_std::HumanAddr;
use fadroma_ensemble::MockEnv;
use cosmwasm_math_compat::Uint128;
use shade_protocol::governance;
use shade_protocol::utils::asset::Contract;
use crate::tests::{admin_only_governance, get_contract};

// TODO: Edit existing contract
// TODO: Edit existing contract as a non gov
// TODO: Add a new contract
// TODO: Add a new contract as a non gov
#[test]
fn add_contract() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain.execute(
        &governance::HandleMsg::AddContract {
            name: "Contract".to_string(),
            metadata: "some description".to_string(),
            contract: Contract {
                address: HumanAddr::from("contract"),
                code_hash: "hash".to_string()
            },
            padding: None
        },
        MockEnv::new(
            // Sender is self
            gov.address.clone(),
            gov.clone()
        )
    ).unwrap();

    let contracts = get_contract(
        &mut chain, &gov, Uint128::zero(), Uint128::new(1)
    ).unwrap();

    assert_eq!(contracts.len(), 2);
}
#[test]
fn unauthorised_add_contract() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain.execute(
        &governance::HandleMsg::AddContract {
            name: "Contract".to_string(),
            metadata: "some description".to_string(),
            contract: Contract {
                address: HumanAddr::from("contract"),
                code_hash: "hash".to_string()
            },
            padding: None
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("random"),
            gov.clone()
        )
    ).is_err();
}
#[test]
fn set_contract() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain.execute(
        &governance::HandleMsg::AddContract {
            name: "Contract".to_string(),
            metadata: "some description".to_string(),
            contract: Contract {
                address: HumanAddr::from("contract"),
                code_hash: "hash".to_string()
            },
            padding: None
        },
        MockEnv::new(
            // Sender is self
            gov.address.clone(),
            gov.clone()
        )
    ).unwrap();

    let old_contract = get_contract(
        &mut chain, &gov, Uint128::new(1), Uint128::new(1)
    ).unwrap()[0].clone();

    chain.execute(
        &governance::HandleMsg::SetContract {
            id: Uint128::new(1),
            name: Some("New name".to_string()),
            metadata: Some("New desc".to_string()),
            contract: Some(Contract {
                address: HumanAddr::from("new contract"),
                code_hash: "other hash".to_string()
            }),
            padding: None
        },
        MockEnv::new(
            // Sender is self
            gov.address.clone(),
            gov.clone()
        )
    ).unwrap();

    let new_contract = get_contract(
        &mut chain, &gov, Uint128::new(1), Uint128::new(1)
    ).unwrap()[0].clone();

    assert_ne!(old_contract.name, new_contract.name);
    assert_ne!(old_contract.metadata, new_contract.metadata);
    assert_ne!(old_contract.contract.address, new_contract.contract.address);
    assert_ne!(old_contract.contract.code_hash, new_contract.contract.code_hash);
}

#[test]
fn unauthorised_set_contract() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain.execute(
        &governance::HandleMsg::SetContract {
            id: Uint128::new(1),
            name: Some("New name".to_string()),
            metadata: Some("New desc".to_string()),
            contract: Some(Contract {
                address: HumanAddr::from("new contract"),
                code_hash: "other hash".to_string()
            }),
            padding: None
        },
        MockEnv::new(
            // Sender is self
            HumanAddr::from("random"),
            gov.clone()
        )
    ).is_err();
}