use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128, Binary};
use crate::asset::Contract;
use crate::generic_response::ResponseStatus;
use secret_toolkit::utils::{InitCallback, HandleCallback, Query};
use secretcli::secretcli::{TestInit, TestHandle, TestQuery};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintConfig {
    pub owner: HumanAddr,
    pub oracle: Contract,
    pub activated: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SupportedAsset {
    pub name: String,
    pub contract: Contract,
    pub burnable: bool,
    pub total_burned: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub oracle: Contract,
    pub initial_assets: Option<Vec<SupportedAsset>>,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

impl TestInit for InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Migrate {
        label: String,
        code_id: u64,
        code_hash: String,
    },
    UpdateConfig {
        owner: Option<HumanAddr>,
        oracle: Option<Contract>,
    },
    RegisterAsset {
        name: Option<String>,
        contract: Contract,
        burnable: Option<bool>,
        total_burned: Option<Uint128>,
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

impl TestHandle<HandleAnswer> for HandleMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SnipMsgHook {
    pub minimum_expected_amount: Uint128,
    pub to_mint: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MintMsgHook {
    pub minimum_expected_amount: Uint128,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init { status: ResponseStatus, address: HumanAddr },
    Migrate { status: ResponseStatus },
    UpdateConfig { status: ResponseStatus},
    RegisterAsset { status: ResponseStatus},
    Burn { status: ResponseStatus, mint_amount: Uint128 }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetSupportedAssets {},
    GetAsset {
        contract: String,
    },
    GetConfig {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

impl TestQuery<QueryAnswer> for QueryMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    SupportedAssets { assets: Vec<String>, },
    Asset { asset: SupportedAsset },
    Config { config: MintConfig },
}
