use crate::shared::is_valid_permission;
use rstest::*;
use shade_multi_test::multi::admin::Admin;
use shade_protocol::{
    admin::{
        AdminAuthStatus, AdminsResponse, ConfigResponse, ExecuteMsg, InstantiateMsg,
        PermissionsResponse, QueryMsg, RegistryAction, ValidateAdminPermissionResponse,
    },
    c_std::Addr,
    multi_test::App,
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

#[rstest]
#[case("VAULT", false)]
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
#[case(AdminAuthStatus::Active, vec![true, true, true, false, true, true, true])]
#[case(AdminAuthStatus::Maintenance, vec![true, true, true, false, true, true, true])]
#[case(AdminAuthStatus::Shutdown, vec![false, false, false, false, false, false, true])]
fn test_status(#[case] status: AdminAuthStatus, #[case] expect_success: Vec<bool>) {
    //init
    let mut chain: App = App::default();
    let contract = InstantiateMsg { super_admin: None }
        .test_init(
            Admin::default(),
            &mut chain,
            Addr::unchecked("admin"),
            "admin_contract",
            &[],
        )
        .unwrap();
    //set state
    ExecuteMsg::ToggleStatus { new_status: status }
        .test_exec(&contract, &mut chain, Addr::unchecked("admin"), &[])
        .unwrap();

    //register 'super' as admin
    let action = RegistryAction::RegisterAdmin {
        user: "super".to_string(),
    };
    let result = ExecuteMsg::UpdateRegistry {
        action: action.clone(),
    }
    .test_exec(&contract, &mut chain, Addr::unchecked("admin"), &[]);
    assert_eq!(&result.is_ok(), expect_success.get(0).unwrap());

    //test bulk update
    let actions = vec![action.clone()];
    let result = ExecuteMsg::UpdateRegistryBulk { actions }.test_exec(
        &contract,
        &mut chain,
        Addr::unchecked("admin"),
        &[],
    );
    assert_eq!(&result.is_ok(), expect_success.get(1).unwrap());

    //set super admin to 'super'
    let result = ExecuteMsg::TransferSuper {
        new_super: "super".to_string(),
    }
    .test_exec(&contract, &mut chain, Addr::unchecked("admin"), &[]);
    assert_eq!(&result.is_ok(), expect_success.get(2).unwrap());

    //register 'admin' as admin without being the super user
    let action = RegistryAction::RegisterAdmin {
        user: "admin".to_string(),
    };
    let result = ExecuteMsg::UpdateRegistry {
        action: action.clone(),
    }
    .test_exec(&contract, &mut chain, Addr::unchecked("admin"), &[]);
    assert_eq!(&result.is_ok(), expect_success.get(3).unwrap());

    //register admin as admin with correct permissions
    let action = RegistryAction::RegisterAdmin {
        user: "admin".to_string(),
    };
    let result = ExecuteMsg::UpdateRegistry {
        action: action.clone(),
    }
    .test_exec(&contract, &mut chain, Addr::unchecked("super"), &[]);
    assert_eq!(&result.is_ok(), expect_success.get(4).unwrap());

    //set super admin to 'admin'
    let result = ExecuteMsg::TransferSuper {
        new_super: "admin".to_string(),
    }
    .test_exec(&contract, &mut chain, Addr::unchecked("super"), &[]);
    assert_eq!(&result.is_ok(), expect_success.get(5).unwrap());

    //self destruct
    let result =
        ExecuteMsg::SelfDestruct {}.test_exec(&contract, &mut chain, Addr::unchecked("admin"), &[]);
    assert_eq!(&result.is_ok(), expect_success.get(6).unwrap());
}

#[rstest]
#[case(vec!["test", "blah"], vec!["test", "blah"], vec![false, false])]
#[case(vec!["test", "blah", "aaaa", "bbbb", "cccc"], vec!["test", "bbbb"], vec![false, true, true, false, true])]
fn test_admin(
    #[case] admins_to_add: Vec<&str>,
    #[case] admins_to_remove: Vec<&str>,
    #[case] expected_in_final_admins: Vec<bool>,
) {
    //init
    let mut chain = App::default();

    let admin_contract = InstantiateMsg { super_admin: None }
        .test_init(
            Admin::default(),
            &mut chain,
            Addr::unchecked("admin"),
            "admin_contract",
            &[],
        )
        .unwrap();

    //check config
    let config: ConfigResponse = QueryMsg::GetConfig {}
        .test_query(&admin_contract, &chain)
        .unwrap();
    assert_eq!(config.super_admin.as_str(), "admin");
    assert_eq!(config.status, AdminAuthStatus::Active);

    //read admins
    let response: AdminsResponse = QueryMsg::GetAdmins {}
        .test_query(&admin_contract, &chain)
        .unwrap();
    assert!(response.admins.is_empty());

    //add admins
    for admin in admins_to_add.iter() {
        ExecuteMsg::UpdateRegistry {
            action: RegistryAction::RegisterAdmin {
                user: admin.to_string(),
            },
        }
        .test_exec(&admin_contract, &mut chain, Addr::unchecked("admin"), &[])
        .unwrap();
    }

    //read admins
    let response: AdminsResponse = QueryMsg::GetAdmins {}
        .test_query(&admin_contract, &chain)
        .unwrap();
    let admin_list = response.admins;
    let admin_list_str: Vec<String> = admin_list.into_iter().map(|x| x.to_string()).collect();
    for admin in admins_to_add.iter() {
        assert!(admin_list_str.contains(&admin.to_string()));
    }

    //remove some admins
    for admin in admins_to_remove.iter() {
        ExecuteMsg::UpdateRegistry {
            action: RegistryAction::DeleteAdmin {
                user: admin.to_string(),
            },
        }
        .test_exec(&admin_contract, &mut chain, Addr::unchecked("admin"), &[])
        .unwrap();
    }

    //read admins
    let response: AdminsResponse = QueryMsg::GetAdmins {}
        .test_query(&admin_contract, &chain)
        .unwrap();
    let admin_list = response.admins;
    let admin_list_str: Vec<String> = admin_list.into_iter().map(|x| x.to_string()).collect();
    for (i, admin) in admins_to_add.iter().enumerate() {
        assert_eq!(
            &admin_list_str.contains(&admin.to_string()),
            expected_in_final_admins.get(i).unwrap()
        );
    }

    //remove all admins with batch
    let mut actions = vec![];
    for admin in &admins_to_add {
        actions.push(RegistryAction::DeleteAdmin {
            user: admin.to_string(),
        });
    }

    ExecuteMsg::UpdateRegistryBulk { actions }
        .test_exec(&admin_contract, &mut chain, Addr::unchecked("admin"), &[])
        .unwrap();

    //read admins
    let response: AdminsResponse = QueryMsg::GetAdmins {}
        .test_query(&admin_contract, &chain)
        .unwrap();
    let admin_list = response.admins;
    let admin_list_str: Vec<String> = admin_list.into_iter().map(|x| x.to_string()).collect();
    for admin in &admins_to_add {
        assert_eq!(&admin_list_str.contains(&admin.to_string()), &false);
    }
}

#[rstest]
#[case(
vec![
("user", vec!["SOME_TARGET"]),
("places", vec!["PLACE_SAN_JUAN", "PLACE_NEW_YORK", "PLACE_CAPRI_ISLAND"]),
("not_admin", vec!["SOME_TARGET_ONE", "SOME_TARGET_TWO", "TARGET_THREE"])
],
vec![
("places", vec!["PLACE_NEW_YORK"]),
("not_admin", vec!["SOME_TARGET_ONE", "TARGET_THREE"]),
("user", vec!["SOME_TARGET"]),
]
)]
fn test_permissions(
    #[case] permissions: Vec<(&str, Vec<&str>)>,
    #[case] revoke_permissions: Vec<(&str, Vec<&str>)>,
) {
    let mut chain = App::default();

    let admin = InstantiateMsg { super_admin: None }
        .test_init(
            Admin::default(),
            &mut chain,
            Addr::unchecked("admin"),
            "admin_contract",
            &[],
        )
        .unwrap();

    let mut actions = vec![];
    for permission in permissions.iter() {
        actions.append(&mut vec![
            RegistryAction::RegisterAdmin {
                user: permission.0.to_string(),
            },
            RegistryAction::GrantAccess {
                permissions: permission.1.iter().map(|&i| i.to_string()).collect(),
                user: permission.0.to_string(),
            },
        ])
    }

    // Check that only super admin chan do this
    assert!(ExecuteMsg::UpdateRegistryBulk {
        actions: actions.clone()
    }
    .test_exec(&admin, &mut chain, Addr::unchecked("user"), &[])
    .is_err());

    assert!(ExecuteMsg::UpdateRegistryBulk { actions }
        .test_exec(&admin, &mut chain, Addr::unchecked("admin"), &[])
        .is_ok());

    // Confirm that all permissions are set
    for permission in permissions.iter() {
        // Check that the permissions are correctly returned
        let stored_permissions: PermissionsResponse = QueryMsg::GetPermissions {
            user: permission.0.to_string(),
        }
        .test_query(&admin, &chain)
        .unwrap();

        assert_eq!(stored_permissions.permissions.len(), permission.1.len());
        for perm in permission.1.iter() {
            assert!(stored_permissions.permissions.contains(&perm.to_string()));

            // Check that no other permission is "accepted"
            let res: ValidateAdminPermissionResponse = QueryMsg::ValidateAdminPermission {
                permission: perm.to_string(),
                user: permission.0.to_string(),
            }
            .test_query(&admin, &chain)
            .unwrap();
            assert!(res.has_permission);
        }
    }

    // Remove permissions
    let revoke_actions: Vec<RegistryAction> = revoke_permissions
        .iter()
        .map(|permission| RegistryAction::RevokeAccess {
            permissions: permission.1.iter().map(|&item| item.to_string()).collect(),
            user: permission.0.to_string(),
        })
        .collect();

    assert!(ExecuteMsg::UpdateRegistryBulk {
        actions: revoke_actions.clone()
    }
    .test_exec(&admin, &mut chain, Addr::unchecked("user"), &[])
    .is_err());

    assert!(ExecuteMsg::UpdateRegistryBulk {
        actions: revoke_actions
    }
    .test_exec(&admin, &mut chain, Addr::unchecked("admin"), &[])
    .is_ok());

    for permission in permissions.iter() {
        // Check that the permissions are correctly returned
        let stored_permissions: PermissionsResponse = QueryMsg::GetPermissions {
            user: permission.0.to_string(),
        }
        .test_query(&admin, &chain)
        .unwrap();

        for perm in permission.1.iter() {
            let mut assertion: Option<bool> = None;
            for p in revoke_permissions.iter() {
                if p.0 == permission.0 {
                    assertion = Some(!p.1.contains(perm));

                    assert_eq!(
                        stored_permissions.permissions.len(),
                        permission.1.len().wrapping_sub(p.1.len())
                    );
                    break;
                }
            }
            assert!(assertion.is_some(), "Never found the required item");

            assert_eq!(
                stored_permissions.permissions.contains(&perm.to_string()),
                assertion.unwrap()
            );

            // Check that no other permission is "accepted"
            let res: ValidateAdminPermissionResponse = QueryMsg::ValidateAdminPermission {
                permission: perm.to_string(),
                user: permission.0.to_string(),
            }
            .test_query(&admin, &chain)
            .unwrap();
            assert_eq!(res.has_permission, assertion.unwrap());
        }
    }
}
