use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TxResponse {
    pub height: String,
    pub txhash: String,
    pub code: Option<String>,
    pub raw_log: String
}

pub struct NetContract {
    pub label: String,
    pub id: String,
    pub address: String,
    pub code_hash: String,
}