use crate::cli_types::{
    ListCodeResponse,
    ListContractCode,
    NetContract,
    SignedTx,
    StoredContract,
    TxCompute,
    TxQuery,
    TxResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::{
    fs::File,
    io::{self, Write},
    process::Command,
    thread,
    time,
};

//secretcli tx sign-doc tx_to_sign --from sign-test

fn vec_str_to_vec_string(str_in: Vec<&str>) -> Vec<String> {
    let mut str_out: Vec<String> = vec![];

    for val in str_in {
        str_out.push(val.to_string());
    }

    str_out
}

///
/// Contains that specific transaction's information
///
#[derive(Serialize, Deserialize)]
pub struct Report {
    pub msg_type: String,
    pub message: String,
    pub gas_used: String,
}

///
/// Will run any scretcli command and return its output
///
/// # Arguments
///
/// * 'command' - a string array that contains the command to forward\
///
fn secretcli_run(command: Vec<String>, max_retry: Option<i32>) -> Result<Value> {
    let retry = max_retry.unwrap_or(30);
    let mut commands = command;
    commands.append(&mut vec_str_to_vec_string(vec!["--output", "json"]));
    let mut cli = Command::new("secretd".to_string());
    if !commands.is_empty() {
        cli.args(commands);
    }

    let mut result = cli.output().expect("Unexpected error");
    // We wait cause sometimes the query/action takes a while
    for _ in 0..retry {
        if !result.stderr.is_empty() {
            thread::sleep(time::Duration::from_secs(1));
        } else {
            break;
        }
        result = cli.output().expect("Unexpected error");
    }
    let out = result.stdout;
    if String::from_utf8_lossy(&out).contains("output_error") {
        println!("{:?}", &String::from_utf8_lossy(&out));
    }
    serde_json::from_str(&String::from_utf8_lossy(&out))
}

///
/// Stores the given `contract
///
/// # Arguments
///
/// * 'contract' - Contract to be stored
/// * 'user' - User that will handle the tx, defaults to a
/// * 'gas' - Gas to pay, defaults to 10000000
/// * 'backend' - The backend keyring, defaults to test
///
fn store_contract(
    contract: &str,
    user: Option<&str>,
    gas: Option<&str>,
    backend: Option<&str>,
) -> Result<TxResponse> {
    let mut command_arr = vec![
        "tx",
        "compute",
        "store",
        contract,
        "--from",
        user.unwrap_or("a"),
        "--gas",
        gas.unwrap_or("10000000"),
        "-y",
    ];

    if let Some(backend) = backend {
        command_arr.push("--keyring-backend");
        command_arr.push(backend);
    }

    let command = vec_str_to_vec_string(command_arr);
    let json = secretcli_run(command, None)?;
    let out: Result<TxResponse> = serde_json::from_value(json);
    out
}

///
/// Queries the hash information
///
fn query_hash(hash: String) -> Result<TxQuery> {
    let command = vec!["q", "tx", &hash];
    let a = secretcli_run(vec_str_to_vec_string(command), None)?;
    serde_json::from_value(a)
}

///
/// Computes the hash information
///
fn compute_hash(hash: String) -> Result<TxCompute> {
    let command = vec!["q", "compute", "tx", &hash];

    serde_json::from_value(secretcli_run(vec_str_to_vec_string(command), None)?)
}

///
/// Lists all uploaded contracts
///
fn list_code() -> Result<Vec<ListCodeResponse>> {
    let command = vec!["query", "compute", "list-code"];

    serde_json::from_value(secretcli_run(vec_str_to_vec_string(command), None)?)
}

pub fn list_contracts_by_code(code: String) -> Result<Vec<ListContractCode>> {
    let command = vec!["query", "compute", "list-contract-by-code", &code];

    serde_json::from_value(secretcli_run(vec_str_to_vec_string(command), None)?)
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

///
/// Displays an account from the keyring
///
/// # Arguments
///
/// * 'acc' - The requested account
///
pub fn account_address(acc: &str) -> Result<String> {
    let command = vec_str_to_vec_string(vec!["keys", "show", "-a", acc]);

    let retry = 40;
    let mut cli = Command::new("secretd".to_string());
    if !command.is_empty() {
        cli.args(command);
    }

    let mut result = cli.output().expect("Unexpected error");

    // We wait cause sometimes the query/action takes a while
    for _ in 0..retry {
        if !result.stderr.is_empty() {
            thread::sleep(time::Duration::from_secs(1));
        } else {
            break;
        }
        result = cli.output().expect("Unexpected error");
    }

    let out = result.stdout;

    let mut s: String = String::from_utf8_lossy(&out).parse().unwrap();

    // Sometimes the resulting string has a newline, so we trim that
    trim_newline(&mut s);

    Ok(s)
}

pub fn create_key_account(name: &str) -> Result<()> {
    let command = vec_str_to_vec_string(vec!["keys", "add", name]);

    let retry = 40;
    let mut cli = Command::new("secretd".to_string());
    if !command.is_empty() {
        cli.args(command);
    }

    let mut result = cli.output().expect("Unexpected error");

    // We wait cause sometimes the query/action takes a while
    for _ in 0..retry {
        if !result.stderr.is_empty() {
            thread::sleep(time::Duration::from_secs(1));
        } else {
            break;
        }
        result = cli.output().expect("Unexpected error");
    }

    Ok(())
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
fn instantiate_contract<Init: serde::Serialize>(
    contract: &NetContract,
    msg: Init,
    label: &str,
    sender: &str,
    gas: Option<&str>,
    backend: Option<&str>,
) -> Result<TxResponse> {
    let message = serde_json::to_string(&msg)?;

    let mut command = vec![
        "tx",
        "compute",
        "instantiate",
        &contract.id,
        &message,
        "--from",
        sender,
        "--label",
        label,
        "--gas",
    ];

    command.push(match gas {
        None => "10000000",
        Some(gas) => gas,
    });

    if let Some(backend) = backend {
        command.push("--keyring-backend");
        command.push(backend);
    }

    command.push("-y");

    let response: TxResponse =
        serde_json::from_value(secretcli_run(vec_str_to_vec_string(command), None)?)?;

    Ok(response)
}

///
/// Store the given contract and return the stored contract information
///
/// * 'contract_file' - Contract file to store
/// * 'sender' - Msg sender
/// * 'store_gas' - Gas price to use when storing the contract, defaults to 10000000
/// * 'backend' - Keyring backend defaults to none
///
pub fn store_and_return_contract(
    contract_file: &str,
    sender: &str,
    store_gas: Option<&str>,
    backend: Option<&str>,
) -> Result<StoredContract> {
    let store_response = store_contract(contract_file, Option::from(&*sender), store_gas, backend)?;
    let store_query = query_hash(store_response.txhash)?;
    let mut contract = StoredContract {
        id: "".to_string(),
        code_hash: "".to_string(),
    };

    for attribute in &store_query.logs[0].events[0].attributes {
        if attribute.msg_key == "code_id" {
            contract.id = attribute.value.clone();
            break;
        }
    }

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
/// Allows contract init to be used in test scripts
///
/// # Arguments
///
/// * `msg` - Contract's init message
/// * 'contract_file' - The contract to interact with
/// * 'label' - The contract label
/// * 'sender' - Msg sender - must be registered in keyring
/// * 'store_gas' - Gas price to use when storing the contract, defaults to 10000000
/// * 'init_gas' - Gas price to use when initializing the contract, defaults to 8000000
/// * 'backend' - Keyring backend defaults to none
/// * `report` - Records the contract`s message and instantiation price
///
pub fn init<Message: serde::Serialize>(
    msg: &Message,
    contract_file: &str,
    label: &str,
    sender: &str,
    store_gas: Option<&str>,
    init_gas: Option<&str>,
    backend: Option<&str>,
    report: &mut Vec<Report>,
) -> Result<NetContract> {
    io::stdout().flush().unwrap();
    let store_response = store_contract(contract_file, Option::from(&*sender), store_gas, backend)?;
    let store_query = query_hash(store_response.txhash)?;
    let mut contract = NetContract {
        label: label.to_string(),
        id: "".to_string(),
        address: "".to_string(),
        code_hash: "".to_string(),
    };

    // Look for the code ID
    for attribute in &store_query.logs[0].events[0].attributes {
        if attribute.msg_key == "code_id" {
            contract.id = attribute.value.clone();
            break;
        }
    }

    // Instantiate and get the info
    let tx = instantiate_contract(&contract, msg, label, sender, init_gas, backend)?;
    let init_query = query_hash(tx.txhash)?;

    // Include the instantiation info in the report
    report.push(Report {
        msg_type: "Instantiate".to_string(),
        message: serde_json::to_string(&msg)?,
        gas_used: init_query.gas_used,
    });

    // Look for the contract's address
    for attribute in &init_query.logs[0].events[0].attributes {
        if attribute.msg_key == "contract_address" {
            contract.address = attribute.value.clone();
            break;
        }
    }
    // Look for the contract's code hash
    let listed_contracts = list_code()?;

    // Find the code_hash
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
fn execute_contract<Handle: serde::Serialize>(
    contract: &NetContract,
    msg: Handle,
    sender: &str,
    gas: Option<&str>,
    backend: Option<&str>,
    amount: Option<&str>,
    max_tries: Option<i32>,
) -> Result<TxResponse> {
    let message = serde_json::to_string(&msg)?;

    let mut command = vec![
        "tx",
        "compute",
        "execute",
        &contract.address,
        &message,
        "--from",
        sender,
        "--gas",
    ];

    command.push(match gas {
        None => "800000",
        Some(gas) => gas,
    });

    if let Some(backend) = backend {
        command.push("--keyring-backend");
        command.push(backend);
    }

    if let Some(amount) = amount {
        command.push("--amount");
        command.push(amount);
    }

    command.push("-y");

    let response: TxResponse =
        serde_json::from_value(secretcli_run(vec_str_to_vec_string(command), max_tries)?)?;

    Ok(response)
}

///
/// Allows contract handle enums to be used in test scripts
///
/// # Arguments
///
/// * `msg` - ExecuteMsg
/// * 'contract' - The contract to interact with
/// * 'sender' - Msg sender
/// * 'gas' - Gas price to use, defaults to 8000000
/// * 'backend' - Keyring backend defaults to none
/// * 'amount' - Included L1 tokens to send, defaults to none
/// * `report` - Records the contract`s message and handle price
///
pub fn handle<Message: serde::Serialize>(
    msg: &Message,
    contract: &NetContract,
    sender: &str,
    gas: Option<&str>,
    backend: Option<&str>,
    amount: Option<&str>,
    report: &mut Vec<Report>,
    max_tries: Option<i32>,
) -> Result<(TxCompute, TxQuery)> {
    let tx = execute_contract(contract, msg, sender, gas, backend, amount, max_tries)?;

    let computed_response = compute_hash(tx.txhash.clone())?;
    let queried_response = query_hash(tx.txhash)?;

    // Include the instantiation info in the report
    report.push(Report {
        msg_type: "Handle".to_string(),
        message: serde_json::to_string(&msg)?,
        gas_used: queried_response.gas_used.clone(),
    });

    Ok((computed_response, queried_response))
}

///
/// Queries a given contract
///
/// # Arguments
///
/// * 'contract' - The contract to query
/// * 'msg' - The query to serialize, must have serde::Serialized
///
pub fn query<Query: serde::Serialize, Response: serde::de::DeserializeOwned>(
    contract: &NetContract,
    msg: Query,
    max_tries: Option<i32>,
) -> Result<Response> {
    let command = vec_str_to_vec_string(vec![
        "query",
        "compute",
        "query",
        &contract.address,
        &serde_json::to_string(&msg)?,
    ]);

    let response: Result<Response> = serde_json::from_value(secretcli_run(command, max_tries)?);
    response
}

///
/// Create a signed permit
///
/// # Arguments
///
/// * 'tx' - The message to sign
/// * 'signer' - The key of the signer
///
pub fn create_permit<Tx: serde::Serialize>(tx: Tx, signer: &str) -> Result<SignedTx> {
    let msg = serde_json::to_string(&tx)?;

    // send to a file
    let mut file = File::create("./tx_to_sign").unwrap();
    file.write_all(msg.as_bytes()).unwrap();

    let command = vec!["tx", "sign-doc", "tx_to_sign", "--from", signer];

    let response: SignedTx =
        serde_json::from_value(secretcli_run(vec_str_to_vec_string(command), None)?)?;

    Ok(response)
}
