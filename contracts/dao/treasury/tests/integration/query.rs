use shade_multi_test::interfaces::{
    dao::{init_dao, mock_adapter_sub_tokens, update_dao},
    snip20,
    treasury,
    utils::{DeployedContracts, SupportedContracts},
};
use shade_protocol::{
    c_std::{BlockInfo, Timestamp, Uint128},
    contract_interfaces::dao::{self, treasury::AllowanceType, treasury_manager::AllocationType},
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
    assert_eq!(
        treasury::batch_balance_query(&app, &contracts, vec!["SSCRT"]).unwrap(),
        vec![Uint128::new(1500)]
    );
    snip20::init(&mut app, "admin", &mut contracts, "Shade", "SHD", 8, None).unwrap();
    assert!(!treasury::batch_balance_query(&app, &contracts, vec!["SSCRT", "SHD"]).is_ok());
    assert!(!treasury::balance_query(&app, &contracts, "SHD",).is_ok());
    assert!(!treasury::reserves_query(&app, &contracts, "SHD",).is_ok());
    assert!(
        !treasury::allowance_query(
            &app,
            &contracts,
            "SHD",
            SupportedContracts::TreasuryManager(0)
        )
        .is_ok()
    );
    treasury::register_asset_exec(&mut app, "admin", &contracts, "SHD").unwrap();
    assert_eq!(
        treasury::batch_balance_query(&app, &contracts, vec!["SSCRT", "SHD"]).unwrap(),
        vec![Uint128::new(1500), Uint128::zero()]
    );
    assert_eq!(
        treasury::run_level_query(&app, &contracts,).unwrap(),
        dao::treasury::RunLevel::Normal
    );
    assert!(
        !treasury::metrics_query(
            &app,
            &contracts,
            Some("1995-11-13T00:00:00.00Z".to_string()),
            None,
            Period::Hour,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury::metrics_query(
            &app,
            &contracts,
            Some("1995-11-13T00:00:00.00Z".to_string()),
            None,
            Period::Day,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury::metrics_query(
            &app,
            &contracts,
            Some("1995-11-13T00:00:00.00Z".to_string()),
            None,
            Period::Month,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury::metrics_query(
            &app,
            &contracts,
            None,
            Some(Uint128::new(816220800)),
            Period::Hour,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury::metrics_query(
            &app,
            &contracts,
            None,
            Some(Uint128::new(816220800)),
            Period::Day,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury::metrics_query(
            &app,
            &contracts,
            None,
            Some(Uint128::new(816220800)),
            Period::Month,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !treasury::metrics_query(&app, &contracts, None, None, Period::Month,)
            .unwrap()
            .is_empty()
    );
    assert!(
        !treasury::metrics_query(
            &app,
            &contracts,
            Some("1995-11-13T00:00:00.00Z".to_string()),
            Some(Uint128::new(816220800)),
            Period::Month,
        )
        .is_ok()
    );
    assert!(
        treasury::metrics_query(
            &app,
            &contracts,
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
        SupportedContracts::MockAdapter(7),
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
    update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    assert!(
        !treasury::metrics_query(
            &app,
            &contracts,
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
        !treasury::metrics_query(
            &app,
            &contracts,
            Some("1995-12-13T00:00:00.00Z".to_string()),
            None,
            Period::Month,
        )
        .unwrap()
        .is_empty()
    );
}
