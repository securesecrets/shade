use shade_multi_test::interfaces::{dao::init_dao, treasury, utils::DeployedContracts};
use shade_protocol::{
    c_std::{Addr, Uint128},
    contract_interfaces::dao::{self, treasury::AllowanceType, treasury_manager::AllocationType},
    multi_test::App,
    utils::{
        asset::{Contract, RawContract},
        cycle::Cycle,
    },
};

#[test]
pub fn update_config() {
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
    treasury::set_config(
        &mut app,
        "admin",
        &contracts,
        Some(RawContract {
            address: "rando2".to_string(),
            code_hash: "rando3".to_string(),
        }),
        Some(Addr::unchecked("rando").into()),
    )
    .unwrap();
    assert_eq!(
        treasury::config_query(&app, &contracts).unwrap(),
        dao::treasury::Config {
            admin_auth: Contract {
                address: Addr::unchecked("rando2"),
                code_hash: "rando3".to_string(),
            },
            multisig: Addr::unchecked("rando"),
        }
    );
}
