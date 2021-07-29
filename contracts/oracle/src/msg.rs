use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub count: i128,
    pub current_price: i128,
    pub initial_oracle_ref: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Increment {},
    Reset { count: i128 },
    SetOracleRef { new_oracle_ref: String },
    SavePrice { symbol: String }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetCount {},
    //GetPrice returns the current price of an asset
    //GetPrice {},
    // query owner address
    Owner {},
    // query oracle address
    OracleRef {},
    // query price that has been saved
    GetPrice { symbol }
}

/*
* used only when a call is made across a contract, which arises internally and is not caused by a direct call
*/
pub enum QueryExtMsg {
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    }
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i128,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceResponse {
    pub currentPrice: i128,
}
