use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, CosmosMsg, Uint128, Binary, WasmMsg, to_binary, StdResult};
use crate::state::{Asset, Config, Contract};
use secret_toolkit::utils::space_pad;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub silk: Contract,
    pub oracle: Contract,
    pub initial_assets: Option<Vec<AssetMsg>>,
}

impl InitMsg {
    pub fn to_cosmos_msg(
        &self,
        mut block_size: usize,
        code_id: u64,
        callback_code_hash: String,
        label: String,
    ) -> StdResult<CosmosMsg> {
        // can not have block size of 0
        if block_size == 0 {
            block_size = 1;
        }
        let mut msg = to_binary(self)?;
        space_pad(&mut msg.0, block_size);
        let execute = WasmMsg::Instantiate {
            code_id,
            callback_code_hash,
            msg,
            send: vec![],
            label
        };
        Ok(execute.into())
    }
}

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
        msg: Option<CosmosMsg>,
    },
}

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

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    SupportedAssets { assets: Vec<String>, },
    Asset { asset: Asset },
    Config { config: Config },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AssetMsg {
    pub contract: Contract,
    pub burned_tokens: Option<Uint128>,
}

// Contract interactions
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleCall {
    pub contract: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}