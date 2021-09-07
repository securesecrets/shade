use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TxResponse {
    pub height: String,
    pub txhash: String,
    pub code: Option<String>,
    pub raw_log: String
}

// NOTE: compute returns type, but it conflicts with rust's type keyword
#[derive(Serialize, Deserialize)]
pub struct TxCompute {
    pub raw_input: String,
    pub output_data: String,
    pub output_data_as_string: String,
    pub output_log: Vec<String>,
    pub plaintext_error: String,

}

pub struct NetContract {
    pub label: String,
    pub id: String,
    pub address: String,
    pub code_hash: String,
}