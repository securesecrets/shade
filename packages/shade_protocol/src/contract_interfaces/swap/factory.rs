use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    c_std::Binary,
    cosmwasm_schema::cw_serde,
    swap::{
        amm_pair::{AMMPair, AMMSettings, StableParams},
        core::{ContractInstantiationInfo, TokenPair},
        staking::StakingContractInstantiateInfo,
        Pagination,
    },
    utils::{ExecuteCallback, InstantiateCallback, Query},
    Contract, BLOCK_SIZE,
};

#[cw_serde]
pub struct InitMsg {
    pub pair_contract: ContractInstantiationInfo,
    pub amm_settings: AMMSettings,
    pub lp_token_contract: ContractInstantiationInfo,
    pub prng_seed: Binary,
    pub api_key: String,
    //Set the default authenticator for all permits on the contracts
    pub authenticator: Option<Contract>,
    pub admin_auth: Contract,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetConfig {
        pair_contract: Option<ContractInstantiationInfo>,
        lp_token_contract: Option<ContractInstantiationInfo>,
        amm_settings: Option<AMMSettings>,
        api_key: Option<String>,
        admin_auth: Option<Contract>,
        padding: Option<String>,
    },
    CreateAMMPair {
        pair: TokenPair,
        entropy: Binary,
        staking_contract: Option<StakingContractInstantiateInfo>,
        stable_params: Option<StableParams>,
        token0_oracle_key: Option<String>,
        token1_oracle_key: Option<String>,
        lp_token_decimals: u8,
        amm_pair_custom_label: Option<String>,
        lp_token_custom_label: Option<String>,
        padding: Option<String>,
    },
    AddAMMPairs {
        amm_pairs: Vec<AMMPair>,
        padding: Option<String>,
    },
}

#[cw_serde]
pub enum QueryResponse {
    ListAMMPairs {
        amm_pairs: Vec<AMMPair>,
    },
    GetConfig {
        pair_contract: ContractInstantiationInfo,
        amm_settings: AMMSettings,
        lp_token_contract: ContractInstantiationInfo,
        authenticator: Option<Contract>,
        admin_auth: Contract,
    },
    GetAMMPairAddress {
        address: String,
    },
    AuthorizeApiKey {
        authorized: bool,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ListAMMPairs { pagination: Pagination },
    GetAMMPairAddress { pair: TokenPair },
    GetConfig {},
    AuthorizeApiKey { api_key: String },
}

impl InstantiateCallback for InitMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
