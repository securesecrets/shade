use crate::tests::{
    admin_only_governance,
    get_assemblies,
    get_assembly_msgs,
    get_config,
    get_contract,
    get_profiles,
};
use shade_protocol::{contract_interfaces::governance, utils::Query};

#[test]
fn query_total_assembly_msg() {
    let (chain, gov) = admin_only_governance().unwrap();

    let query: governance::QueryAnswer = governance::QueryMsg::TotalAssemblyMsgs {}
        .test_query(&gov, &chain)
        .unwrap();

    let total = match query {
        governance::QueryAnswer::Total { total } => total,
        _ => 0,
    };

    assert_eq!(total, 1);
}

#[test]
fn query_assembly_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let assemblies = get_assembly_msgs(&mut chain, &gov, 0, 0).unwrap();

    assert_eq!(assemblies.len(), 1);
}

#[test]
fn query_assembly_msg_large_end() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let assemblies = get_assembly_msgs(&mut chain, &gov, 0, 10).unwrap();

    assert_eq!(assemblies.len(), 1);
}

#[test]
fn query_assembly_msg_wrong_index() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(get_assembly_msgs(&mut chain, &gov, 5, 10).is_err());
}

#[test]
fn query_total_contracts() {
    let (chain, gov) = admin_only_governance().unwrap();

    let query: governance::QueryAnswer = governance::QueryMsg::TotalContracts {}
        .test_query(&gov, &chain)
        .unwrap();

    let total = match query {
        governance::QueryAnswer::Total { total } => total,
        _ => 0,
    };

    assert_eq!(total, 1);
}

#[test]
fn query_contracts() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let contracts = get_contract(&mut chain, &gov, 0, 0).unwrap();

    assert_eq!(contracts.len(), 1);
}

#[test]
fn query_contracts_large_end() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let contracts = get_contract(&mut chain, &gov, 0, 10).unwrap();

    assert_eq!(contracts.len(), 1);
}

#[test]
fn query_contracts_wrong_index() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(get_contract(&mut chain, &gov, 5, 10).is_err());
}

#[test]
fn query_total_profiles() {
    let (chain, gov) = admin_only_governance().unwrap();

    let query: governance::QueryAnswer = governance::QueryMsg::TotalProfiles {}
        .test_query(&gov, &chain)
        .unwrap();

    let total = match query {
        governance::QueryAnswer::Total { total } => total,
        _ => 0,
    };

    assert_eq!(total, 2);
}

#[test]
fn query_profiles() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let profiles = get_profiles(&mut chain, &gov, 0, 0).unwrap();

    assert_eq!(profiles.len(), 1);
}

#[test]
fn query_profiles_large_end() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let profiles = get_profiles(&mut chain, &gov, 0, 10).unwrap();

    assert_eq!(profiles.len(), 2);
}

#[test]
fn query_profiles_wrong_index() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(get_profiles(&mut chain, &gov, 5, 10).is_err());
}

#[test]
fn query_total_assemblies() {
    let (chain, gov) = admin_only_governance().unwrap();

    let query: governance::QueryAnswer = governance::QueryMsg::TotalAssemblies {}
        .test_query(&gov, &chain)
        .unwrap();

    let total = match query {
        governance::QueryAnswer::Total { total } => total,
        _ => 0,
    };

    assert_eq!(total, 2);
}

#[test]
fn query_assemblies() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let assemblies = get_assemblies(&mut chain, &gov, 0, 0).unwrap();

    assert_eq!(assemblies.len(), 1);
}

#[test]
fn query_assemblies_large_end() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let assemblies = get_assemblies(&mut chain, &gov, 0, 10).unwrap();

    assert_eq!(assemblies.len(), 2);
}

#[test]
fn query_assemblies_wrong_index() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(get_assemblies(&mut chain, &gov, 5, 10).is_err());
}

#[test]
fn query_config() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    get_config(&mut chain, &gov).unwrap();
}
