use shade_multi_test::interfaces::{
    dao::{init_dao, system_balance, update_dao},
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
            Uint128::new(100), // Amount - 100
            Uint128::new(6 * 10u128.pow(17)), // Poriton - 60%
            Uint128::new(4 * 10u128.pow(17)), // Portion - 40%
        ], // Allowance amount
        vec![AllowanceType::Amount, AllowanceType::Amount, AllowanceType::Portion, AllowanceType::Portion],
        vec![Cycle::Constant; 4],
        vec![Uint128::zero(); 4],
        vec![Uint128::new(6), Uint128::new(16), Uint128::new(98), Uint128::new(64)],
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
            vec![Uint128::new(48), Uint128::new(5), Uint128::new(16), Uint128::new(15)],
            vec![Uint128::new(294), Uint128::new(5), Uint128::new(98), Uint128::new(15)],
            vec![Uint128::new(192), Uint128::new(5), Uint128::new(64), Uint128::new(15)],
        ]
    ),
}

/*#[test]
pub fn dao_int_gains() {
    let mut app = App::default();
    let mut contracts = DeployedContracts::new();
    init_dao(
        &mut app,
        "admin",
        &mut contracts,
        Uint128::new(100),
        "SSCRT",
        vec![
            AllowanceType::Amount,
            AllowanceType::Portion,
            AllowanceType::Portion,
        ],
        vec![Cycle::Constant, Cycle::Constant, Cycle::Constant],
        vec![
            Uint128::new(50),
            Uint128::new(6 * 10u128.pow(17)),
            Uint128::new(4 * 10u128.pow(17)),
        ], // Allowance amount
        vec![Uint128::zero(), Uint128::zero()], // Allowance tolerance
        vec![vec![AllocationType::Amount, AllocationType::Portion], vec![
            AllocationType::Portion,
        ]],
        vec![
            vec![Uint128::new(20), Uint128::new(1 * 10u128.pow(18))],
            vec![Uint128::new(1 * 10u128.pow(18))],
        ],
        vec![
            vec![Uint128::zero(), Uint128::zero()],
            vec![Uint128::zero()],
        ],
    );
    let bals = system_balance(&app, &contracts, "SSCRT".to_string());
    assert_eq!(
        bals,
        (Uint128::new(25), vec![
            (Uint128::zero(), vec![Uint128::new(20), Uint128::new(30)]),
            (Uint128::zero(), vec![Uint128::new(25)])
        ])
    );
    snip20::send(
        &mut app,
        "admin",
        &contracts,
        "SSCRT".to_string(),
        contracts
            .get(&SupportedContracts::MockAdapter(0))
            .unwrap()
            .address
            .to_string(),
        Uint128::new(5),
        None,
    );
    let bals = system_balance(&app, &contracts, "SSCRT".to_string());
    assert_eq!(
        bals,
        (Uint128::new(50), vec![(Uint128::zero(), vec![
            Uint128::new(55)
        ])])
    );
    update_dao(&mut app, "admin", &contracts, "SSCRT", 1).unwrap();
    let bals = system_balance(&app, &contracts, "SSCRT".to_string());
    println!("{:?}", bals);
    assert_eq!(
        bals,
        (Uint128::new(50), vec![(Uint128::new(5), vec![
            Uint128::new(50)
        ])])
    );
    treasury::update_exec(&mut app, "admin", &contracts, "SSCRT".to_string()).unwrap();
    let bals = system_balance(&app, &contracts, "SSCRT".to_string());
    assert_eq!(
        bals,
        (Uint128::new(50), vec![(Uint128::new(5), vec![
            Uint128::new(50)
        ])])
    );
}*/
