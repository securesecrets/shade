use std::process::{Command};
use serde_json::{Value, Result};
use crate::cli_types::{TxResponse, NetContract, TxCompute, TxQuery, ListCodeResponse, ListContractCode};
use std::{thread, time};

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
    let retry = 20;
    let mut cli = Command::new("secretcli".to_string());
    if command.len() > 0 {
        cli.args(command.clone());
    }

    let mut result = cli.output().expect("Unexpected error");

    // We wait cause sometimes the query/action takes a while
    for _ in 0..retry {
        if result.stderr.len() > 0 {
            thread::sleep(time::Duration::from_secs(1));
        }
        else {
            break
        }
        result = cli.output().expect("Unexpected error");
    }
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
pub fn query_hash(hash: String) -> Result<TxQuery> {
    let command = vec!["q", "tx", &hash];
    let a = secretcli_run(vec_str_to_vec_string(command))?;

    serde_json::from_value(a)
}

///
/// Computes the hash information
///
pub fn compute_hash(hash: String) -> Result<TxCompute> {
    let command = vec!["q", "compute", "tx", &hash];

    serde_json::from_value(secretcli_run(
        vec_str_to_vec_string(command))?)
}

///
/// Lists all uploaded contracts
///
pub fn list_code() -> Result<Vec<ListCodeResponse>> {
    let command = vec!["query", "compute", "list-code"];

    serde_json::from_value(secretcli_run(vec_str_to_vec_string(command))?)
}

pub fn list_contracts_by_code(code: String) -> Result<Vec<ListContractCode>> {
    let command = vec!["query", "compute", "list-contract-by-code", &code];

    serde_json::from_value(secretcli_run(vec_str_to_vec_string(command))?)
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

pub fn account_address(acc: &str) -> Result<String> {
    let command = vec_str_to_vec_string(vec!["keys", "show", "-a", acc]);

    let retry = 20;
    let mut cli = Command::new("secretcli".to_string());
    if command.len() > 0 {
        cli.args(command.clone());
    }

    let mut result = cli.output().expect("Unexpected error");

    // We wait cause sometimes the query/action takes a while
    for _ in 0..retry {
        if result.stderr.len() > 0 {
            thread::sleep(time::Duration::from_secs(1));
        }
        else {
            break
        }
        result = cli.output().expect("Unexpected error");
    }

    let out = result.stdout;

    let mut s: String = String::from_utf8_lossy(&out).parse().unwrap();

    // Sometimes the resulting string has a newline, so we trim that
    trim_newline(&mut s);

    Ok(s)
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
(contract: &NetContract, msg: Init, label: &str, sender: &str,
 gas: Option<&str>, backend: Option<&str>) -> Result<TxResponse> {
    let message = serde_json::to_string(&msg)?;

    let mut command = vec!["tx", "compute", "instantiate", &contract.id,
             &message, "--from", sender, "--label", label, "--gas"];

    command.push(match gas {None => "10000000", Some(gas) => gas});

    if let Some(backend) = backend {
        command.push("--keyring-backend");
        command.push(backend);
    }

    command.push("-y");

    let response: TxResponse = serde_json::from_value(secretcli_run(vec_str_to_vec_string(command))?)?;

    Ok(response)
}

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
    fn t_init(&self, contract: &NetContract, label: &str, sender: &str,
              gas: Option<&str>, backend: Option<&str>) -> Result<TxQuery> {
        let tx = instantiate_contract(contract, self, label, sender,
                                      gas, backend)?;
        query_hash(tx.txhash)
    }

    fn inst_init(&self, contract_file: &str, label: &str, sender: &str, store_gas: Option<&str>,
                 init_gas: Option<&str>, backend: Option<&str>) -> Result<NetContract> {
        let store_response = store_contract(contract_file,
                                            Option::from(&*sender), store_gas, backend)?;

        let store_query = query_hash(store_response.txhash)?;

        let mut contract = NetContract {
            label: label.to_string(),
            id: "".to_string(),
            address: "".to_string(),
            code_hash: "".to_string()
        };

        // Look for the code ID
        for attribute in  &store_query.logs[0].events[0].attributes {
            if attribute.msg_key == "code_id" {
                contract.id = attribute.value.clone();
                break;
            }
        }

        let init_query = self.t_init(&contract, label,
                                     sender, init_gas, backend)?;

        // Look for the contract's address
        for attribute in &init_query.logs[0].events[0].attributes {
            if attribute.msg_key == "contract_address" {
                contract.address = attribute.value.clone();
                break;
            }
        }

        // Look for the contract's code hash
        let listed_contracts = list_code()?;

        for item in listed_contracts {
            if item.id.to_string() == contract.id {
                contract.code_hash = item.data_hash;
                break;
            }
        }

        Ok(contract)
    }
}

pub fn test_init<Message: serde::Serialize>
(msg: &Message, contract: &NetContract, label: &str, sender: &str, gas: Option<&str>,
 backend: Option<&str>) -> Result<TxQuery> {
    let tx = instantiate_contract(contract, msg, label, sender,
                                  gas, backend)?;
    query_hash(tx.txhash)
}

pub fn test_inst_init<Message: serde::Serialize>
(msg: &Message, contract_file: &str, label: &str, sender: &str, store_gas: Option<&str>,
               init_gas: Option<&str>, backend: Option<&str>) -> Result<NetContract> {
    let store_response = store_contract(contract_file,
                                        Option::from(&*sender), store_gas, backend)?;

    let store_query = query_hash(store_response.txhash)?;

    let mut contract = NetContract {
        label: label.to_string(),
        id: "".to_string(),
        address: "".to_string(),
        code_hash: "".to_string()
    };

    // Look for the code ID
    for attribute in  &store_query.logs[0].events[0].attributes {
        if attribute.msg_key == "code_id" {
            contract.id = attribute.value.clone();
            break;
        }
    }

    let init_query = test_init(&msg, &contract, label,
                                 sender, init_gas, backend)?;

    // Look for the contract's address
    for attribute in &init_query.logs[0].events[0].attributes {
        if attribute.msg_key == "contract_address" {
            contract.address = attribute.value.clone();
            break;
        }
    }

    // Look for the contract's code hash
    let listed_contracts = list_code()?;

    for item in listed_contracts {
        if item.id.to_string() == contract.id {
            contract.code_hash = item.data_hash;
            break;
        }
    }

    Ok(contract)
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
(contract: &NetContract, msg: Handle, sender: &str, gas: Option<&str>,
 backend: Option<&str>, amount: Option<&str>) -> Result<TxResponse> {
    let message = serde_json::to_string(&msg)?;

    let mut command =
        vec!["tx", "compute", "execute", &contract.address,
             &message, "--from", &sender, "--gas"];

    command.push(match gas {None => "800000", Some(gas) => gas});

    if let Some(backend) = backend {
        command.push("--keyring-backend");
        command.push(backend);
    }

    if let Some(amount) = amount {
        command.push("--amount");
        command.push(amount);
    }

    command.push("-y");

    let response: TxResponse = serde_json::from_value(secretcli_run(vec_str_to_vec_string(command))?)?;

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
pub trait TestHandle: serde::Serialize {
    fn t_handle(&self, contract: &NetContract, sender: &str, gas: Option<&str>,
                backend: Option<&str>, amount: Option<&str>) -> Result<TxCompute> {
        let tx = execute_contract(contract, self, sender, gas,
                                  backend, amount)?;

        let response: Result<TxCompute> = compute_hash(tx.txhash);
        response
    }
}


///
/// Function equivalent of the TestHandle trait
///
pub fn test_contract_handle<Message: serde::Serialize>(msg: &Message, contract: &NetContract, sender: &str, gas: Option<&str>,
                   backend: Option<&str>, amount: Option<&str>) -> Result<TxCompute> {
    let tx = execute_contract(contract, msg, sender, gas,
                              backend, amount)?;

    let response: Result<TxCompute> = compute_hash(tx.txhash);
    response
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
(contract: &NetContract, msg: Query) -> Result<Response> {
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
    fn t_query(&self, contract: &NetContract) -> Result<Response> {
        query_contract(contract, self)
    }
}

