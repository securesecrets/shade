use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TxResponse {
    pub height: String,
    pub txhash: String,
    pub codespace: String,
    pub code: Option<u128>,
    pub data: String,
    pub raw_log: String,
}

#[derive(Serialize, Deserialize)]
pub struct TxCompute {
    //#[serde(rename="key")]
    //pub msg_key: String,
    pub input: String,
}

#[derive(Serialize, Deserialize)]
pub struct TxQuery {
    pub height: String,
    pub txhash: String,
    pub data: String,
    pub raw_log: String,
    pub logs: Vec<TxQueryLogs>,
    pub gas_wanted: String,
    pub gas_used: String,
    //pub tx: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct TxQueryLogs {
    pub msg_index: i128,
    pub log: String,
    pub events: Vec<TxQueryEvents>,
}

#[derive(Serialize, Deserialize)]
pub struct TxQueryEvents {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub attributes: Vec<TxQueryKeyValue>,
}

#[derive(Serialize, Deserialize)]
pub struct TxQueryKeyValue {
    #[serde(rename = "key")]
    pub msg_key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListCodeResponse {
    pub id: u128,
    pub creator: String,
    pub data_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListContractCode {
    pub code_id: u128,
    pub creator: String,
    pub label: String,
    pub address: String,
}

#[derive(Serialize, Deserialize)]
pub struct NetContract {
    pub label: String,
    pub id: String,
    pub address: String,
    pub code_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct StoredContract {
    pub id: String,
    pub code_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignedTx {
    pub pub_key: PubKey,
    pub signature: String,
}

#[derive(Serialize, Deserialize)]
pub struct PubKey {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub value: String,
}
