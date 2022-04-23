pub mod contract;
pub mod assembly_msg;
pub mod profile;
pub mod assembly;
pub mod proposal;

use cosmwasm_std::HumanAddr;
use fadroma_ensemble::MockEnv;
use shade_protocol::governance;
use shade_protocol::utils::asset::Contract;
use crate::tests::{admin_only_governance, get_config};

#[test]
fn init_contract() {
    admin_only_governance().unwrap();
}

#[test]
fn set_config_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let old_config = get_config(
        &mut chain, &gov
    ).unwrap();

    chain.execute(
        &governance::HandleMsg::SetConfig {
            treasury: Some(HumanAddr::from("random")),
            funding_token: Some(Contract {
                address: HumanAddr::from("random"),
                code_hash: "random".to_string()
            }),
            vote_token: Some(Contract {
                address: HumanAddr::from("random"),
                code_hash: "random".to_string()
            }),
            padding: None
        },
        MockEnv::new(
            // Sender is self
            gov.address.clone(),
            gov.clone()
        )
    ).unwrap();

    let new_config = get_config(
        &mut chain, &gov
    ).unwrap();

    assert_ne!(old_config.treasury, new_config.treasury);
    assert_ne!(old_config.funding_token, new_config.funding_token);
    assert_ne!(old_config.vote_token, new_config.vote_token);
}

#[test]
fn unauthorised_set_config_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain.execute(
        &governance::HandleMsg::SetConfig {
            treasury: None,
            funding_token: None,
            vote_token: None,
            padding: None
        },
        MockEnv::new(
            // Sender is self
            "random",
            gov.clone()
        )
    ).is_err();
}

#[test]
fn reject_disable_config_tokens() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain.execute(
        &governance::HandleMsg::SetConfig {
            treasury: Some(HumanAddr::from("random")),
            funding_token: Some(Contract {
                address: HumanAddr::from("random"),
                code_hash: "random".to_string()
            }),
            vote_token: Some(Contract {
                address: HumanAddr::from("random"),
                code_hash: "random".to_string()
            }),
            padding: None
        },
        MockEnv::new(
            // Sender is self
            gov.address.clone(),
            gov.clone()
        )
    ).unwrap();

    let old_config = get_config(
        &mut chain, &gov
    ).unwrap();

    chain.execute(
        &governance::HandleMsg::SetConfig {
            treasury: None,
            funding_token: None,
            vote_token: None,
            padding: None
        },
        MockEnv::new(
            // Sender is self
            gov.address.clone(),
            gov.clone()
        )
    ).unwrap();

    let new_config = get_config(
        &mut chain, &gov
    ).unwrap();

    assert_eq!(old_config.treasury, new_config.treasury);
    assert_eq!(old_config.funding_token, new_config.funding_token);
    assert_eq!(old_config.vote_token, new_config.vote_token);
}