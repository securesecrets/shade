pub mod handle;
pub mod query;

use shade_protocol::AnyResult;
use shade_protocol::utils::{ExecuteCallback, InstantiateCallback, Query, MultiTestable};
use shade_protocol::multi_test::{App, AppResponse};
use shade_multi_test::multi::snip20::Snip20;
use shade_protocol::c_std::{Binary, Addr, StdResult, ContractInfo};
use shade_protocol::contract_interfaces::{
    snip20,
    snip20::{InitConfig, InitialBalance},
};

//TODO: test rng

pub fn init_snip20(msg: snip20::InstantiateMsg) -> AnyResult<(App, ContractInfo)> {
    let mut app = App::default();
    let admin = Addr::unchecked("admin");
    // Register governance
    let contract = msg.test_init(Snip20::default(), &mut app, admin, "snip20", &[])?;

    Ok((app, contract))
}

pub fn init_snip20_with_config(
    initial_balances: Option<Vec<InitialBalance>>,
    config: Option<InitConfig>,
) -> AnyResult<(App, ContractInfo)> {
    let (mut chain, snip) = init_snip20(snip20::InstantiateMsg {
        name: "Token".into(),
        admin: None,
        symbol: "TKN".into(),
        decimals: 8,
        initial_balances: initial_balances.clone(),
        prng_seed: Binary::from("random".as_bytes()),
        config,
    })?;

    if let Some(balances) = initial_balances {
        for balance in balances.iter() {
            create_vk(&mut chain, &snip, balance.address.as_str(), None)?;
        }
    }

    Ok((chain, snip))
}

pub fn create_vk(
    chain: &mut App,
    snip: &ContractInfo,
    addr: &str,
    key: Option<String>,
) -> AnyResult<AppResponse> {
    snip20::ExecuteMsg::SetViewingKey {
        key: key.unwrap_or("password".into()),
        padding: None,
    }.test_exec(snip, chain, Addr::unchecked(addr), &[])
}
