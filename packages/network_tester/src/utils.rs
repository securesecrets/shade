use serde_json::{Result, Error};
use colored::*;
use rand::{distributions::Alphanumeric, Rng};
use secretcli::{cli_types::NetContract,
                secretcli::{TestInit, TestHandle, TestQuery}};
use shade_protocol::{micro_mint, governance, asset::Contract};
use cosmwasm_std::{HumanAddr, Uint128};
use std::fmt::Display;
use serde::Serialize;

pub const STORE_GAS: &str = "10000000";
pub const GAS: &str = "800000";
pub const VIEW_KEY: &str = "password";
pub const ACCOUNT_KEY: &str = "a";

pub fn generate_label(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

pub fn print_header(header: &str) {
    println!("{}", header.on_blue());
}

pub fn print_warning(warn: &str) {
    println!("{}", warn.on_yellow());
}

pub fn print_contract(contract: &NetContract) {
    println!("\tLabel: {}\n\tID: {}\n\tAddress: {}\n\tHash: {}", contract.label, contract.id,
             contract.address, contract.code_hash);
}

pub fn print_epoch_info(minter: &NetContract) {
    println!("\tEpoch information");
    let query = micro_mint::QueryMsg::GetMintLimit {}.t_query(minter).unwrap();

    if let micro_mint::QueryAnswer::MintLimit { limit } = query {
        println!("\tFrequency: {}\n\tCapacity: {}\n\tTotal Minted: {}\n\tNext Epoch: {}",
                 limit.frequency, limit.mint_capacity, limit.total_minted, limit.next_epoch);
    }
}

pub fn print_struct<Printable: Serialize>(item: Printable) {
    println!("{}", serde_json::to_string_pretty(&item).unwrap());
}

pub fn print_vec<Type: Display>(prefix: &str, vec: Vec<Type>) {
    for e in vec.iter().take(1) {
        print!("{}{}", prefix, e);
    }
    for e in vec.iter().skip(1) {
        print!(", {}", e);
    }
    println!();
}

pub fn gov_init_contract<Init: TestInit>(
    governance: &NetContract, contract_name: String,
    contract_path: &str, contract_init: Init) -> Result<NetContract> {
    print_header(&format!("{}{}", "Initializing ", contract_name.clone()));

    let contract = contract_init.inst_init(contract_path,
                                          &*generate_label(8), ACCOUNT_KEY,
                                          Some(STORE_GAS), Some(GAS),
                                          Some("test"))?;

    print_contract(&contract);

    gov_add_contract(contract_name, &contract, &governance);

    Ok(contract)
}

pub fn gov_get_contract(governance: &NetContract, target: String) -> Result<Contract> {
    let query: governance::QueryAnswer = governance::QueryMsg::GetSupportedContract {
        name: target }.t_query(&governance)?;

    let mut ctrc = Contract {
        address: HumanAddr::from("not_found".to_string()),
        code_hash: "not_found".to_string()
    };

    if let governance::QueryAnswer::SupportedContract { contract } = query {
        ctrc = contract;
    }

    Ok(ctrc)
}

pub fn gov_add_contract(name: String, target: &NetContract, governance: &NetContract) -> Result<()>{
    print_warning(&format!("{}{}", "Adding ", name.clone()));

    governance::HandleMsg::RequestAddSupportedContract {
        name,
        contract: Contract{
            address: HumanAddr::from(target.address.clone()),
            code_hash: target.code_hash.clone()
        },
        description: "Add a contract".to_string()
    }.t_handle(
        &governance, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    gov_trigger_latest_proposal(governance)?;

    Ok(())
}

pub fn gov_custom_proposal<Handle: serde::Serialize>(
    governance: &NetContract, target: String, handle: Handle) -> Result<()> {
    let msg = serde_json::to_string(&handle)?;

    governance::HandleMsg::CreateProposal {
        target_contract: target,
        proposal: msg,
        description: "Custom proposal".to_string()
    }.t_handle(&governance, ACCOUNT_KEY, Some(GAS),
               Some("test"), None)?;

    gov_trigger_latest_proposal(governance)?;

    Ok(())
}

pub fn gov_trigger_latest_proposal(governance: &NetContract) -> Result<Uint128> {
    let query: governance::QueryAnswer = governance::QueryMsg::GetTotalProposals {
    }.t_query(&governance)?;

    let mut proposals = Uint128(1);

    if let governance::QueryAnswer::TotalProposals { total } = query {
        governance::HandleMsg::TriggerProposal { proposal_id: total
        }.t_handle(&governance, ACCOUNT_KEY, Some(GAS),
                   Some("test"), None)?;

        proposals = total;
    }

    Ok(proposals)
}

