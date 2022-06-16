use cosmwasm_std::{
    coins, from_binary, to_binary,
    Extern, HumanAddr, StdError,
    Binary, StdResult, HandleResponse, Env,
    InitResponse, Uint128,
};

use secret_toolkit::snip20;

use shade_protocol::{
    contract_interfaces::{
        dao::{
            treasury_manager,
            adapter,
        },
    },
    utils::{
        asset::Contract,
        price::{normalize_price, translate_price},
    },
};

use contract_harness::{
    Treasury, TreasuryManager, ScrtStaking, Snip20ReferencImpl as Snip20,
};

use fadroma::{
    scrt::{
        ContractLink,
    },
    ensemble::{
       MockEnv, MockDeps, 
       ContractHarness, ContractEnsemble,
    },
};


/* No adapters configured
 * All assets will sit on manager unused as "reserves"
 * No need to "claim" as "unbond" will send up to "reserves"
 */
fn single_asset_holder_no_adapters(
    initial: Uint128, 
    deposit: Uint128,
) {

    let mut ensemble = ContractEnsemble::new(50);

    let reg_manager = ensemble.register(Box::new(TreasuryManager));
    let reg_snip20 = ensemble.register(Box::new(Snip20));

    let viewing_key = "unguessable".to_string();

    let token = ensemble.instantiate(
        reg_snip20.id,
        &snip20_reference_impl::msg::InitMsg {
            name: "token".into(),
            admin: Some("admin".into()),
            symbol: "TKN".into(),
            decimals: 6,
            initial_balances: Some(vec![
                snip20_reference_impl::msg::InitialBalance {
                    address: HumanAddr("holder".into()),
                    amount: initial,
                },
            ]),
            prng_seed: to_binary("").ok().unwrap(),
            config: None,
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("token".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }
        )
    ).unwrap();

    let manager = ensemble.instantiate(
        reg_manager.id,
        &treasury_manager::InitMsg {
            admin: Some(HumanAddr("admin".into())),
            treasury: HumanAddr("treasury".into()),
            viewing_key: viewing_key.clone(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("manager".into()),
                code_hash: reg_manager.code_hash,
            }
        )
    ).unwrap();

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
            holder: HumanAddr("holder".into())
        },
        MockEnv::new(
            "admin",
            manager.clone(),
        ),
    ).unwrap();

    // Deposit funds into manager
    ensemble.execute(
        &snip20::HandleMsg::Send {
            recipient: manager.address.clone(),
            recipient_code_hash: None,
            amount: deposit,
            msg: None,
            memo: None,
            padding: None,
        },
        MockEnv::new(
            "holder",
            token.clone(),
        ),
    ).unwrap();
    
    // Balance Checks

    // manager reported holder balance
    match ensemble.query(
        manager.address.clone(),
        &treasury_manager::QueryMsg::Balance {
            asset: token.address.clone(),
            holder: HumanAddr("holder".into()),
        }
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond Manager Holder Balance");
        },
        _ => assert!(false),
    };

    // manager reported treasury balance
    match ensemble.query(
        manager.address.clone(),
        &treasury_manager::QueryMsg::Balance {
            asset: token.address.clone(),
            holder: HumanAddr("treasury".into()),
        }
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "Pre-unbond Manager Treasury Balance");
        },
        _ => assert!(false),
    };

    // Manager reported total asset balance
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: token.address.clone(),
        })
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond Manager Total Balance");
        }
        _ => assert!(false),
    };

    // holder snip20 bal
    match ensemble.query(
        token.address.clone(),
        &snip20_reference_impl::msg::QueryMsg::Balance {
            address: HumanAddr("holder".into()),
            key: viewing_key.clone(),
        }
    ).unwrap() {
        snip20::AuthenticatedQueryResponse::Balance { amount } => {
            assert_eq!(amount.u128(), initial.u128() - deposit.u128(), "Pre-unbond Holder Snip20 balance");
        },
        _ => {
            assert!(false);
        }
    };

    // Unbondable
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbondable {
            asset: token.address.clone(),
        })
    ).unwrap() {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond unbondable");
        }
        _ => assert!(false),
    };

    // Reserves
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
            asset: token.address.clone(),
        })
    ).unwrap() {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond reserves");
        }
        _ => assert!(false),
    };

    let unbond_amount = Uint128(deposit.u128() / 2);

    // unbond from manager
    ensemble.execute(
        &adapter::HandleMsg::Adapter(adapter::SubHandleMsg::Unbond {
            asset: token.address.clone(),
            amount: unbond_amount,
        }),
        MockEnv::new(
            "holder",
            manager.clone(),
        ),
    ).unwrap();

    // Unbondable
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbondable {
            asset: token.address.clone(),
        })
    ).unwrap() {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, Uint128(deposit.u128() - unbond_amount.u128()), "Post-unbond total unbondable");
        }
        _ => assert!(false),
    };
    match ensemble.query(
        manager.address.clone(),
        &treasury_manager::QueryMsg::Unbondable {
            asset: token.address.clone(),
            holder: HumanAddr("holder".into()),
        }
    ).unwrap() {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, Uint128(deposit.u128() - unbond_amount.u128()), "Post-unbond holder unbondable");
        }
        _ => assert!(false),
    };

    // Unbonding
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbonding {
            asset: token.address.clone(),
        })
    ).unwrap() {
        adapter::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond total unbonding");
        }
        _ => assert!(false),
    };
    match ensemble.query(
        manager.address.clone(),
        &treasury_manager::QueryMsg::Unbonding {
            asset: token.address.clone(),
            holder: HumanAddr("holder".into()),
        }
    ).unwrap() {
        adapter::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond Holder Unbonding");
        }
        _ => assert!(false),
    };

    // Claimable (zero as its immediately claimed)
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Claimable {
            asset: token.address.clone(),
        })
    ).unwrap() {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond total claimable");
        }
        _ => assert!(false),
    };
    match ensemble.query(
        manager.address.clone(),
        //TODO should be manager query not adapter
        &treasury_manager::QueryMsg::Claimable {
            asset: token.address.clone(),
            holder: HumanAddr("holder".into()),
        }
    ).unwrap() {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond holder claimable"); 
        }
        _ => assert!(false),
    };

    // Manager reflects unbonded
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: token.address.clone(),
        }),
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
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
            address: HumanAddr("holder".into()),
            key: viewing_key.clone(),
        },
    ).unwrap() {
        snip20::AuthenticatedQueryResponse::Balance { amount } => {
            assert_eq!(amount.u128(), (initial.u128() - deposit.u128()) + unbond_amount.u128(), "Post-claim holder snip20 balance");
        },
        _ => {
            assert!(false);
        }
    };
}

macro_rules! single_asset_holder_no_adapters_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (initial, deposit) = $value;
                single_asset_holder_no_adapters(initial, deposit);
            }
        )*
    }
}
single_asset_holder_no_adapters_tests! {
    single_asset_holder_no_adapters_0: (
        Uint128(100_000_000),
        Uint128(50_000_000),
    ),
}

/* 1 dummy adapter configured
 * unbondings will need a "claim"
 */
/*
fn single_asset_holder_1_adapter(
    initial: Uint128, 
    deposit: Uint128,
) {

    let mut ensemble = ContractEnsemble::new(50);

    let reg_manager = ensemble.register(Box::new(TreasuryManager));
    let reg_snip20 = ensemble.register(Box::new(Snip20));

    let viewing_key = "unguessable".to_string();

    let token = ensemble.instantiate(
        reg_snip20.id,
        &snip20_reference_impl::msg::InitMsg {
            name: "token".into(),
            admin: Some("admin".into()),
            symbol: "TKN".into(),
            decimals: 6,
            initial_balances: Some(vec![
                snip20_reference_impl::msg::InitialBalance {
                    address: HumanAddr("holder".into()),
                    amount: initial,
                },
            ]),
            prng_seed: to_binary("").ok().unwrap(),
            config: None,
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("token".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }
        )
    ).unwrap();

    let manager = ensemble.instantiate(
        reg_manager.id,
        &treasury_manager::InitMsg {
            admin: Some(HumanAddr("admin".into())),
            treasury: HumanAddr("treasury".into()),
            viewing_key: viewing_key.clone(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("manager".into()),
                code_hash: reg_manager.code_hash,
            }
        )
    ).unwrap();

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
            holder: HumanAddr("holder".into())
        },
        MockEnv::new(
            "admin",
            manager.clone(),
        ),
    ).unwrap();

    // Deposit funds into manager
    ensemble.execute(
        &snip20::HandleMsg::Send {
            recipient: manager.address.clone(),
            recipient_code_hash: None,
            amount: deposit,
            msg: None,
            memo: None,
            padding: None,
        },
        MockEnv::new(
            "holder",
            token.clone(),
        ),
    ).unwrap();
    
    // Balance Checks

    // manager reported holder balance
    match ensemble.query(
        manager.address.clone(),
        &treasury_manager::QueryMsg::Balance {
            asset: token.address.clone(),
            holder: HumanAddr("holder".into()),
        }
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond Manager Holder Balance");
        },
        _ => assert!(false),
    };

    // manager reported treasury balance
    match ensemble.query(
        manager.address.clone(),
        &treasury_manager::QueryMsg::Balance {
            asset: token.address.clone(),
            holder: HumanAddr("treasury".into()),
        }
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "Pre-unbond Manager Treasury Balance");
        },
        _ => assert!(false),
    };

    // Manager reported total asset balance
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: token.address.clone(),
        })
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond Manager Total Balance");
        }
        _ => assert!(false),
    };

    // holder snip20 bal
    match ensemble.query(
        token.address.clone(),
        &snip20_reference_impl::msg::QueryMsg::Balance {
            address: HumanAddr("holder".into()),
            key: viewing_key.clone(),
        }
    ).unwrap() {
        snip20::AuthenticatedQueryResponse::Balance { amount } => {
            assert_eq!(amount.u128(), initial.u128() - deposit.u128(), "Pre-unbond Holder Snip20 balance");
        },
        _ => {
            assert!(false);
        }
    };

    // Unbondable
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbondable {
            asset: token.address.clone(),
        })
    ).unwrap() {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond unbondable");
        }
        _ => assert!(false),
    };

    // Reserves
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Reserves {
            asset: token.address.clone(),
        })
    ).unwrap() {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond reserves");
        }
        _ => assert!(false),
    };

    let unbond_amount = Uint128(deposit.u128() / 2);

    // unbond from manager
    ensemble.execute(
        &adapter::HandleMsg::Adapter(adapter::SubHandleMsg::Unbond {
            asset: token.address.clone(),
            amount: unbond_amount,
        }),
        MockEnv::new(
            "holder",
            manager.clone(),
        ),
    ).unwrap();

    // Unbondable
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbondable {
            asset: token.address.clone(),
        })
    ).unwrap() {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, Uint128(deposit.u128() - unbond_amount.u128()), "Post-unbond total unbondable");
        }
        _ => assert!(false),
    };
    match ensemble.query(
        manager.address.clone(),
        &treasury_manager::QueryMsg::Unbondable {
            asset: token.address.clone(),
            holder: HumanAddr("holder".into()),
        }
    ).unwrap() {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, Uint128(deposit.u128() - unbond_amount.u128()), "Post-unbond holder unbondable");
        }
        _ => assert!(false),
    };

    // Unbonding
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Unbonding {
            asset: token.address.clone(),
        })
    ).unwrap() {
        adapter::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond total unbonding");
        }
        _ => assert!(false),
    };
    match ensemble.query(
        manager.address.clone(),
        &treasury_manager::QueryMsg::Unbonding {
            asset: token.address.clone(),
            holder: HumanAddr("holder".into()),
        }
    ).unwrap() {
        adapter::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond Holder Unbonding");
        }
        _ => assert!(false),
    };

    // Claimable (zero as its immediately claimed)
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Claimable {
            asset: token.address.clone(),
        })
    ).unwrap() {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond total claimable");
        }
        _ => assert!(false),
    };
    match ensemble.query(
        manager.address.clone(),
        //TODO should be manager query not adapter
        &treasury_manager::QueryMsg::Claimable {
            asset: token.address.clone(),
            holder: HumanAddr("holder".into()),
        }
    ).unwrap() {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond holder claimable"); 
        }
        _ => assert!(false),
    };

    // Manager reflects unbonded
    match ensemble.query(
        manager.address.clone(),
        &adapter::QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: token.address.clone(),
        }),
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
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
            address: HumanAddr("holder".into()),
            key: viewing_key.clone(),
        },
    ).unwrap() {
        snip20::AuthenticatedQueryResponse::Balance { amount } => {
            assert_eq!(amount.u128(), (initial.u128() - deposit.u128()) + unbond_amount.u128(), "Post-claim holder snip20 balance");
        },
        _ => {
            assert!(false);
        }
    };
}

macro_rules! single_asset_holder_no_adapters_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (initial, deposit) = $value;
                single_asset_holder_no_adapters(initial, deposit);
            }
        )*
    }
}
single_asset_holder_no_adapters_tests! {
    single_asset_holder_no_adapters_0: (
        Uint128(100_000_000),
        Uint128(50_000_000),
    ),
}
*/
