
use cosmwasm_math_compat as compat;

//use secret_toolkit::snip20;
//use snip20_reference_impl::msg as snip20;

use shade_protocol::{
    contract_interfaces::{
        dao::{
            treasury_manager::{
                self,
                Allocation,
                AllocationType,
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
    c_std::{
        to_binary, Addr, Uint128,
        Decimal, Validator,
        Coin,
    },
    fadroma::{
        core::ContractLink,
        ensemble::{
           MockEnv,
           ContractHarness,
           ContractEnsemble,
        },
    },
};

use contract_harness::harness::{
    treasury_manager::TreasuryManager,
    scrt_staking::ScrtStaking,
    //snip20_reference_impl::Snip20ReferenceImpl as Snip20,
    snip20::Snip20,
};

/* No adapters configured
 * All assets will sit on manager unused as "reserves"
 * No need to "claim" as "unbond" will send up to "reserves"
 */
fn single_holder_scrt_staking_adapter(
    deposit: Uint128,
    alloc_type: AllocationType,
    alloc_amount: Uint128,
    expected_manager: Uint128,
    expected_scrt_staking: Uint128,
    unbond_amount: Uint128,
) {

    let mut ensemble = ContractEnsemble::new(50);

    /*
    ensemble.add_validator(Validator {
        address: Addr("validator".into()),
        commission: Decimal::zero(),
        max_commission: Decimal::one(),
        max_change_rate: Decimal::one(),
    });
    */

    let reg_manager = ensemble.register(Box::new(TreasuryManager));
    let reg_snip20 = ensemble.register(Box::new(Snip20));
    let reg_scrt_staking = ensemble.register(Box::new(ScrtStaking));

    let viewing_key = "unguessable".to_string();

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
                address: Addr("token".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }
        )
    ).unwrap().instance;

    let manager = ensemble.instantiate(
        reg_manager.id,
        &treasury_manager::InitMsg {
            admin: Some(Addr("admin".into())),
            treasury: Addr("treasury".into()),
            viewing_key: viewing_key.clone(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: Addr("manager".into()),
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
                address: Addr("scrt_staking".into()),
                code_hash: reg_scrt_staking.code_hash,
            }
        )
    ).unwrap().instance;

    // set holder viewing key
    ensemble.execute(
        &snip20::HandleMsg::SetViewingKey{
            key: viewing_key.clone(),
            padding: None,
        },
        MockEnv::new(
            "holder", 
            token.clone(),
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

    // Add 'holder' as holder
    ensemble.execute(
        &treasury_manager::HandleMsg::AddHolder {
            holder: Addr("holder".into())
        },
        MockEnv::new(
            "admin",
            manager.clone(),
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
    ensemble.add_funds(Addr::unchecked("holder"), vec![deposit_coin.clone()]);

    assert!(deposit_coin.amount > Uint128::zero());

    // Wrap L1
    ensemble.execute(
        &snip20::HandleMsg::Deposit {
            padding: None,
        },
        MockEnv::new(
            "holder",
            token.clone(),
        ).sent_funds(vec![deposit_coin]),
    ).unwrap();

    // Deposit funds into manager
    ensemble.execute(
        &snip20::HandleMsg::Send {
            recipient: manager.address.clone(),
            recipient_code_hash: None,
            amount: compat::Uint128::new(deposit.u128()),
            msg: None,
            memo: None,
            padding: None,
        },
        MockEnv::new(
            "holder",
            token.clone(),
        ),
    ).unwrap();

    // Update manager
    ensemble.execute(
        &manager::HandleMsg::Manager(
            manager::SubHandleMsg::Update {
                asset: token.address.clone(),
            }
        ),
        MockEnv::new(
            "holder",
            manager.clone(),
        ),
    ).unwrap();
    
    // Balance Checks

    // manager reported holder balance
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(
            manager::SubQueryMsg::Balance {
                asset: token.address.clone(),
                holder: Addr("holder".into()),
            }
        )
    ).unwrap() {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond Manager Holder Balance");
        },
        _ => assert!(false),
    };

    // manager reported treasury balance
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
            asset: token.address.clone(),
            holder: Addr("treasury".into()),
        })
    ).unwrap() {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "Pre-unbond Manager Treasury Balance");
        },
        _ => assert!(false),
    };

    // scrt staking balance
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, expected_scrt_staking, "Pre-unbond scrt staking balance");
        },
        _ => assert!(false),
    };

    // manager unbondable
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(manager::SubQueryMsg::Unbondable {
            asset: token.address.clone(),
            holder: Addr("holder".into()),
        })
    ).unwrap() {
        manager::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond unbondable");
        }
        _ => assert!(false),
    };

    let mut reserves = Uint128::zero();

    // Reserves
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(manager::SubQueryMsg::Reserves {
            asset: token.address.clone(),
            holder: Addr("holder".into()),
        })
    ).unwrap() {
        manager::QueryAnswer::Reserves { amount } => {
            reserves = amount;
            assert_eq!(amount, expected_manager, "Pre-unbond reserves");
        }
        _ => assert!(false),
    };

    // Claimable
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(manager::SubQueryMsg::Claimable {
            asset: token.address.clone(),
            holder: Addr("holder".into()),
        })
    ).unwrap() {
        manager::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Pre-unbond claimable");
        }
        _ => assert!(false),
    };

    // holder unbond from manager
    ensemble.execute(
        &manager::HandleMsg::Manager(
            manager::SubHandleMsg::Unbond {
                asset: token.address.clone(),
                amount: unbond_amount,
            }
        ),
        MockEnv::new(
            "holder",
            manager.clone(),
        ),
    ).unwrap();

    // scrt staking Unbondable
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Unbondable {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, Uint128(deposit.u128() - unbond_amount.u128()), "Post-unbond scrt staking unbondable");
        }
        _ => assert!(false),
    };

    // manager Unbondable
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(
            manager::SubQueryMsg::Unbondable {
                asset: token.address.clone(),
                holder: Addr("holder".into()),
            }
        )
    ).unwrap() {
        manager::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, Uint128(deposit.u128() - unbond_amount.u128()), "Post-unbond manager unbondable");
        }
        _ => assert!(false),
    };

    // Unbonding
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(
            manager::SubQueryMsg::Unbonding {
                asset: token.address.clone(),
                holder: Addr("holder".into()),
            }
        )
    ).unwrap() {
        manager::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, Uint128(unbond_amount.u128() - reserves.u128()), "Post-unbond manager unbonding");
        }
        _ => assert!(false),
    };

    // Manager Claimable
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(
            manager::SubQueryMsg::Claimable {
                asset: token.address.clone(),
                holder: Addr("holder".into()),
            }
        )
    ).unwrap() {
        manager::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Pre-fastforward manager claimable");
        }
        _ => assert!(false),
    };

    //ensemble.fast_forward_delegation_waits();

    // Scrt Staking Claimable
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Claimable {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128(unbond_amount.u128() - reserves.u128()), "Post-fastforward scrt staking claimable");
        }
        _ => assert!(false),
    };

    // Manager Claimable
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(
            manager::SubQueryMsg::Claimable {
                asset: token.address.clone(),
                holder: Addr("holder".into()),
            }
        )
    ).unwrap() {
        manager::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128(unbond_amount.u128() - reserves.u128()), "Post-fastforward manager claimable");
        }
        _ => assert!(false),
    };

    // Claim
    ensemble.execute(
        &manager::HandleMsg::Manager(
            manager::SubHandleMsg::Claim {
                asset: token.address.clone(),
            }
        ),
        MockEnv::new(
            "holder",
            manager.clone(),
        ),
    ).unwrap();

    // Manager
    match ensemble.query(
        manager.address.clone(),
        &manager::QueryMsg::Manager(
            manager::SubQueryMsg::Balance {
                asset: token.address.clone(),
                holder: Addr("holder".into()),
            }
        ),
    ).unwrap() {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount.u128(), deposit.u128() - unbond_amount.u128());
        }
        _ => {
            assert!(false);
        }
    };

    // user received unbonded
    match ensemble.query(
        token.address.clone(),
        &snip20_reference_impl::msg::QueryMsg::Balance {
            address: Addr("holder".into()),
            key: viewing_key.clone(),
        },
    ).unwrap() {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount.u128(), unbond_amount.u128(), "Post-claim holder snip20 balance");
        },
        _ => {
            assert!(false);
        }
    };
}

macro_rules! single_holder_scrt_staking_adapter_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (deposit, alloc_type, alloc_amount, 
                     expected_scrt_staking, expected_manager, unbond_amount) = $value;
                single_holder_scrt_staking_adapter(deposit, alloc_type, alloc_amount, expected_scrt_staking, expected_manager, unbond_amount);
            }
        )*
    }
}
/*
single_holder_scrt_staking_adapter_tests! {
    single_holder_scrt_staking_0: (
        // 100
        Uint128(100_000_000),
        // % 50 alloc
        AllocationType::Portion,
        Uint128(5u128 * 10u128.pow(17)),
        // 50/50
        Uint128(50_000_000),
        Uint128(50_000_000),
        // unbond 75
        Uint128(75_000_000),
    ),
}
*/
