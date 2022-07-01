pub mod handle;
pub mod query;

use contract_harness::harness::snip20::Snip20;
use cosmwasm_std::{Binary, HumanAddr, StdResult};
use fadroma::ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv};
use fadroma::core::ContractLink;
use shade_protocol::contract_interfaces::{
    snip20,
    snip20::{InitConfig, InitialBalance},
};

//TODO: test rng

pub fn init_snip20(msg: snip20::InitMsg) -> StdResult<(ContractEnsemble, ContractLink<HumanAddr>)> {
    let mut chain = ContractEnsemble::new(50);

    // Register governance
    let gov = chain.register(Box::new(Snip20));
    let gov = chain.instantiate(
        gov.id,
        &msg,
        MockEnv::new("admin", ContractLink {
            address: "snip20".into(),
            code_hash: gov.code_hash,
        }),
    )?.instance;

    Ok((chain, gov))
}

pub fn init_snip20_with_config(
    initial_balances: Option<Vec<InitialBalance>>,
    config: Option<InitConfig>,
) -> StdResult<(ContractEnsemble, ContractLink<HumanAddr>)> {
    let (mut chain, snip) = init_snip20(snip20::InitMsg {
        name: "Token".to_string(),
        admin: None,
        symbol: "TKN".to_string(),
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
    chain: &mut ContractEnsemble,
    snip: &ContractLink<HumanAddr>,
    addr: &str,
    key: Option<String>,
) -> StdResult<()> {
    chain.execute(
        &snip20::HandleMsg::SetViewingKey {
            key: key.unwrap_or("password".to_string()),
            padding: None,
        },
        MockEnv::new(addr, snip.clone()),
    )?;
    Ok(())
}
