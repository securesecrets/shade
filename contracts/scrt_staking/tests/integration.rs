use shade_protocol::math_compat as compat;
use shade_protocol::c_std::{
    to_binary,
    HumanAddr, Uint128, Coin, Decimal,
    Validator, Delegation,
};

use shade_protocol::{
    contract_interfaces::{
        dao::{
            scrt_staking,
            adapter,
        },
        snip20,
    },
    utils::{
        asset::Contract,
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
    scrt_staking::ScrtStaking,
    snip20::Snip20,
};


// Add other adapters here as they come
fn basic_scrt_staking_integration(
    deposit: Uint128, 
    rewards: Uint128,
    expected_scrt_staking: Uint128,
) {

    let viewing_key = "unguessable".to_string();

    let mut ensemble = ContractEnsemble::new(50);

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

    let scrt_staking = ensemble.instantiate(
        reg_scrt_staking.id,
        &scrt_staking::InitMsg {
            admins: None,
            owner: HumanAddr("admin".into()),
            sscrt: Contract {
                address: token.address.clone(),
                code_hash: token.code_hash.clone(),
            },
            validator_bounds: None,
            viewing_key: viewing_key.clone(),
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: HumanAddr("scrt_staking".into()),
                code_hash: reg_scrt_staking.code_hash,
            }
        )
    ).unwrap().instance;

    /*
    ensemble.add_validator(Validator {
        address: HumanAddr("validator".into()),
        commission: Decimal::zero(),
        max_commission: Decimal::one(),
        max_change_rate: Decimal::one(),
    });
    */

    // set admin owner key
    ensemble.execute(
        &snip20::HandleMsg::SetViewingKey{
            key: viewing_key.clone(),
            padding: None,
        },
        MockEnv::new(
            "admin", 
            token.clone(),
        ),
    ).unwrap();

    if !deposit.is_zero() {
        let deposit_coin = Coin { denom: "uscrt".into(), amount: deposit };
        ensemble.add_funds(HumanAddr::unchecked("admin"), vec![deposit_coin.clone()]);

        // Wrap L1 into tokens
        ensemble.execute(
            &snip20::HandleMsg::Deposit {
                padding: None,
            },
            MockEnv::new(
                "admin",
                token.clone(),
            ).sent_funds(vec![deposit_coin]),
        ).unwrap();
        //assert!(false, "deposit success");

        // Deposit funds in scrt staking
        ensemble.execute(
            &snip20::HandleMsg::Send {
                recipient: scrt_staking.address.clone(),
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

        // Delegations
        let delegations: Vec<Delegation> = ensemble.query(
            scrt_staking.address.clone(),
            &scrt_staking::QueryMsg::Delegations {},
        ).unwrap();
        assert!(!delegations.is_empty());
    }

    // reserves should be 0 (all staked)
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Reserves {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "Reserves Pre-Rewards");
        },
        _ => assert!(false),
    };

    // Balance
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Balance Pre-Rewards");
        },
        _ => assert!(false),
    };

    // Rewards
    let cur_rewards: Uint128 = ensemble.query(
        scrt_staking.address.clone(),
        &scrt_staking::QueryMsg::Rewards {},
    ).unwrap();
    assert_eq!(cur_rewards, Uint128::zero(), "Rewards Pre-add");

    //ensemble.add_rewards(rewards);

    // Rewards
    let cur_rewards: Uint128 = ensemble.query(
        scrt_staking.address.clone(),
        &scrt_staking::QueryMsg::Rewards {},
    ).unwrap();

    if deposit.is_zero() {
        assert_eq!(cur_rewards, Uint128::zero(), "Rewards Post-add");
    } else {
        assert_eq!(cur_rewards, rewards, "Rewards Post-add");
    }

    // reserves should be rewards
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Reserves {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Reserves { amount } => {
            if deposit.is_zero() {
                assert_eq!(amount, Uint128::zero(), "Reserves Post-Rewards");
            } else {
                assert_eq!(amount, rewards, "Reserves Post-Rewards");
            }
        },
        _ => assert!(false),
    };

    // Balance
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            if deposit.is_zero() {
                assert_eq!(amount, Uint128::zero(), "Balance Post-Rewards");
            } else {
                assert_eq!(amount, deposit + rewards, "Balance Post-Rewards");
            }
        },
        _ => assert!(false),
    };

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

    // reserves/rewards should be staked
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Reserves {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "Reserves Post-Update");
        },
        _ => assert!(false),
    };

    // Balance
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, expected_scrt_staking, "Balance Post-Update");
        },
        _ => assert!(false),
    };

    // Claimable
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Claimable {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Claimable Pre-Unbond");
        },
        _ => assert!(false),
    };

    // Unbondable
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Unbondable {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, expected_scrt_staking, "Unbondable Pre-Unbond");
        },
        _ => assert!(false),
    };

    // Unbond all
    ensemble.execute(
        &adapter::HandleMsg::Adapter(
            adapter::SubHandleMsg::Unbond {
                amount: expected_scrt_staking,
                asset: token.address.clone(),
            }
        ),
        MockEnv::new(
            "admin", 
            scrt_staking.clone(),
        ),
    ).unwrap();

    // Unbonding 
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Unbonding {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, expected_scrt_staking, "Unbonding Pre fast forward");
        },
        _ => assert!(false),
    };

    // Claimable
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Claimable {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Claimable Pre unbond fast forward");
        },
        _ => assert!(false),
    };

    //ensemble.fast_forward_delegation_waits();

    // Claimable
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Claimable {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Claimable { amount } => {
            if deposit.is_zero() {
                assert_eq!(amount, Uint128::zero(), "Claimable post fast forward");
            } else {
                assert_eq!(amount, deposit + rewards, "Claimable post fast forwardd");
            }
        },
        _ => assert!(false),
    };

    // Claim
    ensemble.execute(
        &adapter::HandleMsg::Adapter(
            adapter::SubHandleMsg::Claim {
                asset: token.address.clone(),
            }
        ),
        MockEnv::new(
            "admin", 
            scrt_staking.clone(),
        ),
    ).unwrap();

    // Reserves
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Reserves {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, Uint128::zero(), "Reserves Post Claim");
        },
        _ => assert!(false),
    };

    // Balance
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "Balance Post Claim");
        },
        _ => assert!(false),
    };

    // ensure wrapped tokens were returned
    match ensemble.query(
        token.address.clone(),
        &snip20::QueryMsg::Balance {
            address: HumanAddr("admin".into()),
            key: viewing_key.clone(),
        },
    ).unwrap() {
        snip20::QueryAnswer::Balance { amount } => {
            if deposit.is_zero() {
                assert_eq!(amount.u128(), 0u128, "Final User balance");
            } else {
                assert_eq!(amount.u128(), deposit.u128() + rewards.u128(), "Final user balance");
            }
        },
        _ => {
            panic!("snip20 balance query failed");
        }
    };
}

macro_rules! basic_scrt_staking_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    deposit,
                    rewards,
                    expected_scrt_staking,
                ) = $value;
                basic_scrt_staking_integration(deposit, rewards, expected_scrt_staking);
            }
        )*
    }
}

/*
basic_scrt_staking_tests! {
    basic_scrt_staking_0: (
        Uint128(100), // deposit
        Uint128(0),   // rewards
        Uint128(100), // balance
    ),
    basic_scrt_staking_1: (
        Uint128(100), // deposit
        Uint128(50),   // rewards
        Uint128(150), // balance
    ),

    basic_scrt_staking_2: (
        Uint128(0), // deposit
        Uint128(1000),   // rewards
        Uint128(0), // balance
    ),
}
*/
