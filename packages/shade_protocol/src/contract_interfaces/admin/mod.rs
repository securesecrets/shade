use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::utils::{Query};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg { }

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum HandleMsg {
	AddContract{
		contract_address: String
	},
	RemoveContract{
		contract_address: String
	},
	AddAuthorization {
		contract_address: String,
		admin_address: String
	},
	RemoveAuthorization { 
		contract_address: String,
		admin_address: String
	},
	AddSuper {
		super_address: String
	},
	RemoveSuper {
		super_address: String
	}
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum QueryMsg {
	GetSuperAdmins { },
	GetContracts { },
	GetAuthorizedUsers { contract_address: String },
	ValidateAdminPermission {
		contract_address: String,
		admin_address: String
	},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SuperAdminResponse {
	pub super_admins: Vec<String>
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ContractsResponse {
	pub contracts: Vec<(String, Vec<String>)>
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct AuthorizedUsersResponse {
	pub authorized_users: Vec<String>
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ValidateAdminPermissionResponse {
	pub error_msg: Option<String>
}