/*
use cosmwasm_c_std::Uint128;
use cosmwasm_std::Addr;
use serde_json::Result;
use shade_protocol::contract_interfaces::{governance, governance::GOVERNANCE_SELF};

use crate::utils::{
    generate_label, print_contract, print_header, print_warning, ACCOUNT_KEY, GAS, STORE_GAS,
};

use secretcli::secretcli::Report;
use secretcli::{
    cli_types::NetContract,
    secretcli::{handle, init, query},
};
use shade_protocol::utils::asset::Contract;

pub fn init_with_gov<Init: serde::Serialize>(
    governance: &NetContract,
    contract_name: String,
    contract_path: &str,
    contract_init: Init,
    report: &mut Vec<Report>,
) -> Result<NetContract> {
    print_header(&format!("{}{}", "Initializing ", contract_name));

    let contract = init(
        &contract_init,
        contract_path,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        report,
    )?;

    print_contract(&contract);

    add_contract(contract_name, &contract, governance, report)?;

    Ok(contract)
}

pub fn get_contract(governance: &NetContract, target: String) -> Result<Contract> {
    let msg = governance::QueryMsg::GetSupportedContract { name: target };

    let query: governance::QueryAnswer = query(governance, &msg, None)?;

    let mut ctrc = Contract {
        address: Addr::from("not_found".to_string()),
        code_hash: "not_found".to_string(),
    };

    if let governance::QueryAnswer::SupportedContract { contract } = query {
        ctrc = contract;
    }

    Ok(ctrc)
}

pub fn add_contract(
    name: String,
    target: &NetContract,
    governance: &NetContract,
    report: &mut Vec<Report>,
) -> Result<()> {
    print_warning(&format!("{}{}", "Adding ", name));

    let msg = governance::HandleMsg::AddSupportedContract {
        name: name.clone(),
        contract: Contract {
            address: Addr::from(target.address.clone()),
            code_hash: target.code_hash.clone(),
        },
    };

    create_and_trigger_proposal(
        governance,
        GOVERNANCE_SELF.to_string(),
        &msg,
        Some("Add a contract"),
        report,
    )?;

    {
        let query_msg = governance::QueryMsg::GetSupportedContract { name };

        let query: governance::QueryAnswer = query(governance, query_msg, None)?;

        if let governance::QueryAnswer::SupportedContract { contract } = query {
            assert_eq!(contract.address.to_string(), target.address.to_string());
            assert_eq!(contract.code_hash, target.code_hash);
        } else {
            assert!(false, "Query returned unexpected type");
        }
    }

    Ok(())
}

/// Assumes that governance's staker is not activated
pub fn create_and_trigger_proposal<Handle: serde::Serialize>(
    governance: &NetContract,
    target: String,
    handle: Handle,
    desc: Option<&str>,
    report: &mut Vec<Report>,
) -> Result<Uint128> {
    create_proposal(governance, target, handle, desc, report)?;

    trigger_latest_proposal(governance, report)
}

pub fn get_latest_proposal(governance: &NetContract) -> Result<Uint128> {
    let query_msg = governance::QueryMsg::GetTotalProposals {};

    let query: governance::QueryAnswer = query(governance, &query_msg, None)?;

    let mut proposals = Uint128::new(1u128);

    if let governance::QueryAnswer::TotalProposals { total } = query {
        proposals = total;
    } else {
        assert!(false, "Query returned unexpected type")
    }

    Ok(proposals)
}

pub fn create_proposal<Handle: serde::Serialize>(
    governance: &NetContract,
    target: String,
    msg: Handle,
    desc: Option<&str>,
    report: &mut Vec<Report>,
) -> Result<()> {
    let proposal_msg = governance::HandleMsg::CreateProposal {
        target_contract: target,
        proposal: serde_json::to_string(&msg)?,
        description: match desc {
            None => "Custom proposal".to_string(),
            Some(description) => description.to_string(),
        },
    };

    //let proposals = get_latest_proposal(governance)?;

    handle(
        &proposal_msg,
        governance,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        report,
        None,
    )?;

    //assert_eq!(proposals, get_latest_proposal(governance)?);

    Ok(())
}

pub fn trigger_latest_proposal(
    governance: &NetContract,
    report: &mut Vec<Report>,
) -> Result<Uint128> {
    let proposals = get_latest_proposal(governance)?;

    let handle_msg = governance::HandleMsg::TriggerProposal {
        proposal_id: proposals,
    };

    handle(
        &handle_msg,
        governance,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        report,
        None,
    )?;

    Ok(proposals)
}
*/
