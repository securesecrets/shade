use shade_multi_test::interfaces::{
    dao::{init_dao, system_balance_reserves, update_dao},
    snip20,
    treasury,
    utils::{DeployedContracts, SupportedContracts},
};
use shade_protocol::{
    c_std::{Addr, Uint128},
    contract_interfaces::dao::{treasury::AllowanceType, treasury_manager::AllocationType},
    multi_test::App,
    utils::{asset::Contract, cycle::Cycle},
};

#[test]
pub fn non_manager_allowances() {
    let mut app = App::default();
    let mut contracts = DeployedContracts::new();
    const NOT_A_MANAGER: &str = "no_manager";
    init_dao(
        &mut app,
        "admin",
        &mut contracts,
        Uint128::new(1500),
        "SSCRT",
        vec![
            AllowanceType::Amount,
            AllowanceType::Portion,
            AllowanceType::Amount,
            AllowanceType::Portion,
        ],
        vec![Cycle::Constant; 4],
        vec![
            Uint128::new(200),                // Amount - 50
            Uint128::new(6 * 10u128.pow(17)), // Poriton - 60%
            Uint128::new(300),                // Amount - 100
            Uint128::new(3 * 10u128.pow(17)), // Portion - 40%
        ], // Allowance amount
        vec![Uint128::zero(); 4],
        vec![
            vec![
                AllocationType::Portion,
                AllocationType::Amount,
                AllocationType::Portion,
                AllocationType::Amount
            ];
            4
        ],
        vec![
            vec![
                Uint128::new(6 * 10u128.pow(17)),
                Uint128::new(50),
                Uint128::new(2 * 10u128.pow(17)),
                Uint128::new(75),
            ];
            4
        ],
        vec![vec![Uint128::zero(); 4]; 4],
        true,
        true,
    )
    .unwrap();
    contracts.insert(SupportedContracts::TreasuryManager(5), Contract {
        address: Addr::unchecked(NOT_A_MANAGER),
        code_hash: "".to_string(),
    });
    treasury::allowance_exec(
        &mut app,
        "admin",
        &contracts,
        "SSCRT",
        5,
        AllowanceType::Amount,
        Cycle::Once,
        Uint128::new(100),
        Uint128::zero(),
        true,
    )
    .unwrap();
    update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    snip20::send_from_exec(
        &mut app,
        NOT_A_MANAGER,
        &contracts,
        "SSCRT",
        contracts[&SupportedContracts::Treasury]
            .clone()
            .address
            .into(),
        NOT_A_MANAGER.to_string(),
        Uint128::new(100),
        None,
    )
    .unwrap();
    update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    assert_eq!(
        Uint128::zero(),
        treasury::allowance_query(
            &app,
            &contracts,
            "SSCRT",
            SupportedContracts::TreasuryManager(5)
        )
        .unwrap()
    );
    match snip20::send_from_exec(
        &mut app,
        NOT_A_MANAGER,
        &contracts,
        "SSCRT",
        contracts[&SupportedContracts::Treasury]
            .clone()
            .address
            .into(),
        NOT_A_MANAGER.to_string(),
        Uint128::new(100),
        None,
    ) {
        Ok(_) => assert!(false, "cycle is set to once"),
        Err(_) => assert!(true),
    }
    println!("{:?}", system_balance_reserves(&app, &contracts, "SSCRT"),);
    snip20::set_viewing_key_exec(
        &mut app,
        NOT_A_MANAGER,
        &contracts,
        "SSCRT",
        NOT_A_MANAGER.to_string(),
    )
    .unwrap();
    assert_eq!(
        snip20::balance_query(
            &app,
            NOT_A_MANAGER,
            &contracts,
            "SSCRT",
            NOT_A_MANAGER.to_string()
        )
        .unwrap(),
        Uint128::new(100)
    );
    treasury::allowance_exec(
        &mut app,
        "admin",
        &contracts,
        "SSCRT",
        5,
        AllowanceType::Amount,
        Cycle::Constant,
        Uint128::new(50),
        Uint128::zero(),
        true,
    )
    .unwrap();
    update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    snip20::send_from_exec(
        &mut app,
        NOT_A_MANAGER,
        &contracts,
        "SSCRT",
        contracts[&SupportedContracts::Treasury]
            .clone()
            .address
            .into(),
        NOT_A_MANAGER.to_string(),
        Uint128::new(25),
        None,
    )
    .unwrap();
    assert_eq!(
        snip20::balance_query(
            &app,
            NOT_A_MANAGER,
            &contracts,
            "SSCRT",
            NOT_A_MANAGER.to_string()
        )
        .unwrap(),
        Uint128::new(125)
    );
    update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    assert_eq!(
        Uint128::new(50),
        treasury::allowance_query(
            &app,
            &contracts,
            "SSCRT",
            SupportedContracts::TreasuryManager(5)
        )
        .unwrap()
    );
}
