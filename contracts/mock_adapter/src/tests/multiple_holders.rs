use shade_multi_test::interfaces::{
    dao::{
        init_dao,
        mock_adapter_complete_unbonding,
        mock_adapter_sub_tokens,
        system_balance_reserves,
        system_balance_unbondable,
        update_dao,
    },
    snip20,
    treasury,
    treasury_manager,
    utils::{DeployedContracts, SupportedContracts},
};
use shade_protocol::{
    c_std::{Addr, Uint128},
    contract_interfaces::dao::{
        self,
        treasury::AllowanceType,
        treasury_manager::{AllocationType, Balance, Holding, Status},
    },
    multi_test::App,
    utils::cycle::Cycle,
};

#[test]
pub fn multiple_holders() {
    //is_instant_unbond: bool) {
    const holder: &str = "holder";
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
        true,
    );
    println!(
        "{:?}",
        system_balance_reserves(&app, &contracts, "SSCRT".to_string())
    );
    snip20::set_viewing_key(
        &mut app,
        holder,
        &contracts,
        "SSCRT".to_string(),
        holder.to_string(),
    )
    .unwrap();
    snip20::send(
        &mut app,
        "admin",
        &contracts,
        "SSCRT".to_string(),
        holder.to_string(),
        Uint128::new(1000),
        None,
    );
    treasury_manager::register_holder_exec(
        &mut app,
        "admin",
        &contracts,
        "SSCRT".to_string(),
        SupportedContracts::TreasuryManager(0),
        holder,
    )
    .unwrap();
    snip20::send(
        &mut app,
        holder,
        &contracts,
        "SSCRT".to_string(),
        contracts[&SupportedContracts::TreasuryManager(0)]
            .address
            .to_string(),
        Uint128::new(500),
        None,
    );
    assert_eq!(
        Uint128::new(500),
        treasury_manager::holding_query(
            &app,
            &contracts,
            "SSCRT".to_string(),
            SupportedContracts::TreasuryManager(0),
            holder.to_string(),
        )
        .unwrap()
        .balances[0]
            .amount
    );
    update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    println!(
        "{:?}",
        system_balance_reserves(&app, &contracts, "SSCRT".to_string())
    );
    treasury_manager::unbond_exec(
        &mut app,
        holder,
        &contracts,
        "SSCRT".to_string(),
        SupportedContracts::TreasuryManager(0),
        Uint128::new(300),
    )
    .unwrap();
    update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    treasury_manager::claim_exec(
        &mut app,
        holder,
        &contracts,
        "SSCRT".to_string(),
        SupportedContracts::TreasuryManager(0),
    )
    .unwrap();
    treasury_manager::remove_holder_exec(
        &mut app,
        "admin",
        &contracts,
        "SSCRT".to_string(),
        SupportedContracts::TreasuryManager(0),
        holder.clone(),
    )
    .unwrap();
    treasury_manager::unbond_exec(
        &mut app,
        holder,
        &contracts,
        "SSCRT".to_string(),
        SupportedContracts::TreasuryManager(0),
        Uint128::zero(),
    )
    .unwrap();
    treasury_manager::claim_exec(
        &mut app,
        holder,
        &contracts,
        "SSCRT".to_string(),
        SupportedContracts::TreasuryManager(0),
    )
    .unwrap();
    update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    update_dao(&mut app, "admin", &contracts, "SSCRT", 4).unwrap();
    match (treasury_manager::holding_query(
        &app,
        &contracts,
        "SSCRT".to_string(),
        SupportedContracts::TreasuryManager(0),
        holder.to_string(),
    )) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
    println!(
        "{:?}",
        system_balance_reserves(&app, &contracts, "SSCRT".to_string())
    );
    //assert!(false);
}
