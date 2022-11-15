mod utils;

use crate::utils::{generate_label, print_contract, print_header, GAS, SNIP20_FILE, STORE_GAS};
use secretcli::secretcli::{account_address, handle, init};
use serde::{Deserialize, Serialize};
use shade_protocol::{
    c_std::{Addr, Binary},
    contract_interfaces::{
        snip20,
        snip20::{InitConfig, InitialBalance},
    },
};
use std::{env, fs};

#[derive(Serialize, Deserialize)]
struct Args {
    // Contract signing details
    tx_signer: String,
    label: Option<String>,

    // Snip20 config
    admin: Option<String>,
    seed: Option<String>,
    balances: Vec<InitialBalance>,
    minters: Option<Vec<String>>,
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

    let snip_init_msg = snip20::InstantiateMsg {
        name: NAME.to_string(),
        admin: args.admin,
        symbol: SYMBOL.to_string(),
        decimals: DECIMALS,
        initial_balances: Some(args.balances),
        prng_seed: match args.seed {
            None => Binary::from("random".as_bytes()),
            Some(seed) => Binary::from_base64(&seed).unwrap(),
        },
        config: Some(InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(false),
            enable_redeem: Some(false),
            enable_mint: Some(true),
            enable_burn: Some(true),
        }),
        query_auth: None,
    };

    let snip = init(
        &snip_init_msg,
        SNIP20_FILE,
        &args.label.unwrap_or(generate_label(8)),
        &args.tx_signer,
        Some(STORE_GAS),
        Some(GAS),
        None,
        &mut vec![],
    )?;

    print_contract(&snip);

    if let Some(minters) = args.minters {
        let msg = snip20::ExecuteMsg::SetMinters {
            minters,
            padding: None,
        };

        handle(
            &msg,
            &snip,
            &args.tx_signer,
            Some(GAS),
            None,
            None,
            &mut vec![],
            None,
        )?;
    }

    Ok(())
}
