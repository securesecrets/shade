use shade_multi_test::interfaces::{
    dao::{init_dao, system_balance},
    treasury,
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
    snip20_symbols: Vec<&str>,
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
        snip20_symbols,
        allow_type,
        cycle,
        allow_amount,
        allow_tolerance,
        alloc_type,
        alloc_amount,
        alloc_tolerance,
    );
    //query allowance
    for i in 0..num_managers {
        assert_eq!(
            expected_allowance[i],
            treasury::allowance_query(
                &app,
                "admin",
                &contracts,
                "SSCRT".to_string(),
                SupportedContracts::TreasuryManager(i)
            )
            .unwrap()
        );
    }
    let bals = system_balance(&app, &contracts, "SSCRT".to_string());
    println!("{:?}", bals);
    assert_eq!(bals.0, expected_treasury);
    for (i, manager_tuples) in bals.1.iter().enumerate() {
        assert_eq!(manager_tuples.0, expected_manager[i]);
        for (j, adapter_bals) in manager_tuples.1.iter().enumerate() {
            assert_eq!(adapter_bals.clone(), expected_adapter[i][j]);
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
                    snip20_symbols,
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
                    snip20_symbols,
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
        vec!["SSCRT"],
        vec![Uint128::new(1 * 10u128.pow(17))],
        vec![AllowanceType::Portion],
        vec![Cycle::Constant],
        vec![Uint128::zero()],
        vec![Uint128::new(100_000)],
        vec![vec![Uint128::new(1 * 10u128.pow(17))]],
        vec![vec![AllocationType::Portion]],
        vec![vec![Uint128::zero()]],
        Uint128::new(900_000),
        vec![Uint128::new(90_000)],
        vec![vec![Uint128::new(10_000)]],
    ),
    /*dao_test_1:(
        Uint128::new(1_000_000),
        Uint128::new(1 * 10u128.pow(17)),
        Uint128::new(100_000),
        Uint128::new(1 * 10u128.pow(17)),
        Uint128::new(800_000),
        Uint128::new(80_000),
        Uint128::new(10_000),
        2,
        2,
    ),*/
}
