use crate::contract_interfaces::dex::sienna::{self};
use cosmwasm_std::{Binary, HumanAddr};
use fadroma_platform_scrt::{ContractLink, ContractInstantiationInfo, Callback};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg{
    pub lp_token_contract: ContractInstantiationInfo,
    pub pair_contract: ContractInstantiationInfo,
    pub exchange_settings: ExchangeSettings<HumanAddr>,
    pub admin: Option<HumanAddr>,
    pub prng_seed: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg{
    CreateExchange{
        pair: sienna::Pair,
        entropy: Binary,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug, Clone)]
pub struct ExchangeSettings<A> {
    pub swap_fee: Fee,
    pub sienna_fee: Fee,
    pub sienna_burner: Option<A>,
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Clone, Copy, Debug)]
pub struct Fee {
    pub nom: u8,
    pub denom: u16,
}