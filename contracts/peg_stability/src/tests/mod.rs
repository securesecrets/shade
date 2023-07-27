pub mod handle;
pub mod query;

use shade_multi_test::multi::{admin::Admin, peg_stability::PegStability, snip20::Snip20};
use shade_protocol::{
    admin,
    c_std::{Addr, Binary, ContractInfo, Decimal, Uint128},
    contract_interfaces::peg_stability,
    multi_test::{App, Executor},
    snip20,
    utils::{asset::Contract, MultiTestable},
};

pub fn init_chain() -> (App, ContractInfo) {
    let mut chain = App::default();

    let stored_code = chain.store_code(Admin::default().contract());
    let admin = chain
        .instantiate_contract(
            stored_code,
            Addr::unchecked("admin"),
            &admin::InstantiateMsg { super_admin: None },
            &[],
            "admin",
            None,
        )
        .unwrap();

    //Instantiate band
    //Instantiate oracle
    (chain, admin)
}

pub fn ps_no_oracle(
    chain: App,
    shd_admin: ContractInfo,
    snip20: ContractInfo,
) -> (App, ContractInfo) {
    let mut chain = chain;

    let stored_code = chain.store_code(PegStability::default().contract());
    let init_msg = peg_stability::InstantiateMsg {
        admin_auth: Contract {
            address: shd_admin.address,
            code_hash: shd_admin.code_hash,
        },
        snip20: Contract {
            address: snip20.address,
            code_hash: snip20.code_hash,
        },
        oracle: Contract::default(),
        treasury: Contract {
            address: Addr::unchecked("admin"),
            code_hash: "".into(),
        },
        payback: Decimal::percent(15),
        viewing_key: "SecureSoftware".into(),
        dump_contract: Contract::default(),
    };
    let pstable = chain
        .instantiate_contract(
            stored_code,
            Addr::unchecked("admin"),
            &init_msg,
            &[],
            "admin",
            None,
        )
        .unwrap();

    (chain, pstable)
}

pub fn init_snip20(
    chain: App,
    name: String,
    symbol: String,
    decimals: u128,
) -> (App, ContractInfo) {
    let mut chain = chain;

    let stored_code = chain.store_code(Snip20::default().contract());
    let init_msg = snip20::InstantiateMsg {
        name,
        admin: Some("admin".into()),
        query_auth: None,
        symbol,
        decimals: decimals as u8,
        initial_balances: Some(vec![snip20::InitialBalance {
            address: "admin".into(),
            amount: Uint128::new(1_000_000_000_000_000 * 10 ^ decimals),
        }]),
        prng_seed: Binary::default(),
        config: None,
    };
    let snip20 = chain
        .instantiate_contract(
            stored_code,
            Addr::unchecked("admin"),
            &init_msg,
            &[],
            "admin",
            None,
        )
        .unwrap();

    (chain, snip20)
}

/*#[test]
pub fn test_test() {
    assert!(true);
}*/
