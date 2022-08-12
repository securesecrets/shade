use crate::{shared::is_valid_permission, contract::{instantiate, query, execute}};
use rstest::*;
use shade_protocol::{admin::{self, InstantiateMsg, ConfigResponse, AdminAuthStatus, QueryMsg, ExecuteMsg, RegistryAction, AdminsResponse}, c_std::{testing::{mock_dependencies, mock_env, mock_info}, MessageInfo, Addr, from_binary}, secret_storage_plus::KeyDeserialize};

#[rstest]
#[case("test", false)]
#[case("VAULT_", false)]
#[case("VAULT_TARGET", true)]
#[case("VAULT_TARG3T_2", true)]
#[case("", false)]
#[case("*@#$*!*#!#!#****", false)]
#[case("VAULT_TARGET_addr", false)]
fn test_is_valid_permission(#[case] permission: String, #[case] is_valid: bool) {
    let resp = is_valid_permission(permission.as_str());
    if is_valid {
        assert!(resp.is_ok());
    } else {
        assert!(resp.is_err());
    }
}


#[rstest]
#[case(vec!["test", "blah"], vec!["test", "blah"], vec![false, false])]
#[case(vec!["test", "blah", "aaaa", "bbbb", "cccc"], vec!["test", "bbbb"], vec![false, true, true, false, true])]
fn test_admin(#[case] admins_to_add: Vec<&str>, #[case] admins_to_remove: Vec<&str>, #[case] expected_in_final_admins: Vec<bool>,) {

    //init
    let mut deps = mock_dependencies();
    let env = mock_env();
    let msg_info = mock_info("admin", &[]);
    let init_msg = InstantiateMsg {
        super_admin: Some("admin".into())
    };
    instantiate(deps.as_mut().branch(), env.clone(), msg_info.clone(), init_msg).unwrap();

    //check config
    let config: ConfigResponse = from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::GetConfig{}).unwrap()).unwrap();
    assert_eq!(config.super_admin.as_str(), "admin");
    assert_eq!(config.status, AdminAuthStatus::Active);

    //read admins
    let response: AdminsResponse = from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::GetAdmins {}).unwrap()).unwrap();
    assert!(response.admins.is_empty());

    //add admins
    for admin in &admins_to_add {
        execute(deps.as_mut().branch(), env.clone(), msg_info.clone(), ExecuteMsg::UpdateRegistry { action: RegistryAction::RegisterAdmin { user: admin.to_string() } }).unwrap();
    }

    //read admins
    let response: AdminsResponse = from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::GetAdmins {}).unwrap()).unwrap();
    let admin_list = response.admins;
    let admin_list_str: Vec<String> = admin_list.into_iter().map(|x| x.to_string()).collect();
    for admin in &admins_to_add {
        assert!(admin_list_str.contains(&admin.to_string()));
    }

    //remove some admins
    for admin in &admins_to_remove {
        execute(deps.as_mut().branch(), env.clone(), msg_info.clone(), ExecuteMsg::UpdateRegistry { action: RegistryAction::DeleteAdmin { user: admin.to_string() } }).unwrap();
    }

    //read admins
    let response: AdminsResponse = from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::GetAdmins {}).unwrap()).unwrap();
    let admin_list = response.admins;
    let admin_list_str: Vec<String> = admin_list.into_iter().map(|x| x.to_string()).collect();
    for (i, admin) in admins_to_add.iter().enumerate() {
        assert_eq!(&admin_list_str.contains(&admin.to_string()), expected_in_final_admins.get(i).unwrap());
    }

    //remove all admins with batch
    let mut actions = vec![];
    for admin in &admins_to_add {
        actions.push(RegistryAction::DeleteAdmin { user: admin.to_string() });
    }
    execute(
        deps.as_mut().branch(),
        env.clone(),
        msg_info.clone(),
        ExecuteMsg::UpdateRegistryBulk {
            actions
        }
    ).unwrap();

    //read admins
    let response: AdminsResponse = from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::GetAdmins {}).unwrap()).unwrap();
    let admin_list = response.admins;
    let admin_list_str: Vec<String> = admin_list.into_iter().map(|x| x.to_string()).collect();
    for admin in &admins_to_add {
        assert_eq!(&admin_list_str.contains(&admin.to_string()), &false);
    }
}