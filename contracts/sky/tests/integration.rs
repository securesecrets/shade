//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests.
//! 1. First copy them over verbatum,
//! 2. Then change
//!      let mut deps = mock_dependencies(20, &[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)

use cosmwasm_math_compat::Uint128;
use contract_harness::harness::{
    snip20::Snip20, 
    sky::Sky, 
    oracle::Oracle, 
    mint::Mint, 
    mock_band::MockBand,
    sienna_exchange::SiennaExchange,
};
use fadroma::{
    ensemble::{ContractEnsemble, MockEnv},
    ContractLink,
};
use cosmwasm_math_compat as compat;
use cosmwasm_std::{
    self,
    coins,
    from_binary,
    to_binary,
    Binary,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    InitResponse,
    StdError,
    StdResult,
};
use shade_protocol::{
    utils::asset::{Contract},
    contract_interfaces::{
        snip20::{self},
        sky::sky::{self},
    },
};
fn test_ensemble_sky(
    swap_amount: Uint128,
){
    let mut ensemble = ContractEnsemble::new(50);

    let reg_sky = ensemble.register(Box::new(Sky));
    let reg_snip20 = ensemble.register(Box::new(Snip20));
    let reg_oracle = ensemble.register(Box::new(Oracle));
    let reg_mint = ensemble.register(Box::new(Mint));
    let reg_band = ensemble.register(Box::new(MockBand));
    let reg_sienna_exchange = ensemble.register(Box::new(SiennaExchange));

    println!("Deploying sscrt contract");

    let sscrt = ensemble.instantiate(
        reg_snip20.id,
        &snip20_reference_impl::msg::InitMsg {
            name: "secretSCRT".into(),
            admin: Some(HumanAddr("admin".into())),
            symbol: "SSCRT".into(),
            decimals: 6,
            initial_balances: Some(vec![snip20_reference_impl::msg::InitialBalance {
                address: HumanAddr("admin".into()),
                amount: cosmwasm_std::Uint128(100000000000), // 100,000 SSCRT
            }]),
            prng_seed: to_binary("").ok().unwrap(),
            config: None,
        },
        MockEnv::new("admin", ContractLink {
            address: HumanAddr("sscrt".into()),
            code_hash: reg_snip20.code_hash.clone(),
        }),
    ).unwrap();

    println!("Sscrt contract addr: {}", sscrt.address);
    println!("Deploying sky contract");

    let sky = ensemble.instantiate(
        reg_sky.id,
        &sky::InitMsg {
            admin: Some(HumanAddr("admin".into())),
            mint_addr_shd: Contract {
                address: HumanAddr("admin".into()),
                code_hash: "".to_string(),
            },
            mint_addr_silk: Contract{
                address: HumanAddr("admin".into()),
                code_hash: "".to_string(),
            },
            market_swap_addr:  Contract{
                address: HumanAddr("admin".into()),
                code_hash: "".to_string(),
            },
            shd_token: sky::TokenContract {
                contract: Contract{
                    address: sscrt.address.clone(),
                    code_hash: sscrt.code_hash.clone(),
                },
                decimals: Uint128::new(8),
            },
            silk_token: sky::TokenContract {
                contract: Contract{
                    address: sscrt.address.clone(),
                    code_hash: sscrt.code_hash.clone(),
                },
                decimals: Uint128::new(6),
            },
            treasury:HumanAddr("admin".into()),
            viewing_key: String::from("key"),
            limit: None,
        },
        MockEnv::new("admin", ContractLink {
            address: HumanAddr("sky".into()),
            code_hash: reg_snip20.code_hash.clone(),
        }),
    ).unwrap();

    println!("Sky contract addr: {}", sky.address);
    println!("Deploying shd contract");

    let shd = ensemble.instantiate(
        reg_snip20.id,
        &snip20_reference_impl::msg::InitMsg {
            name: "Shade".into(),
            admin: Some(HumanAddr("admin".into())),
            symbol: "SHD".into(),
            decimals: 8,
            initial_balances: Some(vec![snip20_reference_impl::msg::InitialBalance {
                address: HumanAddr("admin".into()),
                amount: cosmwasm_std::Uint128(10000000000000), // 100,000 SHD
            },
            snip20_reference_impl::msg::InitialBalance {
                address: sky.address.clone(),
                amount: cosmwasm_std::Uint128(10000000000000), // 100,000 SHD
            },]),
            prng_seed: to_binary("").ok().unwrap(),
            config: None,
        },
        MockEnv::new("admin", ContractLink {
            address: HumanAddr("shd".into()),
            code_hash: reg_snip20.code_hash.clone(),
        }),
    ).unwrap();

    println!("Shd contract addr: {}", shd.address);
    println!("Deploying silk contract");

    let silk = ensemble.instantiate(
        reg_snip20.id,
        &snip20_reference_impl::msg::InitMsg {
            name: "Silk".into(),
            admin: Some(HumanAddr("admin".into())),
            symbol: "SILK".into(),
            decimals: 6,
            initial_balances: Some(vec![snip20_reference_impl::msg::InitialBalance {
                address: HumanAddr("admin".into()),
                amount: cosmwasm_std::Uint128(100000000000), // 100,000 SILK
            },
            snip20_reference_impl::msg::InitialBalance {
                address: sky.address.clone(),
                amount: cosmwasm_std::Uint128(100000000000), // 100,000 SILK
            }]),
            prng_seed: to_binary("").ok().unwrap(),
            config: None,
        },
        MockEnv::new("admin", ContractLink {
            address: HumanAddr("silk".into()),
            code_hash: reg_snip20.code_hash.clone(),
        }),
    ).unwrap();

    println!("Silk contract addr: {}", silk.address);

    let key = String::from("key");

    ensemble.execute(
        &snip20_reference_impl::msg::HandleMsg::SetViewingKey { 
            key: key.clone(), 
            padding: None, 
        },
        MockEnv::new("admin", sscrt.clone()),
    ).unwrap();

    ensemble.execute(
        &snip20_reference_impl::msg::HandleMsg::SetViewingKey { 
            key: key.clone(), 
            padding: None, 
        },
        MockEnv::new("admin", shd.clone()),
    ).unwrap();

    ensemble.execute(
        &snip20_reference_impl::msg::HandleMsg::SetViewingKey { 
            key: key.clone(), 
            padding: None, 
        },
        MockEnv::new("admin", silk.clone()),
    ).unwrap();

    let mut query_res = ensemble.query(
        sscrt.address.clone(),
        snip20::QueryMsg::Balance { 
            address: "admin".into(), 
            key: key.clone(), 
        }
    ).unwrap();

    match query_res {
        snip20::QueryAnswer::Balance { 
            amount 
        } => {
            assert_eq!(amount, Uint128::new(100000000000))
        }
        _=> {
            assert!(false)
        }
    }

    query_res = ensemble.query(
        shd.address.clone(),
        snip20::QueryMsg::Balance { 
            address: "admin".into(), 
            key: key.clone(), 
        }
    ).unwrap();

    match query_res {
        snip20::QueryAnswer::Balance { 
            amount 
        } => {
            assert_eq!(amount, Uint128::new(10000000000000))
        }
        _=> {
            assert!(false)
        }
    }

    query_res = ensemble.query(
        silk.address.clone(),
        snip20::QueryMsg::Balance { 
            address: "admin".into(), 
            key: key.clone(), 
        }
    ).unwrap();

    match query_res {
        snip20::QueryAnswer::Balance { 
            amount 
        } => {
            assert_eq!(amount, Uint128::new(100000000000))
        }
        _=> {
            assert!(false)
        }
    }

    ensemble.execute(
        &sky::HandleMsg::UpdateConfig { 
            config: sky::Config {
                admin: HumanAddr("admin".into()),
                mint_addr_shd: Contract {
                    address: HumanAddr("admin".into()),
                    code_hash: "".to_string(),
                },
                mint_addr_silk: Contract{
                    address: HumanAddr("admin".into()),
                    code_hash: "".to_string(),
                },
                market_swap_addr:  Contract{
                    address: HumanAddr("admin".into()),
                    code_hash: "".to_string(),
                },
                shd_token: sky::TokenContract {
                    contract: Contract{
                        address: shd.address.clone(),
                        code_hash: shd.code_hash.clone(),
                    },
                    decimals: Uint128::new(8),
                },
                silk_token: sky::TokenContract {
                    contract: Contract{
                        address: silk.address.clone(),
                        code_hash: silk.code_hash.clone(),
                    },
                    decimals: Uint128::new(6),
                },
                treasury:HumanAddr("admin".into()),
                limit: None,
            }
        },
        MockEnv::new("admin", sky.clone()),
    ).unwrap();

    let mut sky_res = ensemble.query(
        sky.address.clone(),
        sky::QueryMsg::Balance {  },
    ).unwrap();
    
    match sky_res {
        sky::QueryAnswer::Balance { 
            error_status, 
            shd_bal, 
            silk_bal
        } => { 
            assert!(!error_status);
            assert_eq!(shd_bal, Uint128::new(10000000000000)); 
            assert_eq!(silk_bal, Uint128::new(100000000000)); 
        }
        _=>{ assert!(false) }
    }

    let band = ensemble
        .instantiate(
            reg_band.id,
            &shade_protocol::contract_interfaces::oracles::band::InitMsg {},
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("band".into()),
                code_hash: reg_band.code_hash.clone(),
            }),
        )
        .unwrap();

    let oracle = ensemble
        .instantiate(
            reg_oracle.id,
            &shade_protocol::contract_interfaces::oracles::oracle::InitMsg {
                admin: Some(HumanAddr("admin".into())),
                band: Contract {
                    address: band.address.clone(),
                    code_hash: band.code_hash.clone(),
                },
                sscrt: Contract {
                    address: sscrt.address.clone(),
                    code_hash: sscrt.code_hash.clone(),
                },
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("oracle".into()),
                code_hash: reg_oracle.code_hash.clone(),
            }),
        )
        .unwrap();

    let mint_silk = ensemble
        .instantiate(
            reg_mint.id,
            &shade_protocol::contract_interfaces::mint::mint::InitMsg {
                admin: Some(HumanAddr("admin".into())),
                oracle: Contract {
                    address: oracle.address.clone(),
                    code_hash: oracle.code_hash.clone(),
                },
                native_asset: Contract {
                    address: shd.address.clone(),
                    code_hash: shd.code_hash.clone(),
                },
                peg: None,
                treasury: HumanAddr("admin".into()),
                secondary_burn: None,
                limit: None,
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("mint_silk".into()),
                code_hash: reg_mint.code_hash.clone(),
            }),
        )
        .unwrap();

    let mint_shd = ensemble
        .instantiate(
            reg_mint.id,
            &shade_protocol::contract_interfaces::mint::mint::InitMsg {
                admin: Some(HumanAddr("admin".into())),
                oracle: Contract {
                    address: oracle.address.clone(),
                    code_hash: oracle.code_hash.clone(),
                },
                native_asset: Contract {
                    address: silk.address.clone(),
                    code_hash: silk.code_hash.clone(),
                },
                peg: None,
                treasury: HumanAddr("admin".into()),
                secondary_burn: None,
                limit: None,
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("mint_shd".into()),
                code_hash: reg_mint.code_hash,
            }),
        )
        .unwrap();

    ensemble.execute(
        &mock_band::contract::HandleMsg::MockPrice {
            symbol: "SHD".into(), 
            price: Uint128::new(20),
        },
        MockEnv::new("admin", band.clone()),
    ).unwrap();

    ensemble.execute(
        &mock_band::contract::HandleMsg::MockPrice {
            symbol: "SILK".into(), 
            price: Uint128::new(20),
        },
        MockEnv::new("admin", band.clone()),
    ).unwrap();

    ensemble.execute(
        &sky::HandleMsg::UpdateConfig { 
            config: sky::Config {
                admin: HumanAddr("admin".into()),
                mint_addr_shd: Contract {
                    address: mint_shd.address,
                    code_hash: mint_shd.code_hash,
                },
                mint_addr_silk: Contract{
                    address: mint_silk.address,
                    code_hash: mint_silk.code_hash,
                },
                market_swap_addr:  Contract{
                    address: HumanAddr("admin".into()),
                    code_hash: "".to_string(),
                },
                shd_token: sky::TokenContract {
                    contract: Contract{
                        address: shd.address.clone(),
                        code_hash: shd.code_hash.clone(),
                    },
                    decimals: Uint128::new(8),
                },
                silk_token: sky::TokenContract {
                    contract: Contract{
                        address: silk.address.clone(),
                        code_hash: silk.code_hash.clone(),
                    },
                    decimals: Uint128::new(6),
                },
                treasury:HumanAddr("admin".into()),
                limit: None,
            }
        },
        MockEnv::new("admin", sky.clone()),
    ).unwrap();
}

macro_rules! sky_int_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (swap_amount,) = $value;
                test_ensemble_sky(swap_amount);
            }
        )*
    }
}

sky_int_tests! {
    sky_int_0: (
        Uint128::zero(),
    ),
}
