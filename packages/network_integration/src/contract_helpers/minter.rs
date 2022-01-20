use crate::{
    contract_helpers::governance::{create_and_trigger_proposal, get_contract, init_contract},
    utils::{
        print_contract, print_epoch_info, print_header, print_vec, GAS, MICRO_MINT_FILE, VIEW_KEY,
    },
};
use cosmwasm_std::{to_binary, HumanAddr, Uint128};
use secretcli::{
    cli_types::NetContract,
    secretcli::{query_contract, test_contract_handle},
};
use serde_json::Result;
use shade_protocol::utils::asset::Contract;
use shade_protocol::{micro_mint, mint, snip20};

pub fn initialize_minter(
    governance: &NetContract,
    contract_name: String,
    native_asset: &Contract,
) -> Result<NetContract> {
    let minter = init_contract(
        governance,
        contract_name,
        MICRO_MINT_FILE,
        micro_mint::InitMsg {
            admin: Some(HumanAddr::from(governance.address.clone())),
            native_asset: native_asset.clone(),
            oracle: get_contract(governance, "oracle".to_string())?,
            peg: None,
            treasury: None,
            secondary_burn: None,
            start_epoch: None,
            epoch_frequency: Some(Uint128(120)),
            epoch_mint_limit: Some(Uint128(1000000000)),
        },
    )?;

    print_contract(&minter);

    print_epoch_info(&minter);

    Ok(minter)
}

pub fn setup_minters(
    governance: &NetContract,
    mint_shade: &NetContract,
    mint_silk: &NetContract,
    shade: &Contract,
    silk: &Contract,
    sscrt: &NetContract,
) -> Result<()> {
    print_header("Registering allowed tokens in mint contracts");
    create_and_trigger_proposal(
        governance,
        "shade_minter".to_string(),
        micro_mint::HandleMsg::RegisterAsset {
            contract: Contract {
                address: HumanAddr::from(sscrt.address.clone()),
                code_hash: sscrt.code_hash.clone(),
            },
            capture: Some(Uint128(1000)),
        },
        Some("Register asset"),
    )?;
    create_and_trigger_proposal(
        governance,
        "shade_minter".to_string(),
        micro_mint::HandleMsg::RegisterAsset {
            contract: silk.clone(),
            capture: Some(Uint128(1000)),
        },
        Some("Register asset"),
    )?;
    create_and_trigger_proposal(
        governance,
        "silk_minter".to_string(),
        micro_mint::HandleMsg::RegisterAsset {
            contract: shade.clone(),
            capture: Some(Uint128(1000)),
        },
        Some("Register asset"),
    )?;

    print_header("Adding allowed minters in Snip20s");

    create_and_trigger_proposal(
        governance,
        "shade".to_string(),
        snip20::HandleMsg::SetMinters {
            minters: vec![HumanAddr::from(mint_shade.address.clone())],
            padding: None,
        },
        Some("Set minters"),
    )?;

    {
        let msg = snip20::QueryMsg::Minters {};

        let query: snip20::QueryAnswer = query_contract(
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: shade.address.clone().to_string(),
                code_hash: shade.code_hash.clone(),
            },
            &msg,
        )?;

        if let snip20::QueryAnswer::Minters { minters } = query {
            print_vec("Shade minters: ", minters);
        }
    }

    create_and_trigger_proposal(
        governance,
        "silk".to_string(),
        snip20::HandleMsg::SetMinters {
            minters: vec![HumanAddr::from(mint_silk.address.clone())],
            padding: None,
        },
        Some("Set minters"),
    )?;

    {
        let msg = snip20::QueryMsg::Minters {};

        let query: snip20::QueryAnswer = query_contract(
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: silk.address.clone().to_string(),
                code_hash: silk.code_hash.clone(),
            },
            &msg,
        )?;
        if let snip20::QueryAnswer::Minters { minters } = query {
            print_vec("Silk minters: ", minters);
        }
    }

    Ok(())
}

pub fn get_balance(contract: &NetContract, from: String) -> Uint128 {
    let msg = snip20::QueryMsg::Balance {
        address: HumanAddr::from(from),
        key: String::from(VIEW_KEY),
    };

    let balance: snip20::QueryAnswer = query_contract(contract, &msg).unwrap();

    if let snip20::QueryAnswer::Balance { amount } = balance {
        return amount;
    }

    Uint128(0)
}

pub fn mint(
    snip: &NetContract,
    sender: &str,
    minter: String,
    amount: Uint128,
    minimum_expected: Uint128,
    backend: &str,
) {
    let msg = snip20::HandleMsg::Send {
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
    };

    test_contract_handle(&msg, snip, sender, Some(GAS), Some(backend), None).unwrap();
}
