use crate::{
    interfaces::{
        snip20,
        treasury,
        utils::{DeployedContracts, SupportedContracts},
    },
    multi::{admin::init_admin_auth, mock_adapter::MockAdapter, treasury_manager::TreasuryManager},
};
use mock_adapter;
use shade_protocol::{
    c_std::{Addr, StdError, StdResult, Uint128},
    contract_interfaces::dao::{manager, treasury::AllowanceType, treasury_manager},
    multi_test::App,
    utils::{self, asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

pub fn init(chain: &mut App, sender: &str, contracts: &mut DeployedContracts, id: usize) {
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
            let contract = Contract::from(init_admin_auth(chain, &Addr::unchecked(sender)));
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
    id: usize,
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
            .get(&SupportedContracts::TreasuryManager(id))
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )
    .unwrap();
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
    id: usize,
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
            .get(&SupportedContracts::TreasuryManager(id))
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )
    .unwrap();
}

pub fn claimable_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    treasury_manager_contract: SupportedContracts,
    holder: SupportedContracts,
) -> StdResult<Uint128> {
    match treasury_manager::QueryMsg::Manager(manager::SubQueryMsg::Claimable {
        holder: contracts.get(&holder).unwrap().address.to_string(),
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
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
        _ => Err(StdError::generic_err(format!(
            "Failed to.test_query treasury_manager claimable",
        ))),
    }
}

pub fn holding_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    treasury_manager_contract: SupportedContracts,
    holder: String,
) -> StdResult<treasury_manager::Holding> {
    match (treasury_manager::QueryMsg::Holding { holder }.test_query(
        &contracts
            .get(&treasury_manager_contract)
            .unwrap()
            .clone()
            .into(),
        &chain,
    )?) {
        treasury_manager::QueryAnswer::Holding { holding } => Ok(holding),
        _ => Err(StdError::generic_err(format!(
            "Failed to.test_query treasury_manager claimable",
        ))),
    }
}

pub fn unbonding_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    treasury_manager_contract: SupportedContracts,
    holder: SupportedContracts,
) -> StdResult<Uint128> {
    match treasury_manager::QueryMsg::Manager(manager::SubQueryMsg::Unbonding {
        holder: contracts.get(&holder).unwrap().address.to_string(),
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
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
    snip20_symbol: String,
    treasury_manager_contract: SupportedContracts,
    holder: SupportedContracts,
) -> StdResult<Uint128> {
    match treasury_manager::QueryMsg::Manager(manager::SubQueryMsg::Unbondable {
        holder: contracts.get(&holder).unwrap().address.to_string(),
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
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
    snip20_symbol: String,
    treasury_manager_contract: SupportedContracts,
    holder: SupportedContracts,
) -> StdResult<Uint128> {
    match (manager::QueryMsg::Manager(manager::SubQueryMsg::Reserves {
        holder: contracts.get(&holder).unwrap().address.to_string(),
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
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
    )?) {
        manager::QueryAnswer::Reserves { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            "Failed to query treasury_manager reserves",
        )),
    }
}

pub fn balance_query(
    chain: &App,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    treasury_manager_contract: SupportedContracts,
    holder: SupportedContracts,
) -> StdResult<Uint128> {
    match treasury_manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
        holder: contracts.get(&holder).unwrap().address.to_string(),
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
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

pub fn claim_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    treasury_manager_contract: SupportedContracts,
) -> StdResult<()> {
    match (treasury_manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Claim {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
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
    )) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err("claim in treasury manager failed")),
    }
}

pub fn unbond_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    treasury_manager_contract: SupportedContracts,
    amount: Uint128,
) -> StdResult<()> {
    match (treasury_manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Unbond {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
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
    )) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err("update in treasury manager failed")),
    }
}

pub fn update_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    treasury_manager_contract: SupportedContracts,
) -> StdResult<()> {
    match (treasury_manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Update {
        asset: contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
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
    )) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err("update in treasury manager failed")),
    }
}

pub fn register_holder_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
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
    snip20_symbol: String,
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
