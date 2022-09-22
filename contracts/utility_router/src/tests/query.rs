use std::ops::Add;

use crate::{
    query,
    tests::{get_address, get_contract, get_status, init_contract, init_query_auth},
};
use shade_protocol::{
    c_std::{Addr, StdError},
    contract_interfaces::{admin, utility_router},
    utility_router::{UtilityAddresses, UtilityContract},
    utils::{ExecuteCallback, Query},
    Contract,
};

#[test]
fn get_multisig_maintenance_fail() {
    let (mut chain, router, admin) = init_contract().unwrap();

    let msg = utility_router::ExecuteMsg::SetStatus {
        status: utility_router::RouterStatus::UnderMaintenance,
    };

    assert!(
        &msg.test_exec(&router, &mut chain, Addr::unchecked("admin"), &[])
            .is_ok()
    );

    let result: Result<utility_router::QueryAnswer, StdError> =
        utility_router::QueryMsg::GetAddress {
            key: UtilityAddresses::Multisig.into_string(),
        }
        .test_query(&router, &chain);

    match result {
        Err(StdError::GenericErr { msg }) => assert_eq!(msg, "Querier contract error: Generic error: {\"target\":\"utility\",\"code\":4,\"type\":\"under_maintenance\",\"context\":[],\"verbose\":\"Cannot query for information, as router is under maintenance\"}".to_string()),
        _ => assert!(false)
    }
}
