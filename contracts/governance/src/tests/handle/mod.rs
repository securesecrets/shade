pub mod assembly;
pub mod assembly_msg;
pub mod contract;
pub mod migration;
pub mod profile;
pub mod proposal;
pub mod runstate;

use crate::tests::{admin_only_governance, get_config, handle::proposal::init_funding_token};
use shade_protocol::{
    c_std::Addr,
    contract_interfaces::governance,
    utils::{asset::Contract, ExecuteCallback},
};

#[test]
fn set_config_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let old_config = get_config(&mut chain, &gov).unwrap();

    let snip20 = init_funding_token(&mut chain, None, None).unwrap();

    governance::ExecuteMsg::SetConfig {
        query_auth: None,
        treasury: Some(Addr::unchecked("random")),
        funding_token: Some(Contract {
            address: snip20.address.clone(),
            code_hash: snip20.code_hash.clone(),
        }),
        vote_token: Some(Contract {
            address: snip20.address,
            code_hash: snip20.code_hash,
        }),
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    let new_config = get_config(&mut chain, &gov).unwrap();

    assert_ne!(old_config.treasury, new_config.treasury);
    assert_ne!(old_config.funding_token, new_config.funding_token);
    assert_ne!(old_config.vote_token, new_config.vote_token);
}

#[test]
fn unauthorised_set_config_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(
        governance::ExecuteMsg::SetConfig {
            query_auth: None,
            treasury: None,
            funding_token: None,
            vote_token: None,
            padding: None,
        }
        .test_exec(
            // Sender is self
            &gov,
            &mut chain,
            Addr::unchecked("random"),
            &[]
        )
        .is_err()
    );
}

#[test]
fn reject_disable_config_tokens() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let snip20 = init_funding_token(&mut chain, None, None).unwrap();

    governance::ExecuteMsg::SetConfig {
        query_auth: None,
        treasury: Some(Addr::unchecked("random")),
        funding_token: Some(Contract {
            address: snip20.address.clone(),
            code_hash: snip20.code_hash.clone(),
        }),
        vote_token: Some(Contract {
            address: snip20.address,
            code_hash: snip20.code_hash,
        }),
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    let old_config = get_config(&mut chain, &gov).unwrap();

    governance::ExecuteMsg::SetConfig {
        query_auth: None,
        treasury: None,
        funding_token: None,
        vote_token: None,
        padding: None,
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();

    let new_config = get_config(&mut chain, &gov).unwrap();

    assert_eq!(old_config.treasury, new_config.treasury);
    assert_eq!(old_config.funding_token, new_config.funding_token);
    assert_eq!(old_config.vote_token, new_config.vote_token);
}
