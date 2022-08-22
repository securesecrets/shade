use crate::{
    admin::{errors::unauthorized_admin, QueryMsg, ValidateAdminPermissionResponse},
    utils::Query,
    Contract,
};
use cosmwasm_std::{QuerierWrapper, StdError, StdResult};

pub fn validate_admin<T: Into<String> + Clone>(
    querier: &QuerierWrapper,
    permission: AdminPermissions,
    user: T,
    admin_auth: &Contract,
) -> StdResult<()> {
    if admin_is_valid(querier, permission.clone(), user.clone(), admin_auth)? {
        Ok(())
    } else {
        Err(unauthorized_admin(&user.into(), &permission.into_string()))
    }
}

pub fn admin_is_valid<T: Into<String>>(
    querier: &QuerierWrapper,
    permission: AdminPermissions,
    user: T,
    admin_auth: &Contract,
) -> StdResult<bool> {
    let admin_resp: StdResult<ValidateAdminPermissionResponse> =
        QueryMsg::ValidateAdminPermission {
            permission: permission.into_string(),
            user: user.into(),
        }
        .query(querier, admin_auth);

    match admin_resp {
        Ok(resp) => Ok(resp.has_permission),
        Err(err) => Err(err),
    }
}

#[derive(Clone)]
pub enum AdminPermissions {
    QueryAuthAdmin,
    ScrtStakingAdmin,
    TreasuryManager,
    TreasuryAdmin,
    StabilityAdmin,
}

// NOTE: SHADE_{CONTRACT_NAME}_{CONTRACT_ROLE}_{POTENTIAL IDs}

impl AdminPermissions {
    pub fn into_string(self) -> String {
        match self {
            AdminPermissions::QueryAuthAdmin => "SHADE_QUERY_AUTH_ADMIN",
            AdminPermissions::ScrtStakingAdmin => "SHADE_SCRT_STAKING_ADMIN",
            AdminPermissions::TreasuryManager => "SHADE_TREASURY_MANAGER",
            AdminPermissions::TreasuryAdmin => "SHADE_TREASURY_ADMIN",
            AdminPermissions::StabilityAdmin => "SHADE_STABILITY_ADMIN",
        }
        .to_string()
    }
}

/*
use shade_protocol::{
    c_std::{Addr, ContractInfo},
    contract_interfaces::admin::InstantiateMsg,
    multi_test::App,
    utils::InstantiateCallback,
};

/// Initializes an admin auth contract in multitest with superadmin as the superadmin.
pub fn init_admin_auth(app: &mut App, superadmin: &Addr) -> ContractInfo {
    InstantiateMsg {
        super_admin: Some(superadmin.clone().to_string()),
    }
    .test_init(Admin::default(), app, superadmin.clone(), "admin_auth", &[])
    .unwrap()
}
*/
