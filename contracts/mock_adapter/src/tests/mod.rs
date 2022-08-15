use shade_multi_test::interfaces::{
    dao::{init_dao, system_balance},
    utils::{DeployedContracts, SupportedContracts},
};
use shade_protocol::{
    c_std::Uint128,
    contract_interfaces::dao::{treasury::AllowanceType, treasury_manager::AllocationType},
    multi_test::App,
    utils::cycle::Cycle,
};

pub fn dao_int_test(
    allow_amount: Uint128,
    expected_allowance: Uint128,
    alloc_amount: Uint128,
    expected_treasury: Uint128,
    expected_manager: Uint128,
    expected_adapter: Uint128,
) {
    let mut app = App::default();
    let mut contracts = DeployedContracts::new();
    let snip20_symbol = "SSCRT".to_string();
    init_dao(
        &mut app,
        "admin",
        &mut contracts,
        7,
        7,
        Uint128::new(1000000),
        AllowanceType::Portion,
        Cycle::Constant,
        allow_amount,
        Uint128::new(10u128.pow(18)),
        AllocationType::Portion,
        alloc_amount,
        Uint128::zero(),
    );
    let bals = system_balance(&app, &contracts, "SSCRT".to_string());
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
                    allow_amount,
                    expected_allowance,
                    alloc_amount,
                    expected_treasury,
                    expected_manager,
                    expected_adapter,
                ) = $value;
                dao_int_test(
                    allow_amount,
                    expected_allowance,
                    alloc_amount,
                    expected_treasury,
                    expected_manager,
                    expected_adapter
                );
            }
        )*
    }
}

dao_tests! {
    dao_test_0:(
        Uint128::new(9 * 10u128.pow(17)),
        Uint128::new(90),
        Uint128::new(1 * 10u128.pow(18)),
        Uint128::new(10),
        Uint128::new(0),
        Uint128::new(90),
    ),
}
