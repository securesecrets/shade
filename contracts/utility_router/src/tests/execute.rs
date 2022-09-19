use std::ops::Add;

use crate::tests::{init_contract, init_query_auth, get_contract, get_status, get_address, forward_query};
use shade_protocol::{contract_interfaces::{admin, utility_router}, utility_router::{UtilityContracts, UtilityAddresses}, Contract};
use shade_protocol::c_std::Addr;
use shade_protocol::utils::ExecuteCallback;
#[test]
fn set_admin() {
    let (mut chain, router, admin, other_admin) = init_contract().unwrap();

    // assert!(set_contract(
    //     &chain, 
    //     &router, 
    //     UtilityContracts::AdminAuth.into_string(), 
    //     Contract {
    //         address: Addr::unchecked("some_addr".to_string()),
    //         code_hash: "some_hash".to_string()
    //     }, 
    //     "not_admin".to_string(),
    //     None
    // ).is_err());

    

    // let msg = query_auth::ExecuteMsg::SetAdminAuth {
    //     admin: Contract {
    //         address: Addr::unchecked("some_addr"),
    //         code_hash: "some_hash".to_string(),
    //     },
    //     padding: None,
    // };

    let msg = utility_router::ExecuteMsg::SetContract { 
        utility_contract_name: UtilityContracts::AdminAuth.into_string(), 
        contract: Contract {
            address: other_admin.address,
            code_hash: other_admin.code_hash,
        }, 
        query: None, 
        padding: None 
    };

    assert!(
        &msg.test_exec(&router, &mut chain, Addr::unchecked("not_admin"), &[])
            .is_err()
    );

    assert!(
        &msg.test_exec(&router, &mut chain, Addr::unchecked("admin"), &[])
            .is_ok()
    );
}


#[test]
fn get_multisig() {
    let (mut chain, router, admin, other_admin) = init_contract().unwrap();

    match get_address(&chain, &router, UtilityAddresses::Multisig.into_string()) {
        Ok(addr) => {
            assert_eq!(addr, "multisig_address_literal".to_string())
        },
        Err(_) => assert!(false)
    }
}

#[test]
fn set_multisig() {
    let (mut chain, router, admin, other_admin) = init_contract().unwrap();

    let msg = utility_router::ExecuteMsg::SetAddress { address_name: UtilityAddresses::Multisig.into_string(), address: "new_address".to_string(), padding: None };

    assert!(
        &msg.test_exec(&router, &mut chain, Addr::unchecked("not_admin"), &[])
            .is_err()
    );

    assert!(
        &msg.test_exec(&router, &mut chain, Addr::unchecked("admin"), &[])
            .is_ok()
    );

    match get_address(&chain, &router, UtilityAddresses::Multisig.into_string()) {
        Ok(addr) => {
            assert_eq!(addr, "new_address".to_string())
        },
        Err(_) => assert!(false)
    }
}

#[test]
fn set_some_address() {
    let (mut chain, router, admin, other_admin) = init_contract().unwrap();

    let msg = utility_router::ExecuteMsg::SetAddress { address_name: "SHADE_TREASURY_MULTISIG".to_string(), address: "some_address".to_string(), padding: None };

    assert!(
        &msg.test_exec(&router, &mut chain, Addr::unchecked("not_admin"), &[])
            .is_err()
    );

    assert!(
        &msg.test_exec(&router, &mut chain, Addr::unchecked("admin"), &[])
            .is_ok()
    );

    match get_address(&chain, &router, "SHADE_TREASURY_MULTISIG".to_string()) {
        Ok(addr) => {
            assert_eq!(addr, "some_address".to_string())
        },
        Err(_) => assert!(false)
    }
}

#[test]
fn set_contract() {
    let (mut chain, router, admin, other_admin) = init_contract().unwrap();

    let query_auth = init_query_auth(chain, &admin).unwrap();

    let msg = utility_router::ExecuteMsg::SetContract { 
        utility_contract_name: UtilityContracts::QueryAuth.into_string(), 
        contract: Contract {}, 
        query: (), 
        padding: () 
    };
}