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
pub fn query_hash(hash: String) -> Result<Value> {
    let command = vec!["q", "tx", &hash];

    secretcli_run(vec_str_to_vec_string(command))
}

///
/// Computes the hash information
///
pub fn compute_hash(hash: String) -> Result<Value> {
    let command = vec!["q", "compute", "tx", &hash];

    secretcli_run(vec_str_to_vec_string(command))
}

///
/// Lists all uploaded contracts
///
pub fn list_code() -> Result<Value> {
    let command = vec!["query", "compute", "list-code"];

    secretcli_run(vec_str_to_vec_string(command))
}

pub fn list_contracts_by_code(code: String) -> Result<Value> {
    let command = vec!["query", "compute", "list-contract-by-code", &code];

    secretcli_run(vec_str_to_vec_string(command))
}

///
/// Instantiate a contract
///
/// # Arguments
///
/// * 'code_id' - The contract to interact with
/// * 'msg' - The init msg to serialize
/// * 'label' - The contract label
/// * 'sender' - Msg sender
/// * 'gas' - Gas price to use, defaults to 8000000
/// * 'backend' - Keyring backend defaults to none
///
pub fn instantiate_contract<Init: serde::Serialize>
(contract: NetContract, msg: Init, label: String, sender: String,
 gas: Option<String>, backend: Option<String>) -> Result<TxResponse> {
    let mut command = vec_str_to_vec_string(
        vec!["secretcli", "tx", "compute", "instantiate", &contract.id,
             &serde_json::to_string(&msg)?, "--from", &sender, "--label", &label, "--gas"]);

    command.push(match gas {None => "10000000".to_string(), Some(gas) => gas});

    if let Some(backend) = backend {
        command.append(&mut vec!["--keyring-backend".to_string(), backend]);
    }

    command.push("-y".to_string());

    let response: TxResponse = serde_json::from_value(secretcli_run(command)?)?;

    Ok(response)
}
// TODO: Replace the given value with a struct for contract tx queries
///
/// Trait that allows contract init to be used in test scripts
///
/// # Arguments
///
/// * 'contract' - The contract to interact with
/// * 'label' - The contract label
/// * 'sender' - Msg sender
/// * 'gas' - Gas price to use, defaults to 8000000
/// * 'backend' - Keyring backend defaults to none
///
pub trait TestInit: serde::Serialize {
    fn t_init(&self, contract: NetContract, label: String, sender: String,
              gas: Option<String>, backend: Option<String>) -> Result<Value> {
        let tx = instantiate_contract(contract, self, label, sender,
                                      gas, backend)?;
        let response = query_hash(tx.txhash);
        response
    }
}

///
/// Executes a contract's handle
///
/// # Arguments
///
/// * 'contract' - The contract to interact with
/// * 'msg' - The handle msg to serialize
/// * 'sender' - Msg sender
/// * 'gas' - Gas price to use, defaults to 8000000
/// * 'backend' - Keyring backend defaults to none
/// * 'amount' - Included L1 tokens to send, defaults to none
///
pub fn execute_contract<Handle: serde::Serialize>
(contract: NetContract, msg: Handle, sender: String, gas: Option<String>,
 backend: Option<String>, amount: Option<String>) -> Result<TxResponse> {
    let mut command = vec_str_to_vec_string(
        vec!["secretcli", "tx", "compute", "execute", &contract.address,
             &serde_json::to_string(&msg)?, "--from", &sender, "--gas"]);

    command.push(match gas {None => "800000".to_string(), Some(gas) => gas});

    if let Some(backend) = backend {
        command.append(&mut vec!["--keyring-backend".to_string(), backend]);
    }

    if let Some(amount) = amount {
        command.append(&mut vec!["--amount".to_string(), amount]);
    }

    command.push("-y".to_string());

    let response: TxResponse = serde_json::from_value(secretcli_run(command)?)?;

    Ok(response)
}

///
/// Trait that allows contract handle enums to be used in test scripts
///
/// # Arguments
///
/// * 'contract' - The contract to interact with
/// * 'sender' - Msg sender
/// * 'gas' - Gas price to use, defaults to 8000000
/// * 'backend' - Keyring backend defaults to none
/// * 'amount' - Included L1 tokens to send, defaults to none
///
pub trait TestHandle<Response: serde::de::DeserializeOwned>: serde::Serialize {
    fn t_handle(&self, contract: NetContract, sender: String, gas: Option<String>,
                backend: Option<String>, amount: Option<String>) -> Result<Response> {
        let tx = execute_contract(contract, self, sender, gas,
                                  backend, amount)?;

        let response: Result<Response> = serde_json::from_value(compute_hash(tx.txhash)?);
        response
    }
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
    let command = vec_str_to_vec_string(vec!["query", "compute", "query",
                                             &contract.address, &serde_json::to_string(&msg)?]);

    let response: Result<Response> = serde_json::from_value(secretcli_run(command)?);
    response
}

///
/// Trait that allows contract query enums to be used in test scripts
///
/// # Arguments
///
/// * 'contract' - The contract to query
///
pub trait TestQuery<Response: serde::de::DeserializeOwned>: serde::Serialize {
    fn t_query(&self, contract: NetContract) -> Result<Response> {
        query_contract(contract, self)
    }
}