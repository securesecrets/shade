use serde_json::Result;
use rand::{distributions::Alphanumeric, Rng};
use secretcli::{cli_types::NetContract, secretcli::{secretcli_run, account_address}};
use shade_protocol::{initializer::{Snip20ContractInfo}, micro_mint, snip20::{InitConfig, InitialBalance}, oracle, band, snip20, initializer, mint};
use secretcli::secretcli::{TestInit, TestHandle, TestQuery, list_contracts_by_code};
use cosmwasm_std::{HumanAddr, Uint128, to_binary};
use shade_protocol::asset::Contract;

fn generate_label(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

fn main() -> Result<()> {
    let store_gas = "10000000";
    let gas = "800000";
    let view_key = "password";

    let account_key = "a";
    let account = account_address(account_key)?;

    println!("Test");
    println!("Using Account: {}", account);

    // Initialize sSCRT
    println!("Initializing sSCRT");
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
            enable_burn: Some(false)
        })
    }.inst_init("../../compiled/snip20.wasm.gz", &*generate_label(8),
                account_key, Some(store_gas), Some(gas),
                Some("test"))?;
    println!("sSCRT\nAddress: {}\nCode Hash: {}", sSCRT.address, sSCRT.code_hash);

    snip20::HandleMsg::SetViewingKey { key: String::from(view_key), padding: None }.t_handle(
        &sSCRT, account_key, Some(gas), Some("test"), None)?;

    println!("Depositing 1000000000uscrt");

    snip20::HandleMsg::Deposit { padding: None }.t_handle(&sSCRT, account_key,
                                                          Some(gas), Some("test"),
                                                          Some("1000000000uscrt"))?;

    {
        let balance: snip20::QueryAnswer = snip20::QueryMsg::Balance {
            address: HumanAddr::from(account.clone()),
            key: String::from(view_key),
        }.t_query(&sSCRT)?;

        if let snip20::QueryAnswer::Balance { amount } = balance {
            println!("Total sSCRT: {}", amount);
        }
    }

    // Initialize initializer
    println!("Initializing Initializer");
    let mut shade = NetContract {
        label: generate_label(8),
        id: "".to_string(),
        address: "".to_string(),
        code_hash: sSCRT.code_hash.clone()
    };

    let mut silk = NetContract {
        label: generate_label(8),
        id: "".to_string(),
        address: "".to_string(),
        code_hash: sSCRT.code_hash.clone()
    };

    let initializer = initializer::InitMsg {
        snip20_id: sSCRT.id.parse::<u64>().unwrap(),
        snip20_code_hash: sSCRT.code_hash.clone(),
        shade: Snip20ContractInfo {
            label: shade.label.clone(),
            admin: None,
            prng_seed: Default::default(),
            initial_balances: Some(vec![InitialBalance{ address: HumanAddr::from(account.clone()), amount: Uint128(10000000) }])
        },
        silk: Snip20ContractInfo {
            label: silk.label.clone(),
            admin: None,
            prng_seed: Default::default(),
            initial_balances: None
        }
    }.inst_init("../../compiled/initializer.wasm.gz", &*generate_label(8),
                account_key, Some(store_gas), Some(gas),
                Some("test"))?;
    println!("Initializer\nAddress: {}\nCode Hash: {}", initializer.address, initializer.code_hash);


    println!("Getting uploaded Snip20s");

    let contracts = list_contracts_by_code(sSCRT.id.clone())?;

    for contract in contracts {
        if &contract.label == &shade.label {
            shade.id = contract.code_id.to_string();
            shade.address = contract.address;
        }
        else if &contract.label == &silk.label {
            silk.id = contract.code_id.to_string();
            silk.address = contract.address;
        }
    }

    // Set View keys
    snip20::HandleMsg::SetViewingKey { key: String::from(view_key), padding: None }.t_handle(
        &shade, account_key, Some(gas), Some("test"), None)?;

    {
        let balance: snip20::QueryAnswer = snip20::QueryMsg::Balance {
            address: HumanAddr::from(account.clone()),
            key: String::from(view_key),
        }.t_query(&shade)?;

        if let snip20::QueryAnswer::Balance { amount } = balance {
            println!("Total shade: {}", amount);
        }
    }

    snip20::HandleMsg::SetViewingKey { key: String::from(view_key), padding: None }.t_handle(
        &silk, account_key, Some(gas), Some("test"), None)?;

    {
        let balance: snip20::QueryAnswer = snip20::QueryMsg::Balance {
            address: HumanAddr::from(account.clone()),
            key: String::from(view_key),
        }.t_query(&silk)?;

        if let snip20::QueryAnswer::Balance { amount } = balance {
            println!("Total silk: {}", amount);
        }
    }

    println!("Initializing Band Mock");

    let band = band::InitMsg {}.inst_init("../../compiled/mock_band.wasm.gz", 
                                          &*generate_label(8), account_key, 
                                          Some(store_gas), Some(gas),
                                          Some("test"))?;

    println!("Initializing Oracle");
    let oracle = oracle::InitMsg {
        admin: None,
        band: Contract { address: HumanAddr::from(band.address), code_hash: band.code_hash },
        sscrt: Contract { address: HumanAddr::from(sSCRT.address.clone()),
            code_hash: sSCRT.code_hash.clone() }
    }.inst_init("../../compiled/oracle.wasm.gz", &*generate_label(8),
                account_key, Some(store_gas), Some(gas),
                Some("test"))?;

    println!("Initializing Mint-Shade");
    let mint_shade = micro_mint::InitMsg {
        admin: None,
        native_asset: Contract { address: HumanAddr::from(shade.address.clone()),
            code_hash: shade.code_hash.clone() },
        oracle: Contract { address: HumanAddr::from(oracle.address.clone()),
            code_hash: oracle.code_hash.clone() },
        peg: None,
        treasury: None,
        epoch_frequency: None,
        epoch_mint_limit: None,
    }.inst_init("../../compiled/micro_mint.wasm.gz", &*generate_label(8),
                account_key, Some(store_gas), Some(gas),
                Some("test"))?;

    println!("Initializing Mint-Silk");
    let mint_silk = micro_mint::InitMsg {
        admin: None,
        native_asset: Contract { address: HumanAddr::from(silk.address.clone()),
            code_hash: silk.code_hash.clone() },
        oracle: Contract { address: HumanAddr::from(oracle.address.clone()),
            code_hash: oracle.code_hash.clone() },
        peg: None,
        treasury: None,
        epoch_frequency: None,
        epoch_mint_limit: None,
    }.inst_init("../../compiled/micro_mint.wasm.gz", &*generate_label(8),
                account_key, Some(store_gas), Some(gas),
                Some("test"))?;

    println!("Registering allowed tokens");
    micro_mint::HandleMsg::RegisterAsset { contract: Contract {
        address: HumanAddr::from(sSCRT.address.clone()),
        code_hash: sSCRT.code_hash.clone() }, commission: Some(Uint128(1000)) }.t_handle(
        &mint_shade, account_key, Some(gas), Some("test"), None)?;
    micro_mint::HandleMsg::RegisterAsset { contract: Contract {
        address: HumanAddr::from(silk.address.clone()),
        code_hash: silk.code_hash.clone() }, commission: Some(Uint128(1000)) }.t_handle(
        &mint_shade, account_key, Some(gas), Some("test"), None)?;
    micro_mint::HandleMsg::RegisterAsset { contract: Contract {
        address: HumanAddr::from(shade.address.clone()),
        code_hash: shade.code_hash.clone() }, commission: Some(Uint128(1000)) }.t_handle(
        &mint_silk, account_key, Some(gas), Some("test"), None)?;

    println!("Shade allowed tokens: ");
    {
        let query: micro_mint::QueryAnswer = micro_mint::QueryMsg::GetSupportedAssets {}.t_query(
            &mint_shade)?;
        if let micro_mint::QueryAnswer::SupportedAssets { assets } = query {
            for asset in assets {
                print!("{}, ", asset);
            }
        }
    }

    println!("\nSilk allowed tokens: ");
    {
        let query: micro_mint::QueryAnswer = micro_mint::QueryMsg::GetSupportedAssets {}.t_query(
            &mint_silk)?;
        if let micro_mint::QueryAnswer::SupportedAssets { assets } = query {
            for asset in assets {
                print!("{}, ", asset);
            }
        }
    }

    println!("\nSetting minters in snip20s");
    
    snip20::HandleMsg::AddMinters {
        minters: vec![HumanAddr::from(mint_shade.address.clone())], padding: None }.t_handle(
        &shade, account_key, Some(gas), Some("test"), None)?;

    println!("\nShade minters: ");
    {
        let query: snip20::QueryAnswer = snip20::QueryMsg::Minters {}.t_query(&shade)?;
        if let snip20::QueryAnswer::Minters { minters } = query {
            for minter in minters {
                print!("{}, ", minter.to_string());
            }
        }
    }

    snip20::HandleMsg::AddMinters {
        minters: vec![HumanAddr::from(mint_silk.address.clone())], padding: None }.t_handle(
        &silk, account_key, Some(gas), Some("test"), None)?;

    println!("\nSilk minters: ");
    {
        let query: snip20::QueryAnswer = snip20::QueryMsg::Minters {}.t_query(&silk)?;
        if let snip20::QueryAnswer::Minters { minters } = query {
            for minter in minters {
                print!("{}, ", minter.to_string());
            }
        }
    }

    println!("\nSending all the sSCRT to Shade");
    let mut current_sscrt = Uint128(0);

    let balance: snip20::QueryAnswer = snip20::QueryMsg::Balance {
        address: HumanAddr::from(account.clone()),
        key: String::from(view_key),
    }.t_query(&sSCRT)?;

    if let snip20::QueryAnswer::Balance { amount } = balance {
        println!("Total sSCRT: {}", amount);
        current_sscrt = amount;
    }
    
    snip20::HandleMsg::Send {
        recipient: HumanAddr::from(mint_shade.address.clone()),
        amount: current_sscrt,
        msg: Some(to_binary(&mint::MintMsgHook { minimum_expected_amount: Uint128(0)}).unwrap()),
        memo: None,
        padding: None
    }.t_handle(&sSCRT, account_key, Some(gas), Some("test"),
               None)?;

    println!("Sending all the Shade to Silk");

    let mut current_shade = Uint128(0);

    let balance: snip20::QueryAnswer = snip20::QueryMsg::Balance {
        address: HumanAddr::from(account.clone()),
        key: String::from(view_key),
    }.t_query(&shade)?;

    if let snip20::QueryAnswer::Balance { amount } = balance {
        println!("Total shade: {}", amount);
        current_shade = amount;
    }

    snip20::HandleMsg::Send {
        recipient: HumanAddr::from(mint_silk.address.clone()),
        amount: current_shade,
        msg: Some(to_binary(&mint::MintMsgHook { minimum_expected_amount: Uint128(0)}).unwrap()),
        memo: None,
        padding: None
    }.t_handle(&shade, account_key, Some(gas), Some("test"),
               None)?;

    println!("Sending all the Silk to Shade");

    let mut current_silk = Uint128(0);

    let balance: snip20::QueryAnswer = snip20::QueryMsg::Balance {
        address: HumanAddr::from(account.clone()),
        key: String::from(view_key),
    }.t_query(&silk)?;

    if let snip20::QueryAnswer::Balance { amount } = balance {
        println!("Total silk: {}", amount);
        current_silk = amount;
    }

    snip20::HandleMsg::Send {
        recipient: HumanAddr::from(mint_shade.address.clone()),
        msg: Some(to_binary(&mint::MintMsgHook { minimum_expected_amount: Uint128(0)}).unwrap()),
        amount: current_silk,
        memo: None,
        padding: None
    }.t_handle(&silk, account_key, Some(gas), Some("test"),
               None)?;

    // Update MINTER limit and test against that

    Ok(())
}