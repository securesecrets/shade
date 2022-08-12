use crate::{
    interfaces::utils::{DeployedContracts, SupportedContracts},
    multi::treasury::Treasury,
};
use shade_admin_multi_test::multi::helpers::init_admin_auth;
use shade_protocol::{
    c_std::{Addr, Uint128},
    contract_interfaces::dao::treasury,
    multi_test::App,
    utils::{asset::Contract, cycle::Cycle, ExecuteCallback, InstantiateCallback, MultiTestable},
};

pub fn init(chain: &mut App, sender: &str, contracts: &mut DeployedContracts) {
    /*let admin = {
        if contracts.contains_key(&SupportedContracts::AdminAuth) {
            contracts.get(&SupportedContracts::AdminAuth).unwrap()
        } else {
            let contract = Contract::from(init_admin_auth(chain, &Addr::unchecked(sender)));
            contracts.insert(SupportedContracts::AdminAuth, contract.clone());
            &contract
        }
    };*/
    let admin = match contracts.get(&SupportedContracts::AdminAuth) {
        Some(admin) => admin.clone(),
        None => {
            let contract = Contract::from(init_admin_auth(chain, &Addr::unchecked(sender), None));
            contracts.insert(SupportedContracts::AdminAuth, contract.clone());
            contract
        }
    };
    let treasury = Contract::from(
        treasury::InstantiateMsg {
            multisig: admin.address.clone().to_string(),
            admin_auth: admin.clone().into(),
            viewing_key: "viewing_key".to_string(),
        }
        .test_init(
            Treasury::default(),
            chain,
            Addr::unchecked(sender),
            "treasury",
            &[],
        )
        .unwrap(),
    );
    contracts.insert(SupportedContracts::Treasury, treasury);
}

pub fn register_asset(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    symbol: String,
) {
    treasury::ExecuteMsg::RegisterAsset {
        contract: contracts
            .get(&SupportedContracts::Snip20(symbol))
            .unwrap()
            .clone()
            .into(),
    }
    .test_exec(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    );
}

pub fn register_manager(chain: &mut App, sender: &str, contracts: &DeployedContracts) {
    treasury::ExecuteMsg::RegisterManager {
        contract: contracts
            .get(&SupportedContracts::TreasuryManager)
            .unwrap()
            .clone()
            .into(),
    }
    .test_exec(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    );
}

pub fn allowance(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    allowance_type: treasury::AllowanceType,
    cycle: Cycle,
    amount: Uint128,
    tolerance: Uint128,
) {
    treasury::ExecuteMsg::Allowance {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .clone()
            .address
            .to_string(),
        allowance: treasury::Allowance {
            spender: contracts
                .get(&SupportedContracts::TreasuryManager)
                .unwrap()
                .clone()
                .address,
            allowance_type,
            cycle,
            amount,
            tolerance,
        },
    }
    .test_exec(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    );
}
