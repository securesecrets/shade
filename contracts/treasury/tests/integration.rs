use cosmwasm_math_compat as compat;
use cosmwasm_std::{
    coins, from_binary, to_binary,
    Extern, HumanAddr, StdError,
    Binary, StdResult, HandleResponse, Env,
    InitResponse, Uint128,
};

use shade_protocol::{
    mint::{HandleMsg, InitMsg, QueryAnswer, QueryMsg},
    utils::{
        asset::Contract,
        price::{normalize_price, translate_price},
    },
    band::{ ReferenceData, BandQuery },
};

use snip20_reference_impl;
use oracle;
use mock_band;

use mint::{
    contract::{handle, init, query},
    handle::{calculate_mint, calculate_portion, try_burn},
};

use fadroma::{
    ContractLink, 
    ensemble::{
       MockEnv, MockDeps, 
       ContractHarness, ContractEnsemble,
    },
};

pub struct Treasury;

impl ContractHarness for Treasury {
    // Use the method from the default implementation
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        init(
            deps,
            env,
            from_binary(&msg)?,
        )
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
         handle(
            deps,
            env,
            from_binary(&msg)?,
        )
    }

    // Override with some hardcoded value for the ease of testing
    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
         query(
            deps,
            from_binary(&msg)?,
        )
    }
}

pub struct TreasuryManager;

impl ContractHarness for TreasuryManager {
    // Use the method from the default implementation
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        treasury_manager::contract::init(
            deps,
            env,
            from_binary(&msg)?,
        )
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
         treasury_manager::contract::handle(
            deps,
            env,
            from_binary(&msg)?,
        )
    }

    // Override with some hardcoded value for the ease of testing
    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
         treasury_manager::contract::query(
            deps,
            from_binary(&msg)?,
        )
    }
}

pub struct ScrtStaking;

impl ContractHarness for ScrtStaking {
    // Use the method from the default implementation
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        scrt_staking::contract::init(
            deps,
            env,
            from_binary(&msg)?,
        )
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
         scrt_staking::contract::handle(
            deps,
            env,
            from_binary(&msg)?,
        )
    }

    // Override with some hardcoded value for the ease of testing
    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
         scrt_staking::contract::query(
            deps,
            from_binary(&msg)?,
        )
    }
}

pub struct Snip20;

impl ContractHarness for Snip20 {
    // Use the method from the default implementation
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        snip20_reference_impl::contract::init(
            deps,
            env,
            from_binary(&msg)?,
            //mint::DefaultImpl,
        )
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
         snip20_reference_impl::contract::handle(
            deps,
            env,
            from_binary(&msg)?,
            //mint::DefaultImpl,
        )
    }

    // Override with some hardcoded value for the ease of testing
    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
         snip20_reference_impl::contract::query(
            deps,
            from_binary(&msg)?,
            //mint::DefaultImpl,
        )
    }
}

//fn treasury_base(
//fn manager_integration(

// Add other adapters here as they come
fn single_asset_full_dao_integration(
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
        &shade_protocol::treasury::InitMsg {
            admin: Some(HumanAddr("admin".into())),
            viewing_key: "viewing_key",
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
        &shade_protocol::treasury_manager::InitMsg {
            admin: Some(HumanAddr("admin".into())),
            treasury: HumanAddr("treasury".into()),
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
        &shade_protocol::scrt_staking::InitMsg {
            admin: Some(HumanAddr("admin".into())),
            treasury: HumanAddr("treasury".into()),
            sscrt: Contract {
                token.address.clone(),
                token.code_hash.clone(),
            }
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
        &shade_protocol::treasury::HandleMsg::RegisterAsset {
            contract: Contract {
                address: token.address.clone(),
                code_hash: token.code_hash.clone(),
            },
            // unused?
            reserves: Uint128::zero(),
        },
        MockEnv::new(
            "admin", 
            treasury.clone(),
        ),
    ).unwrap();
    
    // Register manager assets
    ensemble.execute(
        &shade_protocol::treasury_manager::HandleMsg::RegisterAsset {
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
        &shade_protocol::treasury::HandleMsg::RegisterManager {
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

    // Register scrt_staking -> manager

    // treasury allowance to manager

    // manager allocation to scrt_staking

    // Deposit funds into treasury
    //ensemble.execute();
    
    //rebalance/update treasury
    //rebalance/update manager
    //check balances are expected
    
   
}

macro_rules! single_asset_full_dao_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (offer_price, offer_amount, mint_price, expected_amount) = $value;
                let (
                    deposit: Uint128, 
                    allowance: Uint128,
                    allocation: Uint128,
                    // expected balances
                    expected_treasury: Uint128,
                    expected_manager: Uint128,
                    expected_scrt_staking: Uint128,
                ) = $value;
                single_asset_full_dao(deposit, allowance, allocation, expected_treasury, expected_manager, expected_scrt_staking);
            }
        )*
    }
}
single_asset_full_dao_tests! {
    single_asset_full_dao_0: (
        Uint128(100), // deposit 
        Uint128(90 * 10.pow(18)), // allow 90%
        Uint128(100 * 10.pow(18)), // allocate 100%
        Uint128(10), // treasury 10
        Uint128(0), // manager 0
        Uint128(90), // scrt_staking 90
    ),
}
