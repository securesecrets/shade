use crate::{
    interfaces::utils::{DeployedContracts, SupportedContracts},
    multi::snip20::Snip20,
};
use shade_protocol::{
    c_std::{Addr, Binary, Coin, StdError, StdResult, Uint128},
    contract_interfaces::snip20,
    multi_test::App,
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

pub fn init(
    chain: &mut App,
    sender: &str,
    contracts: &mut DeployedContracts,
    name: &str,
    snip20_symbol: &str,
    decimals: u8,
    config: Option<snip20::InitConfig>,
) -> StdResult<()> {
    let snip20 = Contract::from(
        match (snip20::InstantiateMsg {
            name: name.to_string(),
            admin: Some(sender.into()),
            symbol: snip20_symbol.to_string(),
            decimals,
            initial_balances: Some(vec![snip20::InitialBalance {
                address: sender.into(),
                amount: Uint128::from(1_000_000_000 * 10 ^ decimals as u128),
            }]),
            prng_seed: Binary::default(),
            query_auth: None,
            config,
        }
        .test_init(
            Snip20::default(),
            chain,
            Addr::unchecked(sender),
            "snip20",
            &[],
        )) {
            Ok(contract_info) => contract_info,
            Err(e) => return Err(StdError::generic_err(e.to_string())),
        },
    );
    contracts.insert(
        SupportedContracts::Snip20(snip20_symbol.to_string()),
        snip20,
    );
    Ok(())
}

pub fn deposit_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    coins: &Vec<Coin>,
) -> StdResult<()> {
    match (snip20::ExecuteMsg::Deposit { padding: None }.test_exec(
        &contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        coins,
    )) {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn set_viewing_key_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    key: String,
) -> StdResult<()> {
    match (snip20::ExecuteMsg::SetViewingKey { key, padding: None }.test_exec(
        &contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
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

pub fn send_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    recipient: String,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<()> {
    match (snip20::ExecuteMsg::Send {
        recipient,
        amount,
        msg,
        memo: None,
        padding: None,
        recipient_code_hash: None,
    }
    .test_exec(
        &contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err("snip20 send failed")),
    }
}

pub fn send_from_exec(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    owner: String,
    recipient: String,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<()> {
    match (snip20::ExecuteMsg::SendFrom {
        owner,
        recipient,
        amount,
        msg,
        memo: None,
        padding: None,
        recipient_code_hash: None,
    }
    .test_exec(
        &contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err("snip20 send failed")),
    }
}

pub fn balance_query(
    chain: &App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: &str,
    key: String,
) -> StdResult<Uint128> {
    let res = snip20::QueryMsg::Balance {
        address: sender.to_string(),
        key,
    }
    .test_query(
        &contracts
            .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
            .unwrap()
            .clone()
            .into(),
        chain,
    )?;
    match res {
        snip20::QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err("SetViewingKey failed")),
    }
}
