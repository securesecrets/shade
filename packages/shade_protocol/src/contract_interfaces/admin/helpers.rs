use crate::{
    admin::{errors::unauthorized_admin, QueryMsg, ValidateAdminPermissionResponse},
    utils::Query,
    Contract,
};
use cosmwasm_std::{QuerierWrapper, StdResult};

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
    SkyAdmin,
    LendAdmin,
    OraclesAdmin,
    OraclesPriceBot,
    SilkAdmin,
    ShadeSwapAdmin,
    StakingAdmin,
    DerivativeAdmin,
    Snip20MigrationAdmin,
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
            AdminPermissions::SkyAdmin => "SHADE_SKY_ADMIN",
            AdminPermissions::LendAdmin => "SHADE_LEND_ADMIN",
            AdminPermissions::OraclesAdmin => "SHADE_ORACLES_ADMIN",
            AdminPermissions::OraclesPriceBot => "SHADE_ORACLES_PRICE_BOT",
            AdminPermissions::SilkAdmin => "SHADE_SILK_ADMIN",
            AdminPermissions::ShadeSwapAdmin => "SHADE_SWAP_ADMIN",
            AdminPermissions::StakingAdmin => "SHADE_STAKING_ADMIN",
            AdminPermissions::DerivativeAdmin => "SHADE_DERIVATIVE_ADMIN",
            AdminPermissions::Snip20MigrationAdmin => "SNIP20_MIGRATION_ADMIN",
        }
        .to_string()
    }
}
