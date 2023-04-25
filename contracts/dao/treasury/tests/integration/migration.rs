use shade_multi_test::interfaces::{
    dao::{
        init_dao,
        mock_adapter_complete_unbonding,
        system_balance_reserves,
        system_balance_unbondable,
        update_dao,
    },
    snip20,
    treasury,
    utils::{DeployedContracts, SupportedContracts},
};
use shade_protocol::{
    c_std::{Addr, Uint128},
    contract_interfaces::dao::{self, treasury::AllowanceType, treasury_manager::AllocationType},
    multi_test::App,
    utils::cycle::Cycle,
};

pub fn migration_test(is_instant_unbond: bool) {
    const MULTISIG: &str = "multisig";
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
        is_instant_unbond,
        true,
    )
    .unwrap();
    snip20::set_viewing_key_exec(
        &mut app,
        MULTISIG,
        &contracts,
        "SSCRT",
        MULTISIG.to_string(),
    )
    .unwrap();
    treasury::set_config(
        &mut app,
        "admin",
        &contracts,
        Some(
            contracts
                .get(&SupportedContracts::AdminAuth)
                .unwrap()
                .clone()
                .into(),
        ),
        Some(Addr::unchecked(MULTISIG).to_string()),
    )
    .unwrap();
    treasury::set_run_level_exec(
        &mut app,
        "admin",
        &contracts,
        dao::treasury::RunLevel::Migrating,
    )
    .unwrap();
    update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    if is_instant_unbond {
        update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    } else {
        let mut k = 0;
        for _i in 0..4 {
            for _j in 0..4 {
                mock_adapter_complete_unbonding(
                    &mut app,
                    "admin",
                    &contracts,
                    SupportedContracts::MockAdapter(k),
                )
                .unwrap();
                k += 1;
            }
            k += 1;
        }
        update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
        update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    }
    println!(
        "{:?}\n{:?}",
        system_balance_reserves(&app, &contracts, "SSCRT"),
        system_balance_unbondable(&app, &contracts, "SSCRT")
    );
    assert_eq!(
        snip20::balance_query(&app, MULTISIG, &contracts, "SSCRT", MULTISIG.to_string()).unwrap(),
        Uint128::new(1500)
    );
}

macro_rules! dao_tests_migration {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    is_instant_unbond,
                ) = $value;
                migration_test(
                    is_instant_unbond,
                );
            }
        )*
    }
}

dao_tests_migration! (
    dao_test_migration_instant_unbond: (
        true,
    ),
    dao_test_migration_non_instant_unbond: (
        false,
    ),
);
