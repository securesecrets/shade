use crate::tests::{admin_only_governance, get_profiles};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::HumanAddr;
use fadroma::ensemble::MockEnv;
use shade_protocol::contract_interfaces::{
    governance,
    governance::profile::{Count, Profile, UpdateFundProfile, UpdateProfile, UpdateVoteProfile},
};

#[test]
fn add_profile() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::AddProfile {
                profile: Profile {
                    name: "Other Profile".to_string(),
                    enabled: false,
                    assembly: None,
                    funding: None,
                    token: None,
                    cancel_deadline: 0,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let profiles = get_profiles(&mut chain, &gov, Uint128::zero(), Uint128::new(10)).unwrap();

    assert_eq!(profiles.len(), 3);
}
#[test]
fn unauthorised_add_profile() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::AddProfile {
                profile: Profile {
                    name: "Other Profile".to_string(),
                    enabled: false,
                    assembly: None,
                    funding: None,
                    token: None,
                    cancel_deadline: 0,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("random"),
                gov.clone(),
            ),
        )
        .is_err();
}

#[test]
fn set_profile() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let old_profile =
        get_profiles(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    chain
        .execute(
            &governance::HandleMsg::SetProfile {
                id: Uint128::new(1),
                profile: UpdateProfile {
                    name: Some("New Name".to_string()),
                    enabled: None,
                    disable_assembly: false,
                    assembly: None,
                    disable_funding: false,
                    funding: None,
                    disable_token: false,
                    token: None,
                    cancel_deadline: None,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let new_profile =
        get_profiles(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    assert_ne!(new_profile.name, old_profile.name);
    assert_eq!(new_profile.assembly, old_profile.assembly);
    assert_eq!(new_profile.funding, old_profile.funding);
    assert_eq!(new_profile.token, old_profile.token);
    assert_eq!(new_profile.enabled, old_profile.enabled);
    assert_eq!(new_profile.cancel_deadline, old_profile.cancel_deadline);
}

#[test]
fn unauthorised_set_profile() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::SetProfile {
                id: Uint128::new(1),
                profile: UpdateProfile {
                    name: Some("New Name".to_string()),
                    enabled: None,
                    disable_assembly: false,
                    assembly: None,
                    disable_funding: false,
                    funding: None,
                    disable_token: false,
                    token: None,
                    cancel_deadline: None,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                HumanAddr::from("random"),
                gov.clone(),
            ),
        )
        .is_err();
}

#[test]
fn set_profile_disable_assembly() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::SetProfile {
                id: Uint128::new(1),
                profile: UpdateProfile {
                    name: None,
                    enabled: None,
                    disable_assembly: false,
                    assembly: Some(UpdateVoteProfile {
                        deadline: Some(0),
                        threshold: Some(Count::LiteralCount {
                            count: Uint128::zero(),
                        }),
                        yes_threshold: Some(Count::LiteralCount {
                            count: Uint128::zero(),
                        }),
                        veto_threshold: Some(Count::LiteralCount {
                            count: Uint128::zero(),
                        }),
                    }),
                    disable_funding: false,
                    funding: None,
                    disable_token: false,
                    token: None,
                    cancel_deadline: None,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let old_profile =
        get_profiles(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    chain
        .execute(
            &governance::HandleMsg::SetProfile {
                id: Uint128::new(1),
                profile: UpdateProfile {
                    name: None,
                    enabled: None,
                    disable_assembly: true,
                    assembly: None,
                    disable_funding: false,
                    funding: None,
                    disable_token: false,
                    token: None,
                    cancel_deadline: None,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let new_profile =
        get_profiles(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    assert_eq!(new_profile.name, old_profile.name);
    assert_ne!(new_profile.assembly, old_profile.assembly);
    assert_eq!(new_profile.funding, old_profile.funding);
    assert_eq!(new_profile.token, old_profile.token);
    assert_eq!(new_profile.enabled, old_profile.enabled);
    assert_eq!(new_profile.cancel_deadline, old_profile.cancel_deadline);
}

#[test]
fn set_profile_set_incomplete_assembly() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::SetProfile {
                id: Uint128::new(1),
                profile: UpdateProfile {
                    name: None,
                    enabled: None,
                    disable_assembly: false,
                    assembly: Some(UpdateVoteProfile {
                        deadline: Some(0),
                        threshold: None,
                        yes_threshold: None,
                        veto_threshold: Some(Count::LiteralCount {
                            count: Uint128::zero(),
                        }),
                    }),
                    disable_funding: false,
                    funding: None,
                    disable_token: false,
                    token: None,
                    cancel_deadline: None,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .is_err();
}

#[test]
fn set_profile_disable_token() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::SetProfile {
                id: Uint128::new(1),
                profile: UpdateProfile {
                    name: None,
                    enabled: None,
                    disable_assembly: false,
                    assembly: None,
                    disable_funding: false,
                    funding: None,
                    disable_token: false,
                    token: Some(UpdateVoteProfile {
                        deadline: Some(0),
                        threshold: Some(Count::LiteralCount {
                            count: Uint128::zero(),
                        }),
                        yes_threshold: Some(Count::LiteralCount {
                            count: Uint128::zero(),
                        }),
                        veto_threshold: Some(Count::LiteralCount {
                            count: Uint128::zero(),
                        }),
                    }),
                    cancel_deadline: None,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let old_profile =
        get_profiles(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    chain
        .execute(
            &governance::HandleMsg::SetProfile {
                id: Uint128::new(1),
                profile: UpdateProfile {
                    name: None,
                    enabled: None,
                    disable_assembly: false,
                    assembly: None,
                    disable_funding: false,
                    funding: None,
                    disable_token: true,
                    token: None,
                    cancel_deadline: None,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let new_profile =
        get_profiles(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    assert_eq!(new_profile.name, old_profile.name);
    assert_eq!(new_profile.assembly, old_profile.assembly);
    assert_eq!(new_profile.funding, old_profile.funding);
    assert_ne!(new_profile.token, old_profile.token);
    assert_eq!(new_profile.enabled, old_profile.enabled);
    assert_eq!(new_profile.cancel_deadline, old_profile.cancel_deadline);
}

#[test]
fn set_profile_set_incomplete_token() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::SetProfile {
                id: Uint128::new(1),
                profile: UpdateProfile {
                    name: None,
                    enabled: None,
                    disable_assembly: false,
                    assembly: None,
                    disable_funding: false,
                    funding: None,
                    disable_token: false,
                    token: Some(UpdateVoteProfile {
                        deadline: Some(0),
                        threshold: None,
                        yes_threshold: None,
                        veto_threshold: Some(Count::LiteralCount {
                            count: Uint128::zero(),
                        }),
                    }),
                    cancel_deadline: None,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .is_err();
}

#[test]
fn set_profile_disable_funding() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::SetProfile {
                id: Uint128::new(1),
                profile: UpdateProfile {
                    name: None,
                    enabled: None,
                    disable_assembly: false,
                    assembly: None,
                    disable_funding: false,
                    funding: Some(UpdateFundProfile {
                        deadline: Some(0),
                        required: Some(Uint128::zero()),
                        privacy: Some(true),
                        veto_deposit_loss: Some(Uint128::zero()),
                    }),
                    disable_token: false,
                    token: None,
                    cancel_deadline: None,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let old_profile =
        get_profiles(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    chain
        .execute(
            &governance::HandleMsg::SetProfile {
                id: Uint128::new(1),
                profile: UpdateProfile {
                    name: None,
                    enabled: None,
                    disable_assembly: false,
                    assembly: None,
                    disable_funding: true,
                    funding: None,
                    disable_token: false,
                    token: None,
                    cancel_deadline: None,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .unwrap();

    let new_profile =
        get_profiles(&mut chain, &gov, Uint128::new(1), Uint128::new(1)).unwrap()[0].clone();

    assert_eq!(new_profile.name, old_profile.name);
    assert_eq!(new_profile.assembly, old_profile.assembly);
    assert_ne!(new_profile.funding, old_profile.funding);
    assert_eq!(new_profile.token, old_profile.token);
    assert_eq!(new_profile.enabled, old_profile.enabled);
    assert_eq!(new_profile.cancel_deadline, old_profile.cancel_deadline);
}

#[test]
fn set_profile_set_incomplete_fuding() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain
        .execute(
            &governance::HandleMsg::SetProfile {
                id: Uint128::new(1),
                profile: UpdateProfile {
                    name: None,
                    enabled: None,
                    disable_assembly: false,
                    assembly: None,
                    disable_funding: false,
                    funding: Some(UpdateFundProfile {
                        deadline: Some(0),
                        required: None,
                        privacy: Some(true),
                        veto_deposit_loss: None,
                    }),
                    disable_token: false,
                    token: None,
                    cancel_deadline: None,
                },
                padding: None,
            },
            MockEnv::new(
                // Sender is self
                gov.address.clone(),
                gov.clone(),
            ),
        )
        .is_err();
}
