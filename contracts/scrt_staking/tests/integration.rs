use cosmwasm_math_compat as compat;
use cosmwasm_std::{
    to_binary,
    HumanAddr, Uint128, Coin, Decimal,
    Validator,
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
fn single_asset_portion_manager_integration(
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

    // Deposit funds
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
    
    /*
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
    */

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
            assert_eq!(amount, Uint128::zero(), "Reserves Pre-Unbond");
        },
        _ => assert!(false),
    };

    // balance check
    match ensemble.query(
        scrt_staking.address.clone(),
        &adapter::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, expected_scrt_staking, "Balance Pre-Unbond");
        },
        _ => assert!(false),
    };

    // claimable check
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
            assert_eq!(amount, expected_scrt_staking, "Claimable Pre-Unbond");
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

macro_rules! single_asset_portion_manager_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    deposit,
                    rewards,
                    expected_scrt_staking,
                ) = $value;
                single_asset_portion_manager_integration(deposit, rewards, expected_scrt_staking);
            }
        )*
    }
}

single_asset_portion_manager_tests! {
    single_asset_portion_manager_0: (
        Uint128(100), // deposit
        Uint128(0),   // rewards
        Uint128(100), // balance 90
    ),
}
