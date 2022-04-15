use colored::*;
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{to_binary, HumanAddr};
use rand::{distributions::Alphanumeric, Rng};
use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, list_contracts_by_code, TestHandle, TestInit, TestQuery},
};
use serde::Serialize;
use serde_json::Result;
use shade_protocol::{
    asset::Contract,
    band, initializer,
    initializer::Snip20ContractInfo,
    mint, mint,
    mint::MintLimit,
    oracle, snip20,
    snip20::{InitConfig, InitialBalance},
};
use std::fmt::Display;

const STORE_GAS: &str = "10000000";
const GAS: &str = "800000";
const VIEW_KEY: &str = "password";
const ACCOUNT_KEY: &str = "a";

fn generate_label(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

fn main() -> Result<()> {
    let account = account_address(ACCOUNT_KEY)?;

    println!("Using Account: {}", account.blue());

    // Initialize sSCRT
    print_header("Initializing sSCRT");
    let sSCRT = snip20::InitMsg {
        name: "sSCRT".to_string(),
        admin: None,
        symbol: "SSCRT".to_string(),
        decimals: 6,
        initial_balances: None,
        prng_seed: Default::default(),
        config: Some(InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
        }),
    }
    .inst_init(
        "../../compiled/snip20.wasm.gz",
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
    )?;
    print_contract(&sSCRT);

    snip20::HandleMsg::SetViewingKey {
        key: String::from(VIEW_KEY),
        padding: None,
    }
    .t_handle(&sSCRT, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    println!("Depositing 1000000000uscrt");

    snip20::HandleMsg::Deposit { padding: None }.t_handle(
        &sSCRT,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        Some("1000000000uscrt"),
    )?;

    println!("Total sSCRT: {}", get_balance(&sSCRT, account.clone()));

    // Initialize initializer
    print_header("Initializing Initializer");
    let mut shade = NetContract {
        label: generate_label(8),
        id: "".to_string(),
        address: "".to_string(),
        code_hash: sSCRT.code_hash.clone(),
    };

    let mut silk = NetContract {
        label: generate_label(8),
        id: "".to_string(),
        address: "".to_string(),
        code_hash: sSCRT.code_hash.clone(),
    };

    let initializer = initializer::InitMsg {
        snip20_id: sSCRT.id.parse::<u64>().unwrap(),
        snip20_code_hash: sSCRT.code_hash.clone(),
        shade: Snip20ContractInfo {
            label: shade.label.clone(),
            admin: None,
            prng_seed: Default::default(),
            initial_balances: Some(vec![InitialBalance {
                address: HumanAddr::from(account.clone()),
                amount: Uint128(10000000),
            }]),
        },
        silk: Snip20ContractInfo {
            label: silk.label.clone(),
            admin: None,
            prng_seed: Default::default(),
            initial_balances: None,
        },
    }
    .inst_init(
        "../../compiled/initializer.wasm.gz",
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
    )?;
    print_contract(&initializer);

    print_header("Getting uploaded Snip20s");

    let contracts = list_contracts_by_code(sSCRT.id.clone())?;

    for contract in contracts {
        if &contract.label == &shade.label {
            print_warning("Found Shade");
            shade.id = contract.code_id.to_string();
            shade.address = contract.address;
            print_contract(&shade);
        } else if &contract.label == &silk.label {
            print_warning("Found Silk");
            silk.id = contract.code_id.to_string();
            silk.address = contract.address;
            print_contract(&silk);
        }
    }

    // Set View keys
    snip20::HandleMsg::SetViewingKey {
        key: String::from(VIEW_KEY),
        padding: None,
    }
    .t_handle(&shade, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    println!("Total shade: {}", get_balance(&shade, account.clone()));

    snip20::HandleMsg::SetViewingKey {
        key: String::from(VIEW_KEY),
        padding: None,
    }
    .t_handle(&silk, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    println!("Total silk: {}", get_balance(&silk, account.clone()));

    print_header("Initializing Band Mock");

    let band = band::InitMsg {}.inst_init(
        "../../compiled/mock_band.wasm.gz",
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
    )?;

    print_contract(&band);

    print_header("Initializing Oracle");
    let oracle = oracle::InitMsg {
        admin: None,
        band: Contract {
            address: HumanAddr::from(band.address),
            code_hash: band.code_hash,
        },
        sscrt: Contract {
            address: HumanAddr::from(sSCRT.address.clone()),
            code_hash: sSCRT.code_hash.clone(),
        },
    }
    .inst_init(
        "../../compiled/oracle.wasm.gz",
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
    )?;

    print_contract(&oracle);

    print_header("Initializing Mint-Shade");
    let mint_shade = mint::InitMsg {
        admin: None,
        native_asset: Contract {
            address: HumanAddr::from(shade.address.clone()),
            code_hash: shade.code_hash.clone(),
        },
        oracle: Contract {
            address: HumanAddr::from(oracle.address.clone()),
            code_hash: oracle.code_hash.clone(),
        },
        peg: None,
        treasury: None,
        epoch_frequency: Some(Uint128(120)),
        epoch_mint_limit: Some(Uint128(1000000000)),
    }
    .inst_init(
        "../../compiled/mint.wasm.gz",
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
    )?;

    print_contract(&mint_shade);

    print_epoch_info(&mint_shade);

    print_header("Initializing Mint-Silk");
    let mint_silk = mint::InitMsg {
        admin: None,
        native_asset: Contract {
            address: HumanAddr::from(silk.address.clone()),
            code_hash: silk.code_hash.clone(),
        },
        oracle: Contract {
            address: HumanAddr::from(oracle.address.clone()),
            code_hash: oracle.code_hash.clone(),
        },
        peg: None,
        treasury: None,
        epoch_frequency: Some(Uint128(120)),
        epoch_mint_limit: Some(Uint128(1000000000)),
    }
    .inst_init(
        "../../compiled/mint.wasm.gz",
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
    )?;

    print_contract(&mint_silk);

    print_epoch_info(&mint_silk);

    print_header("Registering allowed tokens");
    mint::HandleMsg::RegisterAsset {
        contract: Contract {
            address: HumanAddr::from(sSCRT.address.clone()),
            code_hash: sSCRT.code_hash.clone(),
        },
        commission: Some(Uint128(1000)),
    }
    .t_handle(&mint_shade, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;
    mint::HandleMsg::RegisterAsset {
        contract: Contract {
            address: HumanAddr::from(silk.address.clone()),
            code_hash: silk.code_hash.clone(),
        },
        commission: Some(Uint128(1000)),
    }
    .t_handle(&mint_shade, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;
    mint::HandleMsg::RegisterAsset {
        contract: Contract {
            address: HumanAddr::from(shade.address.clone()),
            code_hash: shade.code_hash.clone(),
        },
        commission: Some(Uint128(1000)),
    }
    .t_handle(&mint_silk, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    {
        let query: mint::QueryAnswer =
            mint::QueryMsg::GetSupportedAssets {}.t_query(&mint_shade)?;
        if let mint::QueryAnswer::SupportedAssets { assets } = query {
            print_vec("Shade allowed tokens: ", assets);
        }
    }

    {
        let query: mint::QueryAnswer = mint::QueryMsg::GetSupportedAssets {}.t_query(&mint_silk)?;
        if let mint::QueryAnswer::SupportedAssets { assets } = query {
            print_vec("Silk allowed tokens: ", assets);
        }
    }

    print_header("Setting minters in snip20s");

    snip20::HandleMsg::SetMinters {
        minters: vec![HumanAddr::from(mint_shade.address.clone())],
        padding: None,
    }
    .t_handle(&shade, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    {
        let query: snip20::QueryAnswer = snip20::QueryMsg::Minters {}.t_query(&shade)?;
        if let snip20::QueryAnswer::Minters { minters } = query {
            print_vec("Shade minters: ", minters);
        }
    }

    snip20::HandleMsg::SetMinters {
        minters: vec![HumanAddr::from(mint_silk.address.clone())],
        padding: None,
    }
    .t_handle(&silk, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    {
        let query: snip20::QueryAnswer = snip20::QueryMsg::Minters {}.t_query(&silk)?;
        if let snip20::QueryAnswer::Minters { minters } = query {
            print_vec("Silk minters: ", minters);
        }
    }

    print_header("Testing minting");

    {
        let amount = get_balance(&sSCRT, account.clone());
        println!("Burning {} usSCRT for Shade", amount.to_string().blue());

        mint(
            &sSCRT,
            ACCOUNT_KEY,
            mint_shade.address.clone(),
            amount,
            Uint128(0),
            "test",
        );
    }

    {
        let amount = get_balance(&shade, account.clone());
        println!("Minted {} uShade", amount.to_string().blue());
        print_epoch_info(&mint_shade);
    }

    // Test Mint Limit
    {
        let amount = Uint128(1000000000);
        println!("Burning {} uShade for Silk ", amount.to_string().blue());
        mint(
            &shade,
            ACCOUNT_KEY,
            mint_silk.address.clone(),
            amount,
            Uint128(0),
            "test",
        );
        print_epoch_info(&mint_silk);
        println!("Minted {} uSilk", get_balance(&silk, account.clone()));
    }

    // Try to send whats left
    {
        let amount = Uint128(10000000);
        let expected_total = Uint128(1010000000);
        while get_balance(&silk, account.clone()) != expected_total {
            mint(
                &shade,
                ACCOUNT_KEY,
                mint_silk.address.clone(),
                amount,
                Uint128(0),
                "test",
            );
        }
        print_epoch_info(&mint_silk);
        println!("Finally minted {} uSilk", amount);
    }

    Ok(())
}

fn print_header(header: &str) {
    println!("{}", header.on_blue());
}

fn print_warning(warn: &str) {
    println!("{}", warn.on_yellow());
}

fn print_contract(contract: &NetContract) {
    println!(
        "\tLabel: {}\n\tID: {}\n\tAddress: {}\n\tHash: {}",
        contract.label, contract.id, contract.address, contract.code_hash
    );
}

fn print_epoch_info(minter: &NetContract) {
    println!("\tEpoch information");
    let query = mint::QueryMsg::GetMintLimit {}.t_query(minter).unwrap();

    if let mint::QueryAnswer::MintLimit { limit } = query {
        println!(
            "\tFrequency: {}\n\tCapacity: {}\n\tTotal Minted: {}\n\tNext Epoch: {}",
            limit.frequency, limit.mint_capacity, limit.total_minted, limit.next_epoch
        );
    }
}

fn print_struct<Printable: Serialize>(item: Printable) {
    println!("{}", serde_json::to_string_pretty(&item).unwrap());
}

fn print_vec<Type: Display>(prefix: &str, vec: Vec<Type>) {
    for e in vec.iter().take(1) {
        print!("{}{}", prefix, e);
    }
    for e in vec.iter().skip(1) {
        print!(", {}", e);
    }
    println!();
}

fn get_balance(contract: &NetContract, from: String) -> Uint128 {
    let balance: snip20::QueryAnswer = snip20::QueryMsg::Balance {
        address: HumanAddr::from(from),
        key: String::from(VIEW_KEY),
    }
    .t_query(contract)
    .unwrap();

    if let snip20::QueryAnswer::Balance { amount } = balance {
        return amount;
    }

    Uint128(0)
}

fn mint(
    snip: &NetContract,
    sender: &str,
    minter: String,
    amount: Uint128,
    minimum_expected: Uint128,
    backend: &str,
) {
    snip20::HandleMsg::Send {
        recipient: HumanAddr::from(minter),
        amount,
        msg: Some(
            to_binary(&mint::MintMsgHook {
                minimum_expected_amount: minimum_expected,
            })
            .unwrap(),
        ),
        memo: None,
        padding: None,
    }
    .t_handle(snip, sender, Some(GAS), Some(backend), None)
    .unwrap();
}
