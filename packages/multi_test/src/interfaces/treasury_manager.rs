use crate::{
    interfaces::{
        snip20,
        treasury,
        utils::{DeployedContracts, SupportedContracts},
    },
    multi::{mock_adapter::MockAdapter, treasury_manager::TreasuryManager},
};
use mock_adapter;
use shade_admin_multi_test::multi::helpers::init_admin_auth;
use shade_protocol::{
    c_std::{Addr, Uint128},
    contract_interfaces::dao::{treasury::AllowanceType, treasury_manager},
    multi_test::App,
    utils::{self, asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable},
};

pub fn init(chain: &mut App, sender: &str, contracts: &mut DeployedContracts, id: u8) {
    /*let admin_auth = match admin_auth {
        Some(admin) => admin,
        None => Contract::from(init_admin_auth(chain, Addr::unchecked(sender), None)),
    };
    let treasury = match treasury {
        Some(treasury) => treasury,
        None => treasury::init(chain, sender, Some(admin_auth)),
    };*/
    let treasury = match contracts.get(&SupportedContracts::Treasury) {
        Some(treasury) => treasury.clone(),
        None => {
            treasury::init(chain, sender, contracts);
            contracts
                .get(&SupportedContracts::Treasury)
                .unwrap()
                .clone()
        }
    };
    let admin_auth = match contracts.get(&SupportedContracts::AdminAuth) {
        Some(admin) => admin.clone(),
        None => {
            let contract = Contract::from(init_admin_auth(chain, &Addr::unchecked(sender), None));
            contracts.insert(SupportedContracts::AdminAuth, contract.clone());
            contract
        }
    };
    let treasury_manager = Contract::from(
        treasury_manager::InstantiateMsg {
            admin_auth: admin_auth.into(),
            viewing_key: "viewing_key".to_string(),
            treasury: treasury.address.into(),
        }
        .test_init(
            TreasuryManager::default(),
            chain,
            Addr::unchecked(sender),
            "manager",
            &[],
        )
        .unwrap(),
    );
    contracts.insert(SupportedContracts::TreasuryManager(id), treasury_manager);
}

pub fn register_asset(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    symbol: String,
) {
    treasury_manager::ExecuteMsg::RegisterAsset {
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

pub fn allocate(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    nickname: Option<String>,
    contract_to_allocate_to: &SupportedContracts,
    alloc_type: treasury_manager::AllocationType,
    amount: Uint128,
    tolerance: Uint128,
) {
    treasury_manager::ExecuteMsg::Allocate {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .clone()
            .address
            .to_string(),
        allocation: treasury_manager::Allocation {
            nick: nickname,
            contract: contracts.get(contract_to_allocate_to).unwrap().clone(),
            alloc_type,
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
