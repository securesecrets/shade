use crate::contract_interfaces::dex::sienna::{self};
use cosmwasm_std::{HumanAddr, Binary};
use fadroma_platform_scrt::{ContractLink, ContractInstantiationInfo, Callback};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg{
    /// The tokens that will be managed by the exchange
    pub pair: sienna::Pair,
    /// LP token instantiation info
    pub lp_token_contract: ContractInstantiationInfo,
    /// Used by the exchange contract to
    /// send back its address to the factory on init
    pub factory_info: ContractLink<HumanAddr>,
    pub callback: Callback<HumanAddr>,
    pub prng_seed: Binary,
    pub entropy: Binary,
}