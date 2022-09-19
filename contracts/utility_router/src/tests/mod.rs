pub mod execute;
pub mod query;

use shade_multi_test::multi::{utility_router::UtilityRouter, admin::Admin};
use shade_protocol::{
    admin::{self, helpers::AdminPermissions},
    c_std::{Addr, Binary, ContractInfo, StdError, StdResult, Response},
    contract_interfaces::utility_router::*,
    multi_test::{App, Executor, Router, AppResponse},
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query}, utility_router, query_auth,
};

pub fn init_contract() -> StdResult<(App, ContractInfo, ContractInfo, ContractInfo)> {
    let mut chain = App::default();

    let admin = admin::InstantiateMsg {
        super_admin: Some("admin".into()),
    }
    .test_init(
        Admin::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "admin_auth",
        &[],
    )
    .unwrap();

    let other_admin = admin::InstantiateMsg {
        super_admin: Some("admin".into()),
    }
    .test_init(
        Admin::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "admin_auth",
        &[],
    )
    .unwrap();

    let router = utility_router::InstantiateMsg {
        admin_auth: Contract {
            address: admin.address.clone(),
            code_hash: admin.code_hash.clone(),
        },
        multisig_address: "multisig_address_literal".to_string()
    }
    .test_init(
        UtilityRouter::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "utility_router",
        &[],
    )
    .unwrap();

    // admin::ExecuteMsg::UpdateRegistryBulk {
    //     actions: vec![
    //         admin::RegistryAction::RegisterAdmin {
    //             user: "admin".to_string(),
    //         },
    //         admin::RegistryAction::GrantAccess {
    //             permissions: vec![AdminPermissions::QueryAuthAdmin.into_string()],
    //             user: "admin".to_string(),
    //         },
    //     ],
    // }
    // .test_exec(&admin, &mut chain, Addr::unchecked("admin"), &[])
    // .unwrap();

    Ok((chain, router, admin, other_admin))
}

pub fn init_query_auth(chain: &App, admin: &ContractInfo) -> StdResult<ContractInfo> {
    query_auth = query_auth::InstantiateMsg { 
        admin_auth: Contract { 
            address: admin.address, 
            code_hash: admin.code_hash 
        }, 
        prng_seed: "" 
    }
    .test_init(
        QueryAuth::default(), 
        &mut chain, 
        Addr::unchecked("admin"), 
        "query_auth", 
        &[]
    ).unwrap()
}

pub fn get_contract(chain: &App, router: &ContractInfo, name: String) -> StdResult<Contract> {
    let query: utility_router::QueryAnswer = 
    utility_router::QueryMsg::GetContract { utility_name: name }.test_query(router, chain)?;

    match query {
        utility_router::QueryAnswer::GetContract { status, contract } => {
            Ok(contract)
        },
        // Err(e) => Err(e),
        _ => Err(StdError::GenericErr { msg: "get_contract error".to_string() })
    }
}

pub fn get_address(chain: &App, router: &ContractInfo, name: String) -> StdResult<String> {
    let query: utility_router::QueryAnswer =
    utility_router::QueryMsg::GetAddress { address_name: name }.test_query(router, chain)?;

    match query {
        utility_router::QueryAnswer::GetAddress { status, address } => {
            Ok(address)
        },
        _ => Err(StdError::GenericErr { msg: "get_address error".to_string() })
    }
}

pub fn forward_query(chain: &App, router: &ContractInfo, name: String, query: Binary ) -> StdResult<Binary> {
    let query: utility_router::QueryAnswer = 
    utility_router::QueryMsg::ForwardQuery { utility_name: name, query  }.test_query(router, chain)?;

    match query {
        utility_router::QueryAnswer::ForwardQuery { status, result  } => {
            Ok(result)
        },
        // Err(e) => Err(e),
        _ => Err(StdError::GenericErr { msg: "forward_query error".to_string()  })
    }
}

pub fn get_status(chain: &App, router: &ContractInfo) -> StdResult<RouterStatus> {
    let query: utility_router::QueryAnswer = 
    utility_router::QueryMsg::Status {  }.test_query(router, chain)?;

    match query {
        utility_router::QueryAnswer::Status { contract_status } => {
            Ok(contract_status)
        },
        // Err(e) => Err(e),
        _ => Err(StdError::GenericErr { msg: "get_status error".to_string() })
    }
}

// pub fn set_contract(chain: &App, router: &ContractInfo, name: String, contract: Contract, sender: String, query: Option<Binary>) -> StdResult<AppResponse> {
//     // let execute: utility_router::HandleAnswer = 
//     let response: Result<AppResponse, Error> = utility_router::ExecuteMsg::SetContract { utility_contract_name: name, contract, query, padding: None }
//     .test_exec(router, &mut chain, Addr::unchecked(sender), &[]);
//     Ok(response)
// }

// pub fn toggle_status(chain: &App, router: &ContractInfo, status: RouterStatus, sender: String) -> StdResult<AppResponse> {
//     let response: Result<AppResponse, Error> = utility_router::ExecuteMsg::ToggleStatus { status, padding: None }
//     .test_exec(router, &mut chain, Addr::unchecked(sender), &[]);
//     Ok(response)
// }