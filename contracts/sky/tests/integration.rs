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

/*use contract_harness::harness::snip20::Snip20;
use cosmwasm_math_compat as compat;
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{
    self,
    coins,
    from_binary,
    to_binary,
    Binary,
    Env,
    Extern,
    HandleResponse,
    Addr,
    InitResponse,
    StdError,
    StdResult,
};
use fadroma::{
    ensemble::{ContractEnsemble, MockEnv},
    prelude::{Callback, ContractInstantiationInfo, ContractLink},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::contract_interfaces::{
    dex::{self, shadeswap},
    snip20::{self},
};

fn test_ensemble_sky(swap_amount: Uint128) {
    let mut ensemble = ContractEnsemble::new(50);

    let reg_snip20 = ensemble.register(Box::new(Snip20));

    //let reg_mock_shdswp = ensemble.register(Box::new(MockShdSwp));
    //let reg_shadeswap_exchange = ensemble.register(Box::new(ShadeswapExchange));
    //let reg_shadeswap_factory = ensemble.register(Box::new(ShadeswapFactory));
    //let reg_sienna_lp_token = ensemble.register(Box::new(SiennaLpToken));

    println!("Deploying sscrt contract");

    let sscrt = ensemble
        .instantiate(
            reg_snip20.id,
            &snip20_reference_impl::msg::InitMsg {
                name: "secretSCRT".into(),
                admin: Some(Addr("admin".into())),
                symbol: "SSCRT".into(),
                decimals: 6,
                initial_balances: Some(vec![snip20_reference_impl::msg::InitialBalance {
                    address: Addr("admin".into()),
                    amount: cosmwasm_std::Uint128(100000000000), // 100,000 SSCRT
                }]),
                prng_seed: to_binary("").ok().unwrap(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: Addr("sscrt".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }),
        )
        .unwrap();

    println!("Sscrt contract addr: {}", sscrt.instance.address);
    println!("Deploying shd contract");

    let shd = ensemble
        .instantiate(
            reg_snip20.id,
            &snip20_reference_impl::msg::InitMsg {
                name: "Shade".into(),
                admin: Some(Addr("admin".into())),
                symbol: "SHD".into(),
                decimals: 8,
                initial_balances: Some(vec![snip20_reference_impl::msg::InitialBalance {
                    address: Addr("admin".into()),
                    amount: cosmwasm_std::Uint128(10000000000000), // 100,000 SHD
                }]),
                prng_seed: to_binary("").ok().unwrap(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: Addr("secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }),
        )
        .unwrap();

    println!("Shd contract addr: {}", shd.instance.address);
    println!("Deploying silk contract");

    let silk = ensemble
        .instantiate(
            reg_snip20.id,
            &snip20_reference_impl::msg::InitMsg {
                name: "Silk".into(),
                admin: Some(Addr("admin".into())),
                symbol: "SILK".into(),
                decimals: 6,
                initial_balances: Some(vec![snip20_reference_impl::msg::InitialBalance {
                    address: Addr("admin".into()),
                    amount: cosmwasm_std::Uint128(100000000000), // 100,000 SILK
                }]),
                prng_seed: to_binary("").ok().unwrap(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: Addr("secret14m2ffr7fyjhzv8cdknn2yp8sneht3luvsh9495".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }),
        )
        .unwrap();

    println!("Silk contract addr: {}", silk.instance.address);

    let key = String::from("key");

    ensemble
        .execute(
            &snip20_reference_impl::msg::HandleMsg::SetViewingKey {
                key: key.clone(),
                padding: None,
            },
            MockEnv::new("admin", sscrt.instance.clone()),
        )
        .unwrap();

    ensemble
        .execute(
            &snip20_reference_impl::msg::HandleMsg::SetViewingKey {
                key: key.clone(),
                padding: None,
            },
            MockEnv::new("admin", shd.instance.clone()),
        )
        .unwrap();

    ensemble
        .execute(
            &snip20_reference_impl::msg::HandleMsg::SetViewingKey {
                key: key.clone(),
                padding: None,
            },
            MockEnv::new("admin", silk.instance.clone()),
        )
        .unwrap();

    let mut query_res = ensemble
        .query(sscrt.instance.address.clone(), &snip20::QueryMsg::Balance {
            address: "admin".into(),
            key: key.clone(),
        })
        .unwrap();

    match query_res {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::new(100000000000))
        }
        _ => {
            assert!(false)
        }
    }

    query_res = ensemble
        .query(shd.instance.address.clone(), &snip20::QueryMsg::Balance {
            address: "admin".into(),
            key: key.clone(),
        })
        .unwrap();

    match query_res {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::new(10000000000000))
        }
        _ => {
            assert!(false)
        }
    }

    query_res = ensemble
        .query(silk.instance.address.clone(), &snip20::QueryMsg::Balance {
            address: "admin".into(),
            key: key.clone(),
        })
        .unwrap();

    match query_res {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::new(100000000000))
        }
        _ => {
            assert!(false)
        }
    }

    /*println!("{}", reg_sienna_factory.code_hash);
    println!("{}", reg_sienna_lp_token.code_hash);

    let sienna_factory = ensemble.instantiate(
        reg_sienna_factory.id,
        &factory::InitMsg {
            lp_token_contract: reg_sienna_lp_token.clone(),
            pair_contract: reg_sienna_exchange.clone(),
            exchange_settings: factory::ExchangeSettings{
                swap_fee: factory::Fee { nom: 0, denom: 0 },
                sienna_fee: factory::Fee { nom: 0, denom: 0 },
                sienna_burner: None,
            },
            admin: Some(Addr("admin".into())),
            prng_seed: to_binary("").ok().unwrap(),
        },
        MockEnv::new("admin", ContractLink {
            address: Addr("reg_sienna_factory".into()),
            code_hash: reg_sienna_factory.code_hash.clone(),
        }),
    ).unwrap();

    println!("{}", silk.address);
    println!("{}", shd.address);

    let mut res = ensemble.execute(
        &factory::HandleMsg::CreateExchange {
            pair: sienna::Pair {
                token_0: sienna::TokenType::CustomToken {
                    contract_addr: shd.address,
                    token_code_hash: shd.code_hash,
                },
                token_1: sienna::TokenType::CustomToken {
                    contract_addr: silk.address,
                    token_code_hash: silk.code_hash,
                }
            },
            entropy: to_binary("").ok().unwrap(),
        },
        MockEnv::new("admin", sienna_factory.clone()),
    ).unwrap();*/

    println!("here");

    /*    let sienna_pair = ensemble.instantiate(
        reg_sienna_exchange.id,
        &amm_pair::InitMsg{
            pair: sienna::Pair {
                token_0: sienna::TokenType::CustomToken{
                    contract_addr: shd.address.clone(),
                    token_code_hash: shd.code_hash.clone(),
                },
                token_1: sienna::TokenType::CustomToken{
                    contract_addr: silk.address.clone(),
                    token_code_hash: silk.code_hash.clone(),
                },
            },
            lp_token_contract: reg_sienna_lp_token.clone(),
            factory_info: sienna_factory.clone(),
            callback: Callback {
                msg: to_binary("").ok().unwrap(),
                contract: sienna_factory.clone(),
            },
            prng_seed: to_binary("").ok().unwrap(),
            entropy: to_binary("").ok().unwrap(),
        },
        MockEnv::new("admin", ContractLink {
            address: Addr("reg_sienna_exchange".into()),
            code_hash: reg_sienna_exchange.code_hash.clone(),
        }),
    ).unwrap();*/
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    /// The tokens that will be managed by the exchange
    pub pair: dex::sienna::Pair,
    /// LP token instantiation info
    pub lp_token_contract: ContractInstantiationInfo,
    /// Used by the exchange contract to
    /// send back its address to the factory on init
    pub factory_info: ContractLink<Addr>,
    pub callback: Callback<Addr>,
    pub prng_seed: Binary,
    pub entropy: Binary,
}*/
