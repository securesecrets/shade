use crate::{
    query,
    tests::{
        check_balances,
        init_contracts,
        query::{query_no_opps, query_opp_parameters},
        set_prices,
        set_viewing_key,
    },
};

use shade_protocol::{
    c_std::HumanAddr,
    contract_interfaces::{
        bonds,
        query_auth,
        snip20::{self, helpers::Snip20Asset},
    },
    fadroma::{
        core::ContractLink,
        ensemble::{ContractEnsemble, MockEnv},
    },
    math_compat::Uint128,
    secret_toolkit::snip20::{TokenConfig, TokenInfo},
    utils::asset::Contract,
};

use shade_protocol::c_std::StdError;

use super::{
    increase_allowance,
    query::{query_acccount_parameters, query_bonds_balance, query_config},
    setup_admin,
};

#[test]
pub fn test_bonds() {
    let (mut chain, bonds, issu, depo, atom, band, _oracle, query_auth, shade_admins) =
        init_contracts(false, Uint128::new(10_000)).unwrap();

    set_prices(
        &mut chain,
        &band,
        Uint128::new(10_000_000_000_000_000_000),
        Uint128::new(5_000_000_000_000_000_000),
        Uint128::new(20_000_000_000_000_000_000),
    )
    .unwrap();

    setup_admin(&mut chain, &shade_admins, &bonds);

    increase_allowance(&mut chain, &bonds, &issu);

    // No bond, so fail
    // buy_opp_fail(&mut chain, &bonds, &depo);

    open_opp(
        &mut chain,
        &bonds,
        &depo,
        "admin",
        Some(100),
        Some(Uint128::new(10_000_000_000)),
        Some(0),
        Some(Uint128::new(1000)),
        Uint128::new(10_000_000_000_000_000_000_000_000),
        Uint128::new(10_000_000_000_000_000_000_000_000),
        false,
    );

    buy_opp(&mut chain, &bonds, &depo, Uint128::new(2_000_000_000));

    query_acccount_parameters(
        &mut chain,
        &bonds.clone(),
        &query_auth.clone(),
        "secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq",
        None,
        None,
        Some(Uint128::new(2_000_000_000)),
        None,
        None,
        None,
        None,
        None,
    );

    query_opp_parameters(
        &mut chain,
        &bonds,
        None,
        Some(Uint128::new(1000000000)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    update_config(
        &mut chain,
        &bonds,
        "admin",
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(Uint128::new(9_000_000_000_000_000_000)),
        None,
        None,
        None,
        None,
    );

    buy_opp(&mut chain, &bonds, &depo, Uint128::new(2_000_000_000));

    query_opp_parameters(
        &mut chain,
        &bonds,
        None,
        Some(Uint128::new(2010101010)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    set_viewing_key(&mut chain, &query_auth);

    claim(&mut chain, &bonds);

    check_balances(
        &mut chain,
        &issu,
        &depo,
        Uint128::new(2010101010),
        Uint128::new(4_000_000_000),
    )
    .unwrap();

    close_opp(&mut chain, &bonds, &depo, "admin");

    query_no_opps(&mut chain, &bonds);

    open_opp(
        &mut chain,
        &bonds,
        &depo,
        "admin",
        None,
        None,
        None,
        None,
        Uint128::new(1),
        Uint128::new(1),
        false,
    );
    open_opp_fail(
        &mut chain,
        &bonds,
        &depo,
        "secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq",
        None,
        None,
        None,
        None,
        Uint128::new(1),
        Uint128::new(1),
        false,
        "21", // Not an admin, can't start opp
    );
    open_opp_fail(
        &mut chain,
        &bonds,
        &depo,
        "admin",
        None,
        None,
        None,
        Some(Uint128::new(10000000000000000000)),
        Uint128::new(1),
        Uint128::new(1),
        false,
        "12", // Discount percentage is too high
    );
    open_opp(
        &mut chain,
        &bonds,
        &depo,
        "admin",
        None,
        None,
        None,
        Some(Uint128::new(4_347)),
        Uint128::new(1_000_000_000_000_000_000),
        Uint128::new(950_000_000_000_000_000),
        false,
    );

    set_prices(
        &mut chain,
        &band,
        Uint128::new(7_500_000_000_000_000_000),
        Uint128::new(980_000_000_000_000_000),
        Uint128::new(20_000_000_000_000_000_000),
    )
    .unwrap();

    buy_opp(&mut chain, &bonds, &depo, Uint128::new(5));
    open_opp(
        &mut chain,
        &bonds,
        &depo,
        "admin",
        Some(200),
        None,
        None,
        Some(Uint128::new(4_347)),
        Uint128::new(1_000_000_000_000_000_000),
        Uint128::new(3_000_000_000_000_000_000),
        false,
    );
    buy_opp(&mut chain, &bonds, &depo, Uint128::new(500_000_000)); // 5 units
    // 4.9/9 for amount purchased, due to config issu_limit of $9 and current depo price of $.98
    query_opp_parameters(
        &mut chain,
        &bonds,
        None,
        Some(Uint128::new(54444444)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    set_prices(
        &mut chain,
        &band,
        Uint128::new(4_000_000_000_000_000_000),
        Uint128::new(980_000_000_000_000_000),
        Uint128::new(20_000_000_000_000_000_000),
    )
    .unwrap();

    buy_opp_fail(&mut chain, &bonds, &depo, "16");

    set_prices(
        &mut chain,
        &band,
        Uint128::new(6_000_000_000_000_000_000),
        Uint128::new(4_000_000_000_000_000_000),
        Uint128::new(20_000_000_000_000_000_000),
    )
    .unwrap();

    buy_opp_fail(&mut chain, &bonds, &depo, "15");

    set_prices(
        &mut chain,
        &band,
        Uint128::new(6_000_000_000_000_000_000),
        Uint128::new(2_000_000_000_000_000_000),
        Uint128::new(20_000_000_000_000_000_000),
    )
    .unwrap();

    buy_opp(&mut chain, &bonds, &depo, Uint128::new(1_000_000_000));

    query_opp_parameters(
        &mut chain,
        &bonds,
        None,
        Some(Uint128::new(165_555_555)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    open_opp_fail(
        &mut chain,
        &bonds,
        &atom,
        "admin",
        None,
        Some(Uint128::new(1000000000000000000)),
        None,
        None,
        Uint128::new(1),
        Uint128::new(1),
        false,
        "10", // Bond limit + previous limits exceeds global limit, so error
    );
    open_opp(
        &mut chain,
        &bonds,
        &atom,
        "admin",
        None,
        Some(Uint128::new(1000000000050)),
        None,
        None,
        Uint128::new(1),
        Uint128::new(1),
        false,
    );
    open_opp(
        &mut chain,
        &bonds,
        &depo,
        "admin",
        None,
        None,
        None,
        Some(Uint128::new(4_347)),
        Uint128::new(1_000_000_000_000_000_000),
        Uint128::new(950_000_000_000_000_000),
        false,
    );
    close_opp(&mut chain, &bonds, &depo, "admin");
    query_opp_parameters(
        &mut chain,
        &bonds,
        Some(Uint128::new(1000000000050)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
}

#[test]
fn buy_no_opp() -> () {
    let (mut chain, bonds, issu, depo, atom, band, _oracle, query_auth, shade_admins) =
        init_contracts(false, Uint128::new(10_000)).unwrap();

    set_prices(
        &mut chain,
        &band,
        Uint128::new(10_000_000_000_000_000_000),
        Uint128::new(5_000_000_000_000_000_000),
        Uint128::new(20_000_000_000_000_000_000),
    )
    .unwrap();

    setup_admin(&mut chain, &shade_admins, &bonds);

    increase_allowance(&mut chain, &bonds, &issu);

    // No bond, so fail. Error code 6 is "No Bond Found"
    buy_opp_fail(&mut chain, &bonds, &depo, "6");
}

#[test]
fn contract_inactive() -> () {
    let (mut chain, bonds, issu, depo, atom, band, _oracle, query_auth, shade_admins) =
        init_contracts(false, Uint128::new(10_000)).unwrap();

    set_prices(
        &mut chain,
        &band,
        Uint128::new(10_000_000_000_000_000_000),
        Uint128::new(5_000_000_000_000_000_000),
        Uint128::new(20_000_000_000_000_000_000),
    )
    .unwrap();

    setup_admin(&mut chain, &shade_admins, &bonds);

    increase_allowance(&mut chain, &bonds, &issu);

    update_config(
        &mut chain,
        &bonds,
        "admin",
        None,
        None,
        None,
        Some(false),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    // Contract not active, error out with code 5
    open_opp_fail(
        &mut chain,
        &bonds,
        &depo,
        "admin",
        Some(100),
        Some(Uint128::new(10_000_000_000)),
        Some(0),
        Some(Uint128::new(1000)),
        Uint128::new(10_000_000_000_000_000_000_000_000),
        Uint128::new(10_000_000_000_000_000_000_000_000),
        false,
        "5",
    );
}

fn claim(chain: &mut ContractEnsemble, bonds: &ContractLink<HumanAddr>) -> () {
    let msg = bonds::HandleMsg::Claim { padding: None };

    chain
        .execute(
            &msg,
            MockEnv::new(
                "secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq",
                bonds.clone(),
            ),
        )
        .unwrap();
}

fn buy_opp(
    chain: &mut ContractEnsemble,
    bonds: &ContractLink<HumanAddr>,
    depo: &ContractLink<HumanAddr>,
    amount: Uint128,
) -> () {
    let msg = snip20::HandleMsg::Send {
        recipient: bonds.address.clone(),
        recipient_code_hash: Some(bonds.code_hash.clone()),
        amount,
        msg: None,
        memo: None,
        padding: None,
    };

    chain
        .execute(
            &msg,
            MockEnv::new(
                "secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq",
                depo.clone(),
            ),
        )
        .unwrap();
}

fn buy_opp_fail(
    chain: &mut ContractEnsemble,
    bonds: &ContractLink<HumanAddr>,
    depo: &ContractLink<HumanAddr>,
    code: &str,
) -> () {
    let msg = snip20::HandleMsg::Send {
        recipient: bonds.address.clone(),
        recipient_code_hash: Some(bonds.code_hash.clone()),
        amount: Uint128::new(2_000_000_000), //20
        msg: None,
        memo: None,
        padding: None,
    };

    match chain.execute(
        &msg,
        MockEnv::new(
            "secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq",
            depo.clone(),
        ),
    ) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            StdError::GenericErr { msg, backtrace: _ } => {
                let mut str = String::from("code\":{},");
                str = str.replace("{}", code);
                if msg.contains(&str) {
                    assert!(true)
                } else {
                    println!("{}", msg);
                    assert!(false)
                }
            }
            _ => assert!(false),
        },
    }
}

fn open_opp(
    chain: &mut ContractEnsemble,
    bonds: &ContractLink<HumanAddr>,
    depo: &ContractLink<HumanAddr>,
    sender: &str,
    time_till_opp_end: Option<u64>,
    bond_issuance_limit: Option<Uint128>,
    bonding_period: Option<u64>,
    discount: Option<Uint128>,
    max_accepted_deposit_price: Uint128,
    err_deposit_price: Uint128,
    minting_bond: bool,
) -> () {
    let mut add: u64 = 50;
    if time_till_opp_end.is_some() {
        add = time_till_opp_end.unwrap();
    }

    let msg = bonds::HandleMsg::OpenBond {
        deposit_asset: Contract {
            address: depo.address.clone(),
            code_hash: depo.code_hash.clone(),
        },
        start_time: chain.block().time,
        end_time: (chain.block().time + add),
        bond_issuance_limit,
        bonding_period,
        discount,
        max_accepted_deposit_price,
        err_deposit_price,
        minting_bond,
        padding: None,
    };

    chain
        .execute(&msg, MockEnv::new(sender, bonds.clone()))
        .unwrap();
}

fn open_opp_fail(
    chain: &mut ContractEnsemble,
    bonds: &ContractLink<HumanAddr>,
    depo: &ContractLink<HumanAddr>,
    sender: &str,
    time_till_opp_end: Option<u64>,
    bond_issuance_limit: Option<Uint128>,
    bonding_period: Option<u64>,
    discount: Option<Uint128>,
    max_accepted_deposit_price: Uint128,
    err_deposit_price: Uint128,
    minting_bond: bool,
    code: &str,
) -> () {
    let mut add: u64 = 0;
    if time_till_opp_end.is_some() {
        add = time_till_opp_end.unwrap();
    }

    let msg = bonds::HandleMsg::OpenBond {
        deposit_asset: Contract {
            address: depo.address.clone(),
            code_hash: depo.code_hash.clone(),
        },
        start_time: chain.block().time,
        end_time: (chain.block().time + add),
        bond_issuance_limit,
        bonding_period,
        discount,
        max_accepted_deposit_price,
        err_deposit_price,
        minting_bond,
        padding: None,
    };

    match chain.execute(&msg, MockEnv::new(sender, bonds.clone())) {
        Ok(_) => assert!(false),
        Err(e) => match e {
            StdError::GenericErr { msg, backtrace: _ } => {
                let mut str = String::from("code\":{},");
                str = str.replace("{}", code);
                if msg.contains(&str) {
                    assert!(true)
                } else {
                    println!("{}", msg);
                    assert!(false)
                }
            }
            _ => assert!(false),
        },
    }
}

fn close_opp(
    chain: &mut ContractEnsemble,
    bonds: &ContractLink<HumanAddr>,
    depo: &ContractLink<HumanAddr>,
    sender: &str,
) -> () {
    let msg = bonds::HandleMsg::CloseBond {
        deposit_asset: Contract {
            address: depo.address.clone(),
            code_hash: depo.code_hash.clone(),
        },
        padding: None,
    };

    chain
        .execute(&msg, MockEnv::new(sender, bonds.clone()))
        .unwrap();
}

fn update_config(
    chain: &mut ContractEnsemble,
    bonds: &ContractLink<HumanAddr>,
    sender: &str,
    oracle: Option<Contract>,
    treasury: Option<HumanAddr>,
    issued_asset: Option<Contract>,
    activated: Option<bool>,
    bond_issuance_limit: Option<Uint128>,
    bonding_period: Option<u64>,
    discount: Option<Uint128>,
    global_min_accepted_issued_price: Option<Uint128>,
    global_err_issued_price: Option<Uint128>,
    allowance_key: Option<String>,
    airdrop: Option<Contract>,
    query_auth: Option<Contract>,
) -> () {
    let msg = bonds::HandleMsg::UpdateConfig {
        oracle,
        treasury,
        issued_asset,
        activated,
        bond_issuance_limit,
        bonding_period,
        discount,
        global_min_accepted_issued_price,
        global_err_issued_price,
        allowance_key,
        airdrop,
        query_auth,
        padding: None,
    };

    chain
        .execute(&msg, MockEnv::new(sender, bonds.clone()))
        .unwrap();
}

#[test]
pub fn test_shd_shd_bond_mimic() {
    let (mut chain, bonds, issu, depo, _atom, band, _oracle, query_auth, shade_admins) =
        init_contracts(true, Uint128::new(5000)).unwrap();

    update_config(
        &mut chain,
        &bonds,
        "admin",
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(Uint128::new(8_000_000_000_000_000_000)),
        Some(Uint128::new(6_450_000_000_000_000_000)),
        None,
        None,
        None,
    );

    set_prices(
        &mut chain,
        &band,
        Uint128::new(9_213_000_000_000_000_000),
        Uint128::new(5_000_000_000_000_000_000),
        Uint128::new(20_000_000_000_000_000_000),
    )
    .unwrap();

    setup_admin(&mut chain, &shade_admins, &bonds);

    increase_allowance(&mut chain, &bonds, &issu);

    open_opp(
        &mut chain,
        &bonds,
        &issu,
        "admin",
        Some(100),
        Some(Uint128::new(10_000_000_000)),
        Some(0),
        Some(Uint128::new(192)),
        Uint128::new(100_000_000_000_000_000_000),
        Uint128::new(100_000_000_000_000_000_000),
        false,
    );

    // No opp, so fail
    buy_opp_fail(&mut chain, &bonds, &depo, "6");

    // Buy opp successfully, hopefully
    buy_opp(&mut chain, &bonds, &issu, Uint128::new(2_000_000_000));

    query_bonds_balance(&mut chain, &bonds, Uint128::new(2_003_847_386));

    query_opp_parameters(
        &mut chain,
        &bonds,
        None,
        Some(Uint128::new(2_003_847_386)),
        Some(Snip20Asset {
            contract: Contract {
                address: issu.address.clone(),
                code_hash: issu.code_hash.clone(),
            },
            token_info: TokenInfo {
                name: "Issued".to_string(),
                symbol: "ISSU".to_string(),
                decimals: 8,
                total_supply: None,
            },
            token_config: Some(TokenConfig {
                public_total_supply: false,
                deposit_enabled: false,
                redeem_enabled: false,
                mint_enabled: false,
                burn_enabled: false,
            }),
        }),
        None,
        None,
        None,
        Some(Uint128::new(192)),
        None,
        None,
        None,
    );

    query_acccount_parameters(
        &mut chain,
        &bonds,
        &query_auth,
        "secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq",
        Some(Snip20Asset {
            contract: Contract {
                address: issu.address.clone(),
                code_hash: issu.code_hash.clone(),
            },
            token_info: TokenInfo {
                name: "Issued".to_string(),
                symbol: "ISSU".to_string(),
                decimals: 8,
                total_supply: None,
            },
            token_config: Some(TokenConfig {
                public_total_supply: false,
                deposit_enabled: false,
                redeem_enabled: false,
                mint_enabled: false,
                burn_enabled: false,
            }),
        }),
        None,
        Some(Uint128::new(2_000_000_000)),
        Some(Uint128::new(9_213_000_000_000_000_000)),
        Some(Uint128::new(2_003_847_386)),
        Some(Uint128::new(9_213_000_000_000_000_000)),
        Some(Uint128::new(192)),
        Some(Uint128::new(9_195_311_040_000_000_000)),
    );

    query_bonds_balance(&mut chain, &bonds, Uint128::new(2_003_847_386));
}
