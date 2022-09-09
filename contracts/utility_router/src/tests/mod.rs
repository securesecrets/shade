
use shade_protocol::{
    admin::{self, helpers::AdminPermissions},
    c_std::{Addr, Binary, ContractInfo, StdError, StdResult},
    contract_interfaces::utility_router::*,
    multi_test::{App, Executor},
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query}, utility_router,
};

pub fn init_contract() -> StdResult<(App, ContractInfo)> {
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

    let router = utility_router::InstantiateMsg {
        admin_auth: Contract {
            address: admin.address.clone(),
            code_hash: admin.code_hash.clone(),
        },
    }
    .test_init(
        UtilityRouter::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "query_auth",
        &[],
    )
    .unwrap();

    admin::ExecuteMsg::UpdateRegistryBulk {
        actions: vec![
            admin::RegistryAction::RegisterAdmin {
                user: "admin".to_string(),
            },
            admin::RegistryAction::GrantAccess {
                permissions: vec![AdminPermissions::QueryAuthAdmin.into_string()],
                user: "admin".to_string(),
            },
        ],
    }
    .test_exec(&admin, &mut chain, Addr::unchecked("admin"), &[])
    .unwrap();

    Ok((chain, auth))
}