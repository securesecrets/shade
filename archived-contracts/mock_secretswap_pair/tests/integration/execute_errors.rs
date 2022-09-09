use shade_multi_test::interfaces::{
    dao::{
        init_dao,
        mock_adapter_complete_unbonding,
        mock_adapter_sub_tokens,
        system_balance_reserves,
        system_balance_unbondable,
        update_dao,
    },
    snip20,
    treasury,
    treasury_manager,
    utils::{DeployedContracts, SupportedContracts},
};
use shade_protocol::{
    c_std::{Addr, Uint128},
    contract_interfaces::{
        self,
        dao::{
            self,
            treasury::AllowanceType,
            treasury_manager::{AllocationType, Balance, Holding, Status},
        },
    },
    multi_test::App,
    utils::{
        asset::{Contract, RawContract},
        cycle::Cycle,
    },
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
    );
    match treasury::allowance_exec(
        &mut app,
        "admin",
        &contracts,
        "SSCRT".to_string(),
        0,
        AllowanceType::Portion,
        Cycle::Constant,
        Uint128::new(1),
        Uint128::new(10u128.pow(18u32)),
    ) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
    match treasury::allowance_exec(
        &mut app,
        "admin",
        &contracts,
        "SSCRT".to_string(),
        0,
        AllowanceType::Portion,
        Cycle::Constant,
        Uint128::new(101 * 10u128.pow(16u32)),
        Uint128::zero(),
    ) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
    snip20::init(
        &mut app,
        "admin",
        &mut contracts,
        "Shade".to_string(),
        "SHD".to_string(),
        8,
        None,
    );
    match treasury::allowance_exec(
        &mut app,
        "admin",
        &contracts,
        "SHD".to_string(),
        0,
        AllowanceType::Portion,
        Cycle::Constant,
        Uint128::new(1),
        Uint128::zero(),
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
    match treasury::update_exec(&mut app, "admin", &contracts, "SHD".to_string()) {
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
    match treasury::update_exec(&mut app, "admin", &contracts, "SSCRT".to_string()) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
    treasury::register_asset_exec(&mut app, "admin", &contracts, "SHD".to_string()).unwrap();
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
