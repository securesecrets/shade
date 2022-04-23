
// TODO: Queries
// TODO: Query a range of proposals
// TODO: Query where end range is greater than total
// TODO: Check proposal without voting or funding and see how it returns

// TODO: Verify proposal history

// TODO: Query a range of assemblies
// TODO: Query where end range is greater than total

// TODO: Query a range of assembly msgs
// TODO: Query where end range is greater than total

// TODO: Query a range of profiles
// TODO: Query where end range is greater than total

// TODO: Query a range of contracts
// TODO: Query where end range is greater than total

// TODO: Query user funding
// TODO: Query where theres no user funding

// TODO: Query user assembly vote
// TODO: Query where theres no user vote

// TODO: Query user vote
// TODO: Query where theres no user vote

// TODO: funding privacy

use cosmwasm_math_compat::Uint128;
use crate::tests::{admin_only_governance, get_assemblies, get_assembly_msgs, get_config, get_contract, get_profiles};

#[test]
fn query_assembly_msg() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let assemblies = get_assembly_msgs(
        &mut chain, &gov, Uint128::zero(), Uint128::zero()
    ).unwrap();

    assert_eq!(assemblies.len(), 1);
}

#[test]
fn query_assembly_msg_large_end() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let assemblies = get_assembly_msgs(
        &mut chain, &gov, Uint128::zero(), Uint128::new(10)
    ).unwrap();

    assert_eq!(assemblies.len(), 1);
}

#[test]
fn query_assembly_msg_wrong_index() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let assemblies = get_assembly_msgs(
        &mut chain, &gov, Uint128::new(5), Uint128::new(10)
    ).is_err();
}

#[test]
fn query_contracts() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let contracts = get_contract(
        &mut chain, &gov, Uint128::zero(), Uint128::zero()
    ).unwrap();

    assert_eq!(contracts.len(), 1);
}

#[test]
fn query_contracts_large_end() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let contracts = get_contract(
        &mut chain, &gov, Uint128::zero(), Uint128::new(10)
    ).unwrap();

    assert_eq!(contracts.len(), 1);
}

#[test]
fn query_contracts_wrong_index() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    get_contract(
        &mut chain, &gov, Uint128::new(5), Uint128::new(10)
    ).is_err();
}

#[test]
fn query_profiles() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let profiles = get_profiles(
        &mut chain, &gov, Uint128::zero(), Uint128::zero()
    ).unwrap();

    assert_eq!(profiles.len(), 1);
}

#[test]
fn query_profiles_large_end() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let profiles = get_profiles(
        &mut chain, &gov, Uint128::zero(), Uint128::new(10)
    ).unwrap();

    assert_eq!(profiles.len(), 2);
}

#[test]
fn query_profiles_wrong_index() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    get_profiles(
        &mut chain, &gov, Uint128::new(5), Uint128::new(10)
    ).is_err();
}

#[test]
fn query_assemblies() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let assemblies = get_assemblies(
        &mut chain, &gov, Uint128::zero(), Uint128::zero()
    ).unwrap();

    assert_eq!(assemblies.len(), 1);
}

#[test]
fn query_assemblies_large_end() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    let assemblies = get_assemblies(
        &mut chain, &gov, Uint128::zero(), Uint128::new(10)
    ).unwrap();

    assert_eq!(assemblies.len(), 2);
}

#[test]
fn query_assemblies_wrong_index() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    get_assemblies(
        &mut chain, &gov, Uint128::new(5), Uint128::new(10)
    ).is_err();
}

#[test]
fn query_config() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    get_config(
        &mut chain, &gov
    ).unwrap();
}