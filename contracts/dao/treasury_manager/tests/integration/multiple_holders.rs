use shade_multi_test::interfaces::{
    dao::{
        init_dao,
        mock_adapter_complete_unbonding,
        system_balance_reserves,
        system_balance_unbondable,
        update_dao,
    },
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

pub fn multiple_holders(
    is_instant_unbond: bool,
    after_holder_adds_tokens: (Uint128, Vec<(Uint128, Vec<Uint128>)>),
    after_holder_removed: (Uint128, Vec<(Uint128, Vec<Uint128>)>),
) {
    let num_managers = 4;
    const HOLDER: &str = "holder";
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
    let bals = {
        if is_instant_unbond {
            system_balance_reserves(&app, &contracts, "SSCRT")
        } else {
            system_balance_unbondable(&app, &contracts, "SSCRT")
        }
    };
    assert_eq!(bals, after_holder_removed);
    snip20::set_viewing_key_exec(&mut app, HOLDER, &contracts, "SSCRT", HOLDER.to_string())
        .unwrap();
    snip20::send_exec(
        &mut app,
        "admin",
        &contracts,
        "SSCRT",
        HOLDER.to_string(),
        Uint128::new(1000),
        None,
    )
    .unwrap();
    treasury_manager::register_holder_exec(
        &mut app,
        "admin",
        &contracts,
        SupportedContracts::TreasuryManager(0),
        HOLDER,
    )
    .unwrap();
    snip20::send_exec(
        &mut app,
        HOLDER,
        &contracts,
        "SSCRT",
        contracts[&SupportedContracts::TreasuryManager(0)]
            .address
            .to_string(),
        Uint128::new(200),
        None,
    )
    .unwrap();
    snip20::send_exec(
        &mut app,
        HOLDER,
        &contracts,
        "SSCRT",
        contracts[&SupportedContracts::TreasuryManager(0)]
            .address
            .to_string(),
        Uint128::new(300),
        None,
    )
    .unwrap();
    assert_eq!(
        Uint128::new(500),
        treasury_manager::holding_query(
            &app,
            &contracts,
            SupportedContracts::TreasuryManager(0),
            HOLDER.to_string(),
        )
        .unwrap()
        .balances[0]
            .amount
    );
    update_dao(&mut app, "admin", &contracts, "SSCRT", num_managers).unwrap();
    let bals = {
        if is_instant_unbond {
            system_balance_reserves(&app, &contracts, "SSCRT")
        } else {
            system_balance_unbondable(&app, &contracts, "SSCRT")
        }
    };
    assert_eq!(bals, after_holder_adds_tokens);
    treasury_manager::unbond_exec(
        &mut app,
        HOLDER,
        &contracts,
        "SSCRT",
        SupportedContracts::TreasuryManager(0),
        Uint128::new(300),
    )
    .unwrap();
    if !is_instant_unbond {
        update_dao(&mut app, "admin", &contracts, "SSCRT", num_managers).unwrap();
        let mut k = 0;
        for _i in 0..num_managers {
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
    }
    update_dao(&mut app, "admin", &contracts, "SSCRT", num_managers).unwrap();
    treasury_manager::claim_exec(
        &mut app,
        HOLDER,
        &contracts,
        "SSCRT",
        SupportedContracts::TreasuryManager(0),
    )
    .unwrap();
    match treasury_manager::remove_holder_exec(
        &mut app,
        "rando",
        &contracts,
        SupportedContracts::TreasuryManager(0),
        HOLDER.clone(),
    ) {
        Ok(_) => assert!(false, "unauthorized removing of HOLDER"),
        Err(_) => assert!(true),
    }
    treasury_manager::remove_holder_exec(
        &mut app,
        "admin",
        &contracts,
        SupportedContracts::TreasuryManager(0),
        HOLDER.clone(),
    )
    .unwrap();
    match treasury_manager::remove_holder_exec(
        &mut app,
        "admin",
        &contracts,
        SupportedContracts::TreasuryManager(0),
        &contracts[&SupportedContracts::Treasury].address.to_string(),
    ) {
        Ok(_) => assert!(false, "removed treasury as a HOLDER"),
        Err(_) => assert!(true),
    }
    match snip20::send_exec(
        &mut app,
        HOLDER,
        &contracts,
        "SSCRT",
        contracts[&SupportedContracts::TreasuryManager(0)]
            .address
            .to_string(),
        Uint128::new(300),
        None,
    ) {
        Ok(_) => assert!(false, "closed HOLDERs shouldn't be able to send to TM"),
        Err(_) => assert!(true),
    }
    treasury_manager::unbond_exec(
        &mut app,
        HOLDER,
        &contracts,
        "SSCRT",
        SupportedContracts::TreasuryManager(0),
        Uint128::zero(),
    )
    .unwrap();
    if !is_instant_unbond {
        let mut k = 0;
        for _i in 0..num_managers {
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
    }
    treasury_manager::claim_exec(
        &mut app,
        HOLDER,
        &contracts,
        "SSCRT",
        SupportedContracts::TreasuryManager(0),
    )
    .unwrap();
    update_dao(&mut app, "admin", &contracts, "SSCRT", num_managers).unwrap();
    if !is_instant_unbond {
        let mut k = 0;
        for _i in 0..num_managers {
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
        treasury_manager::claim_exec(
            &mut app,
            HOLDER,
            &contracts,
            "SSCRT",
            SupportedContracts::TreasuryManager(0),
        )
        .unwrap();
    }
    update_dao(&mut app, "admin", &contracts, "SSCRT", num_managers).unwrap();
    match treasury_manager::holding_query(
        &app,
        &contracts,
        SupportedContracts::TreasuryManager(0),
        HOLDER.to_string(),
    ) {
        Ok(_) => assert!(false, "HOLDER was not removed"),
        Err(_) => assert!(true),
    }
    let bals = {
        if is_instant_unbond {
            system_balance_reserves(&app, &contracts, "SSCRT")
        } else {
            system_balance_unbondable(&app, &contracts, "SSCRT")
        }
    };
    assert_eq!(bals, after_holder_removed);
}

#[test]
pub fn mul_holders() {
    multiple_holders(
        true,
        (Uint128::new(280), vec![
            (Uint128::new(100), vec![
                Uint128::new(345),
                Uint128::new(50),
                Uint128::new(115),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(285),
                Uint128::new(50),
                Uint128::new(95),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(105),
                Uint128::new(50),
                Uint128::new(35),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(105),
                Uint128::new(50),
                Uint128::new(35),
                Uint128::new(75),
            ]),
        ]),
        (Uint128::new(280), vec![
            (Uint128::new(0), vec![
                Uint128::new(45),
                Uint128::new(50),
                Uint128::new(15),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(285),
                Uint128::new(50),
                Uint128::new(95),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(105),
                Uint128::new(50),
                Uint128::new(35),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(105),
                Uint128::new(50),
                Uint128::new(35),
                Uint128::new(75),
            ]),
        ]),
    );
}

#[test]
pub fn mul_holders_unbond() {
    multiple_holders(
        false,
        (Uint128::new(280), vec![
            (Uint128::new(100), vec![
                Uint128::new(345),
                Uint128::new(50),
                Uint128::new(115),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(285),
                Uint128::new(50),
                Uint128::new(95),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(105),
                Uint128::new(50),
                Uint128::new(35),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(105),
                Uint128::new(50),
                Uint128::new(35),
                Uint128::new(75),
            ]),
        ]),
        (Uint128::new(280), vec![
            (Uint128::new(0), vec![
                Uint128::new(45),
                Uint128::new(50),
                Uint128::new(15),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(285),
                Uint128::new(50),
                Uint128::new(95),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(105),
                Uint128::new(50),
                Uint128::new(35),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(105),
                Uint128::new(50),
                Uint128::new(35),
                Uint128::new(75),
            ]),
        ]),
    );
}
