use cosmwasm_math_compat as compat;
use cosmwasm_std::{
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
};

use contract_harness::harness::{
    scrt_staking::ScrtStaking,
    snip20_reference_impl::Snip20ReferenceImpl as Snip20,
    //snip20::Snip20,
};

use fadroma::{
    core::ContractLink,
    ensemble::{
       MockEnv,
       ContractHarness,
       ContractEnsemble,
    },
};

// Add other adapters here as they come
fn basic_scrt_staking_integration(
    deposit: Uint128, 
    rewards: Uint128,
    expected_scrt_staking: Uint128,
) {

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

    let deposit_coin = Coin { denom: "uscrt".into(), amount: deposit };
    ensemble.add_funds(HumanAddr::from("admin"), vec![deposit_coin.clone()]);

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

    // Delegations
    let delegations: Vec<Delegation> = ensemble.query(
        scrt_staking.address.clone(),
        &scrt_staking::QueryMsg::Delegations {},
    ).unwrap();
    assert!(!delegations.is_empty());

    // Rewards
    let cur_rewards: Uint128 = ensemble.query(
        scrt_staking.address.clone(),
        &scrt_staking::QueryMsg::Rewards {},
    ).unwrap();
    assert_eq!(cur_rewards, Uint128::zero(), "Rewards Pre-add");

    ensemble.add_rewards(rewards);

    // Rewards
    let cur_rewards: Uint128 = ensemble.query(
        scrt_staking.address.clone(),
        &scrt_staking::QueryMsg::Rewards {},
    ).unwrap();
    assert_eq!(cur_rewards, rewards, "Rewards Post-add");

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
            assert_eq!(amount, rewards, "Reserves Post-Rewards");
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
            assert_eq!(amount, expected_scrt_staking, "Balance Post-Rewards");
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

    ensemble.fast_forward_delegation_waits();

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
            assert_eq!(amount, expected_scrt_staking, "Claimable Post unbond fast forward");
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
}
