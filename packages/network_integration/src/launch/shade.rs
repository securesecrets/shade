use std::{env, fs};
use cosmwasm_std::{Binary, HumanAddr};
use serde::{Deserialize, Serialize};
use network_integration::utils::{GAS, generate_label, print_contract, print_header, SNIP20_FILE, STORE_GAS};
use secretcli::secretcli::{account_address, init};
use shade_protocol::contract_interfaces::snip20;
use shade_protocol::contract_interfaces::snip20::{InitConfig, InitialBalance};

#[derive(Serialize, Deserialize)]
struct Args {
    // Contract signing details
    tx_signer: String,
    label: Option<String>,

    // Snip20 config
    admin: Option<HumanAddr>,
    seed: String,
    balances: Vec<InitialBalance>
}

const NAME: &str = "Shade";
const SYMBOL: &str = "SHD";
const DECIMALS: u8 = 8;

fn main() -> serde_json::Result<()> {
    let bin_args: Vec<String> = env::args().collect();
    let args_file = fs::read_to_string(&bin_args.get(1).expect("No argument provided"))
        .expect("Unable to read args");
    let args: Args = serde_json::from_str(&args_file)?;

    // Initialize snip20
    print_header("Initializing Snip20");

    let snip_init_msg = snip20::InitMsg {
        name: NAME.to_string(),
        admin: args.admin,
        symbol: SYMBOL.to_string(),
        decimals: DECIMALS,
        initial_balances: Some(args.balances),
        prng_seed: Binary::from_base64(&args.seed).unwrap(),
        config: Some(InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(false),
            enable_redeem: Some(false),
            enable_mint: Some(true),
            enable_burn: Some(true),
            enable_transfer: Some(true),
        }),
    };

    let snip = init(
        &snip_init_msg,
        SNIP20_FILE,
        &args.label.unwrap_or(generate_label(8)),
        &args.tx_signer,
        Some(STORE_GAS),
        Some(GAS),
        None,
        &mut vec![]
    )?;

    print_contract(&snip);

    Ok(())
}
