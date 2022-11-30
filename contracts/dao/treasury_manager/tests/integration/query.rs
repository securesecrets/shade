use shade_multi_test::interfaces::{
    dao::{init_dao, mock_adapter_sub_tokens, update_dao},
    snip20,
    treasury_manager,
    utils::{DeployedContracts, SupportedContracts},
};
use shade_protocol::{
    c_std::{BlockInfo, Timestamp, Uint128},
    contract_interfaces::dao::{treasury::AllowanceType, treasury_manager::AllocationType},
    multi_test::App,
    utils::{
        cycle::{parse_utc_datetime, Cycle},
        storage::plus::period_storage::Period,
    },
};

#[test]
pub fn query() {
    let mut app = App::default();
    let mut contracts = DeployedContracts::new();
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(
            parse_utc_datetime(&"1995-11-13T00:00:00.00Z".to_string())
                .unwrap()
                .timestamp() as u64,
        ),
        chain_id: "chain_id".to_string(),
    });
    init_dao(
        &mut app,
        "admin",
        &mut contracts,
        Uint128::new(1500),
        "SSCRT",
        vec![AllowanceType::Amount],
        vec![Cycle::Constant],
        vec![
            Uint128::new(1500), // Amount - 50
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
    assert_eq!(
        treasury_manager::batch_balance_query(
            &app,
            &contracts,
            vec!["SSCRT"],
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury
        )
        .unwrap()[0]
            + treasury_manager::pending_allowance_query(
                &app,
                &contracts,
                SupportedContracts::TreasuryManager(0),
                "SSCRT"
            )
            .unwrap(),
        Uint128::new(1500)
    );
    snip20::init(&mut app, "admin", &mut contracts, "Shade", "SHD", 8, None).unwrap();
    assert!(
        !treasury_manager::pending_allowance_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            "SHD"
        )
        .is_ok()
    );
    assert!(
        !treasury_manager::assets_query(&app, &contracts, SupportedContracts::TreasuryManager(0),)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        treasury_manager::allocations_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            "SHD"
        )
        .unwrap(),
        vec![]
    );
    assert!(
        !treasury_manager::allocations_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            "SSCRT"
        )
        .unwrap()
        .is_empty(),
    );
    assert!(
        !treasury_manager::holders_query(&app, &contracts, SupportedContracts::TreasuryManager(0),)
            .unwrap()
            .is_empty(),
    );
    assert_eq!(
        treasury_manager::batch_balance_query(
            &app,
            &contracts,
            vec!["SSCRT", "SHD"],
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury
        )
        .unwrap(),
        vec![Uint128::new(1225), Uint128::zero()]
    );
    assert!(
        !treasury_manager::batch_balance_query(
            &app,
            &contracts,
            vec!["SSCRT", "SHD"],
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::AdminAuth
        )
        .is_ok()
    );
    assert!(
        !treasury_manager::balance_query(
            &app,
            &contracts,
            "SHD",
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury
        )
        .is_ok()
    );
    assert!(
        !treasury_manager::balance_query(
            &app,
            &contracts,
            "SSCRT",
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::AdminAuth
        )
        .is_ok()
    );
    assert!(
        !treasury_manager::unbonding_query(
            &app,
            &contracts,
            "SHD",
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury
        )
        .is_ok()
    );
    assert!(
        !treasury_manager::unbonding_query(
            &app,
            &contracts,
            "SSCRT",
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::AdminAuth
        )
        .is_ok()
    );
    assert!(
        !treasury_manager::unbondable_query(
            &app,
            &contracts,
            "SHD",
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury
        )
        .is_ok()
    );
    assert!(
        !treasury_manager::unbondable_query(
            &app,
            &contracts,
            "SSCRT",
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::AdminAuth
        )
        .is_ok()
    );
    assert!(
        !treasury_manager::reserves_query(
            &app,
            &contracts,
            "SHD",
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury
        )
        .is_ok()
    );
    assert!(
        !treasury_manager::claimable_query(
            &app,
            &contracts,
            "SHD",
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury
        )
        .is_ok()
    );
    assert!(
        !treasury_manager::claimable_query(
            &app,
            &contracts,
            "SSCRT",
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::AdminAuth
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
        !treasury_manager::metrics_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            Some("1995-11-13T00:00:00.00Z".to_string()),
            None,
            Period::Hour,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury_manager::metrics_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            Some("1995-11-13T00:00:00.00Z".to_string()),
            None,
            Period::Day,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury_manager::metrics_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            Some("1995-11-13T00:00:00.00Z".to_string()),
            None,
            Period::Month,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury_manager::metrics_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            None,
            Some(Uint128::new(816220800)),
            Period::Hour,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury_manager::metrics_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            None,
            Some(Uint128::new(816220800)),
            Period::Day,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury_manager::metrics_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            None,
            Some(Uint128::new(816220800)),
            Period::Month,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury_manager::metrics_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            None,
            None,
            Period::Month,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury_manager::metrics_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            Some("1995-11-13T00:00:00.00Z".to_string()),
            Some(Uint128::new(816220800)),
            Period::Month,
        )
        .is_ok()
    );
    assert!(
        treasury_manager::metrics_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            None,
            Some(Uint128::new(
                parse_utc_datetime(&"1995-12-13T00:00:00.00Z".to_string())
                    .unwrap()
                    .timestamp() as u128
            )),
            Period::Month,
        )
        .unwrap()
        .is_empty()
    );
    mock_adapter_sub_tokens(
        &mut app,
        "admin",
        &contracts,
        Uint128::new(10),
        SupportedContracts::MockAdapter(3),
    )
    .unwrap();
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(
            parse_utc_datetime(&"1995-12-13T00:00:00.00Z".to_string())
                .unwrap()
                .timestamp() as u64,
        ),
        chain_id: "chain_id".to_string(),
    });
    update_dao(&mut app, "admin", &contracts, "SSCRT", 1).unwrap();
    update_dao(&mut app, "admin", &contracts, "SSCRT", 1).unwrap();
    assert!(
        !treasury_manager::metrics_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            None,
            Some(Uint128::new(
                parse_utc_datetime(&"1995-12-13T00:00:00.00Z".to_string())
                    .unwrap()
                    .timestamp() as u128
            )),
            Period::Month,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury_manager::metrics_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            Some("1995-12-13T00:00:00.00Z".to_string()),
            None,
            Period::Month,
        )
        .unwrap()
        .is_empty()
    );
}
