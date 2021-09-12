use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TxResponse {
    pub height: String,
    pub txhash: String,
    pub code: Option<String>,
    pub raw_log: String
}

#[derive(Serialize, Deserialize)]
pub struct TxCompute {
    //#[serde(rename="key")]
    //pub msg_key: String,
    pub raw_input: String,
    //pub output_data: String,
    pub output_data_as_string: String,
    //pub output_log: Vec<String>,
    pub plaintext_error: String,

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
    pub events: Vec<TxQueryEvents>
}

#[derive(Serialize, Deserialize)]
pub struct TxQueryEvents {
    #[serde(rename="type")]
    pub msg_type: String,
    pub attributes: Vec<TxQueryKeyValue>
}

#[derive(Serialize, Deserialize)]
pub struct TxQueryKeyValue {
    #[serde(rename="key")]
    pub msg_key: String,
    pub value: String
}

#[derive(Serialize, Deserialize)]
pub struct ListCodeResponse {
    pub id: u128,
    pub creator: String,
    pub data_hash: String,
    pub source: String,
    pub builder: String
}

#[derive(Serialize, Deserialize)]
pub struct ListContractCode {
    pub code_id: u128,
    pub creator: String,
    pub label: String,
    pub address: String
}

#[derive(Serialize, Deserialize)]
pub struct NetContract {
    pub label: String,
    pub id: String,
    pub address: String,
    pub code_hash: String,
}