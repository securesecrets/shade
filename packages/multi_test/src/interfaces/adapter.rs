use crate::{
    interfaces::{
        snip20,
        treasury,
        treasury_manager,
        utils::{DeployedContracts, SupportedContracts},
    },
    multi::{admin::init_admin_auth, scrt_staking::ScrtStaking},
};
use shade_protocol::{
    c_std::{Addr, ContractInfo, Uint128},
    contract_interfaces::dao::adapter,
    multi_test::App,
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

pub fn update(chain: &mut App, asset: Addr, sender: Addr, contract: &ContractInfo) {
    adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Update {
        asset: asset.to_string().clone(),
    })
    .test_exec(&contract, chain, sender.clone(), &[])
    .unwrap();
}

pub fn claim(chain: &mut App, asset: Addr, sender: Addr, contract: &ContractInfo) {
    adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Claim {
        asset: asset.to_string().clone(),
    })
    .test_exec(&contract, chain, sender.clone(), &[])
    .unwrap();
}

pub fn unbond(
    chain: &mut App,
    asset: Addr,
    amount: Uint128,
    sender: Addr,
    contract: &ContractInfo,
) {
    adapter::ExecuteMsg::Adapter(adapter::SubExecuteMsg::Unbond {
        asset: asset.to_string().clone(),
        amount,
    })
    .test_exec(&contract, chain, sender.clone(), &[])
    .unwrap();
}

pub fn unbondable(chain: &App, asset: Addr, contract: &ContractInfo) -> Uint128 {
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbondable {
        asset: asset.to_string().clone(),
    })
    .test_query(&contract, chain)
    .unwrap())
    {
        adapter::QueryAnswer::Unbondable { amount } => amount,
        _ => panic!("Unbondable query failed"),
    }
}

pub fn unbonding(chain: &App, asset: Addr, contract: &ContractInfo) -> Uint128 {
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbonding {
        asset: asset.to_string().clone(),
    })
    .test_query(&contract, chain)
    .unwrap())
    {
        adapter::QueryAnswer::Unbonding { amount } => amount,
        _ => panic!("Unbondable query failed"),
    }
}

pub fn claimable(chain: &App, asset: Addr, contract: &ContractInfo) -> Uint128 {
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Claimable {
        asset: asset.to_string().clone(),
    })
    .test_query(&contract, chain)
    .unwrap())
    {
        adapter::QueryAnswer::Claimable { amount } => amount,
        _ => panic!("Claimable query failed"),
    }
}

pub fn balance(chain: &App, asset: Addr, contract: &ContractInfo) -> Uint128 {
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: asset.to_string().clone(),
    })
    .test_query(&contract, chain)
    .unwrap())
    {
        adapter::QueryAnswer::Balance { amount } => amount,
        _ => panic!("Balance query failed"),
    }
}

pub fn reserves(chain: &App, asset: Addr, contract: &ContractInfo) -> Uint128 {
    match (adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
        asset: asset.to_string().clone(),
    })
    .test_query(&contract, chain)
    .unwrap())
    {
        adapter::QueryAnswer::Reserves { amount } => amount,
        _ => panic!("Reserves query failed"),
    }
}
