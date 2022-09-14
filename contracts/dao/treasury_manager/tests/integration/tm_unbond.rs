use shade_multi_test::interfaces::{
    dao::{init_dao, system_balance_reserves},
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
        true,
        true,
    )
    .unwrap();
    let bals = system_balance_reserves(&app, &contracts, "SSCRT");
    assert_eq!(bals, expected_before_unbond);
    match adapter_gain_amount {
        Some(x) => {
            for i in vec![1, 3] {
                snip20::send_exec(
                    &mut app,
                    "admin",
                    &contracts,
                    "SSCRT",
                    contracts
                        .get(&SupportedContracts::MockAdapter(i))
                        .unwrap()
                        .address
                        .to_string(),
                    x,
                    None,
                )
                .unwrap();
            }
        }
        None => {}
    }
    treasury_manager::unbond_exec(
        &mut app,
        "admin",
        &contracts,
        "SSCRT",
        SupportedContracts::TreasuryManager(0),
        unbond_amount,
    )
    .unwrap();

    let bals = system_balance_reserves(&app, &contracts, "SSCRT");
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
    unbond_only_extra_from_amount_adapters:(
        Uint128::new(20),
        Some(Uint128::new(10)),
        (Uint128::new(10), Uint128::new(20)),
        (Uint128::new(594), vec![(Uint128::new(0), vec![
            Uint128::new(282),
            Uint128::new(10),
            Uint128::new(94),
            Uint128::new(20),
        ])]),
        (Uint128::new(594), vec![(Uint128::new(20), vec![
            Uint128::new(282),
            Uint128::new(10),
            Uint128::new(94),
            Uint128::new(20)
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
        Uint128::new(459),
        None,
        (Uint128::new(101), Uint128::new(200)),
        (Uint128::new(541), vec![(Uint128::new(0), vec![
            Uint128::new(119),
            Uint128::new(101),
            Uint128::new(39),
            Uint128::new(200),
        ])]),
        (Uint128::new(541), vec![(Uint128::new(459), vec![
            Uint128::new(0),
            Uint128::new(0),
            Uint128::new(0),
            Uint128::new(0)
        ])])
    ),
}
