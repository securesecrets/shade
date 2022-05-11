use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{Binary, HumanAddr};
use network_integration::utils::{
    generate_label, print_contract, print_header, store_struct, AIRDROP_FILE, GAS, STORE_GAS,
};
use rs_merkle::algorithms::Sha256;
use rs_merkle::{Hasher, MerkleTree};
use secretcli::cli_types::NetContract;
use secretcli::secretcli::{handle, init};
use serde::{Deserialize, Serialize};
use shade_protocol::utils::asset::Contract;
use shade_protocol::{
    contract_interfaces::airdrop,
    contract_interfaces::snip20
};
use std::{env, fs};

#[derive(Serialize, Deserialize)]
struct Args {
    // Contract signing details
    tx_signer: String,
    label: Option<String>,

    // Merkle Tree stuff
    db_path: String,

    // Airdrop config
    admin: Option<HumanAddr>,
    dump_address: Option<String>,
    start_date: u64,
    end_date: Option<u64>,
    decay_start: Option<u64>,

    // Other stuff
    fund_airdrop: bool,
    shade: Contract,
}

#[derive(Serialize, Deserialize)]
struct Reward {
    pub address: String,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize)]
struct Tree {
    pub address: String,
    pub amount: Uint128,
}

const QUERY_ROUNDING: Uint128 = Uint128::new(1_000_000_000_000_u128);
const DEFAULT_CLAIM: Uint128 = Uint128::new(20u128);

fn main() -> serde_json::Result<()> {
    let bin_args: Vec<String> = env::args().collect();
    let args_file = fs::read_to_string(&bin_args.get(1).expect("No argument provided"))
        .expect("Unable to read args");
    let args: Args = serde_json::from_str(&args_file)?;

    print_header("Importing DB");
    let file_data = fs::read_to_string(args.db_path).expect("Unable to read db");
    let rewards: Vec<Reward> = serde_json::from_str(&file_data)?;

    print_header("Converting into merkle tree");
    let mut max_amount = Uint128::zero();
    let mut airdrop_amount = Uint128::zero();
    let leaves: Vec<[u8; 32]> = rewards
        .iter()
        .map(|x| {
            airdrop_amount += x.amount;
            if x.amount > max_amount {
                max_amount = x.amount
            }
            Sha256::hash((x.address.clone() + &x.amount.to_string()).as_bytes())
        })
        .collect();

    let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);
    let root = merkle_tree.root().unwrap();

    // Store the tree
    print_header("Storing tree");
    let mut stored_tree: Vec<Vec<Binary>> = vec![];
    for layer in merkle_tree.layers().iter() {
        let mut new_layer: Vec<Binary> = vec![];
        for node in layer.iter() {
            new_layer.push(Binary(node.to_vec()));
        }
        stored_tree.push(new_layer);
    }

    println!(
        "Merkle tree height: {}, amount: {}, max: {}",
        merkle_tree.layers().len(),
        airdrop_amount,
        max_amount
    );
    store_struct("merkle_tree.json", &stored_tree);

    // Initialize airdrop
    print_header("Initializing airdrop");

    let airdrop_init_msg = airdrop::InitMsg {
        admin: args.admin,
        dump_address: match args.dump_address {
            Some(addr) => Some(HumanAddr(addr)),
            None => None,
        },
        airdrop_token: args.shade.clone(),
        airdrop_amount,
        start_date: Some(args.start_date),
        end_date: args.end_date,
        decay_start: args.decay_start,
        merkle_root: Binary(root.to_vec()),
        total_accounts: leaves.len() as u32,
        max_amount,
        default_claim: DEFAULT_CLAIM,
        task_claim: vec![],
        query_rounding: QUERY_ROUNDING,
    };

    let airdrop = init(
        &airdrop_init_msg,
        AIRDROP_FILE,
        &args.label.unwrap_or(generate_label(8)),
        &args.tx_signer,
        Some(STORE_GAS),
        Some(GAS),
        None,
        &mut vec![],
    )?;

    print_contract(&airdrop);

    if args.fund_airdrop {
        print_header("Funding airdrop");
        let snip = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: args.shade.address.to_string(),
            code_hash: args.shade.code_hash.to_string(),
        };
        handle(
            &snip20::HandleMsg::Send {
                recipient: HumanAddr(airdrop.address),
                amount: airdrop_amount,
                msg: None,
                memo: None,
                padding: None,
            },
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
