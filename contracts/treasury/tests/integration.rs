use cosmwasm_math_compat as compat;
use cosmwasm_std::{
    coins, from_binary, to_binary,
    Extern, HumanAddr, StdError,
    Binary, StdResult, HandleResponse, Env,
    InitResponse, Uint128,
};

use shade_protocol::{
    contract_interfaces::{
        dao::{
            treasury,
            treasury_manager,
            scrt_staking,
        },
        mint::mint::{HandleMsg, InitMsg, QueryAnswer, QueryMsg},
        oracles::band::{ ReferenceData, BandQuery },
    },
    utils::{
        asset::Contract,
        price::{normalize_price, translate_price},
    },
};

use contract_harness::harness::{
    Treasury, TreasuryManager, ScrtStaking, Snip20,
};

use fadroma::{
    scrt::ContractLink,
    ensemble::{
       MockEnv, MockDeps, 
       ContractHarness, ContractEnsemble,
    },
};

//fn treasury_base(
//fn manager_integration(

// Add other adapters here as they come
fn single_asset_portion_full_dao_integration(
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
        &snip20_reference_impl::msg::InitMsg {
            name: "secretSCRT".into(),
            admin: Some("admin".into()),
            symbol: "SSCRT".into(),
            decimals: 6,
            initial_balances: None,
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
    ).unwrap();

    let scrt_staking = ensemble.instantiate(
        reg_scrt_staking.id,
        &scrt_staking::InitMsg {
            admins: Some(vec![HumanAddr("admin".into())]),
            owner: HumanAddr("manager".into()),
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
    ).unwrap();


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

    // Allocate scrt_staking -> manager
    ensemble.execute(
        &treasury_manager::HandleMsg::Allocate {
            asset: token.address.clone(),
            allocation: treasury_manager::Allocation {
                nick: Some("sSCRT Staking".to_string()),
                contract: Contract {
                    address: scrt_staking.address.clone(),
                    code_hash: scrt_staking.code_hash.clone(),
                },
                alloc_type: treasury_manager::AllocationType::Portion,
                amount: allocation,
                tolerance: Uint128::zero(),
            },
        },
        MockEnv::new(
            "admin", 
            manager.clone(),
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

    // Deposit funds into treasury
    //ensemble.execute();
    
    //rebalance/update treasury
    //rebalance/update manager
    //check balances are expected
}

macro_rules! single_asset_portion_full_dao_tests {
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
                single_asset_portion_full_dao_integration(deposit, allowance, allocation, expected_treasury, expected_manager, expected_scrt_staking);
            }
        )*
    }
}
single_asset_portion_full_dao_tests! {
    single_asset_portion_full_dao_0: (
        Uint128(100), // deposit 
        Uint128(9 * 10u128.pow(17)), // allow 90%
        Uint128(1 * 10u128.pow(18)), // allocate 100%
        Uint128(10), // treasury 10
        Uint128(0), // manager 0
        Uint128(90), // scrt_staking 90
    ),
}
