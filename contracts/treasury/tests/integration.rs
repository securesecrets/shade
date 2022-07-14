use shade_protocol::math_compat as compat;
use shade_protocol::c_std::{
    coins, from_binary, to_binary,
    Extern, HumanAddr, StdError,
    Binary, StdResult, HandleResponse, Env,
    InitResponse, Uint128,
};

use shade_protocol::{
    contract_interfaces::{
        dao::{
            treasury,
            treasury_manager::{
                self, Allocation, AllocationType,
            },
            scrt_staking,
            adapter,
            manager,
        },
        snip20,
    },
    utils::{
        asset::Contract,
    },
};

use contract_harness::harness::{
    treasury::Treasury,
    treasury_manager::TreasuryManager,
    scrt_staking::ScrtStaking,
    snip20_reference_impl::Snip20ReferenceImpl as Snip20,
    //snip20::Snip20,
};

use shade_protocol::fadroma::{
    core::ContractLink,
    ensemble::{
       MockEnv,
       ContractHarness,
       ContractEnsemble,
    },
};

// Add other adapters here as they come
fn single_asset_portion_manager_integration(
    deposit: Uint128, 
    allowance: Uint128,
    expected_allowance: Uint128,
    alloc_type: AllocationType,
    alloc_amount: Uint128,
    // expected balances
    expected_treasury: Uint128,
    expected_manager: Uint128,
    expected_scrt_staking: Uint128,
) {

    let mut ensemble = ContractEnsemble::new(50);

    let reg_treasury = ensemble.register(Box::new(Treasury));
    let reg_manager = ensemble.register(Box::new(TreasuryManager));
    let reg_scrt_staking = ensemble.register(Box::new(ScrtStaking));
    let reg_snip20 = ensemble.register(Box::new(Snip20));

    let token = ensemble.instantiate(
        reg_snip20.id,
        &snip20::InitMsg {
            name: "secretSCRT".into(),
            admin: Some("admin".into()),
            symbol: "SSCRT".into(),
            decimals: 6,
            initial_balances: None,
            prng_seed: to_binary("").ok().unwrap(),
            config: Some(snip20::InitConfig {
                public_total_supply: Some(true),
                enable_deposit: Some(true),
                enable_redeem: Some(true),
                enable_mint: Some(false),
                enable_burn: Some(false),
                enable_transfer: Some(true),
            }),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("token".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }
        )
    ).unwrap().instance;

    let treasury = ensemble.instantiate(
        reg_treasury.id,
        &treasury::InitMsg {
            admin: Some(HumanAddr("admin".into())),
            viewing_key: "viewing_key".to_string(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("treasury".into()),
                code_hash: reg_treasury.code_hash,
            }
        )
    ).unwrap().instance;

    let manager = ensemble.instantiate(
        reg_manager.id,
        &treasury_manager::InitMsg {
            admin: Some(HumanAddr("admin".into())),
            treasury: HumanAddr("treasury".into()),
            viewing_key: "viewing_key".to_string(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("manager".into()),
                code_hash: reg_manager.code_hash,
            }
        )
    ).unwrap().instance;

    let scrt_staking = ensemble.instantiate(
        reg_scrt_staking.id,
        &scrt_staking::InitMsg {
            admins: None,
            owner: manager.address.clone(),
            sscrt: Contract {
                address: token.address.clone(),
                code_hash: token.code_hash.clone(),
            },
            validator_bounds: None,
            viewing_key: "viewing_key".to_string(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("scrt_staking".into()),
                code_hash: reg_scrt_staking.code_hash,
            }
        )
    ).unwrap().instance;

    ensemble.add_validator(Validator {
        address: HumanAddr("validator".into()),
        commission: Decimal::zero(),
        max_commission: Decimal::one(),
        max_change_rate: Decimal::one(),
    });

    // Register treasury assets
    ensemble.execute(
        &treasury::HandleMsg::RegisterAsset {
            contract: Contract {
                address: token.address.clone(),
                code_hash: token.code_hash.clone(),
            },
        },
        MockEnv::new(
            "admin", 
            treasury.clone(),
        ),
    ).unwrap();
    
    // Register manager assets
    ensemble.execute(
        &treasury_manager::HandleMsg::RegisterAsset {
            contract: Contract {
                address: token.address.clone(),
                code_hash: token.code_hash.clone(),
            },
        },
        MockEnv::new(
            "admin", 
            manager.clone(),
        ),
    ).unwrap();

    // Register manager w/ treasury
    ensemble.execute(
        &treasury::HandleMsg::RegisterManager {
            contract: Contract {
                address: manager.address.clone(),
                code_hash: manager.code_hash.clone(),
            },
        },
        MockEnv::new(
            "admin", 
            treasury.clone(),
        ),
    ).unwrap();

    // treasury allowance to manager
    ensemble.execute(
        &treasury::HandleMsg::Allowance {
            asset: token.address.clone(),
            allowance: treasury::Allowance::Portion {
                //nick: "Mid-Stakes-Manager".to_string(),
                spender: manager.address.clone(),
                portion: allowance,
                // to be removed
                last_refresh: "".to_string(),
                // 100% (adapter balance will 2x before unbond)
                tolerance: Uint128(10u128.pow(18)),
            },
        },
        MockEnv::new(
            "admin", 
            treasury.clone(),
        ),
    ).unwrap();

    // Allocate to scrt_staking from manager
    ensemble.execute(
        &treasury_manager::HandleMsg::Allocate {
            asset: token.address.clone(),
            allocation: Allocation {
                nick: Some("scrt_staking".to_string()),
                contract: Contract {
                    address: scrt_staking.address.clone(),
                    code_hash: scrt_staking.code_hash.clone(),
                },
                alloc_type: alloc_type,
                amount: alloc_amount,
                tolerance: Uint128::zero(),
            }
        },
        MockEnv::new(
            "admin", 
            manager.clone(),
        ),
    ).unwrap();

    let deposit_coin = Coin { denom: "uscrt".into(), amount: deposit };
    ensemble.add_funds(HumanAddr::from("admin"), vec![deposit_coin.clone()]);

    assert!(deposit_coin.amount > Uint128::zero());

    // Wrap L1
    ensemble.execute(
        &snip20::HandleMsg::Deposit {
            padding: None,
        },
        MockEnv::new(
            "admin",
            token.clone(),
        ).sent_funds(vec![deposit_coin]),
    ).unwrap();

    // Deposit funds into treasury
    ensemble.execute(
        &snip20::HandleMsg::Send {
            recipient: treasury.address.clone(),
            recipient_code_hash: None,
            amount: compat::Uint128::new(deposit.u128()),
            msg: None,
            memo: None,
            padding: None,
        },
        MockEnv::new(
            "admin",
            token.clone(),
        ),
    ).unwrap();
    
    // Update treasury
    ensemble.execute(
        &adapter::HandleMsg::Adapter(
            adapter::SubHandleMsg::Update {
                asset: token.address.clone(),
            }
        ),
        MockEnv::new(
            "admin", 
            treasury.clone(),
        ),
    ).unwrap();

    // Check treasury allowance to manager
    match ensemble.query(
        treasury.address.clone(),
        &treasury::QueryMsg::Allowance {
            asset: token.address.clone(),
            spender: manager.address.clone(),
        }
    ).unwrap() {
        treasury::QueryAnswer::Allowance { amount } => {
            assert_eq!(amount, expected_allowance, "Treasury->Manager Allowance");
        },
        _ => assert!(false),
    };

    // Update manager
    ensemble.execute(
        &manager::HandleMsg::Manager(
            manager::SubHandleMsg::Update {
                asset: token.address.clone(),
            }
        ),
        MockEnv::new(
            "admin", 
            manager.clone(),
        ),
    ).unwrap();

    // Update SCRT Staking
    ensemble.execute(
        &adapter::HandleMsg::Adapter(
            adapter::SubHandleMsg::Update {
                asset: token.address.clone(),
            }
        ),
        MockEnv::new(
            "admin", 
            scrt_staking.clone(),
        ),
    ).unwrap();

    // Treasury reserves check
    match ensemble.query(
        treasury.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Reserves {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, expected_treasury, "Treasury Reserves");
        },
        _ => assert!(false),
    };

    // Manager reserves
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(
            manager::SubQueryMsg::Reserves {
                asset: token.address.clone(),
                holder: treasury.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, expected_manager, "Manager Reserves");
        },
        _ => assert!(false),
    };

    // Scrt Staking reserves should be 0 (all staked)
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Reserves {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "SCRT Staking Reserves");
        },
        _ => assert!(false),
    };

    // Scrt Staking balance check
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, expected_scrt_staking, "SCRT Staking Balance");
        },
        _ => assert!(false),
    };

    // Treasury unbondable check
    match ensemble.query(
        treasury.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Unbondable {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, expected_scrt_staking + expected_manager, "Treasury Unbondable");
        },
        _ => assert!(false),
    };

    // Unbond all w/ treasury
    ensemble.execute(
        &adapter::HandleMsg::Adapter(
            adapter::SubHandleMsg::Unbond {
                amount: expected_scrt_staking + expected_manager, 
                asset: token.address.clone(),
            }
        ),
        MockEnv::new(
            "admin", 
            treasury.clone(),
        ),
    ).unwrap();

    // scrt staking unbonding
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Unbonding {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, expected_scrt_staking, "Scrt Staking Unbonding Pre-fastforward");
        },
        _ => assert!(false),
    };

    // scrt staking claimable
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Claimable {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Scrt Staking Claimable Pre-fastforward");
        },
        _ => assert!(false),
    };

    ensemble.fast_forward_delegation_waits();

    // scrt staking unbonding
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Unbonding {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, Uint128::zero(), "Scrt Staking Unbonding Post-fastforward");
        },
        _ => assert!(false),
    };

    // scrt staking claimable
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Claimable {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, expected_scrt_staking, "Scrt Staking Claimable Post-fastforward");
        },
        _ => assert!(false),
    };

    /*
    // Claim Treasury Manager
    ensemble.execute(
        &manager::HandleMsg::Manager(
            manager::SubHandleMsg::Claim {
                asset: token.address.clone(),
            }
        ),
        MockEnv::new(
            "admin", 
            manager.clone(),
        ),
    ).unwrap();
    */

    // Claim Treasury
    ensemble.execute(
        &adapter::HandleMsg::Adapter(
            adapter::SubHandleMsg::Claim {
                asset: token.address.clone(),
            }
        ),
        MockEnv::new(
            "admin", 
            treasury.clone(),
        ),
    ).unwrap();

    // Treasury reserves check
    match ensemble.query(
        treasury.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Reserves {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, deposit, "Treasury Reserves Post-Unbond");
        },
        _ => panic!("Bad Reserves Query Response"),
    };

    /*
    // Treasury balance check
    match ensemble.query(
        treasury.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Treasury Balance Post-Unbond");
        },
        _ => assert!(false),
    };
    */

    // Scrt Staking reserves
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Reserves {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "SCRT Staking Reserves Post Unbond");
        },
        _ => assert!(false),
    };

    // Scrt Staking balance
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "SCRT Staking Balance Post Unbond");
        },
        _ => assert!(false),
    };

    // Manager balance check
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(
            manager::SubQueryMsg::Balance {
                asset: token.address.clone(),
                holder: treasury.address.clone(),
            }
        )
    ).unwrap() {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "Manager Balance Post-Claim");
        },
        _ => assert!(false),
    };





    // Treasury balance check
    match ensemble.query(
        treasury.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Treasury Balance Post-Unbond");
        },
        _ => assert!(false),
    };

    // Manager reserves check
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(
            manager::SubQueryMsg::Reserves {
                asset: token.address.clone(),
                holder: treasury.address.clone(),
            }
        )
    ).unwrap() {
        manager::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "Manager Reserves Post-Unbond");
        },
        _ => assert!(false),
    };

    // Manager balance check
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(
            manager::SubQueryMsg::Balance {
                asset: token.address.clone(),
                holder: treasury.address.clone(),
            }
        )
    ).unwrap() {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "Manager Balance Post-Unbond");
        },
        _ => assert!(false),
    };

    // Scrt Staking reserves
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Reserves {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "SCRT Staking Reserves Post Unbond");
        },
        _ => assert!(false),
    };

    // Scrt Staking balance
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "SCRT Staking Balance Post Unbond");
        },
        _ => assert!(false),
    };
}

macro_rules! single_asset_portion_manager_tests {
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
                    // expected balances
                    expected_treasury,
                    expected_manager,
                    expected_scrt_staking,
                ) = $value;
                single_asset_portion_manager_integration(
                    deposit,
                    allowance,
                    expected_allowance,
                    alloc_type,
                    alloc_amount,
                    expected_treasury,
                    expected_manager,
                    expected_scrt_staking
                );
            }
        )*
    }
}

single_asset_portion_manager_tests! {
    single_asset_portion_manager_0: (
        Uint128(100), // deposit 
        Uint128(9 * 10u128.pow(17)), // manager allowance 90%
        Uint128(90), // expected manager allowance
        AllocationType::Portion,
        Uint128(1 * 10u128.pow(18)), // allocate 100%
        Uint128(10), // treasury 10
        Uint128(0), // manager 0
        Uint128(90), // scrt_staking 90
    ),
    /*
    single_asset_portion_manager_1: (
        Uint128(100), // deposit 
        Uint128(9 * 10u128.pow(17)), // manager allowance 90%
        Uint128(90), // expected manager allowance
        AllocationType::Portion,
        Uint128(5 * 10u128.pow(17)), // 50% allocate
        Uint128(55), // treasury 55 (manager won't pull unused allowance
        Uint128(0), // manager 0
        Uint128(45), // scrt_staking 90
    ),
    */
}
