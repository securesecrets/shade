use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128, Binary};
use crate::asset::Contract;
use crate::generic_response::ResponseStatus;
use crate::msg_traits::{Init, Handle, Query};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintConfig {
    pub owner: HumanAddr,
    pub silk: Contract,
    pub oracle: Contract,
    pub activated: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BurnableAsset {
    pub contract: Contract,
    pub burned_tokens: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub silk: Contract,
    pub oracle: Contract,
    pub initial_assets: Option<Vec<AssetMsg>>,
}

impl Init<'_> for InitMsg {}

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
        silk: Option<Contract>,
        oracle: Option<Contract>,
    },
    RegisterAsset {
        contract: Contract,
    },
    UpdateAsset {
        asset: HumanAddr,
        contract: Contract,
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SnipMsgHook {
    pub minimum_expected_amount: Uint128,
    pub mint_type: MintType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MintType {
    MintSilk {}
}

impl Handle<'_> for HandleMsg{}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init { status: ResponseStatus, address: HumanAddr },
    Migrate { status: ResponseStatus },
    UpdateConfig { status: ResponseStatus},
    RegisterAsset { status: ResponseStatus},
    UpdateAsset { status: ResponseStatus},
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

impl Query for QueryMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    SupportedAssets { assets: Vec<String>, },
    Asset { asset: BurnableAsset },
    Config { config: MintConfig },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AssetMsg {
    pub contract: Contract,
    pub burned_tokens: Option<Uint128>,
}
