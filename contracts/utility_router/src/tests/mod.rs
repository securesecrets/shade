pub mod integration;

use shade_multi_test::multi::{
    admin::Admin,
    query_auth::QueryAuth,
    snip20::Snip20,
    utility_router::UtilityRouter,
};
use shade_protocol::{
    admin::{self, helpers::AdminPermissions},
    c_std::{Addr, Binary, ContractInfo, Response, StdResult},
    contract_interfaces::utility_router::*,
    multi_test::{App, AppResponse, Executor, Router},
    query_auth,
    snip20,
    utility_router,
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
    AnyResult,
};

pub fn init_contract() -> AnyResult<(App, ContractInfo, ContractInfo)> {
    let mut chain = App::default();

    let admin = admin::InstantiateMsg {
        super_admin: Some("admin".into()),
    }
    .test_init(
        Admin::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "admin_auth",
        &[],
    )
    .unwrap();

    let router = utility_router::InstantiateMsg {
        admin_auth: Contract {
            address: admin.address.clone(),
            code_hash: admin.code_hash.clone(),
        },
    }
    .test_init(
        UtilityRouter::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "utility_router",
        &[],
    )
    .unwrap();

    Ok((chain, router, admin))
}

pub fn init_query_auth(chain: &mut App, admin: &ContractInfo) -> AnyResult<ContractInfo> {
    let query_auth = query_auth::InstantiateMsg {
        admin_auth: Contract {
            address: admin.address.clone(),
            code_hash: admin.code_hash.clone(),
        },
        prng_seed: Binary::default(),
    }
    .test_init(
        QueryAuth::default(),
        chain,
        Addr::unchecked("admin"),
        "query_auth",
        &[],
    )
    .unwrap();

    Ok(query_auth)
}

pub fn init_snip20(chain: &mut App) -> AnyResult<ContractInfo> {
    let snip20 = snip20::InstantiateMsg {
        name: "Issued".into(),
        admin: Some("admin".to_string()),
        symbol: "ISSU".into(),
        decimals: 8,
        initial_balances: None,
        prng_seed: Default::default(),
        config: None,
        query_auth: None,
    }
    .test_init(
        Snip20::default(),
        chain,
        Addr::unchecked("admin".to_string()),
        "snip20",
        &[],
    )
    .unwrap();

    Ok(snip20)
}

pub fn get_contract(chain: &App, router: &ContractInfo, key: UtilityKey) -> AnyResult<Contract> {
    match (utility_router::QueryMsg::GetContract {
        key: key.to_string(),
    }
    .test_query(router, chain))
    {
        Ok(resp) => match resp {
            utility_router::QueryAnswer::GetContract { contract } => Ok(contract),
            _ => panic!("get_contract error"),
        },
        Err(e) => Err(e.into()),
    }
}

pub fn get_contracts(
    chain: &App,
    router: &ContractInfo,
    keys: Vec<UtilityKey>,
) -> AnyResult<Vec<Contract>> {
    match (utility_router::QueryMsg::GetContracts {
        keys: keys.iter().map(|k| k.to_string()).collect(),
    }
    .test_query(router, chain))
    {
        Ok(resp) => match resp {
            utility_router::QueryAnswer::GetContracts { contracts } => Ok(contracts),
            _ => panic!("get_contract error"),
        },
        Err(e) => Err(e.into()),
    }
}

pub fn get_address(chain: &App, router: &ContractInfo, key: UtilityKey) -> AnyResult<Addr> {
    match (utility_router::QueryMsg::GetAddress {
        key: key.to_string(),
    }
    .test_query(router, chain))
    {
        Ok(resp) => match resp {
            utility_router::QueryAnswer::GetAddress { address } => Ok(address),
            _ => panic!("get_address error"),
        },
        Err(e) => Err(e.into()),
    }
}

pub fn get_addresses(
    chain: &App,
    router: &ContractInfo,
    keys: Vec<UtilityKey>,
) -> AnyResult<Vec<Addr>> {
    match (utility_router::QueryMsg::GetAddresses {
        keys: keys.iter().map(|k| k.to_string()).collect(),
    }
    .test_query(router, chain))
    {
        Ok(resp) => match resp {
            utility_router::QueryAnswer::GetAddresses { addresses } => Ok(addresses),
            _ => panic!("get_address error"),
        },
        Err(e) => Err(e.into()),
    }
}

pub fn get_keys(
    chain: &App,
    router: &ContractInfo,
    start: usize,
    limit: usize,
) -> AnyResult<Vec<String>> {
    match (utility_router::QueryMsg::GetKeys { start, limit }.test_query(router, chain)) {
        Ok(resp) => match resp {
            utility_router::QueryAnswer::GetKeys { keys } => Ok(keys),
            _ => panic!("get_address error"),
        },
        Err(e) => Err(e.into()),
    }
}

pub fn get_status(chain: &App, router: &ContractInfo) -> AnyResult<RouterStatus> {
    match (utility_router::QueryMsg::Status {}
        .test_query(router, chain)
        .unwrap())
    {
        utility_router::QueryAnswer::Status { contract_status } => Ok(contract_status),
        _ => panic!("get_status error"),
    }
}

pub fn set_contract(
    chain: &mut App,
    router: &ContractInfo,
    key: UtilityKey,
    contract: Contract,
    sender: Addr,
) -> AnyResult<AppResponse> {
    utility_router::ExecuteMsg::SetContract {
        key: key.to_string(),
        contract: contract.into(),
    }
    .test_exec(router, chain, sender, &[])
}

pub fn set_address(
    chain: &mut App,
    router: &ContractInfo,
    key: UtilityKey,
    address: Addr,
    sender: Addr,
) -> AnyResult<AppResponse> {
    utility_router::ExecuteMsg::SetAddress {
        key: key.to_string(),
        address: address.to_string(),
    }
    .test_exec(router, chain, sender, &[])
}

pub fn set_status(
    chain: &mut App,
    router: &ContractInfo,
    status: RouterStatus,
    sender: Addr,
) -> AnyResult<AppResponse> {
    utility_router::ExecuteMsg::SetStatus { status }.test_exec(router, chain, sender, &[])
}
