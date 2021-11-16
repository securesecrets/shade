use serde_json::{Result};
use shade_protocol::{governance, asset::Contract};
use cosmwasm_std::{HumanAddr, Uint128};
use shade_protocol::governance::GOVERNANCE_SELF;

use crate::utils::{print_header, generate_label, ACCOUNT_KEY, STORE_GAS, GAS,
                   print_contract, print_warning};

use secretcli::{cli_types::NetContract, secretcli::{query_contract, test_contract_handle, test_inst_init}};

pub fn init_contract<Init: serde::Serialize>(
    governance: &NetContract, contract_name: String,
    contract_path: &str, contract_init: Init) -> Result<NetContract> {
    print_header(&format!("{}{}", "Initializing ", contract_name.clone()));

    let contract = test_inst_init(&contract_init, contract_path,
                                           &*generate_label(8), ACCOUNT_KEY,
                                           Some(STORE_GAS), Some(GAS),
                                           Some("test"))?;

    print_contract(&contract);

    add_contract(contract_name, &contract, &governance)?;

    Ok(contract)
}

pub fn get_contract(governance: &NetContract, target: String) -> Result<Contract> {

    let msg = governance::QueryMsg::GetSupportedContract { name: target };

    let query: governance::QueryAnswer = query_contract(&governance, &msg)?;

    let mut ctrc = Contract {
        address: HumanAddr::from("not_found".to_string()),
        code_hash: "not_found".to_string()
    };

    if let governance::QueryAnswer::SupportedContract { contract } = query {
        ctrc = contract;
    }

    Ok(ctrc)
}

pub fn add_contract(name: String, target: &NetContract, governance: &NetContract) -> Result<()>{
    print_warning(&format!("{}{}", "Adding ", name.clone()));

    let msg = governance::HandleMsg::AddSupportedContract {
        name: name.clone(),
        contract: Contract{
            address: HumanAddr::from(target.address.clone()),
            code_hash: target.code_hash.clone()
        }
    };

    create_and_trigger_proposal(governance, GOVERNANCE_SELF.to_string(),
                                &msg, Some("Add a contract"))?;

    {
        let query_msg = governance::QueryMsg::GetSupportedContract { name };

        let query: governance::QueryAnswer  = query_contract(governance, query_msg)?;

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
    governance: &NetContract, target: String, handle: Handle, desc: Option<&str>) -> Result<Uint128> {
    create_proposal(governance, target, handle, desc)?;

    Ok(trigger_latest_proposal(governance)?)
}

pub fn get_latest_proposal(governance: &NetContract) -> Result<Uint128> {
    let query_msg = governance::QueryMsg::GetTotalProposals {};

    let query: governance::QueryAnswer = query_contract(&governance, &query_msg)?;

    let mut proposals = Uint128(1);

    if let governance::QueryAnswer::TotalProposals { total } = query {

        proposals = total;
    }
    else {
        assert!(false, "Query returned unexpected type")
    }

    Ok(proposals)
}

pub fn create_proposal<Handle: serde::Serialize>(
    governance: &NetContract, target: String, handle: Handle, desc: Option<&str>) -> Result<()> {
    let msg = serde_json::to_string(&handle)?;

    let proposal_msg = governance::HandleMsg::CreateProposal {
        target_contract: target,
        proposal: msg,
        description: match desc {
            None => "Custom proposal".to_string(),
            Some(description) => description.to_string()
        }
    };

    //let proposals = get_latest_proposal(governance)?;

    test_contract_handle(&proposal_msg, &governance, ACCOUNT_KEY, Some(GAS),
                         Some("test"), None)?;

    //assert_eq!(proposals, get_latest_proposal(governance)?);

    Ok(())
}

pub fn trigger_latest_proposal(governance: &NetContract) -> Result<Uint128> {

    let proposals = get_latest_proposal(governance)?;

    let handle_msg = governance::HandleMsg::TriggerProposal { proposal_id: proposals };

    test_contract_handle(&handle_msg, &governance, ACCOUNT_KEY, Some(GAS),
               Some("test"), None)?;

    Ok(proposals)
}
