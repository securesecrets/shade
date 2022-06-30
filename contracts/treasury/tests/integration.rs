use cosmwasm_math_compat as compat;
use cosmwasm_std::{
    to_binary,
    HumanAddr, Uint128, Coin,
};

use shade_protocol::{
    contract_interfaces::{
        dao::{
            treasury,
            treasury_manager,
            //scrt_staking,
            adapter,
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
    //scrt_staking::ScrtStaking,
    //snip20_reference_impl::Snip20ReferenceImpl as Snip20,
    snip20::Snip20,
};

use fadroma::{
    core::ContractLink,
    ensemble::{
       MockEnv, 
       ContractHarness, ContractEnsemble,
    },
};

//fn treasury_base(
//fn manager_integration(

// Add other adapters here as they come
fn single_asset_portion_manager_integration(
    deposit: Uint128, 
    allowance: Uint128,
    allocation: Uint128,
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

    ensemble.execute(
        &treasury::HandleMsg::RegisterAsset {
            contract: Contract {
                address: token.address.clone(),
                code_hash: token.code_hash.clone(),
            },
            // unused?
            reserves: None,
        },
        MockEnv::new(
            "admin", 
            treasury.clone(),
        ),
    ).unwrap();

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

    // Register treasury assets
    ensemble.execute(
        &treasury::HandleMsg::RegisterAsset {
            contract: Contract {
                address: token.address.clone(),
                code_hash: token.code_hash.clone(),
            },
            // unused?
            reserves: Some(Uint128::zero()),
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

    // Register manager -> treasury
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

    //update manager
    ensemble.execute(
        &treasury::HandleMsg::Adapter(
            adapter::SubHandleMsg::Update {
                asset: token.address.clone(),
            }
        ),
        MockEnv::new(
            "admin", 
            manager.clone(),
        ),
    ).unwrap();

    // Treasury balance check
    match ensemble.query(
        treasury.address.clone(),
        &treasury::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, expected_treasury, "Treasury Balance");
        },
        _ => assert!(false),
    };

    // Manager balance check
    match ensemble.query(
        manager.address.clone(),
        &treasury_manager::QueryMsg::Adapter(
            adapter::SubQueryMsg::Balance {
                asset: token.address.clone(),
            }
        )
    ).unwrap() {
        adapter::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, expected_manager, "Manager Balance");
        },
        _ => assert!(false),
    };

    ensemble.execute(
        &treasury::HandleMsg::Adapter(
            adapter::SubHandleMsg::Unbond {
                amount: 
                asset: token.address.clone(),
            }
        ),
        MockEnv::new(
            "admin", 
            manager.clone(),
        ),
    ).unwrap();
}

macro_rules! single_asset_portion_manager_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    deposit,
                    allowance,
                    allocation,
                    // expected balances
                    expected_treasury,
                    expected_manager,
                    expected_scrt_staking,
                ) = $value;
                single_asset_portion_manager_integration(deposit, allowance, allocation, expected_treasury, expected_manager, expected_scrt_staking);
            }
        )*
    }
}

single_asset_portion_manager_tests! {
    single_asset_portion_manager_0: (
        Uint128(100), // deposit 
        Uint128(9 * 10u128.pow(17)), // allow 90%
        Uint128(1 * 10u128.pow(18)), // allocate 100%
        Uint128(10), // treasury 10
        Uint128(0), // manager 0
        Uint128(90), // scrt_staking 90
    ),
}
