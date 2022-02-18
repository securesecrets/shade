use crate::{
    contract_helpers::minter::get_balance,
    utils::{
        generate_label, print_contract, print_header, print_warning, ACCOUNT_KEY, GAS,
        INITIALIZER_FILE, STORE_GAS, VIEW_KEY,
    },
};
use cosmwasm_std::{HumanAddr, Uint128};
use secretcli::{
    cli_types::NetContract,
    secretcli::{list_contracts_by_code, handle, init},
};
use serde_json::Result;
use secretcli::secretcli::Report;
use shade_protocol::{
    initializer, initializer::Snip20ContractInfo, snip20, snip20::InitialBalance,
};

pub fn initialize_initializer(
    admin: String,
    sscrt: &NetContract,
    account: String,
    report: &mut Vec<Report>
) -> Result<(NetContract, NetContract, NetContract)> {
    print_header("Initializing Initializer");
    let mut shade = NetContract {
        label: generate_label(8),
        id: "".to_string(),
        address: "".to_string(),
        code_hash: sscrt.code_hash.clone(),
    };

    let mut silk = NetContract {
        label: generate_label(8),
        id: "".to_string(),
        address: "".to_string(),
        code_hash: sscrt.code_hash.clone(),
    };

    let init_msg = initializer::InitMsg {
        admin: None,
        snip20_id: sscrt.id.parse::<u64>().unwrap(),
        snip20_code_hash: sscrt.code_hash.clone(),
        shade: Snip20ContractInfo {
            label: shade.label.clone(),
            admin: Some(HumanAddr::from(admin.clone())),
            prng_seed: Default::default(),
            initial_balances: Some(vec![InitialBalance {
                address: HumanAddr::from(account.clone()),
                amount: Uint128(10000000),
            }]),
        },
    };

    let initializer = init(
        &init_msg,
        INITIALIZER_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        report
    )?;

    handle(&initializer::HandleMsg::InitSilk {
        silk: Snip20ContractInfo {
            label: silk.label.clone(),
            admin: Some(HumanAddr::from(admin)),
            prng_seed: Default::default(),
            initial_balances: None,
        },
        ticker: "SILK".to_string(),
        decimals: 6,
    }, &initializer, ACCOUNT_KEY, Some(GAS), Some("test"), None, report, None)?;

    print_contract(&initializer);

    print_header("Getting uploaded Snip20s");

    let contracts = list_contracts_by_code(sscrt.id.clone())?;

    for contract in contracts {
        if contract.label == shade.label {
            print_warning("Found Shade");
            shade.id = contract.code_id.to_string();
            shade.address = contract.address;
            print_contract(&shade);
        } else if contract.label == silk.label {
            print_warning("Found Silk");
            silk.id = contract.code_id.to_string();
            silk.address = contract.address;
            print_contract(&silk);
        }
    }

    // Set View keys
    {
        let msg = snip20::HandleMsg::SetViewingKey {
            key: String::from(VIEW_KEY),
            padding: None,
        };

        handle(&msg, &shade, ACCOUNT_KEY, Some(GAS), Some("test"), None, report, None)?;
        handle(&msg, &silk, ACCOUNT_KEY, Some(GAS), Some("test"), None, report, None)?;
    }

    println!("\n\tTotal shade: {}", get_balance(&shade, account.clone()));
    println!("\tTotal silk: {}", get_balance(&silk, account));

    Ok((initializer, shade, silk))
}
