use shade_multi_test::interfaces::{
    dao::init_dao,
    snip20,
    treasury,
    utils::{DeployedContracts, SupportedContracts},
};
use shade_protocol::{
    c_std::Uint128,
    contract_interfaces::dao::{self, treasury::AllowanceType, treasury_manager::AllocationType},
    multi_test::App,
    utils::{asset::RawContract, cycle::Cycle},
};

#[test]
pub fn execute_error() {
    let mut app = App::default();
    let mut contracts = DeployedContracts::new();
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
    match treasury::allowance_exec(
        &mut app,
        "admin",
        &contracts,
        "SSCRT",
        0,
        AllowanceType::Portion,
        Cycle::Constant,
        Uint128::new(1),
        Uint128::new(10u128.pow(18u32)),
        true,
    ) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
    match treasury::allowance_exec(
        &mut app,
        "admin",
        &contracts,
        "SSCRT",
        0,
        AllowanceType::Portion,
        Cycle::Constant,
        Uint128::new(101 * 10u128.pow(16u32)),
        Uint128::zero(),
        true,
    ) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
    snip20::init(&mut app, "admin", &mut contracts, "Shade", "SHD", 8, None).unwrap();
    match treasury::allowance_exec(
        &mut app,
        "admin",
        &contracts,
        "SHD",
        0,
        AllowanceType::Portion,
        Cycle::Constant,
        Uint128::new(1),
        Uint128::zero(),
        true,
    ) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
    match treasury::register_manager_exec(&mut app, "admin", &contracts, 0) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
    match treasury::register_wrap_exec(
        &mut app,
        "admin",
        &contracts,
        "SHD".to_string(),
        RawContract {
            address: "rando".to_string(),
            code_hash: "code_hash".to_string(),
        },
    ) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
    match treasury::update_exec(&mut app, "admin", &contracts, "SHD") {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
    treasury::set_run_level_exec(
        &mut app,
        "admin",
        &contracts,
        dao::treasury::RunLevel::Deactivated,
    )
    .unwrap();
    match treasury::update_exec(&mut app, "admin", &contracts, "SSCRT") {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
    treasury::register_asset_exec(&mut app, "admin", &contracts, "SHD").unwrap();
    match treasury::register_wrap_exec(
        &mut app,
        "admin",
        &contracts,
        "SHD".to_string(),
        contracts[&SupportedContracts::Snip20("SHD".to_string())]
            .clone()
            .into(),
    ) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
}

#[test]
pub fn admin_errors() {
    const NOT_ADMIN: &str = "not_admin";
    let mut app = App::default();
    let mut contracts = DeployedContracts::new();
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
    assert!(!treasury::set_config(&mut app, NOT_ADMIN, &contracts, None, None).is_ok());
    assert!(
        !treasury::set_run_level_exec(
            &mut app,
            NOT_ADMIN,
            &contracts,
            dao::treasury::RunLevel::Migrating
        )
        .is_ok()
    );
    assert!(!treasury::register_asset_exec(&mut app, NOT_ADMIN, &contracts, "SSCRT").is_ok());
    assert!(
        !treasury::register_wrap_exec(
            &mut app,
            NOT_ADMIN,
            &contracts,
            "SSCRT".to_string(),
            RawContract {
                address: "nana".to_string(),
                code_hash: "nana".to_string()
            }
        )
        .is_ok()
    );
    assert!(!treasury::register_manager_exec(&mut app, NOT_ADMIN, &contracts, 0).is_ok());
    assert!(
        !treasury::allowance_exec(
            &mut app,
            NOT_ADMIN,
            &contracts,
            "SSCRT",
            0,
            AllowanceType::Amount,
            Cycle::Daily {
                days: Uint128::new(1)
            },
            Uint128::zero(),
            Uint128::zero(),
            true,
        )
        .is_ok()
    );
}
