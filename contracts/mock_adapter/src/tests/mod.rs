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
    allow_amount: Uint128,
    expected_allowance: Uint128,
    alloc_amount: Uint128,
    expected_treasury: Uint128,
    expected_manager: Uint128,
    expected_adapter: Uint128,
    num_managers: u8,
    num_adapters: u8,
) {
    let mut app = App::default();
    let mut contracts = DeployedContracts::new();
    let snip20_symbol = "SSCRT".to_string();
    init_dao(
        &mut app,
        "admin",
        &mut contracts,
        num_managers,
        num_adapters,
        initial_treasury_bal,
        AllowanceType::Portion,
        Cycle::Constant,
        allow_amount,
        Uint128::zero(),
        AllocationType::Portion,
        alloc_amount,
        Uint128::zero(),
    );
    //query allowance
    for i in 0..num_managers {
        assert_eq!(
            expected_manager,
            treasury::allowance_query(
                &app,
                "admin",
                &contracts,
                "SSCRT".to_string(),
                SupportedContracts::TreasuryManager(i)
            )
            .unwrap(),
            "Treasury->Manager Allowance",
        );
    }
    let bals = system_balance(&app, &contracts, "SSCRT".to_string());
    println!("{:?}", bals);
    assert_eq!(bals.0, expected_treasury);
    for manager_tuples in bals.1 {
        assert_eq!(manager_tuples.0, expected_manager);
        for adapter_bals in manager_tuples.1 {
            assert_eq!(adapter_bals, expected_adapter);
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
                    allow_amount,
                    expected_allowance,
                    alloc_amount,
                    expected_treasury,
                    expected_manager,
                    expected_adapter,
                    num_managers,
                    num_adapters,
                ) = $value;
                dao_int_test(
                    initial_treasury_bal,
                    allow_amount,
                    expected_allowance,
                    alloc_amount,
                    expected_treasury,
                    expected_manager,
                    expected_adapter,
                    num_managers,
                    num_adapters,
                );
            }
        )*
    }
}

dao_tests! {
    dao_test_0:(
        Uint128::new(1_000_000),          // initial
        Uint128::new(1 * 10u128.pow(17)), // allowance portion
        Uint128::new(900_000),            // expected allowance
        Uint128::new(1 * 10u128.pow(17)), // alloc portion
        Uint128::new(900_000),            // expected treasury
        Uint128::new(90_000),             // expected manager
        Uint128::new(10_000),             // expected adapter
        1,                                // managers
        1,                                // adapters per manager
    ),
    dao_test_1:(
        Uint128::new(1_000_000),
        Uint128::new(1 * 10u128.pow(17)),
        Uint128::new(100_000),
        Uint128::new(1 * 10u128.pow(17)),
        Uint128::new(800_000),
        Uint128::new(80_000),
        Uint128::new(10_000),
        2,
        2,
    ),
}
