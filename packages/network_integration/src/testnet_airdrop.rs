use cosmwasm_std::{Binary, HumanAddr, Uint128};
use network_integration::utils::{
    generate_label, print_contract, print_header, store_struct, AIRDROP_FILE, GAS, SNIP20_FILE,
    STORE_GAS,
};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use secretcli::cli_types::NetContract;
use secretcli::secretcli::{account_address, handle, init};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use shade_protocol::utils::asset::Contract;
use shade_protocol::{
    airdrop,
    airdrop::claim_info::RequiredTask,
    snip20,
    snip20::{InitConfig, InitialBalance},
};
use std::{env, fs};

#[derive(Serialize, Deserialize)]
pub struct Reward {
    pub address: String,
    pub amount: String,
}

#[derive(Serialize, Deserialize)]
pub struct Args {
    db_path: String,
    total_amount: Uint128,
    max_amount: Uint128,
    admin: String,
    start_date: u64,
    end_date: u64,
    decay_start: u64,
    fund_airdrop: bool,
    shade: Option<Contract>,
}

fn main() -> Result<()> {
    let bin_args: Vec<String> = env::args().collect();
    //println!("ARGUMENT: {}", bin_args.get(2).expect("No argument provided"));
    let args_file = fs::read_to_string(&bin_args.get(2).expect("No argument provided"))
        .expect("Unable to read args");
    let args: Args = serde_json::from_str(&args_file)?;

    let account_addr = account_address(&args.admin)?;

    print_header("Importing DB");
    let file_data = fs::read_to_string(args.db_path).expect("Unable to read db");
    let rewards: Vec<Reward> = serde_json::from_str(&file_data)?;

    print_header("Converting into merkle tree");
    let raw_leaves: Vec<String> = rewards
        .iter()
        .map(|x| x.address.clone() + &x.amount)
        .collect();
    let leaves: Vec<[u8; 32]> = raw_leaves
        .iter()
        .map(|x| Sha256::hash(x.as_bytes()))
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
    println!("Merkle tree height: {}", merkle_tree.layers().len());
    store_struct("merkle_tree.json", &stored_tree);
    fs::write("merkle_tree.json", serde_json::to_string(&stored_tree)?)
        .expect("Could not store merkle tree");

    let mut reports = vec![];

    let snip: NetContract;

    if args.shade.is_none() {
        // Initialize snip20
        print_header("Initializing Snip20");

        let snip_init_msg = snip20::InitMsg {
            name: "Shade".to_string(),
            admin: None,
            symbol: "SHD".to_string(),
            decimals: 8,
            initial_balances: Some(vec![InitialBalance {
                address: HumanAddr::from(account_addr.clone()),
                amount: args.total_amount,
            }]),
            prng_seed: Default::default(),
            config: Some(InitConfig {
                public_total_supply: Some(true),
                enable_deposit: Some(false),
                enable_redeem: Some(false),
                enable_mint: Some(true),
                enable_burn: Some(true),
            }),
        };

        snip = init(
            &snip_init_msg,
            SNIP20_FILE,
            &*generate_label(8),
            &args.admin,
            Some(STORE_GAS),
            Some(GAS),
            None,
            &mut reports,
        )?;
    } else {
        print_header("Using Shade");
        snip = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: args.shade.clone().unwrap().address.to_string(),
            code_hash: args.shade.clone().unwrap().code_hash,
        }
    }

    print_contract(&snip);

    // Initialize airdrop
    print_header("Initializing airdrop");

    let airdrop_init_msg = airdrop::InitMsg {
        admin: None,
        dump_address: Some(HumanAddr::from(account_addr.clone())),
        airdrop_token: Contract {
            address: HumanAddr::from(snip.address.clone()),
            code_hash: snip.code_hash.clone(),
        },
        airdrop_amount: args.total_amount,
        start_date: Some(args.start_date),
        end_date: Some(args.end_date),
        decay_start: Some(args.decay_start),
        merkle_root: Binary(root.to_vec()),
        total_accounts: leaves.len() as u32,
        max_amount: args.max_amount,
        default_claim: Uint128(20),
        task_claim: vec![RequiredTask {
            address: HumanAddr::from(account_addr),
            percent: Uint128(50),
        }],
        query_rounding: Uint128(10000000000),
    };

    let airdrop = init(
        &airdrop_init_msg,
        AIRDROP_FILE,
        &*generate_label(8),
        &args.admin,
        Some(STORE_GAS),
        Some(GAS),
        None,
        &mut reports,
    )?;

    print_contract(&airdrop);

    if args.fund_airdrop {
        print_header("Funding airdrop");
        handle(
            &snip20::HandleMsg::Send {
                recipient: HumanAddr::from(airdrop.address),
                amount: args.total_amount,
                msg: None,
                memo: None,
                padding: None,
            },
            &snip,
            &args.admin,
            Some(GAS),
            None,
            None,
            &mut reports,
            None,
        )?;
    }

    store_struct("testnet_airdrop.json", &reports);

    Ok(())
}
