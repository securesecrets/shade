use shade_multi_test::interfaces::{
    dao::init_dao,
    snip20,
    treasury_manager,
    utils::{DeployedContracts, SupportedContracts},
};
use shade_protocol::{
    c_std::Uint128,
    contract_interfaces::dao::{treasury::AllowanceType, treasury_manager::AllocationType},
    multi_test::App,
    utils::cycle::Cycle,
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
    assert!(
        !treasury_manager::allocate_exec(
            &mut app,
            "admin",
            &contracts,
            "SSCRT",
            None,
            &SupportedContracts::MockAdapter(0),
            AllocationType::Amount,
            Uint128::new(1),
            Uint128::new(10u128.pow(18u32)),
            0,
        )
        .is_ok()
    );
    assert!(
        !treasury_manager::allocate_exec(
            &mut app,
            "admin",
            &contracts,
            "SSCRT",
            None,
            &SupportedContracts::MockAdapter(0),
            AllocationType::Portion,
            Uint128::new(10u128.pow(18u32)),
            Uint128::new(1),
            0,
        )
        .is_ok()
    );
    snip20::init(&mut app, "admin", &mut contracts, "Shade", "SHD", 8, None).unwrap();
    assert!(
        !treasury_manager::claim_exec(
            &mut app,
            "admin",
            &contracts,
            "SHD",
            SupportedContracts::TreasuryManager(0)
        )
        .is_ok()
    );
    treasury_manager::register_holder_exec(
        &mut app,
        "admin",
        &contracts,
        SupportedContracts::TreasuryManager(0),
        "holder",
    )
    .unwrap();
    assert!(
        !treasury_manager::unbond_exec(
            &mut app,
            "holder",
            &contracts,
            "SSCRT",
            SupportedContracts::TreasuryManager(0),
            Uint128::new(1)
        )
        .is_ok()
    );
    snip20::send_exec(
        &mut app,
        "admin",
        &contracts,
        "SSCRT",
        "holder".to_string(),
        Uint128::new(2),
        None,
    )
    .unwrap();
    snip20::send_exec(
        &mut app,
        "holder",
        &contracts,
        "SSCRT",
        contracts[&SupportedContracts::TreasuryManager(0)]
            .address
            .clone()
            .into(),
        Uint128::new(1),
        None,
    )
    .unwrap();
    assert!(
        !treasury_manager::unbond_exec(
            &mut app,
            "holder",
            &contracts,
            "SSCRT",
            SupportedContracts::TreasuryManager(0),
            Uint128::new(2)
        )
        .is_ok()
    );
    treasury_manager::register_asset_exec(
        &mut app,
        "admin",
        &contracts,
        "SHD",
        SupportedContracts::TreasuryManager(0),
    )
    .unwrap();
    assert!(
        !treasury_manager::register_holder_exec(
            &mut app,
            "admin",
            &contracts,
            SupportedContracts::TreasuryManager(0),
            "holder",
        )
        .is_ok()
    );
    assert!(
        !treasury_manager::remove_holder_exec(
            &mut app,
            "admin",
            &contracts,
            SupportedContracts::TreasuryManager(0),
            "not_a_holdler"
        )
        .is_ok()
    );
}
