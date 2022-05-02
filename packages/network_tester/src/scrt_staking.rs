use colored::*;
use serde_json::Result;
use rand::{distributions::Alphanumeric, Rng};
use secretcli::{cli_types::NetContract,
                secretcli::{account_address, TestInit, TestHandle,
                            TestQuery, list_contracts_by_code}};
use shade_protocol::{
    snip20::{
        InitConfig,
        InitialBalance,
    },
    snip20,
    scrt_staking,
};
use cosmwasm_std::{HumanAddr, to_binary};
use cosmwasm_math_compat::Uint128;
use shade_protocol::asset::Contract;
use std::fmt::Display;
use serde::Serialize;
use shade_protocol::mint::MintLimit;
use shade_protocol::governance::Proposal;

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
            enable_burn: Some(false)
        })
    }.inst_init("../../compiled/snip20.wasm.gz", &*generate_label(8),
                ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                Some("test"))?;
    print_contract(&sSCRT);

    snip20::HandleMsg::SetViewingKey { key: String::from(VIEW_KEY), padding: None }.t_handle(
        &sSCRT, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    println!("Depositing 1000000000uscrt");
    snip20::HandleMsg::Deposit { padding: None }.t_handle(&sSCRT, ACCOUNT_KEY,
                                                          Some(GAS), Some("test"),
                                                          Some("1000000000uscrt"))?;

    println!("Total sSCRT: {}", get_balance(&sSCRT, account.clone()));

    let scrt_staking = scrt_staking::InitMsg {
        admin: account,
        treasury: account,
        sscrt: sSCRT.address,
    }.inst_init("../../compiled/scrt_staking.wasm.gz", &*generate_label(8),
                ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                Some("test"))?;

    snip20::HandleMsg::Send {
        recipient: HumanAddr::from(minter),
        Uint128(100),
        memo: None,
        padding: None
    }.t_handle(snip, sender, Some(GAS), Some(backend), None).unwrap();

    // Initialize initializer
    /*
    print_header("Initializing Initializer");
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
                ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                Some("test"))?;
    print_contract(&initializer);


    print_header("Getting uploaded Snip20s");

    let contracts = list_contracts_by_code(sSCRT.id.clone())?;

    for contract in contracts {
        if &contract.label == &shade.label {
            print_warning("Found Shade");
            shade.id = contract.code_id.to_string();
            shade.address = contract.address;
            print_contract(&shade);
        }
        else if &contract.label == &silk.label {
            print_warning("Found Silk");
            silk.id = contract.code_id.to_string();
            silk.address = contract.address;
            print_contract(&silk);
        }
    }

    // Set View keys
    snip20::HandleMsg::SetViewingKey { key: String::from(VIEW_KEY), padding: None }.t_handle(
        &shade, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    println!("Total shade: {}", get_balance(&shade, account.clone()));

    snip20::HandleMsg::SetViewingKey { key: String::from(VIEW_KEY), padding: None }.t_handle(
        &silk, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    println!("Total silk: {}", get_balance(&silk, account.clone()));

    print_header("Initializing Band Mock");

    let band = band::InitMsg {}.inst_init("../../compiled/mock_band.wasm.gz",
                                          &*generate_label(8), ACCOUNT_KEY,
                                          Some(STORE_GAS), Some(GAS),
                                          Some("test"))?;

    print_contract(&band);

    print_header("Initializing Oracle");
    let oracle = oracle::InitMsg {
        admin: None,
        band: Contract { address: HumanAddr::from(band.address), code_hash: band.code_hash },
        sscrt: Contract { address: HumanAddr::from(sSCRT.address.clone()),
            code_hash: sSCRT.code_hash.clone() }
    }.inst_init("../../compiled/oracle.wasm.gz", &*generate_label(8),
                ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                Some("test"))?;

    print_contract(&oracle);

    print_header("Initializing Governance");
    let governance = governance::InitMsg {
        admin: None,
        proposal_deadline: 0,
        quorum: Uint128(0)
    }.inst_init("../../compiled/governance.wasm.gz", &*generate_label(8),
                ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                Some("test"))?;

    print_contract(&governance);

    print_header("Initializing Mint-Shade");
    let mint_shade = mint::InitMsg {
        admin: Some(HumanAddr::from(governance.address.clone())),
        native_asset: Contract { address: HumanAddr::from(shade.address.clone()),
            code_hash: shade.code_hash.clone() },
        oracle: Contract { address: HumanAddr::from(oracle.address.clone()),
            code_hash: oracle.code_hash.clone() },
        peg: None,
        treasury: None,
        secondary_burn: None,
        start_epoch: None,
        epoch_frequency: Some(Uint128(120)),
        epoch_mint_limit: Some(Uint128(1000000000)),
    }.inst_init("../../compiled/mint.wasm.gz", &*generate_label(8),
                ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                Some("test"))?;

    print_contract(&mint_shade);

    print_epoch_info(&mint_shade);

    print_header("Request add mint-shade to governance");

    governance::HandleMsg::CreateProposal {
        target_contract: "SELF".to_string(),
        proposal: serde_json::to_string(&governance::HandleMsg::AddSupportedContract {
            name: "mint-shade".to_string(),
            contract: Contract{
                address: HumanAddr::from(mint_shade.address.clone()),
                code_hash: mint_shade.code_hash.clone()
            }
        })?,
        description: "This is some description".to_string()
    }.t_handle(
        &governance, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    {
        let query: governance::QueryAnswer = governance::QueryMsg::GetProposals {
            start: Uint128(0), total: Uint128(4)
        }.t_query(&governance)?;

        if let governance::QueryAnswer::Proposals { proposals } = query {
            print_proposal(&proposals[0]);
        }
    }

    print_header("Trigger add mint-shade to governance");

    governance::HandleMsg::TriggerProposal { proposal_id: Uint128(1)
    }.t_handle(&governance, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    {
        let query: governance::QueryAnswer = governance::QueryMsg::GetSupportedContract {
            name: "mint-shade".to_string()
        }.t_query(&governance)?;

        if let governance::QueryAnswer::SupportedContract { contract } = query {
            println!("{}", contract.address);
        }
    }

    print_header("Request a mint limit change");
    // Print mint config
    {
        let query: mint::QueryAnswer = mint::QueryMsg::GetMintLimit {
        }.t_query(&mint_shade)?;

        if let mint::QueryAnswer::MintLimit { limit } = query {
            println!("Mint limit before change request");
            print_struct(limit);
        }
    }
    // Request mint config update
    {
        let msg = serde_json::to_string(&mint::HandleMsg::UpdateMintLimit {
            start_epoch: None,
            epoch_frequency: None,
            epoch_limit: Some(Uint128(2000000000)),
        })?;

        println!("{}",msg);

        governance::HandleMsg::CreateProposal {
            target_contract: "mint-shade".to_string(),
            proposal: msg,
            description: "Extend mint limit because of x and y reason".to_string()
        }.t_handle(&governance, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

        governance::HandleMsg::TriggerProposal { proposal_id: Uint128(2)
        }.t_handle(&governance, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;
    }

    // Print mint config
    {
        let query: mint::QueryAnswer = mint::QueryMsg::GetMintLimit {
        }.t_query(&mint_shade)?;

        if let mint::QueryAnswer::MintLimit { limit } = query {
            println!("Mint limit after change request");
            print_struct(limit);
        }
    }

    print_header("Give governance admin power");
    {
        // Using {} will allow us to replace with values
        governance::HandleMsg::CreateProposal {
            target_contract: "SELF".to_string(),
            proposal: serde_json::to_string(&governance::HandleMsg::AddAdminCommand {
                name: "update-mint-limit".to_string(),
                proposal: "{\"update_mint_limit\":{\"start_epoch\":null,\"epoch_frequency\":null,\"epoch_limit\":\"{}\"}}".to_string()
            })?,
            description: "Give admin power to modify whenever for x and y reason".to_string()
        }.t_handle(&governance, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

        governance::HandleMsg::TriggerProposal { proposal_id: Uint128(3)
        }.t_handle(&governance, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;
    }
    {
        let query: governance::QueryAnswer = governance::QueryMsg::GetAdminCommand {
            name: "update-mint-limit".to_string()
        }.t_query(&governance)?;

        if let governance::QueryAnswer::AdminCommand { command } = query {
            println!("\t{}\n\tTotal commands: {}", command.msg, command.total_arguments);
        }
    }
    print_header("Run admin command");
    // Print mint config
    {
        let query: mint::QueryAnswer = mint::QueryMsg::GetMintLimit {
        }.t_query(&mint_shade)?;

        if let mint::QueryAnswer::MintLimit { limit } = query {
            println!("Mint limit before change request");
            print_struct(limit);
        }
    }
    {
        governance::HandleMsg::TriggerAdminCommand {
            target: "mint-shade".to_string(),
            command: "update-mint-limit".to_string(),
            variables: vec!["1000000000".to_string()],
            description: "Admin triggered command".to_string()
        }.t_handle(&governance, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;
    }
    // Print mint config
    {
        let query: mint::QueryAnswer = mint::QueryMsg::GetMintLimit {
        }.t_query(&mint_shade)?;

        if let mint::QueryAnswer::MintLimit { limit } = query {
            println!("Mint limit after change request");
            print_struct(limit);
        }
    }
    */

    Ok(())
}

fn print_header(header: &str) {
    println!("{}", header.on_blue());
}

fn print_warning(warn: &str) {
    println!("{}", warn.on_yellow());
}

fn print_contract(contract: &NetContract) {
    println!("\tLabel: {}\n\tID: {}\n\tAddress: {}\n\tHash: {}", contract.label, contract.id,
             contract.address, contract.code_hash);
}

fn print_proposal(proposal: &Proposal) {
    println!("\tID: {}\n\tTarget: {}\n\tMsg: {}\n\tDescription: {}\n\tDue Date: {}",
             proposal.id, proposal.target, proposal.msg, proposal.description, proposal.due_date);
}

fn print_epoch_info(minter: &NetContract) {
    println!("\tEpoch information");
    let query = mint::QueryMsg::GetMintLimit {}.t_query(minter).unwrap();

    if let mint::QueryAnswer::MintLimit { limit } = query {
        println!("\tFrequency: {}\n\tCapacity: {}\n\tTotal Minted: {}\n\tNext Epoch: {}",
                 limit.frequency, limit.mint_capacity, limit.total_minted, limit.next_epoch);
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

fn get_balance(contract: &NetContract, from: String, ) -> Uint128 {
    let balance: snip20::QueryAnswer = snip20::QueryMsg::Balance {
        address: HumanAddr::from(from),
        key: String::from(VIEW_KEY),
    }.t_query(contract).unwrap();

    if let snip20::QueryAnswer::Balance { amount } = balance {
        return amount
    }

    Uint128(0)
}

fn mint(snip: &NetContract, sender: &str, minter: String, amount: Uint128,
        minimum_expected: Uint128, backend: &str) {
    snip20::HandleMsg::Send {
        recipient: HumanAddr::from(minter),
        amount,
        msg: Some(to_binary(&mint::MintMsgHook {
            minimum_expected_amount: minimum_expected}).unwrap()),
        memo: None,
        padding: None
    }.t_handle(snip, sender, Some(GAS), Some(backend), None).unwrap();
}
