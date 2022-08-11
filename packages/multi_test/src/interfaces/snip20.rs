use super::super::multi::snip20::Snip20;
use shade_protocol::{
    c_std::{Addr, Binary, Uint128},
    contract_interfaces::snip20,
    multi_test::App,
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable},
};

pub fn init(
    chain: &mut App,
    sender: &str,
    name: String,
    symbol: String,
    decimals: u8,
    config: Option<snip20::InitConfig>,
) -> Contract {
    Contract::from(
        snip20::InstantiateMsg {
            name,
            admin: Some(sender.into()),
            symbol,
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
    )
}

pub fn send(
    chain: &mut App,
    snip20: Contract,
    recipient: String,
    amount: Uint128,
    sender: &str,
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
    .test_exec(&snip20.into(), chain, Addr::unchecked(sender), &[])
    .unwrap();
}
