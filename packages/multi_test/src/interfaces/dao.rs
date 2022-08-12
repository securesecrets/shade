use crate::{
    interfaces::{
        snip20,
        treasury,
        treasury_manager,
        utils::{DeployedContracts, SupportedContracts},
    },
    multi::mock_adapter::MockAdapter,
};
use shade_protocol::{
    c_std::{Addr, Uint128},
    contract_interfaces::dao::{
        adapter,
        treasury::AllowanceType,
        treasury_manager::AllocationType,
    },
    multi_test::App,
    utils::{self, asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable},
};

pub fn init_dao(
    chain: &mut App,
    sender: &str,
    contracts: &mut DeployedContracts,
    num_managers: u8,
    num_adapters: u8,
    treasury_start_bal: Uint128,
    allowance_type: AllowanceType,
    cycle: utils::cycle::Cycle,
    allowance_amount: Uint128,
    allowance_tolerance: Uint128,
    tm_allowance_type: AllocationType,
    tm_alocation_amount: Uint128,
    tm_alocation_tolerance: Uint128,
) {
    treasury::init(chain, sender, contracts);
    snip20::init(
        chain,
        sender,
        contracts,
        "secretSCRT".to_string(),
        "SSCRT".to_string(),
        6,
        None,
    );
    snip20::send(
        chain,
        sender,
        contracts,
        "secretSCRT".to_string(),
        contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .address
            .to_string(),
        treasury_start_bal,
        None,
    );
    treasury::register_asset(chain, sender, contracts, "SSCRT".to_string());
    for i in 0..num_managers {
        treasury_manager::init(chain, sender, contracts, i);
        treasury::register_manager(chain, sender, contracts, i);
        treasury::allowance(
            chain,
            sender,
            contracts,
            "SSCRT".to_string(),
            i,
            allowance_type.clone(),
            cycle.clone(),
            allowance_amount,
            allowance_tolerance.clone(),
        );
        for j in 0..num_adapters {
            let mock_adap_contract = Contract::from(
                mock_adapter::contract::Config {
                    owner: contracts
                        .get(&SupportedContracts::TreasuryManager(i))
                        .unwrap()
                        .clone()
                        .address,
                    unbond_blocks: Uint128::zero(),
                    token: contracts
                        .get(&SupportedContracts::Snip20("SSCRT".to_string()))
                        .unwrap()
                        .clone(),
                }
                .test_init(
                    MockAdapter::default(),
                    chain,
                    Addr::unchecked(sender),
                    "mock_adapter",
                    &[],
                )
                .unwrap(),
            );
            contracts.insert(SupportedContracts::MockAdapter(j), mock_adap_contract);
            treasury_manager::allocate(
                chain,
                sender,
                contracts,
                "SSCRT".to_string(),
                Some(j.to_string()),
                &SupportedContracts::MockAdapter(j),
                tm_allowance_type.clone(),
                tm_alocation_amount,
                tm_alocation_tolerance,
            );
        }
    }
    update(
        chain,
        sender,
        contracts,
        "SSCRT".to_string(),
        SupportedContracts::Treasury,
    );
    for i in 0..num_managers {
        update(
            chain,
            sender,
            contracts,
            "SSCRT".to_string(),
            SupportedContracts::TreasuryManager(i),
        );
    }
}

pub fn update(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    adapter_contract: SupportedContracts,
) {
    adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Update {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .clone()
            .address
            .to_string(),
    })
    .test_exec(
        &contracts.get(&adapter_contract).unwrap().clone().into(),
        chain,
        Addr::unchecked(sender),
        &[],
    );
}
