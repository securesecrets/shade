use std::process::{Command};
use serde_json::{Value, Result};
use crate::cli_types::{TxResponse, NetContract};

fn vec_str_to_vec_string (str_in: Vec<&str>) -> Vec<String> {
    let mut str_out: Vec<String> = vec![];

    for val in str_in {
        str_out.push(val.to_string());
    }

    str_out
}

///
/// Will run any scretcli command and return its output
///
/// # Arguments
///
/// * 'command' - a string array that contains the command to forward\
///
pub fn secretcli_run(command: Vec<String>) -> Result<Value> {
    let mut cli = Command::new("secretcli".to_string());
    if command.len() > 0 {
        cli.args(command);
    }
    let result = cli.output().expect("Unexpected error");
    let out = result.stdout;
    //println!("{}", String::from_utf8_lossy(&result.stderr));
    let json= serde_json::from_str(&String::from_utf8_lossy(&out));
    json
}

///
/// Stores the given contract
///
/// # Arguments
///
/// * 'contract' - Contract to be stored
/// * 'user' - User that will handle the tx, defaults to a
/// * 'gas' - Gas to pay, defaults to 10000000
/// * 'backend' - The backend keyring, defaults to test
///
pub fn store_contract(contract: &str, user: Option<&str>,
                      gas: Option<&str>, backend: Option<&str>) -> Result<TxResponse> {

    let mut command_arr = vec!["tx", "compute", "store", contract,
                   "--from", match user {None => "a", Some(usr) => usr },
                   "--gas", match gas {None => "10000000", Some(gas) => gas}, "-y",
    ];

    match backend {
        None => {},
        Some(backend) => {
            command_arr.push("--keyring-backend");
            command_arr.push(backend);
        }
    }

    let command = vec_str_to_vec_string(command_arr);
    let json = secretcli_run(command)?;
    let out: Result<TxResponse> = serde_json::from_value(json);
    out
}

///
/// Queries the hash information
///
pub fn query_hash(hash: &str) -> Result<Value> {
    let command = vec!["q", "tx", hash];

    secretcli_run(vec_str_to_vec_string(command))
}

///
/// Computes the hash information
///
pub fn compute_hash(hash: &str) -> Result<Value> {
    let command = vec!["q", "compute", "tx", hash];

    secretcli_run(vec_str_to_vec_string(command))
}

///
/// Queries a given contract
///
/// # Arguments
///
/// * 'contract' - The contract to query
/// * 'msg' - The query to serialize, must have serde::Serialized
///
pub fn query_contract<Query: serde::Serialize, Response: serde::de::DeserializeOwned>
(contract: NetContract, msg: Query) -> Result<Response> {
    let mut command = vec_str_to_vec_string(vec!["query", "compute", "query"]);
    command.append(&mut vec![contract.address, serde_json::to_string(&msg)?]);

    let response: Response = serde_json::from_value(secretcli_run(command)?)?;
    Ok(response)
}

pub trait TestQuery<Response: serde::de::DeserializeOwned>: serde::Serialize {
    fn t_query(&self, contract: NetContract) -> Result<Response> {
        query_contract(contract, self)
    }
}