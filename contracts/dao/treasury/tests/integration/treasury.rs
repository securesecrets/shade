use mock_adapter;
use shade_multi_test::{
    interfaces::{
        self,
        utils::{DeployedContracts, SupportedContracts},
    },
    multi::{
        admin::init_admin_auth,
        mock_adapter::MockAdapter,
        snip20::Snip20,
        treasury::Treasury,
        treasury_manager::TreasuryManager,
    },
};
use shade_protocol::{
    c_std::{to_binary, Addr, Coin, Uint128},
    contract_interfaces::{
        dao::{
            treasury,
            treasury::{AllowanceType, RunLevel},
            treasury_manager::{self, Allocation, AllocationType},
        },
        snip20,
    },
    multi_test::{App, BankSudo, StakingSudo, SudoMsg},
    utils::{
        asset::Contract,
        cycle::Cycle,
        ExecuteCallback,
        InstantiateCallback,
        MultiTestable,
        Query,
    },
};

// Add other adapters here as they come
fn bonded_adapter_int(
    deposit: Uint128,
    allowance: Uint128,
    expected_allowance: Uint128,
    alloc_type: AllocationType,
    alloc_amount: Uint128,
    rewards: Uint128,
    // expected balances
    pre_rewards: (Uint128, Uint128, Uint128),
    post_rewards: (Uint128, Uint128, Uint128),
) {
    let mut app = App::default();
    let mut contracts = DeployedContracts::new();

    let admin = Addr::unchecked("admin");
    let admin_auth = init_admin_auth(&mut app, &admin);

    let viewing_key = "viewing_key".to_string();
    let symbol = "TKN";

    interfaces::dao::init_dao(
        &mut app,
        &admin.to_string(),
        &mut contracts,
        deposit,
        symbol,
        vec![AllowanceType::Portion],
        vec![Cycle::Constant],
        vec![allowance],
        vec![Uint128::zero()],
        vec![vec![alloc_type]],
        vec![vec![alloc_amount]],
        vec![vec![Uint128::zero()]],
        false,
        false,
    );

    // Update treasury
    interfaces::treasury::update_exec(&mut app, &admin.to_string(), &contracts, symbol).unwrap();

    // Check initial allowance
    assert_eq!(
        interfaces::treasury::allowance_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::TreasuryManager(0),
        )
        .unwrap(),
        expected_allowance,
        "Treasury->Manager Allowance"
    );

    // Update manager
    interfaces::treasury_manager::update_exec(
        &mut app,
        &admin.to_string(),
        &contracts,
        symbol,
        SupportedContracts::TreasuryManager(0),
    )
    .unwrap();

    // Update Adapter
    interfaces::dao::update_exec(
        &mut app,
        &admin.to_string(),
        &contracts,
        symbol,
        SupportedContracts::MockAdapter(0),
    )
    .unwrap();

    // Treasury reserves check
    assert_eq!(
        interfaces::treasury::reserves_query(&app, &contracts, symbol).unwrap(),
        pre_rewards.0,
        "Treasury Reserves",
    );

    // Manager reserves
    assert_eq!(
        interfaces::treasury_manager::reserves_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury,
        )
        .unwrap(),
        pre_rewards.1,
        "Manager Reserves",
    );

    // Adapter reserves should be 0 (all staked)
    assert_eq!(
        interfaces::dao::reserves_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::MockAdapter(0)
        )
        .unwrap(),
        Uint128::zero(),
        "Bonded Adapter Reserves",
    );

    // Adapter balance
    assert_eq!(
        interfaces::dao::balance_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::MockAdapter(0)
        )
        .unwrap(),
        pre_rewards.2,
        "Adapter Balance",
    );

    // Add Rewards
    interfaces::snip20::send_exec(
        &mut app,
        &admin.to_string(),
        &contracts,
        symbol,
        contracts
            .get(&SupportedContracts::MockAdapter(0))
            .unwrap()
            .address
            .to_string(),
        rewards,
        None,
    )
    .unwrap();

    // Adapter Balance
    assert_eq!(
        interfaces::dao::balance_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::MockAdapter(0)
        )
        .unwrap(),
        pre_rewards.2 + rewards,
        "Adapter Balance Post-Rewards Pre-Update",
    );

    // Update manager
    interfaces::treasury_manager::update_exec(
        &mut app,
        &admin.to_string(),
        &contracts,
        symbol,
        SupportedContracts::TreasuryManager(0),
    )
    .unwrap();

    // Update treasury
    interfaces::treasury::update_exec(&mut app, &admin.to_string(), &contracts, symbol).unwrap();

    // Adapter Balance
    assert_eq!(
        interfaces::dao::balance_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::MockAdapter(0)
        )
        .unwrap(),
        pre_rewards.2 + rewards,
        "Adapter Balance Post-Rewards Post-Update"
    );

    // Adapter Unbondable
    assert_eq!(
        interfaces::dao::unbondable_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::MockAdapter(0)
        )
        .unwrap(),
        post_rewards.2,
        "Adapter Unbondable",
    );

    // Manager unbondable check
    assert_eq!(
        interfaces::treasury_manager::unbondable_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury,
        )
        .unwrap(),
        post_rewards.1 + post_rewards.2,
        "Manager Unbondable"
    );

    // Unbond all w/ manager
    interfaces::treasury_manager::unbond_exec(
        &mut app,
        &admin.to_string(),
        &contracts,
        symbol,
        SupportedContracts::TreasuryManager(0),
        post_rewards.1 + post_rewards.2,
    )
    .unwrap();

    // Adapter Reserves
    assert_eq!(
        interfaces::dao::reserves_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::MockAdapter(0)
        )
        .unwrap(),
        Uint128::zero(),
        "Adapter Reserves Pre-fastforward"
    );

    // Adapter Unbonding
    assert_eq!(
        interfaces::dao::unbonding_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::MockAdapter(0)
        )
        .unwrap(),
        pre_rewards.2 + rewards,
        "Adapter Unbonding Pre-fastforward"
    );

    // Adapter Claimable
    assert_eq!(
        interfaces::dao::claimable_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::MockAdapter(0)
        )
        .unwrap(),
        Uint128::zero(),
        "Adapter Claimable Pre-fastforward",
    );

    // Manager Claimable
    assert_eq!(
        interfaces::treasury_manager::claimable_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury,
        )
        .unwrap(),
        Uint128::zero(),
        "Manager Claimable Pre-fastforward"
    );

    // Manager Unbonding
    assert_eq!(
        interfaces::treasury_manager::unbonding_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury,
        )
        .unwrap(),
        pre_rewards.2 + rewards,
        "Manager Claimable Pre-fastforward"
    );

    // Complete unbondings
    interfaces::dao::mock_adapter_complete_unbonding(
        &mut app,
        &admin.to_string(),
        &contracts,
        SupportedContracts::MockAdapter(0),
    )
    .unwrap();

    // adapter unbonding
    assert_eq!(
        interfaces::dao::unbonding_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::MockAdapter(0)
        )
        .unwrap(),
        Uint128::zero(),
        "Adapter Unbonding Post-fastforward"
    );

    // adapter claimable
    assert_eq!(
        interfaces::dao::claimable_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::MockAdapter(0)
        )
        .unwrap(),
        pre_rewards.2 + rewards,
        "Adapter Claimable Post-fastforward"
    );

    // Claim Treasury Manager
    interfaces::treasury_manager::claim_exec(
        &mut app,
        &admin.to_string(),
        &contracts,
        symbol,
        SupportedContracts::TreasuryManager(0),
    )
    .unwrap();

    // Adapter balance
    assert_eq!(
        interfaces::dao::balance_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::MockAdapter(0)
        )
        .unwrap(),
        Uint128::zero(),
        "Adapter Balance Post-Claim"
    );

    // Treasury balance check
    assert_eq!(
        interfaces::treasury::balance_query(&app, &contracts, symbol,).unwrap(),
        deposit + rewards,
        "Treasury Balance Post Claim"
    );

    // Adapter reserves
    assert_eq!(
        interfaces::dao::reserves_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::MockAdapter(0)
        )
        .unwrap(),
        Uint128::zero(),
        "Adapter Reserves Post-Claim"
    );

    // Manager unbonding check
    assert_eq!(
        interfaces::treasury_manager::reserves_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury,
        )
        .unwrap(),
        Uint128::zero(),
        "Manager Unbonding Post-Claim"
    );

    // Manager balance check
    assert_eq!(
        interfaces::treasury_manager::balance_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury,
        )
        .unwrap(),
        Uint128::zero(),
        "Manager Balance Post-Claim"
    );

    // Manager reserves check
    assert_eq!(
        interfaces::treasury_manager::reserves_query(
            &app,
            &contracts,
            symbol,
            SupportedContracts::TreasuryManager(0),
            SupportedContracts::Treasury,
        )
        .unwrap(),
        Uint128::zero(),
        "Manager Reserves Post-Unbond"
    );

    // Treasury reserves check
    assert_eq!(
        interfaces::treasury::reserves_query(&app, &contracts, symbol,).unwrap(),
        deposit + rewards,
        "Treasury Reserves Post-Unbond"
    );
    assert_eq!(
        interfaces::treasury::balance_query(&app, &contracts, symbol,).unwrap(),
        deposit + rewards,
        "Treasury Balance Post-Unbond"
    );

    // Migration
    interfaces::treasury::set_run_level_exec(
        &mut app,
        &admin.to_string(),
        &contracts,
        RunLevel::Migrating,
    )
    .unwrap();

    interfaces::treasury::update_exec(&mut app, &admin.to_string(), &contracts, symbol).unwrap();

    assert_eq!(
        interfaces::treasury::balance_query(&app, &contracts, symbol,).unwrap(),
        Uint128::zero(),
        "post-migration full unbond"
    );
}

macro_rules! bonded_adapter_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    deposit,
                    allowance,
                    expected_allowance,
                    alloc_type,
                    alloc_amount,
                    rewards,
                    pre_rewards,
                    post_rewards,
                ) = $value;
                bonded_adapter_int(
                    deposit,
                    allowance,
                    expected_allowance,
                    alloc_type,
                    alloc_amount,
                    rewards,
                    pre_rewards,
                    post_rewards,
                );
            }
        )*
    }
}

bonded_adapter_tests! {
    portion_with_rewards_0: (
        Uint128::new(100), // deposit
        Uint128::new(1 * 10u128.pow(18)), // manager allowance 100%
        Uint128::new(100), // expected manager allowance
        AllocationType::Portion,
        Uint128::new(1 * 10u128.pow(18)), // allocate 100%
        Uint128::new(100), // rewards
        // pre-rewards
        (
            Uint128::new(0), // treasury 10
            Uint128::new(0), // manager 0
            Uint128::new(100), // mock_adapter 90
        ),
        //post-rewards
        (
            Uint128::new(0), // treasury 10
            Uint128::new(0), // manager 0
            Uint128::new(200), // mock_adapter 90
        ),
    ),
    portion_with_rewards_1: (
        Uint128::new(1000), // deposit
        Uint128::new(5 * 10u128.pow(17)), // %50 manager allowance
        Uint128::new(500), // expected manager allowance
        AllocationType::Portion,
        Uint128::new(1 * 10u128.pow(18)), // 100% allocate
        Uint128::new(10), // rewards
        (
            Uint128::new(500), // treasury 55 (manager won't pull unused allowance
            Uint128::new(0), // manager 0
            Uint128::new(500), // mock_adapter
        ),
        (
            Uint128::new(505),
            Uint128::new(0),
            Uint128::new(505),
        ),
    ),
    /*
    // TODO: this needs separate test logic bc of update
    amount_with_rewards_0: (
        Uint128::new(1_000_000), // deposit
        Uint128::new(5 * 10u128.pow(17)), // %50 manager allowance
        Uint128::new(500_000), // expected manager allowance
        AllocationType::Amount,
        Uint128::new(500_000), // .5 tkn (all) allocate
        Uint128::new(500), // rewards
        (
            Uint128::new(500_000), // treasury
            Uint128::new(0), // manager 0
            Uint128::new(500_000), // mock_adapter
        ),
        (
            Uint128::new(500_250),
            Uint128::new(250),
            Uint128::new(500_000),
        ),
    ),
    */
}
