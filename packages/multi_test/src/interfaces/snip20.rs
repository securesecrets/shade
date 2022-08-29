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
    name: String,
    symbol: String,
    decimals: u8,
    config: Option<snip20::InitConfig>,
) {
    let snip20 = Contract::from(
        snip20::InstantiateMsg {
            name,
            admin: Some(sender.into()),
            symbol: symbol.clone(),
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
        )
        .unwrap(),
    );
    contracts.insert(SupportedContracts::Snip20(symbol), snip20);
}

pub fn deposit(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    coins: &Vec<Coin>,
) {
    snip20::ExecuteMsg::Deposit { padding: None }.test_exec(
        &contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        coins,
    );
}

pub fn set_viewing_key(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    key: String,
) -> StdResult<()> {
    match (snip20::ExecuteMsg::SetViewingKey { key, padding: None }.test_exec(
        &contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err("SetViewingKey failed")),
    }
}

pub fn send(
    chain: &mut App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    recipient: String,
    amount: Uint128,
    msg: Option<Binary>,
) {
    snip20::ExecuteMsg::Send {
        recipient,
        amount,
        msg,
        memo: None,
        padding: None,
        recipient_code_hash: None,
    }
    .test_exec(
        &contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .clone()
            .into(),
        chain,
        Addr::unchecked(sender),
        &[],
    )
    .unwrap();
}

pub fn balance_query(
    chain: &App,
    sender: &str,
    contracts: &DeployedContracts,
    snip20_symbol: String,
    key: String,
) -> StdResult<Uint128> {
    match (snip20::QueryMsg::Balance {
        address: sender.to_string(),
        key,
    }
    .test_query(
        &contracts
            .get(&SupportedContracts::Snip20(snip20_symbol))
            .unwrap()
            .clone()
            .into(),
        chain,
    )?) {
        snip20::QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err("SetViewingKey failed")),
    }
}
