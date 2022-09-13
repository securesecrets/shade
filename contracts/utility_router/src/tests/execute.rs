use std::ops::Add;

use crate::tests::{init_contract, get_contract, get_status, forward_query};
use shade_protocol::{contract_interfaces::{admin, utility_router}, utility_router::UtilityContracts, Contract};
use shade_protocol::c_std::Addr;
use shade_protocol::utils::ExecuteCallback;
#[test]
fn set_admin() {
    let (mut chain, router, admin) = init_contract().unwrap();

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

    // let msg = utility_router::ExecuteMsg::SetContract { 
    //     utility_contract_name: UtilityContracts::AdminAuth.into_string(), 
    //     contract: Contract {
    //         address: Addr::unchecked("some_addr"),
    //         code_hash: "some_hash".to_string(),
    //     }, 
    //     query: admin::QueryMsg::GetConfig {  }, 
    //     padding: None 
    // };



    // assert!(
    //     &msg.test_exec(&auth, &mut chain, Addr::unchecked("not_admin"), &[])
    //         .is_err()
    // );

    // assert!(
    //     &msg.test_exec(&auth, &mut chain, Addr::unchecked("admin"), &[])
    //         .is_ok()
    // );

    // match get_config(&chain, &auth) {
    //     Ok((admin, _)) => assert_eq!(admin.address, Addr::unchecked("some_addr")),
    //     Err(_) => assert!(false),
    // };

    
}