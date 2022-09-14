pub mod handle;
pub mod query;

use shade_multi_test::multi::{admin::Admin, query_auth::QueryAuth};
use shade_protocol::{
    admin::{self, helpers::AdminPermissions},
    c_std::{Addr, Binary, ContractInfo, StdError, StdResult},
    contract_interfaces::query_auth::{self, PermitData, QueryPermit},
    multi_test::{App},
    query_auth::{ContractStatus},
    query_authentication::transaction::{PermitSignature, PubKey},
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
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

    let auth = query_auth::InstantiateMsg {
        admin_auth: Contract {
            address: admin.address.clone(),
            code_hash: admin.code_hash.clone(),
        },
        prng_seed: Binary::from("random".as_bytes()),
    }
    .test_init(
        QueryAuth::default(),
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

pub fn get_permit() -> QueryPermit {
    QueryPermit {
        params: PermitData {
            key: "key".to_string(),
            data: Binary::from_base64("c29tZSBzdHJpbmc=").unwrap()
        },
        signature: PermitSignature {
            pub_key: PubKey::new(
                Binary::from_base64(
                    "A9NjbriiP7OXCpoTov9ox/35+h5k0y1K0qCY/B09YzAP"
                ).unwrap()
            ),
            signature: Binary::from_base64(
                "XRzykrPmMs0ZhksNXX+eU0TM21fYBZXZogr5wYZGGy11t2ntfySuQNQJEw6D4QKvPsiU9gYMsQ259dOzMZNAEg=="
            ).unwrap()
        },
        account_number: None,
        chain_id: Some(String::from("chain")),
        sequence: None,
        memo: None
    }
}

pub fn get_config(chain: &App, auth: &ContractInfo) -> StdResult<(Contract, ContractStatus)> {
    let query: query_auth::QueryAnswer =
        query_auth::QueryMsg::Config {}.test_query(&auth, &chain)?;

    match query {
        query_auth::QueryAnswer::Config { admin, state } => Ok((admin, state)),
        _ => Err(StdError::generic_err("Config not found")),
    }
}

pub fn validate_vk(chain: &App, auth: &ContractInfo, user: &str, key: &str) -> StdResult<bool> {
    let query: query_auth::QueryAnswer = query_auth::QueryMsg::ValidateViewingKey {
        user: Addr::unchecked(user),
        key: key.to_string(),
    }
    .test_query(&auth, &chain)?;

    match query {
        query_auth::QueryAnswer::ValidateViewingKey { is_valid } => Ok(is_valid),
        _ => Err(StdError::generic_err("VK not found")),
    }
}

pub fn validate_permit(chain: &App, auth: &ContractInfo) -> StdResult<(Addr, bool)> {
    let query: query_auth::QueryAnswer = query_auth::QueryMsg::ValidatePermit {
        permit: get_permit(),
    }
    .test_query(&auth, &chain)?;

    match query {
        query_auth::QueryAnswer::ValidatePermit { user, is_revoked } => Ok((user, is_revoked)),
        _ => Err(StdError::generic_err("VK not found")),
    }
}
