use std::ops::Add;

use crate::{
    query,
    tests::{get_address, get_contract, get_status, init_contract, init_query_auth},
};
use shade_protocol::{
    c_std::{Addr, StdError},
    contract_interfaces::{admin, utility_router},
    utility_router::{UtilityAddresses, UtilityContracts},
    utils::{ExecuteCallback, Query},
    Contract,
};

// #[test]
// pub fn router_query() {
//     let (mut chain, router, local_admin, other_admin) = init_contract().unwrap();

//     // let query_auth = init_query_auth(&mut chain, &local_admin).unwrap();

//     let snip20 = init_snip20(&mut chain).unwrap();

//     // let msg = utility_router::ExecuteMsg::SetContract {
//     //     utility_contract_name: UtilityContracts::QueryAuth.into_string(),
//     //     contract: Contract {
//     //         address: query_auth.address,
//     //         code_hash: query_auth.code_hash
//     //     },
//     //     padding: None
//     // };

//     let msg = utility_router::ExecuteMsg::SetContract { utility_contract_name: "SHADE_SNIP20".to_string(), contract: Contract { address: snip20.address.clone(), code_hash: snip20.code_hash.clone()}, padding: None };

//     assert!(
//         msg.test_exec(&router, &mut chain, Addr::unchecked("admin"), &[])
//             .is_ok()
//     );

//     // let query: utility_router::QueryAnswer = utility_router::QueryMsg::ForwardQuery {
//     //     utility_name: UtilityContracts::QueryAuth.into_string(),
//     //     query: to_binary(&query_auth::QueryMsg::Config{ }).unwrap()
//     //     // query: to_binary("hey").unwrap()
//     // }.test_query(&router, &chain).unwrap();

//     let query: utility_router::QueryAnswer = utility_router::QueryMsg::ForwardQuery {
//         utility_name: "SHADE_SNIP20".to_string(),
//         query: to_binary(&snip20::QueryMsg::TokenInfo{}).unwrap()
//         // query: to_binary("hey").unwrap()
//     }.test_query(&router, &chain).unwrap();

//     match query {
//         utility_router::QueryAnswer::ForwardQuery { status, result } => {
//             // Config {
//             //     admin: Contract,
//             //     state: ContractStatus
//             // },
//             // let msg: SlipMsg = from_binary(&message)?;
//             let response: query_auth::QueryAnswer = from_binary(&result).unwrap();
//             match response {
//                 query_auth::QueryAnswer::Config { admin, state } => {
//                     assert_eq!(
//                         admin,
//                         Contract {
//                             address: local_admin.address,
//                             code_hash: local_admin.code_hash
//                         }
//                     )
//                 },
//                 _ => assert!(false)
//             }

//         },
//         _ => assert!(false)
//     }
// }

#[test]
fn get_multisig_maintenance_fail() {
    let (mut chain, router, admin, other_admin) = init_contract().unwrap();

    let msg = utility_router::ExecuteMsg::ToggleStatus {
        status: utility_router::RouterStatus::UnderMaintenance,
        padding: None,
    };

    assert!(
        &msg.test_exec(&router, &mut chain, Addr::unchecked("admin"), &[])
            .is_ok()
    );

    let result: Result<utility_router::QueryAnswer, StdError> =
        utility_router::QueryMsg::GetAddress {
            address_name: UtilityAddresses::Multisig.into_string(),
        }
        .test_query(&router, &chain);

    match result {
        Err(StdError::GenericErr { msg }) => assert_eq!(msg, "Querier contract error: Generic error: {\"target\":\"utility\",\"code\":4,\"type\":\"under_maintenance\",\"context\":[],\"verbose\":\"Cannot query for information, as router is under maintenance\"}".to_string()),
        _ => assert!(false)
    }
}

#[test]
fn get_multisig() {
    let (mut chain, router, admin, other_admin) = init_contract().unwrap();

    match get_address(&chain, &router, UtilityAddresses::Multisig.into_string()) {
        Ok(addr) => {
            assert_eq!(addr, "multisig_address_literal".to_string())
        }
        Err(_) => assert!(false),
    }
}