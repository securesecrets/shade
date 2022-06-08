use cosmwasm_std::HumanAddr;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cosmwasm_math_compat::Uint128;
use crate::utils::asset::Contract;


pub enum HandleMsg {
    // Domain creation
    RegisterDomain{ admin: Option<HumanAddr>, domain: String },
    CreateSubDomain { domain_id: Uint128, path: String, contract: Contract },
    RemoveSubDomain { domain_id: Uint128, path: String, contract: Contract },

    // If we want to use a domain well nned to change how handleMsgs work,
    // youll need a trusted sender (contract) and then have a sort of custom setup where the mst input is ReceiveFromContract{msg: Binary, env: Env}
}