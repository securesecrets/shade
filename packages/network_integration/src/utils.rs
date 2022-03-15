use colored::*;
use rand::{distributions::Alphanumeric, Rng};
use secretcli::{cli_types::NetContract, secretcli::query_contract};
use serde::Serialize;
use shade_protocol::mint;
use std::fmt::Display;

// Smart contracts
pub const SNIP20_FILE: &str = "../../compiled/snip20.wasm.gz";
pub const AIRDROP_FILE: &str = "../../compiled/airdrop.wasm.gz";
pub const GOVERNANCE_FILE: &str = "../../compiled/governance.wasm.gz";
pub const MOCK_BAND_FILE: &str = "../../compiled/mock_band.wasm.gz";
pub const ORACLE_FILE: &str = "../../compiled/oracle.wasm.gz";
pub const INITIALIZER_FILE: &str = "../../compiled/initializer.wasm.gz";
pub const MICRO_MINT_FILE: &str = "../../compiled/mint.wasm.gz";
pub const STAKING_FILE: &str = "../../compiled/staking.wasm.gz";
pub const SHD_STAKING_FILE: &str = "../../compiled/shd_staking.wasm.gz";

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
    println!(
        "\tLabel: {}\n\tID: {}\n\tAddress: {}\n\tHash: {}",
        contract.label, contract.id, contract.address, contract.code_hash
    );
}

pub fn print_epoch_info(minter: &NetContract) {
    println!("\tEpoch information");
    let msg = mint::QueryMsg::Limit {};

    let query: mint::QueryAnswer = query_contract(minter, &msg).unwrap();

    if let mint::QueryAnswer::Limit { limit, minted, last_refresh } = query {
        println!(
            "\tLast Refresh: {}\n\tCapacity: {}\n\tTotal Minted: {}\n\t",
            last_refresh, limit, minted
        );
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
