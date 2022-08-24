use shade_multi_test::interfaces::{
    dao::{init_dao, mock_adapter_sub_tokens, system_balance, unbond_exec, update_dao},
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

pub fn dao_int_test(
    initial_treasury_bal: Uint128,
    snip20_symbol: &str,
    allow_amount: Vec<Uint128>,
    allow_type: Vec<AllowanceType>,
    cycle: Vec<Cycle>,
    allow_tolerance: Vec<Uint128>,
    expected_allowance: Vec<Uint128>,
    alloc_amount: Vec<Vec<Uint128>>,
    alloc_type: Vec<Vec<AllocationType>>,
    alloc_tolerance: Vec<Vec<Uint128>>,
    expected_treasury: Uint128,
    expected_manager: Vec<Uint128>,
    expected_adapter: Vec<Vec<Uint128>>,
) {
    let mut app = App::default();
    let mut contracts = DeployedContracts::new();
    let num_managers = allow_amount.len();
    init_dao(
        &mut app,
        "admin",
        &mut contracts,
        initial_treasury_bal,
        snip20_symbol.clone(),
        allow_type.clone(),
        cycle.clone(),
        allow_amount.clone(),
        allow_tolerance.clone(),
        alloc_type.clone(),
        alloc_amount.clone(),
        alloc_tolerance.clone(),
    );
    //query assets
    let assets_query_res = treasury::assets_query(&app, &contracts).unwrap();
    println!("{:?}", assets_query_res);
    assert!(
        assets_query_res.contains(
            &contracts
                .get(&SupportedContracts::Snip20(snip20_symbol.to_string()))
                .unwrap()
                .address
        )
    );
    //query allowance
    for i in 0..num_managers {
        assert_eq!(
            expected_allowance[i],
            treasury::allowance_query(
                &app,
                "admin",
                &contracts,
                snip20_symbol.to_string(),
                SupportedContracts::TreasuryManager(i)
            )
            .unwrap(),
            "Treasury->Manager Allowance",
        );
    }
    let mut bals = system_balance(&app, &contracts, snip20_symbol.to_string());
    println!("{:?}", bals);
    assert_eq!(bals.0, expected_treasury);
    for (i, manager_tuples) in bals.1.iter().enumerate() {
        assert_eq!(manager_tuples.0, expected_manager[i]);
        for (j, adapter_bals) in manager_tuples.1.iter().enumerate() {
            assert_eq!(adapter_bals.clone(), expected_adapter[i][j]);
        }
    }
    let mut k = 0;
    for i in 0..num_managers {
        treasury::allowance(
            &mut app,
            "admin",
            &contracts,
            snip20_symbol.to_string(),
            i,
            allow_type[i].clone(),
            cycle[i].clone(),
            Uint128::zero(),
            allow_tolerance[i].clone(),
        );
        for j in 0..alloc_amount[i].len() {
            treasury_manager::allocate(
                &mut app,
                "admin",
                &contracts,
                snip20_symbol.to_string(),
                Some(j.to_string()),
                &SupportedContracts::MockAdapter(k),
                alloc_type[i][j].clone(),
                Uint128::zero(),
                alloc_tolerance[i][j].clone(),
                i,
            );
            k += 1;
        }
        k += 1;
    }
    update_dao(&mut app, "admin", &contracts, "SSCRT", num_managers);
    treasury::update_exec(&mut app, "admin", &contracts, "SSCRT".to_string());
    bals = system_balance(&app, &contracts, "SSCRT".to_string());
    println!("{:?}", bals);
    assert_eq!(bals.0, initial_treasury_bal);
    for (i, manager_tuples) in bals.1.iter().enumerate() {
        assert_eq!(manager_tuples.0, Uint128::zero());
        for (j, adapter_bals) in manager_tuples.1.iter().enumerate() {
            assert_eq!(adapter_bals.clone(), Uint128::zero());
        }
    }
}

macro_rules! dao_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    initial_treasury_bal,
                    snip20_symbol,
                    allow_amount,
                    allow_type,
                    cycle,
                    allow_tolerance,
                    expected_allowance,
                    alloc_amount,
                    alloc_type,
                    alloc_tolerance,
                    expected_treasury,
                    expected_manager,
                    expected_adapter,
                ) = $value;
                dao_int_test(
                    initial_treasury_bal,
                    snip20_symbol,
                    allow_amount,
                    allow_type,
                    cycle,
                    allow_tolerance,
                    expected_allowance,
                    alloc_amount,
                    alloc_type,
                    alloc_tolerance,
                    expected_treasury,
                    expected_manager,
                    expected_adapter,
                );
            }
        )*
    }
}

dao_tests! {
    dao_test_0:(
        Uint128::new(1_000_000),
        "SSCRT",
        vec![Uint128::new(5 * 10u128.pow(17))],
        vec![AllowanceType::Portion],
        vec![Cycle::Constant],
        vec![Uint128::zero()],
        vec![Uint128::new(90_000)],
        vec![vec![Uint128::new(1 * 10u128.pow(17)), Uint128::new(400_000)]],
        vec![vec![AllocationType::Portion, AllocationType::Amount]],
        vec![vec![Uint128::zero(), Uint128::zero()]],
        Uint128::new(590_000),
        vec![Uint128::new(0)],
        vec![vec![Uint128::new(10_000), Uint128::new(400_000)]],
    ),
    dao_test_1:(
        Uint128::new(100_000_000),
        "SSCRT",
        vec![Uint128::new(50_000_000), Uint128::new(40_000_000)],
        vec![AllowanceType::Amount, AllowanceType::Amount],
        vec![Cycle::Constant, Cycle::Constant],
        vec![Uint128::zero(), Uint128::zero()],
        vec![Uint128::new(21_000_000), Uint128::new(18_000_000)],
        vec![vec![Uint128::new(5 * 10u128.pow(17)), Uint128::new(4_000_000), Uint128::new(4_000_000)], vec![Uint128::new(5 * 10u128.pow(17)), Uint128::new(4_000_000)]],
        vec![vec![AllocationType::Portion, AllocationType::Amount, AllocationType::Amount],vec![AllocationType::Portion, AllocationType::Amount]],
        vec![vec![Uint128::zero(), Uint128::zero(), Uint128::zero()],vec![Uint128::zero(), Uint128::zero()]],
        Uint128::new(49_000_000),
        vec![Uint128::new(0), Uint128::new(0)],
        vec![vec![Uint128::new(21_000_000), Uint128::new(4_000_000), Uint128::new(4_000_000)],vec![Uint128::new(18_000_000), Uint128::new(4_000_000)]],
    ),
    dao_test_2:(
        Uint128::new(100),
        "SSCRT",
        vec![Uint128::new(5 * 10u128.pow(17))],
        vec![AllowanceType::Portion],
        vec![Cycle::Constant],
        vec![Uint128::zero()],
        vec![Uint128::new(9)],
        vec![vec![Uint128::new(1 * 10u128.pow(17)), Uint128::new(40)]],
        vec![vec![AllocationType::Portion, AllocationType::Amount]],
        vec![vec![Uint128::zero(), Uint128::zero()]],
        Uint128::new(59),
        vec![Uint128::new(0)],
        vec![vec![Uint128::new(1), Uint128::new(40)]],
    ),
    dao_test_3: (
        Uint128::new(1000),
        "SSCRT",
        vec![
            Uint128::new(50), // Amount - 50
            Uint128::new(6 * 10u128.pow(17)), // Poriton - 60%
            Uint128::new(100), // Amount - 100
            Uint128::new(4 * 10u128.pow(17)), // Portion - 40%
        ], // Allowance amount
        vec![AllowanceType::Amount, AllowanceType::Portion, AllowanceType::Amount,  AllowanceType::Portion],
        vec![Cycle::Constant; 4],
        vec![Uint128::zero(); 4],
        vec![Uint128::new(6), Uint128::new(98), Uint128::new(16),  Uint128::new(64)],
        vec![
            vec![Uint128::new(6 * 10u128.pow(17)), Uint128::new(5), Uint128::new(2 * 10u128.pow(17)), Uint128::new(15)];4
        ],
        vec![
            vec![AllocationType::Portion, AllocationType::Amount, AllocationType::Portion, AllocationType::Amount];4
        ],
        vec![
            vec![Uint128::zero(); 4]; 4
        ],
        Uint128::new(184),
        vec![Uint128::zero(); 4],
        vec![
            vec![Uint128::new(18), Uint128::new(5), Uint128::new(6), Uint128::new(15)],
            vec![Uint128::new(294), Uint128::new(5), Uint128::new(98), Uint128::new(15)],
            vec![Uint128::new(48), Uint128::new(5), Uint128::new(16), Uint128::new(15)],
            vec![Uint128::new(192), Uint128::new(5), Uint128::new(64), Uint128::new(15)],
        ]
    ),
}

pub fn dao_int_gains_losses(
    initial_treasury_bal: Uint128,
    allow_type: Vec<AllowanceType>,
    t_cycle: Vec<Cycle>,
    allow_amount: Vec<Uint128>,
    allow_tolerance: Vec<Uint128>,
    alloc_type: Vec<Vec<AllocationType>>,
    alloc_amount: Vec<Vec<Uint128>>,
    alloc_tolerance: Vec<Vec<Uint128>>,
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
        alloc_amount,
        alloc_tolerance,
    );
    let bals = system_balance(&app, &contracts, "SSCRT".to_string());
    assert_eq!(bals, expected_after_init, "AFTER INITIALIZATION");
    for (i, adap) in adapters_to_send_to.clone().iter().enumerate() {
        if is_adapters_gain[i] {
            snip20::send(
                &mut app,
                "admin",
                &contracts,
                "SSCRT".to_string(),
                contracts
                    .get(&SupportedContracts::MockAdapter(adap.clone()))
                    .unwrap()
                    .address
                    .to_string(),
                snip20_send_amount,
                None,
            );
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
    for _ in 0..2 {
        for tm in 0..num_managers {
            treasury_manager::update_exec(
                &mut app,
                "admin",
                &contracts,
                "SSCRT".to_string(),
                SupportedContracts::TreasuryManager(tm),
            );
        }
        treasury::update_exec(&mut app, "admin", &contracts, "SSCRT".to_string()).unwrap();
    }
    let bals = system_balance(&app, &contracts, "SSCRT".to_string());
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
}

pub fn test_tm_unbond(
    unbond_amount: Uint128,
    adapter_gain_amount: Option<Uint128>,
    amount_adapter_bal: (Uint128, Uint128),
    expected_before_unbond: (Uint128, Vec<(Uint128, Vec<Uint128>)>),
    expected_after_unbond: (Uint128, Vec<(Uint128, Vec<Uint128>)>),
) {
    let mut app = App::default();
    let mut contracts = DeployedContracts::new();
    init_dao(
        &mut app,
        "admin",
        &mut contracts,
        Uint128::new(1000),
        "SSCRT",
        vec![AllowanceType::Amount],
        vec![Cycle::Constant],
        vec![
            Uint128::new(500), // Amount - 500
        ], // Allowance amount
        vec![Uint128::zero()],
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
                amount_adapter_bal.0,
                Uint128::new(2 * 10u128.pow(17)),
                amount_adapter_bal.1,
            ];
            4
        ],
        vec![vec![Uint128::zero(); 4]; 4],
    );
    let bals = system_balance(&app, &contracts, "SSCRT".to_string());
    assert_eq!(bals, expected_before_unbond);
    match adapter_gain_amount {
        Some(x) => {
            for i in vec![1, 3] {
                snip20::send(
                    &mut app,
                    "admin",
                    &contracts,
                    "SSCRT".to_string(),
                    contracts
                        .get(&SupportedContracts::MockAdapter(i))
                        .unwrap()
                        .address
                        .to_string(),
                    x,
                    None,
                );
            }
        }
        None => {}
    }
    unbond_exec(
        &mut app,
        "admin",
        &contracts,
        "SSCRT".to_string(),
        unbond_amount,
        SupportedContracts::Treasury,
    )
    .unwrap();

    let bals = system_balance(&app, &contracts, "SSCRT".to_string());
    assert_eq!(bals, expected_after_unbond);
}

macro_rules! dao_tests_tm_unbond {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    unbond_amount,
                    adapter_gain_amount,
                    amount_adapter_bal,
                    expected_before_unbond,
                    expected_after_unbond,
                ) = $value;
                test_tm_unbond(
                    unbond_amount,
                    adapter_gain_amount,
                    amount_adapter_bal,
                    expected_before_unbond,
                    expected_after_unbond,
                );
            }
        )*
    }
}

dao_tests_tm_unbond! {
    unbond_only_from_amount_adapters:(
        Uint128::new(7),
        Some(Uint128::new(10)),
        (Uint128::new(10), Uint128::new(20)),
        (Uint128::new(594), vec![(Uint128::new(0), vec![
            Uint128::new(282),
            Uint128::new(10),
            Uint128::new(94),
            Uint128::new(20),
        ])]),
        (Uint128::new(594), vec![(Uint128::new(7), vec![
            Uint128::new(282),
            Uint128::new(17),
            Uint128::new(94),
            Uint128::new(26)
        ])])
    ),
    unbond_case_extra_from_amount_adapters_and_some_from_portion_adapters:(
        Uint128::new(21),
        Some(Uint128::new(10)),
        (Uint128::new(10), Uint128::new(20)),
        (Uint128::new(594), vec![(Uint128::new(0), vec![
            Uint128::new(282),
            Uint128::new(10),
            Uint128::new(94),
            Uint128::new(20),
        ])]),
        (Uint128::new(594), vec![(Uint128::new(21), vec![
            Uint128::new(282),
            Uint128::new(10),
            Uint128::new(93),
            Uint128::new(20)
        ])])
    ),
    unbond_case_extra_from_amount_adapters_and_all_from_portion_adapters:(
        Uint128::new(396),
        Some(Uint128::new(10)),
        (Uint128::new(10), Uint128::new(20)),
        (Uint128::new(594), vec![(Uint128::new(0), vec![
            Uint128::new(282),
            Uint128::new(10),
            Uint128::new(94),
            Uint128::new(20),
        ])]),
        (Uint128::new(594), vec![(Uint128::new(396), vec![
            Uint128::new(0),
            Uint128::new(10),
            Uint128::new(0),
            Uint128::new(20)
        ])])
    ),
    unbond_case_extra_and_some_from_amount_adapters_and_all_from_portion_adapters:(
        Uint128::new(391),
        None,
        (Uint128::new(100), Uint128::new(200)),
        (Uint128::new(540), vec![(Uint128::new(0), vec![
            Uint128::new(120),
            Uint128::new(100),
            Uint128::new(40),
            Uint128::new(200),
        ])]),
        (Uint128::new(540), vec![(Uint128::new(391), vec![
            Uint128::new(0),
            Uint128::new(23),
            Uint128::new(0),
            Uint128::new(46)
        ])])
    ),
    unbond_all:(
        Uint128::new(460),
        None,
        (Uint128::new(100), Uint128::new(200)),
        (Uint128::new(540), vec![(Uint128::new(0), vec![
            Uint128::new(120),
            Uint128::new(100),
            Uint128::new(40),
            Uint128::new(200),
        ])]),
        (Uint128::new(540), vec![(Uint128::new(460), vec![
            Uint128::new(0),
            Uint128::new(0),
            Uint128::new(0),
            Uint128::new(0)
        ])])
    ),
}
