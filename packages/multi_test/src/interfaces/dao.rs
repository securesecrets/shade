use crate::{
    interfaces::{
        snip20,
        treasury,
        treasury_manager,
        utils::{DeployedContracts, SupportedContracts},
    },
    multi::mock_adapter::MockAdapter,
};
use mock_adapter;
use shade_protocol::{
    c_std::{Addr, StdError, StdResult, Uint128},
    contract_interfaces::dao::{
        adapter,
        treasury::AllowanceType,
        treasury_manager::AllocationType,
    },
    multi_test::App,
    utils::{self, asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

pub fn init_dao(
    chain: &mut App,
    sender: &str,
    contracts: &mut DeployedContracts,
    treasury_start_bal: Uint128,
    snip20_symbol: &str,
    allowance_type: Vec<AllowanceType>,
    cycle: Vec<utils::cycle::Cycle>,
    allowance_amount: Vec<Uint128>,
    allowance_tolerance: Vec<Uint128>,
    tm_allowance_type: Vec<Vec<AllocationType>>,
    tm_allocation_amount: Vec<Vec<Uint128>>,
    tm_allocation_tolerance: Vec<Vec<Uint128>>,
    is_instant_unbond: bool,
) {
    let num_managers = allowance_amount.len();
    treasury::init(chain, sender, contracts);
    let mut offset = 0;
    snip20::init(
        chain,
        sender,
        contracts,
        format!("snip20_1",),
        snip20_symbol.to_string(),
        6,
        None,
    );
    treasury::register_asset(chain, sender, contracts, snip20_symbol.to_string());
    snip20::send(
        chain,
        sender,
        contracts,
        snip20_symbol.to_string(),
        contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .address
            .to_string(),
        treasury_start_bal,
        None,
    )
    .unwrap();
    for i in 0..num_managers {
        let num_adapters = tm_allocation_amount[i].len();
        treasury_manager::init(chain, sender, contracts, i);
        treasury_manager::register_asset(chain, "admin", contracts, snip20_symbol.to_string(), i);
        treasury::register_manager(chain, sender, contracts, i);
        treasury::allowance(
            chain,
            sender,
            contracts,
            snip20_symbol.to_string(),
            i,
            allowance_type[i].clone(),
            cycle[i].clone(),
            allowance_amount[i].clone(),
            allowance_tolerance[i].clone(),
        );
        for j in 0..num_adapters {
            let mock_adap_contract = Contract::from(
                mock_adapter::contract::Config {
                    owner: contracts
                        .get(&SupportedContracts::TreasuryManager(i))
                        .unwrap()
                        .clone()
                        .address,
                    instant: is_instant_unbond,
                    token: contracts
                        .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
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
            contracts.insert(
                SupportedContracts::MockAdapter(j + offset),
                mock_adap_contract,
            );
            treasury_manager::allocate(
                chain,
                sender,
                contracts,
                snip20_symbol.to_string(),
                Some(j.to_string()),
                &SupportedContracts::MockAdapter(j + offset),
                tm_allowance_type.clone()[i][j].clone(),
                tm_allocation_amount[i][j].clone(),
                tm_allocation_tolerance[i][j].clone(),
                i,
            );
        }
        offset += num_adapters + 1;
    }
    update_dao(chain, sender, contracts, snip20_symbol, num_managers).unwrap();
}

pub fn update_dao(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    num_managers: usize,
) -> StdResult<()> {
    println!("HERE DAO UPDATE 1");
    treasury::update_exec(chain, sender, contracts, snip20_symbol.to_string())?;
    println!("HERE UPDATE DAO");
    for i in 0..num_managers {
        treasury_manager::update_exec(
            chain,
            sender,
            contracts,
            snip20_symbol.to_string(),
            SupportedContracts::TreasuryManager(i),
        )?;
    }
    Ok(())
}

pub fn system_balance_reserves(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: String,
) -> (Uint128, Vec<(Uint128, Vec<Uint128>)>) {
    println!("{}", snip20_symbol);
    let mut ret_struct = (Uint128::zero(), vec![]);
    ret_struct.0 = treasury::reserves_query(chain, contracts, snip20_symbol.clone()).unwrap();
    let (mut i, mut j) = (0, 0);
    let mut offset = 0;
    while true {
        let mut manager_tuple = (Uint128::zero(), vec![]);
        if contracts.get(&SupportedContracts::TreasuryManager(i)) == None {
            break;
        } else {
            manager_tuple.0 = match (treasury_manager::reserves_query(
                chain,
                contracts,
                snip20_symbol.clone(),
                SupportedContracts::TreasuryManager(i),
                SupportedContracts::Treasury,
            )) {
                Ok(bal) => bal,
                Err(_) => {
                    i += 1;
                    continue;
                }
            };
            j = 0;
            while true {
                if contracts.get(&SupportedContracts::MockAdapter(j + offset)) == None {
                    offset += j + 1;
                    break;
                } else {
                    manager_tuple.1.push(
                        reserves_query(
                            chain,
                            contracts,
                            snip20_symbol.clone(),
                            SupportedContracts::MockAdapter(j + offset),
                        )
                        .unwrap(),
                    );
                }
                j += 1;
            }
        }
        ret_struct.1.push(manager_tuple);
        i += 1;
    }
    ret_struct
}

pub fn system_balance_unbondable(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: String,
) -> (Uint128, Vec<(Uint128, Vec<Uint128>)>) {
    println!("{}", snip20_symbol);
    let mut ret_struct = (Uint128::zero(), vec![]);
    ret_struct.0 = treasury::reserves_query(chain, contracts, snip20_symbol.clone()).unwrap();
    let (mut i, mut j) = (0, 0);
    let mut offset = 0;
    while true {
        let mut manager_tuple = (Uint128::zero(), vec![]);
        if contracts.get(&SupportedContracts::TreasuryManager(i)) == None {
            break;
        } else {
            manager_tuple.0 = match (treasury_manager::reserves_query(
                chain,
                contracts,
                snip20_symbol.clone(),
                SupportedContracts::TreasuryManager(i),
                SupportedContracts::Treasury,
            )) {
                Ok(bal) => bal,
                Err(_) => {
                    i += 1;
                    continue;
                }
            };
            j = 0;
            while true {
                if contracts.get(&SupportedContracts::MockAdapter(j + offset)) == None {
                    offset += j + 1;
                    break;
                } else {
                    manager_tuple.1.push(
                        unbondable_query(
                            chain,
                            contracts,
                            snip20_symbol.clone(),
                            SupportedContracts::MockAdapter(j + offset),
                        )
                        .unwrap(),
                    );
                }
                j += 1;
            }
        }
        ret_struct.1.push(manager_tuple);
        i += 1;
    }
    ret_struct
}

pub fn claimable_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    adapter_contract: SupportedContracts,
) -> StdResult<Uint128> {
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Claimable {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .address
            .to_string(),
    })
    .test_query(
        &contracts.get(&adapter_contract).unwrap().clone().into(),
        &chain,
    )? {
        adapter::QueryAnswer::Claimable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to.test_query adapter claimable",
        ))),
    }
}

pub fn unbonding_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    adapter_contract: SupportedContracts,
) -> StdResult<Uint128> {
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbonding {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .address
            .to_string(),
    })
    .test_query(
        &contracts.get(&adapter_contract).unwrap().clone().into(),
        &chain,
    )? {
        adapter::QueryAnswer::Unbonding { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            "Failed to.test_query adapter unbonding",
        )),
    }
}

pub fn unbondable_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    adapter_contract: SupportedContracts,
) -> StdResult<Uint128> {
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbondable {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .address
            .to_string(),
    })
    .test_query(
        &contracts.get(&adapter_contract).unwrap().clone().into(),
        &chain,
    )? {
        adapter::QueryAnswer::Unbondable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            "Failed to.test_query adapter unbondable",
        )),
    }
}

pub fn reserves_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    adapter_contract: SupportedContracts,
) -> StdResult<Uint128> {
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .address
            .to_string(),
    })
    .test_query(
        &contracts.get(&adapter_contract).unwrap().clone().into(),
        &chain,
    )? {
        adapter::QueryAnswer::Reserves { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            "Failed to.test_query adapter unbondable",
        )),
    }
}

pub fn balance_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    adapter_contract: SupportedContracts,
) -> StdResult<Uint128> {
    match adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .address
            .to_string(),
    })
    .test_query(
        &contracts.get(&adapter_contract).unwrap().clone().into(),
        &chain,
    )? {
        adapter::QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            "Failed to.test_query adapter balance",
        )),
    }
}

pub fn claim_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    adapter_contract: SupportedContracts,
) -> StdResult<()> {
    let res = adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Claim {
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
    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn update_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    adapter_contract: SupportedContracts,
) -> StdResult<()> {
    println!(
        "{}",
        contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.clone()))
            .unwrap()
            .clone()
            .address
            .to_string()
    );
    println!(
        "{:?}",
        contracts.get(&adapter_contract.clone()).unwrap().clone()
    );
    match adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Update {
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
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn unbond_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    amount: Uint128,
    adapter_contract: SupportedContracts,
) -> StdResult<()> {
    match adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Unbond {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .clone()
            .address
            .to_string(),
        amount,
    })
    .test_exec(
        &contracts.get(&adapter_contract).unwrap().clone().into(),
        chain,
        Addr::unchecked(sender),
        &[],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn mock_adapter_sub_tokens(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    amount: Uint128,
    adapter_contract: SupportedContracts,
) -> StdResult<()> {
    match (mock_adapter::contract::ExecuteMsg::GiveMeMoney { amount }.test_exec(
        &contracts.get(&adapter_contract).unwrap().clone().into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn mock_adapter_complete_unbonding(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    adapter_contract: SupportedContracts,
) -> StdResult<()> {
    match (mock_adapter::contract::ExecuteMsg::CompleteUnbonding {}.test_exec(
        &contracts.get(&adapter_contract).unwrap().clone().into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}
