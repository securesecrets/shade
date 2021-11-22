use colored::*;
use rand::{distributions::Alphanumeric, Rng};
use shade_protocol::{micro_mint};
use std::fmt::Display;
use serde::Serialize;
use secretcli::{cli_types::NetContract,
                secretcli::{query_contract}};

// Smart contracts
pub const SNIP20_FILE: &str = "../../compiled/snip20.wasm.gz";
pub const AIRDROP_FILE: &str = "../../compiled/airdrop.wasm.gz";
pub const GOVERNANCE_FILE: &str = "../../compiled/governance.wasm.gz";
pub const MOCK_BAND_FILE: &str = "../../compiled/mock_band.wasm.gz";
pub const ORACLE_FILE: &str = "../../compiled/oracle.wasm.gz";
pub const INITIALIZER_FILE: &str = "../../compiled/initializer.wasm.gz";
pub const MICRO_MINT_FILE: &str = "../../compiled/micro_mint.wasm.gz";
pub const STAKING_FILE: &str = "../../compiled/staking.wasm.gz";


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
    let msg = micro_mint::QueryMsg::GetMintLimit {};

    let query: micro_mint::QueryAnswer = query_contract(minter, &msg).unwrap();

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
