use shade_multi_test::interfaces::{
    dao::{
        init_dao,
        mock_adapter_complete_unbonding,
        mock_adapter_sub_tokens,
        system_balance_reserves,
        system_balance_unbondable,
    },
    snip20,
    treasury,
    treasury_manager,
    utils::{DeployedContracts, SupportedContracts},
};
use shade_protocol::{
    c_std::Uint128,
    contract_interfaces::dao::{treasury::AllowanceType, treasury_manager::AllocationType},
    multi_test::App,
    utils::cycle::Cycle,
};

pub fn dao_int_gains_losses(
    initial_treasury_bal: Uint128,
    allow_type: Vec<AllowanceType>,
    t_cycle: Vec<Cycle>,
    allow_amount: Vec<Uint128>,
    allow_tolerance: Vec<Uint128>,
    alloc_type: Vec<Vec<AllocationType>>,
    alloc_amount: Vec<Vec<Uint128>>,
    alloc_tolerance: Vec<Vec<Uint128>>,
    is_instant_unbond: bool,
    expected_after_init: (Uint128, Vec<(Uint128, Vec<Uint128>)>),
    snip20_send_amount: Uint128,
    adapters_to_send_to: Vec<usize>,
    is_adapters_gain: Vec<bool>,
    expected_in_between_updates: (Uint128, Vec<(Uint128, Vec<Uint128>)>),
    expected_after_updates: (Uint128, Vec<(Uint128, Vec<Uint128>)>),
) {
    let mut app = App::default();
    let mut contracts = DeployedContracts::new();
    let num_managers = allow_type.len();
    init_dao(
        &mut app,
        "admin",
        &mut contracts,
        initial_treasury_bal,
        "SSCRT",
        allow_type,
        t_cycle,
        allow_amount,
        allow_tolerance,
        alloc_type,
        alloc_amount.clone(),
        alloc_tolerance,
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
    assert_eq!(bals, expected_after_init, "AFTER INITIALIZATION");
    for (i, adap) in adapters_to_send_to.clone().iter().enumerate() {
        if is_adapters_gain[i] {
            snip20::send_exec(
                &mut app,
                "admin",
                &contracts,
                "SSCRT",
                contracts
                    .get(&SupportedContracts::MockAdapter(adap.clone()))
                    .unwrap()
                    .address
                    .to_string(),
                snip20_send_amount,
                None,
            )
            .unwrap();
        } else {
            mock_adapter_sub_tokens(
                &mut app,
                "admin",
                &contracts,
                snip20_send_amount,
                SupportedContracts::MockAdapter(adap.clone()),
            )
            .unwrap();
        }
    }
    // Needs 2 full cycles to reballance fully
    for tm in 0..num_managers {
        treasury_manager::update_exec(
            &mut app,
            "admin",
            &contracts,
            "SSCRT",
            SupportedContracts::TreasuryManager(tm),
        )
        .unwrap();
    }
    treasury::update_exec(&mut app, "admin", &contracts, "SSCRT").unwrap();
    let bals = {
        if is_instant_unbond {
            let sys_bal = system_balance_reserves(&app, &contracts, "SSCRT");
            assert_eq!(sys_bal, expected_in_between_updates, "AFTER FIRST UPDATE");
            for tm in 0..num_managers {
                treasury_manager::update_exec(
                    &mut app,
                    "admin",
                    &contracts,
                    "SSCRT",
                    SupportedContracts::TreasuryManager(tm),
                )
                .unwrap();
            }
            treasury::update_exec(&mut app, "admin", &contracts, "SSCRT").unwrap();
            for tm in 0..num_managers {
                treasury_manager::update_exec(
                    &mut app,
                    "admin",
                    &contracts,
                    "SSCRT",
                    SupportedContracts::TreasuryManager(tm),
                )
                .unwrap();
            }
            treasury::update_exec(&mut app, "admin", &contracts, "SSCRT").unwrap();
            system_balance_reserves(&app, &contracts, "SSCRT")
        } else {
            let _sys_bal = system_balance_unbondable(&app, &contracts, "SSCRT");
            //assert_eq!(sys_bal, expected_in_between_updates, "AFTER FIRST UPDATE");
            let mut k = 0;
            for i in 0..num_managers {
                for _j in 0..alloc_amount[i].len() {
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
            for tm in 0..num_managers {
                treasury_manager::update_exec(
                    &mut app,
                    "admin",
                    &contracts,
                    "SSCRT",
                    SupportedContracts::TreasuryManager(tm),
                )
                .unwrap();
            }
            treasury::update_exec(&mut app, "admin", &contracts, "SSCRT").unwrap();
            for tm in 0..num_managers {
                treasury_manager::update_exec(
                    &mut app,
                    "admin",
                    &contracts,
                    "SSCRT",
                    SupportedContracts::TreasuryManager(tm),
                )
                .unwrap();
            }
            treasury::update_exec(&mut app, "admin", &contracts, "SSCRT").unwrap();
            let mut k = 0;
            for i in 0..num_managers {
                for _j in 0..alloc_amount[i].len() {
                    println!("{}", k);
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
            for tm in 0..num_managers {
                treasury_manager::update_exec(
                    &mut app,
                    "admin",
                    &contracts,
                    "SSCRT",
                    SupportedContracts::TreasuryManager(tm),
                )
                .unwrap();
            }
            treasury::update_exec(&mut app, "admin", &contracts, "SSCRT").unwrap();
            for tm in 0..num_managers {
                treasury_manager::update_exec(
                    &mut app,
                    "admin",
                    &contracts,
                    "SSCRT",
                    SupportedContracts::TreasuryManager(tm),
                )
                .unwrap();
            }
            treasury::update_exec(&mut app, "admin", &contracts, "SSCRT").unwrap();
            //update_dao(&mut app, "admin", &contracts, "SSCRT", num_managers);
            system_balance_unbondable(&app, &contracts, "SSCRT")
        }
    };
    assert_eq!(bals, expected_after_updates, "AFTER BOTH UPDATES");
}

macro_rules! dao_tests_gains_losses {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    initial_treasury_bal,
                    allow_type,
                    t_cycle,
                    allow_amount,
                    allow_tolerance,
                    alloc_type,
                    alloc_amount,
                    alloc_tolerance,
                    is_instant_unbond,
                    expected_after_init,
                    snip20_send_amount,
                    adapters_to_send_to,
                    is_adapters_gain,
                    expected_in_between_updates,
                    expected_after_updates,
                ) = $value;
                dao_int_gains_losses(
                    initial_treasury_bal,
                    allow_type,
                    t_cycle,
                    allow_amount,
                    allow_tolerance,
                    alloc_type,
                    alloc_amount,
                    alloc_tolerance,
                    is_instant_unbond,
                    expected_after_init,
                    snip20_send_amount,
                    adapters_to_send_to,
                    is_adapters_gain,
                    expected_in_between_updates,
                    expected_after_updates,
                );
            }
        )*
    }
}

dao_tests_gains_losses! {
    dao_test_gains:(
        Uint128::new(1000),
        vec![AllowanceType::Portion],
        vec![Cycle::Constant],
        vec![
            Uint128::new(6 * 10u128.pow(17)), // Poriton - 60%
        ], // Allowance amount
        vec![Uint128::zero()],
        vec![vec![
            AllocationType::Portion,
            AllocationType::Amount,
            AllocationType::Portion,
            AllocationType::Amount,
        ]],
        vec![vec![
            Uint128::new(6 * 10u128.pow(17)),
            Uint128::new(5),
            Uint128::new(2 * 10u128.pow(17)),
            Uint128::new(15),
        ]],
        vec![vec![Uint128::zero(); 4]],
        true,
        (Uint128::new(516), vec![(Uint128::new(0), vec![
            Uint128::new(348),
            Uint128::new(5),
            Uint128::new(116),
            Uint128::new(15),
        ])]),
        Uint128::new(100),
        vec![0, 1, 2],
        vec![true, true, true],
        (Uint128::new(520), vec![(Uint128::new(56), vec![
            Uint128::new(528),
            Uint128::new(5),
            Uint128::new(176),
            Uint128::new(15),
        ])]),
        (Uint128::new(520), vec![(Uint128::new(152), vec![
            Uint128::new(456),
            Uint128::new(5),
            Uint128::new(152),
            Uint128::new(15),
        ])]),
    ),
    dao_test_gains_4_managers: (
        Uint128::new(1000),
        vec![
            AllowanceType::Amount,
            AllowanceType::Portion,
            AllowanceType::Amount,
            AllowanceType::Portion,
        ],
        vec![Cycle::Constant; 4],
        vec![
            Uint128::new(50),                 // Amount - 50
            Uint128::new(6 * 10u128.pow(17)), // Poriton - 60%
            Uint128::new(100),                // Amount - 100
            Uint128::new(2 * 10u128.pow(17)), // Portion - 40%
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
                Uint128::new(5),
                Uint128::new(2 * 10u128.pow(17)),
                Uint128::new(15)
            ];
            4
        ],
        vec![vec![Uint128::zero(); 4]; 4],
        true,
        (Uint128::new(320), vec![
            (Uint128::new(0), vec![
                Uint128::new(18),
                Uint128::new(5),
                Uint128::new(6),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(294),
                Uint128::new(5),
                Uint128::new(98),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(48),
                Uint128::new(5),
                Uint128::new(16),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(90),
                Uint128::new(5),
                Uint128::new(30),
                Uint128::new(15),
            ]),
        ]),
        Uint128::new(100),
        vec![0, 1, 2, 3, 5, 7, 10, 12, 16],
        vec![true; 11],
        (Uint128::new(528), vec![
            (Uint128::new(180), vec![
                Uint128::new(18),
                Uint128::new(5),
                Uint128::new(12),
                Uint128::new(15),
            ]),
            (Uint128::new(60), vec![
                Uint128::new(414),
                Uint128::new(5),
                Uint128::new(138),
                Uint128::new(15),
            ]),
            (Uint128::new(140), vec![
                Uint128::new(60),
                Uint128::new(5),
                Uint128::new(20),
                Uint128::new(15),
            ]),
            (Uint128::new(100), vec![
                Uint128::new(120),
                Uint128::new(5),
                Uint128::new(30),
                Uint128::new(15),
            ]),
        ]),
        (Uint128::new(622), vec![
            (Uint128::new(6), vec![
                Uint128::new(18),
                Uint128::new(5),
                Uint128::new(6),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(618),
                Uint128::new(5),
                Uint128::new(206),
                Uint128::new(15),
            ]),
            (Uint128::new(16), vec![
                Uint128::new(48),
                Uint128::new(5),
                Uint128::new(16),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(198),
                Uint128::new(5),
                Uint128::new(66),
                Uint128::new(15),
            ]),
        ]),
    ),
    dao_test_losses: (
        Uint128::new(1000),
        vec![
            AllowanceType::Amount,
            AllowanceType::Portion,
            AllowanceType::Amount,
            AllowanceType::Portion,
        ],
        vec![Cycle::Constant; 4],
        vec![
            Uint128::new(50),                 // Amount - 50
            Uint128::new(6 * 10u128.pow(17)), // Poriton - 60%
            Uint128::new(100),                // Amount - 100
            Uint128::new(2 * 10u128.pow(17)), // Portion - 40%
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
                Uint128::new(5),
                Uint128::new(2 * 10u128.pow(17)),
                Uint128::new(15)
            ];
            4
        ],
        vec![vec![Uint128::zero(); 4]; 4],
        true,
        (Uint128::new(320), vec![
            (Uint128::new(0), vec![
                Uint128::new(18),
                Uint128::new(5),
                Uint128::new(6),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(294),
                Uint128::new(5),
                Uint128::new(98),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(48),
                Uint128::new(5),
                Uint128::new(16),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(90),
                Uint128::new(5),
                Uint128::new(30),
                Uint128::new(15),
            ]),
        ]),
        Uint128::new(5),
        vec![0, 1, 2, 3, 5, 7, 10, 12, 16],
        vec![false; 9],
        (Uint128::new(303), vec![
            (Uint128::new(7), vec![
                Uint128::new(6),
                Uint128::new(5),
                Uint128::new(1),
                Uint128::new(11),
            ]),
            (Uint128::new(1), vec![
                Uint128::new(288),
                Uint128::new(5),
                Uint128::new(96),
                Uint128::new(15),
            ]),
            (Uint128::new(1), vec![
                Uint128::new(42),
                Uint128::new(5),
                Uint128::new(14),
                Uint128::new(15),
            ]),
            (Uint128::new(4), vec![
                Uint128::new(87),
                Uint128::new(5),
                Uint128::new(29),
                Uint128::new(15),
            ]),
        ]),
        (Uint128::new(282), vec![
            (Uint128::new(0), vec![
                Uint128::new(18),
                Uint128::new(5),
                Uint128::new(6),
                Uint128::new(15),
            ]),
            (Uint128::new(16), vec![
                Uint128::new(277),
                Uint128::new(5),
                Uint128::new(92),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(48),
                Uint128::new(5),
                Uint128::new(16),
                Uint128::new(15),
            ]),
            (Uint128::new(8), vec![
                Uint128::new(84),
                Uint128::new(5),
                Uint128::new(28),
                Uint128::new(15),
            ]),
        ]),
    ),
    dao_test_losses_and_gains: (
        Uint128::new(1500),
        vec![
            AllowanceType::Amount,
            AllowanceType::Portion,
            AllowanceType::Amount,
            AllowanceType::Portion,
        ],
        vec![Cycle::Constant; 4],
        vec![
            Uint128::new(200),                 // Amount - 50
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
                Uint128::new(75)
            ];
            4
        ],
        vec![vec![Uint128::zero(); 4]; 4],
        true,
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
        Uint128::new(50),
        vec![0, 1, 2, 3, 5, 7, 10, 12, 16],
        vec![true, false, true, false, false, true, true, true, false],
        (Uint128::new(200), vec![
            (Uint128::new(100), vec![
                Uint128::new(45),
                Uint128::new(15),
                Uint128::new(15),
                Uint128::new(25),
            ]),
            (Uint128::new(50), vec![
                Uint128::new(285),
                Uint128::new(50),
                Uint128::new(95),
                Uint128::new(75),
            ]),
            (Uint128::new(45), vec![
                Uint128::new(132),
                Uint128::new(50),
                Uint128::new(43),
                Uint128::new(75),
            ]),
            (Uint128::new(40), vec![
                Uint128::new(75),
                Uint128::new(35),
                Uint128::new(25),
                Uint128::new(75),
            ]),
        ]),
        (Uint128::new(218), vec![
            (Uint128::new(15), vec![
                Uint128::new(45),
                Uint128::new(50),
                Uint128::new(15),
                Uint128::new(75),
            ]),
            (Uint128::new(26), vec![
                Uint128::new(303),
                Uint128::new(50),
                Uint128::new(101),
                Uint128::new(75),
            ]),
            (Uint128::new(35), vec![
                Uint128::new(105),
                Uint128::new(50),
                Uint128::new(35),
                Uint128::new(75),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(114),
                Uint128::new(50),
                Uint128::new(38),
                Uint128::new(75),
            ]),
        ]),
    ),
    dao_test_gains_4_managers_with_unbond: (
        Uint128::new(1000),
        vec![
            AllowanceType::Amount,
            AllowanceType::Portion,
            AllowanceType::Amount,
            AllowanceType::Portion,
        ],
        vec![Cycle::Constant; 4],
        vec![
            Uint128::new(50),                 // Amount - 50
            Uint128::new(6 * 10u128.pow(17)), // Poriton - 60%
            Uint128::new(100),                // Amount - 100
            Uint128::new(2 * 10u128.pow(17)), // Portion - 40%
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
                Uint128::new(5),
                Uint128::new(2 * 10u128.pow(17)),
                Uint128::new(15)
            ];
            4
        ],
        vec![vec![Uint128::zero(); 4]; 4],
        false,
        (Uint128::new(320), vec![
            (Uint128::new(0), vec![
                Uint128::new(18),
                Uint128::new(5),
                Uint128::new(6),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(294),
                Uint128::new(5),
                Uint128::new(98),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(48),
                Uint128::new(5),
                Uint128::new(16),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(90),
                Uint128::new(5),
                Uint128::new(30),
                Uint128::new(15),
            ]),
        ]),
        Uint128::new(100),
        vec![0, 1, 2, 3, 5, 7, 10, 12, 16],
        vec![true; 11],
        (Uint128::new(528), vec![
            (Uint128::new(180), vec![
                Uint128::new(18),
                Uint128::new(5),
                Uint128::new(12),
                Uint128::new(15),
            ]),
            (Uint128::new(60), vec![
                Uint128::new(414),
                Uint128::new(5),
                Uint128::new(138),
                Uint128::new(15),
            ]),
            (Uint128::new(140), vec![
                Uint128::new(60),
                Uint128::new(5),
                Uint128::new(20),
                Uint128::new(15),
            ]),
            (Uint128::new(100), vec![
                Uint128::new(120),
                Uint128::new(5),
                Uint128::new(30),
                Uint128::new(15),
            ]),
        ]),
        (Uint128::new(622), vec![
            (Uint128::new(6), vec![
                Uint128::new(18),
                Uint128::new(5),
                Uint128::new(6),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(618),
                Uint128::new(5),
                Uint128::new(206),
                Uint128::new(15),
            ]),
            (Uint128::new(16), vec![
                Uint128::new(48),
                Uint128::new(5),
                Uint128::new(16),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(198),
                Uint128::new(5),
                Uint128::new(66),
                Uint128::new(15),
            ]),
        ]),
    ),
    dao_test_losses_with_unbond: (
        Uint128::new(1000),
        vec![
            AllowanceType::Amount,
            AllowanceType::Portion,
            AllowanceType::Amount,
            AllowanceType::Portion,
        ],
        vec![Cycle::Constant; 4],
        vec![
            Uint128::new(50),                 // Amount - 50
            Uint128::new(6 * 10u128.pow(17)), // Poriton - 60%
            Uint128::new(100),                // Amount - 100
            Uint128::new(2 * 10u128.pow(17)), // Portion - 40%
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
                Uint128::new(5),
                Uint128::new(2 * 10u128.pow(17)),
                Uint128::new(15)
            ];
            4
        ],
        vec![vec![Uint128::zero(); 4]; 4],
        false,
        (Uint128::new(320), vec![
            (Uint128::new(0), vec![
                Uint128::new(18),
                Uint128::new(5),
                Uint128::new(6),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(294),
                Uint128::new(5),
                Uint128::new(98),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(48),
                Uint128::new(5),
                Uint128::new(16),
                Uint128::new(15),
            ]),
            (Uint128::new(0), vec![
                Uint128::new(90),
                Uint128::new(5),
                Uint128::new(30),
                Uint128::new(15),
            ]),
        ]),
        Uint128::new(5),
        vec![0, 1, 2, 3, 5, 7, 10, 12, 16],
        vec![false; 9],
        (Uint128::new(303), vec![
            (Uint128::new(7), vec![
                Uint128::new(6),
                Uint128::new(5),
                Uint128::new(1),
                Uint128::new(11),
            ]),
            (Uint128::new(1), vec![
                Uint128::new(288),
                Uint128::new(5),
                Uint128::new(96),
                Uint128::new(15),
            ]),
            (Uint128::new(1), vec![
                Uint128::new(42),
                Uint128::new(5),
                Uint128::new(14),
                Uint128::new(15),
            ]),
            (Uint128::new(4), vec![
                Uint128::new(87),
                Uint128::new(5),
                Uint128::new(29),
                Uint128::new(15),
            ]),
        ]),
        (Uint128::new(275), vec![
            (Uint128::new(6), vec![
                Uint128::new(18),
                Uint128::new(5),
                Uint128::new(6),
                Uint128::new(15),
            ]),
            (Uint128::new(16), vec![
                Uint128::new(277),
                Uint128::new(5),
                Uint128::new(92),
                Uint128::new(15),
            ]),
            (Uint128::new(1), vec![
                Uint128::new(48),
                Uint128::new(5),
                Uint128::new(16),
                Uint128::new(15),
            ]),
            (Uint128::new(8), vec![
                Uint128::new(84),
                Uint128::new(5),
                Uint128::new(28),
                Uint128::new(15),
            ]),
        ]),
    ),
    dao_test_losses_and_gains_with_unbond: (
        Uint128::new(1500),
        vec![
            AllowanceType::Amount,
            AllowanceType::Portion,
            AllowanceType::Amount,
            AllowanceType::Portion,
        ],
        vec![Cycle::Constant; 4],
        vec![
            Uint128::new(200),                 // Amount - 50
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
                Uint128::new(75)
            ];
            4
        ],
        vec![vec![Uint128::zero(); 4]; 4],
        false,
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
        Uint128::new(50),
        vec![0, 1, 2, 3, 5, 7, 10, 12, 16],
        vec![true, false, true, false, false, true, true, true, false],
        (Uint128::new(200), vec![
            (Uint128::new(100), vec![
                Uint128::new(45),
                Uint128::new(15),
                Uint128::new(15),
                Uint128::new(25),
            ]),
            (Uint128::new(50), vec![
                Uint128::new(285),
                Uint128::new(50),
                Uint128::new(95),
                Uint128::new(75),
            ]),
            (Uint128::new(45), vec![
                Uint128::new(132),
                Uint128::new(50),
                Uint128::new(43),
                Uint128::new(75),
            ]),
            (Uint128::new(40), vec![
                Uint128::new(75),
                Uint128::new(35),
                Uint128::new(25),
                Uint128::new(75),
            ]),
        ]),
        (Uint128::new(156), vec![
            (Uint128::new(15), vec![
                Uint128::new(45),
                Uint128::new(50),
                Uint128::new(15),
                Uint128::new(75),
            ]),
            (Uint128::new(50), vec![
                Uint128::new(303),
                Uint128::new(50),
                Uint128::new(101),
                Uint128::new(75),
            ]),
            (Uint128::new(35), vec![
                Uint128::new(105),
                Uint128::new(50),
                Uint128::new(35),
                Uint128::new(75),
            ]),
            (Uint128::new(38), vec![
                Uint128::new(114),
                Uint128::new(50),
                Uint128::new(38),
                Uint128::new(75),
            ]),
        ]),
    ),
}
