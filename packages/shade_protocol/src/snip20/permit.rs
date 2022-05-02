use cosmwasm_std::HumanAddr;
use query_authentication::{
    permit::{bech32_to_canonical, Permit},
    transaction::SignedTx,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

pub type Snip20Permit = Permit<Params>;

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Params {
    pub allowed_tokens: Vec<HumanAddr>,
    pub permissions: Vec<Permission>,
    pub permit_name: String,
}

impl Params {
    pub fn check_token(&self, token: &HumanAddr) -> bool {
        self.allowed_tokens.contains(token)
    }

    pub fn check_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Allowance for SNIP-20 - Permission to query allowance of the owner & spender
    Allowance,
    /// Balance for SNIP-20 - Permission to query balance
    Balance,
    /// History for SNIP-20 - Permission to query transfer_history & transaction_hisotry
    History,
    /// Owner permission indicates that the bearer of this permit should be granted all
    /// the access of the creator/signer of the permit.  SNIP-721 uses this to grant
    /// viewing access to all data that the permit creator owns and is whitelisted for.
    /// For SNIP-721 use, a permit with Owner permission should NEVER be given to
    /// anyone else.  If someone wants to share private data, they should whitelist
    /// the address they want to share with via a SetWhitelistedApproval tx, and that
    /// address will view the data by creating their own permit with Owner permission
    Owner,
}
