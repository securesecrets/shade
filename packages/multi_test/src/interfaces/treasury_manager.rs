use crate::{
    interfaces::{
        treasury,
        utils::{DeployedContracts, SupportedContracts},
    },
    multi::{admin::init_admin_auth, treasury_manager::TreasuryManager},
};
use shade_protocol::{
    c_std::{Addr, StdError, StdResult, Uint128},
    contract_interfaces::dao::{manager, treasury_manager},
    multi_test::App,
    utils::{
        asset::{Contract, RawContract},
        storage::plus::period_storage::Period,
        ExecuteCallback,
        InstantiateCallback,
        MultiTestable,
        Query,
    },
};

pub fn init(
    chain: &mut App,
    sender: &str,
    contracts: &mut DeployedContracts,
    id: usize,
) -> StdResult<()> {
    let treasury = match contracts.get(&SupportedContracts::Treasury) {
        Some(treasury) => treasury.clone(),
        None => {
            treasury::init(chain, sender, contracts)?;
            contracts
                .get(&SupportedContracts::Treasury)
                .unwrap()
                .clone()
        }
    };
    let admin_auth = match contracts.get(&SupportedContracts::AdminAuth) {
        Some(admin) => admin.clone(),
        None => {
            let contract = Contract::from(init_admin_auth(chain, &Addr::unchecked(sender)));
            contracts.insert(SupportedContracts::AdminAuth, contract.clone());
            contract
        }
    };
    let treasury_manager = Contract::from(
        match (treasury_manager::InstantiateMsg {
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
        )) {
            Ok(contract_info) => contract_info,
            Err(e) => return Err(StdError::generic_err(e.to_string())),
        },
    );
    contracts.insert(SupportedContracts::TreasuryManager(id), treasury_manager);
    Ok(())
}

pub fn claimable_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    treasury_manager_contract: SupportedContracts,
    holder: SupportedContracts,
) -> StdResult<Uint128> {
    match treasury_manager::QueryMsg::Manager(manager::SubQueryMsg::Claimable {
        holder: contracts.get(&holder).unwrap().address.to_string(),
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .address
            .to_string(),
    })
    .test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )? {
        manager::QueryAnswer::Claimable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            "Failed to test query treasury_manager claimable",
        )),
    }
}

pub fn config_query(
    chain: &App,
    contracts: &DeployedContracts,
    treasury_manager_contract: SupportedContracts,
) -> StdResult<treasury_manager::Config> {
    let res = treasury_manager::QueryMsg::Config {}.test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )?;
    match res {
        treasury_manager::QueryAnswer::Config { config } => Ok(config),
        _ => Err(StdError::generic_err(format!(
            "Failed to.test_query treasury_manager claimable",
        ))),
    }
}

pub fn pending_allowance_query(
    chain: &App,
    contracts: &DeployedContracts,
    treasury_manager_contract: SupportedContracts,
    snip20_symbol: &str,
) -> StdResult<Uint128> {
    let res = treasury_manager::QueryMsg::PendingAllowance {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .address
            .to_string(),
    }
    .test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )?;
    match res {
        treasury_manager::QueryAnswer::PendingAllowance { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to.test_query treasury_manager pending_allowance",
        ))),
    }
}

pub fn holding_query(
    chain: &App,
    contracts: &DeployedContracts,
    treasury_manager_contract: SupportedContracts,
    holder: String,
) -> StdResult<treasury_manager::Holding> {
    let res = treasury_manager::QueryMsg::Holding { holder }.test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )?;
    match res {
        treasury_manager::QueryAnswer::Holding { holding } => Ok(holding),
        _ => Err(StdError::generic_err(format!(
            "Failed to.test_query treasury_manager claimable",
        ))),
    }
}

pub fn holders_query(
    chain: &App,
    contracts: &DeployedContracts,
    treasury_manager_contract: SupportedContracts,
) -> StdResult<Vec<Addr>> {
    let res = treasury_manager::QueryMsg::Holders {}.test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )?;
    match res {
        treasury_manager::QueryAnswer::Holders { holders } => Ok(holders),
        _ => Err(StdError::generic_err(format!(
            "Failed to.test_query treasury_manager holders",
        ))),
    }
}

pub fn assets_query(
    chain: &App,
    contracts: &DeployedContracts,
    treasury_manager_contract: SupportedContracts,
) -> StdResult<Vec<Addr>> {
    let res = treasury_manager::QueryMsg::Assets {}.test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )?;
    match res {
        treasury_manager::QueryAnswer::Assets { assets } => Ok(assets),
        _ => Err(StdError::generic_err(format!(
            "Failed to.test_query treasury_manager holders",
        ))),
    }
}

pub fn allocations_query(
    chain: &App,
    contracts: &DeployedContracts,
    treasury_manager_contract: SupportedContracts,
    snip20_symbol: &str,
) -> StdResult<Vec<treasury_manager::AllocationMeta>> {
    let res = treasury_manager::QueryMsg::Allocations {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .address
            .to_string(),
    }
    .test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )?;
    match res {
        treasury_manager::QueryAnswer::Allocations { allocations } => Ok(allocations),
        _ => Err(StdError::generic_err(format!(
            "Failed to.test_query treasury_manager allocations",
        ))),
    }
}

pub fn metrics_query(
    chain: &App,
    contracts: &DeployedContracts,
    treasury_manager_contract: SupportedContracts,
    date: Option<String>,
    epoch: Option<Uint128>,
    period: Period,
) -> StdResult<Vec<treasury_manager::Metric>> {
    let res = treasury_manager::QueryMsg::Metrics {
        date,
        epoch,
        period,
    }
    .test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )?;
    match res {
        treasury_manager::QueryAnswer::Metrics { metrics } => Ok(metrics),
        _ => Err(StdError::generic_err(format!(
            "Failed to.test_query treasury_manager metrics",
        ))),
    }
}

pub fn unbonding_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    treasury_manager_contract: SupportedContracts,
    holder: SupportedContracts,
) -> StdResult<Uint128> {
    match treasury_manager::QueryMsg::Manager(manager::SubQueryMsg::Unbonding {
        holder: contracts.get(&holder).unwrap().address.to_string(),
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .address
            .to_string(),
    })
    .test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )? {
        manager::QueryAnswer::Unbonding { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            "Failed to.test_query treasury_manager unbonding",
        )),
    }
}

pub fn unbondable_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    treasury_manager_contract: SupportedContracts,
    holder: SupportedContracts,
) -> StdResult<Uint128> {
    match treasury_manager::QueryMsg::Manager(manager::SubQueryMsg::Unbondable {
        holder: contracts.get(&holder).unwrap().address.to_string(),
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .address
            .to_string(),
    })
    .test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )? {
        manager::QueryAnswer::Unbondable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            "Failed to.test_query treasury_manager unbondable",
        )),
    }
}

pub fn reserves_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    treasury_manager_contract: SupportedContracts,
    holder: SupportedContracts,
) -> StdResult<Uint128> {
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Reserves {
        holder: contracts.get(&holder).unwrap().address.to_string(),
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .address
            .to_string(),
    })
    .test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )? {
        manager::QueryAnswer::Reserves { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            "Failed to query treasury_manager reserves",
        )),
    }
}

pub fn batch_balance_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbols: Vec<&str>,
    treasury_manager_contract: SupportedContracts,
    holder: SupportedContracts,
) -> StdResult<Vec<Uint128>> {
    let assets = {
        let mut vec = vec![];
        for symbols in snip20_symbols {
            vec.push(
                contracts
                    .get(&SupportedContracts::Snip20(symbols.to_string()))
                    .unwrap()
                    .clone()
                    .address
                    .to_string(),
            );
        }
        vec
    };
    match manager::QueryMsg::Manager(manager::SubQueryMsg::BatchBalance {
        holder: contracts.get(&holder).unwrap().address.to_string(),
        assets,
    })
    .test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )? {
        manager::QueryAnswer::BatchBalance { amounts } => Ok(amounts),
        _ => Err(StdError::generic_err(
            "Failed to query treasury_manager reserves",
        )),
    }
}

pub fn balance_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    treasury_manager_contract: SupportedContracts,
    holder: SupportedContracts,
) -> StdResult<Uint128> {
    match treasury_manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
        holder: contracts.get(&holder).unwrap().address.to_string(),
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .address
            .to_string(),
    })
    .test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )? {
        manager::QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            "Failed to query treasury_manager balance",
        )),
    }
}

pub fn update_config_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    treasury_manager_contract: SupportedContracts,
    admin_auth: Option<RawContract>,
    treasury: Option<String>,
) -> StdResult<()> {
    match (treasury_manager::ExecuteMsg::UpdateConfig {
        admin_auth,
        treasury,
    }
    .test_exec(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err("claim in treasury manager failed")),
    }
}

pub fn claim_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    treasury_manager_contract: SupportedContracts,
) -> StdResult<()> {
    match treasury_manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Claim {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .address
            .to_string(),
    })
    .test_exec(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
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
    snip20_symbol: &str,
    treasury_manager_contract: SupportedContracts,
    amount: Uint128,
) -> StdResult<()> {
    match treasury_manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Unbond {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .address
            .to_string(),
        amount,
    })
    .test_exec(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    ) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err("update in treasury manager failed")),
    }
}

pub fn update_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    treasury_manager_contract: SupportedContracts,
) -> StdResult<()> {
    match treasury_manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Update {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .address
            .to_string(),
    })
    .test_exec(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn register_holder_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    treasury_manager_contract: SupportedContracts,
    holder: &str,
) -> StdResult<()> {
    match (treasury_manager::ExecuteMsg::AddHolder {
        holder: holder.to_string(),
    }
    .test_exec(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err(
            "register_holder in treasury manager failed",
        )),
    }
}

pub fn remove_holder_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    treasury_manager_contract: SupportedContracts,
    holder: &str,
) -> StdResult<()> {
    match (treasury_manager::ExecuteMsg::RemoveHolder {
        holder: holder.to_string(),
    }
    .test_exec(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err(
            "register_holder in treasury manager failed",
        )),
    }
}

pub fn register_asset_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    tm_contract: SupportedContracts,
) -> StdResult<()> {
    match (treasury_manager::ExecuteMsg::RegisterAsset {
        contract: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .into(),
    }
    .test_exec(
        &contracts.get(&tm_contract).unwrap().clone().into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn allocate_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    nickname: Option<String>,
    contract_to_allocate_to: &SupportedContracts,
    alloc_type: treasury_manager::AllocationType,
    amount: Uint128,
    tolerance: Uint128,
    id: usize,
) -> StdResult<()> {
    match (treasury_manager::ExecuteMsg::Allocate {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .address
            .to_string(),
        allocation: treasury_manager::RawAllocation {
            nick: nickname,
            contract: RawContract::from(contracts.get(contract_to_allocate_to).unwrap().clone()),
            alloc_type,
            amount,
            tolerance,
        },
    }
    .test_exec(
        &contracts
            .get(&SupportedContracts::TreasuryManager(id))
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}
