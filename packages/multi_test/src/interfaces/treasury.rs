use crate::{
    interfaces::utils::{DeployedContracts, SupportedContracts},
    multi::{admin::init_admin_auth, treasury::Treasury},
};
use shade_protocol::{
    c_std::{Addr, StdError, StdResult, Uint128},
    contract_interfaces::dao::treasury,
    multi_test::App,
    utils::{
        asset::{Contract, RawContract},
        cycle::Cycle,
        storage::plus::period_storage::Period,
        ExecuteCallback,
        InstantiateCallback,
        MultiTestable,
        Query,
    },
};

pub fn init(chain: &mut App, sender: &str, contracts: &mut DeployedContracts) -> StdResult<()> {
    let admin = match contracts.get(&SupportedContracts::AdminAuth) {
        Some(admin) => admin.clone(),
        None => {
            let contract = Contract::from(init_admin_auth(chain, &Addr::unchecked(sender)));
            contracts.insert(SupportedContracts::AdminAuth, contract.clone());
            contract
        }
    };
    let treasury = Contract::from(
        match (treasury::InstantiateMsg {
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
        )) {
            Ok(contract_info) => contract_info,
            Err(e) => return Err(StdError::generic_err(e.to_string())),
        },
    );
    contracts.insert(SupportedContracts::Treasury, treasury);
    Ok(())
}

pub fn config_query(chain: &App, contracts: &DeployedContracts) -> StdResult<treasury::Config> {
    let res = treasury::QueryMsg::Config {}.test_query(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
    )?;
    match res {
        treasury::QueryAnswer::Config { config } => Ok(config),
        _ => Err(StdError::generic_err("query failed")),
    }
}

pub fn allowance_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    spender: SupportedContracts,
) -> StdResult<Uint128> {
    let res = treasury::QueryMsg::Allowance {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .address
            .to_string(),
        spender: contracts.get(&spender).unwrap().clone().address.to_string(),
    }
    .test_query(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
    )?;
    match res {
        treasury::QueryAnswer::Allowance { amount } => Ok(amount),
        _ => Err(StdError::generic_err("query failed")),
    }
}

pub fn allowances_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
) -> StdResult<Vec<treasury::AllowanceMeta>> {
    let res = treasury::QueryMsg::Allowances {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .address
            .to_string(),
    }
    .test_query(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
    )?;
    match res {
        treasury::QueryAnswer::Allowances { allowances } => Ok(allowances),
        _ => Err(StdError::generic_err("query failed")),
    }
}

pub fn assets_query(chain: &App, contracts: &DeployedContracts) -> StdResult<Vec<Addr>> {
    let res = treasury::QueryMsg::Assets {}.test_query(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
    )?;
    match res {
        treasury::QueryAnswer::Assets { assets } => Ok(assets),
        _ => Err(StdError::generic_err("query failed")),
    }
}

pub fn reserves_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
) -> StdResult<Uint128> {
    let res = treasury::QueryMsg::Reserves {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .address
            .to_string(),
    }
    .test_query(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
    )?;
    match res {
        treasury::QueryAnswer::Reserves { amount } => Ok(amount),
        _ => Err(StdError::generic_err("query failed")),
    }
}

pub fn balance_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
) -> StdResult<Uint128> {
    let res = treasury::QueryMsg::Balance {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .address
            .to_string(),
    }
    .test_query(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
    )?;
    match res {
        treasury::QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err("query failed")),
    }
}

pub fn run_level_query(
    chain: &App,
    contracts: &DeployedContracts,
) -> StdResult<treasury::RunLevel> {
    let res = treasury::QueryMsg::RunLevel {}.test_query(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
    )?;
    match res {
        treasury::QueryAnswer::RunLevel { run_level } => Ok(run_level),
        _ => Err(StdError::generic_err("query failed")),
    }
}

pub fn metrics_query(
    chain: &App,
    contracts: &DeployedContracts,
    date: Option<String>,
    epoch: Option<Uint128>,
    period: Period,
) -> StdResult<Vec<treasury::Metric>> {
    let res = treasury::QueryMsg::Metrics {
        date,
        epoch,
        period,
    }
    .test_query(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
    )?;
    match res {
        treasury::QueryAnswer::Metrics { metrics } => Ok(metrics),
        _ => Err(StdError::generic_err("query failed")),
    }
}

pub fn batch_balance_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbols: Vec<&str>,
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
    match (treasury::QueryMsg::BatchBalance { assets }.test_query(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
    )) {
        Ok(a) => Ok(a),
        _ => Err(StdError::generic_err("query failed")),
    }
}

pub fn register_asset_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
) -> StdResult<()> {
    match (treasury::ExecuteMsg::RegisterAsset {
        contract: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
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
    )) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err("register wrap failed")),
    }
}

pub fn register_manager_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    manager_id: usize,
) -> StdResult<()> {
    match (treasury::ExecuteMsg::RegisterManager {
        contract: contracts
            .get(&SupportedContracts::TreasuryManager(manager_id))
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
    )) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err("register wrap failed")),
    }
}

pub fn register_wrap_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    denom: String,
    contract: RawContract,
) -> StdResult<()> {
    match (treasury::ExecuteMsg::RegisterWrap { denom, contract }.test_exec(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err("register wrap failed")),
    }
}

pub fn allowance_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    manager_id: usize,
    allowance_type: treasury::AllowanceType,
    cycle: Cycle,
    amount: Uint128,
    tolerance: Uint128,
    refresh_now: bool,
) -> StdResult<()> {
    match (treasury::ExecuteMsg::Allowance {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .address
            .to_string(),
        allowance: treasury::RawAllowance {
            spender: contracts
                .get(&SupportedContracts::TreasuryManager(manager_id))
                .unwrap()
                .clone()
                .address
                .to_string(),
            allowance_type,
            cycle,
            amount,
            tolerance,
        },
        refresh_now,
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
    )) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err("allowance exec failed")),
    }
}

pub fn set_run_level_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    run_level: treasury::RunLevel,
) -> StdResult<()> {
    match (treasury::ExecuteMsg::SetRunLevel { run_level }.test_exec(
        &contracts
            .get(&SupportedContracts::Treasury)
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => {
            return Err(StdError::generic_err(e.to_string()));
        }
    }
}

pub fn set_config(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    admin_auth: Option<RawContract>,
    multisig: Option<String>,
) -> StdResult<()> {
    match (treasury::ExecuteMsg::UpdateConfig {
        admin_auth,
        multisig,
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
    )) {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn update_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
) -> StdResult<()> {
    let res = treasury::ExecuteMsg::Update {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .address
            .to_string(),
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
    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}
