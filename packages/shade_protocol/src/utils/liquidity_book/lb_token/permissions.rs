use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::expiration::Expiration;
use cosmwasm_std::{Addr, BlockInfo, Uint256};

/// struct to store permission for a `[token_id, owner, allowed_addr]` combination

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct Permission {
    pub view_balance_perm: bool,
    pub view_balance_exp: Expiration,
    pub view_pr_metadata_perm: bool,
    pub view_pr_metadata_exp: Expiration,
    pub trfer_allowance_perm: Uint256,
    pub trfer_allowance_exp: Expiration,
}

impl Permission {
    pub fn check_view_balance_perm(&self, blockinfo: &BlockInfo) -> bool {
        self.view_balance_perm && !self.view_balance_exp.is_expired(blockinfo)
    }
    pub fn check_view_pr_metadata_perm(&self, blockinfo: &BlockInfo) -> bool {
        self.view_pr_metadata_perm && !self.view_pr_metadata_exp.is_expired(blockinfo)
    }
}

/// to store all keys to access all permissions for a given `owner`
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PermissionKey {
    pub token_id: String,
    pub allowed_addr: Addr,
}
