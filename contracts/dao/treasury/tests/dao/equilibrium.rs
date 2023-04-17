use shade_multi_test::interfaces::{
    dao::{
        init_dao,
        mock_adapter_complete_unbonding,
        system_balance_reserves,
        system_balance_unbondable,
    },
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

pub fn equilibrium_test(
    is_instant_unbond: bool,
    initial_bals: (Uint128, Vec<(Uint128, Vec<Uint128>)>),
) {
    let mut app = App::default();
    let mut contracts = DeployedContracts::new();
    let num_managers = 8;
    init_dao(
        &mut app,
        "admin",
        &mut contracts,
        Uint128::new(1500),
        "SSCRT",
        vec![
            AllowanceType::Portion,
            AllowanceType::Amount,
            AllowanceType::Portion,
            AllowanceType::Amount,
            AllowanceType::Portion,
            AllowanceType::Amount,
            AllowanceType::Portion,
            AllowanceType::Amount,
        ],
        vec![Cycle::Constant; 8],
        vec![
            Uint128::new(5 * 10u128.pow(16)),  // Poriton - 5%
            Uint128::new(30),                  // Amount - 30
            Uint128::new(15 * 10u128.pow(16)), // Portion - 15%
            Uint128::new(40),                  // Amount - 40
            Uint128::new(25 * 10u128.pow(16)), // Poriton - 25%
            Uint128::new(50),                  // Amount - 50
            Uint128::new(35 * 10u128.pow(16)), // Portion - 35%
            Uint128::new(20),                  // Amount - 20
        ], // Allowance amount
        vec![Uint128::zero(); 8],
        vec![
            vec![
                AllocationType::Amount,
                AllocationType::Portion,
                AllocationType::Amount,
                AllocationType::Portion,
                AllocationType::Amount,
                AllocationType::Portion,
                AllocationType::Amount,
                AllocationType::Portion,
            ];
            8
        ],
        vec![
            vec![
                Uint128::new(1),                   // Amount - 1
                Uint128::new(4 * 10u128.pow(16)),  // Portion - 4%
                Uint128::new(2),                   // Amount - 2
                Uint128::new(16 * 10u128.pow(16)), //Portion - 16%
                Uint128::new(3),                   // Amount - 3
                Uint128::new(1 * 10u128.pow(17)),  //Portion - 10%
                Uint128::new(4),                   // Amount - 4
                Uint128::new(2 * 10u128.pow(17)),  // Portion - 20%
            ];
            8
        ],
        vec![vec![Uint128::zero(); 8]; 8],
        is_instant_unbond,
        true,
    )
    .unwrap();
    for i in 0..20 {
        let bals = {
            if is_instant_unbond {
                system_balance_reserves(&app, &contracts, "SSCRT")
            } else {
                system_balance_unbondable(&app, &contracts, "SSCRT")
            }
        };
        assert_eq!(bals, initial_bals, "loop: {}", i);
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
        if !is_instant_unbond {
            let mut k = 0;
            for _i in 0..num_managers {
                for _j in 0..num_managers {
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
        }
    }
}

macro_rules! dao_tests_migration {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    is_instant_unbond,
                    initial_bals,
                ) = $value;
                equilibrium_test(
                    is_instant_unbond,
                    initial_bals,
                );
            }
        )*
    }
}

dao_tests_migration! (
    dao_test_equilibrium_instant_unbond: (
        true,
        (Uint128::new(857), vec![
            (Uint128::new(0), vec![ // used - 38
                Uint128::new(1),
                Uint128::new(2),
                Uint128::new(2),
                Uint128::new(9),
                Uint128::new(3),
                Uint128::new(5),
                Uint128::new(4),
                Uint128::new(11),
            ]),
            (Uint128::new(0), vec![ // used - 18
                Uint128::new(1),
                Uint128::new(0),
                Uint128::new(2),
                Uint128::new(3),
                Uint128::new(3),
                Uint128::new(2),
                Uint128::new(4),
                Uint128::new(4),
            ]),
            (Uint128::new(0), vec![ // used - 105
                Uint128::new(1),
                Uint128::new(7),
                Uint128::new(2),
                Uint128::new(31),
                Uint128::new(3),
                Uint128::new(19),
                Uint128::new(4),
                Uint128::new(38),
            ]),
            (Uint128::new(0), vec![ // used - 25
                Uint128::new(1),
                Uint128::new(1),
                Uint128::new(2),
                Uint128::new(4),
                Uint128::new(3),
                Uint128::new(3),
                Uint128::new(4),
                Uint128::new(6),
            ]),
            (Uint128::new(0), vec![ // used - 174
                Uint128::new(1),
                Uint128::new(13),
                Uint128::new(2),
                Uint128::new(52),
                Uint128::new(3),
                Uint128::new(33),
                Uint128::new(4),
                Uint128::new(66),
            ]),
            (Uint128::new(0), vec![ // used - 29
                Uint128::new(1),
                Uint128::new(1),
                Uint128::new(2),
                Uint128::new(6),
                Uint128::new(3),
                Uint128::new(4),
                Uint128::new(4),
                Uint128::new(8),
            ]),
            (Uint128::new(0), vec![ // used - 241
                Uint128::new(1),
                Uint128::new(18),
                Uint128::new(2),
                Uint128::new(74),
                Uint128::new(3),
                Uint128::new(46),
                Uint128::new(4),
                Uint128::new(93),
            ]),
            (Uint128::new(0), vec![ // used - 14
                Uint128::new(1),
                Uint128::new(0),
                Uint128::new(2),
                Uint128::new(1),
                Uint128::new(3),
                Uint128::new(1),
                Uint128::new(4),
                Uint128::new(2),
            ]),
        ]),
    ),
    dao_test_equilibrium_non_instant_unbond: (
        false,
        (Uint128::new(857), vec![
            (Uint128::new(0), vec![ // used - 38
                Uint128::new(1),
                Uint128::new(2),
                Uint128::new(2),
                Uint128::new(9),
                Uint128::new(3),
                Uint128::new(5),
                Uint128::new(4),
                Uint128::new(11),
            ]),
            (Uint128::new(0), vec![ // used - 18
                Uint128::new(1),
                Uint128::new(0),
                Uint128::new(2),
                Uint128::new(3),
                Uint128::new(3),
                Uint128::new(2),
                Uint128::new(4),
                Uint128::new(4),
            ]),
            (Uint128::new(0), vec![ // used - 105
                Uint128::new(1),
                Uint128::new(7),
                Uint128::new(2),
                Uint128::new(31),
                Uint128::new(3),
                Uint128::new(19),
                Uint128::new(4),
                Uint128::new(38),
            ]),
            (Uint128::new(0), vec![ // used - 25
                Uint128::new(1),
                Uint128::new(1),
                Uint128::new(2),
                Uint128::new(4),
                Uint128::new(3),
                Uint128::new(3),
                Uint128::new(4),
                Uint128::new(6),
            ]),
            (Uint128::new(0), vec![ // used - 174
                Uint128::new(1),
                Uint128::new(13),
                Uint128::new(2),
                Uint128::new(52),
                Uint128::new(3),
                Uint128::new(33),
                Uint128::new(4),
                Uint128::new(66),
            ]),
            (Uint128::new(0), vec![ // used - 29
                Uint128::new(1),
                Uint128::new(1),
                Uint128::new(2),
                Uint128::new(6),
                Uint128::new(3),
                Uint128::new(4),
                Uint128::new(4),
                Uint128::new(8),
            ]),
            (Uint128::new(0), vec![ // used - 241
                Uint128::new(1),
                Uint128::new(18),
                Uint128::new(2),
                Uint128::new(74),
                Uint128::new(3),
                Uint128::new(46),
                Uint128::new(4),
                Uint128::new(93),
            ]),
            (Uint128::new(0), vec![ // used - 14
                Uint128::new(1),
                Uint128::new(0),
                Uint128::new(2),
                Uint128::new(1),
                Uint128::new(3),
                Uint128::new(1),
                Uint128::new(4),
                Uint128::new(2),
            ]),
        ]),
    ),
);
