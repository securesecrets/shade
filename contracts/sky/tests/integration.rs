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

use contract_harness::harness::{
    admin::Admin,
    mock_secretswap_pair::MockSecretswapPair,
    sky::Sky,
    snip20::Snip20,
};
use fadroma::{
    ensemble::{ContractEnsemble, MockEnv},
    prelude::{Callback, ContractInstantiationInfo, ContractLink},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::{
    c_std::{
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
    },
    contract_interfaces::{
        dex::{self, dex::Dex, shadeswap},
        sky,
        snip20,
    },
    math_compat::{Decimal, Uint128},
    utils::asset::Contract,
};

fn test_ensemble_sky(swap_amount: Uint128) {
    let mut ensemble = ContractEnsemble::new(50);

    let reg_snip20 = ensemble.register(Box::new(Snip20));
    let reg_admin = ensemble.register(Box::new(Admin));
    let reg_sky = ensemble.register(Box::new(Sky));
    let reg_mock_secretswap = ensemble.register(Box::new(MockSecretswapPair));
    //let reg_mock_shdswp = ensemble.register(Box::new(MockShdSwp));
    //let reg_shadeswap_exchange = ensemble.register(Box::new(ShadeswapExchange));
    //let reg_shadeswap_factory = ensemble.register(Box::new(ShadeswapFactory));
    //let reg_sienna_lp_token = ensemble.register(Box::new(SiennaLpToken));

    println!("Deploying sscrt contract");

    let sscrt = ensemble
        .instantiate(
            reg_snip20.id,
            &snip20::InitMsg {
                name: "secretSCRT".into(),
                admin: Some(HumanAddr("admin".into())),
                symbol: "SSCRT".into(),
                decimals: 6,
                initial_balances: Some(vec![snip20::InitialBalance {
                    address: HumanAddr("admin".into()),
                    amount: Uint128::new(10000000000000), // 10,000,000 SSCRT
                }]),
                prng_seed: to_binary("").ok().unwrap(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("sscrt".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }),
        )
        .unwrap();

    println!("Sscrt contract addr: {}", sscrt.instance.address);
    println!("Deploying shd contract");

    let shd = ensemble
        .instantiate(
            reg_snip20.id,
            &snip20::InitMsg {
                name: "Shade".into(),
                admin: Some(HumanAddr("admin".into())),
                symbol: "SHD".into(),
                decimals: 8,
                initial_balances: Some(vec![snip20::InitialBalance {
                    address: HumanAddr("admin".into()),
                    amount: Uint128::new(1000000000000000), // 10,000,000 SHD
                }]),
                prng_seed: to_binary("").ok().unwrap(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }),
        )
        .unwrap();

    println!("Shd contract addr: {}", shd.instance.address);
    println!("Deploying silk contract");

    let silk = ensemble
        .instantiate(
            reg_snip20.id,
            &snip20::InitMsg {
                name: "Silk".into(),
                admin: Some(HumanAddr("admin".into())),
                symbol: "SILK".into(),
                decimals: 6,
                initial_balances: Some(vec![snip20::InitialBalance {
                    address: HumanAddr("admin".into()),
                    amount: Uint128::new(10000000000000), // 10,000,000 SILK
                }]),
                prng_seed: to_binary("").ok().unwrap(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("secret14m2ffr7fyjhzv8cdknn2yp8sneht3luvsh9495".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }),
        )
        .unwrap();

    println!("Silk contract addr: {}", silk.instance.address);

    let key = String::from("key");

    ensemble
        .execute(
            &snip20::HandleMsg::SetViewingKey {
                key: key.clone(),
                padding: None,
            },
            MockEnv::new("admin", sscrt.instance.clone()),
        )
        .unwrap();

    ensemble
        .execute(
            &snip20::HandleMsg::SetViewingKey {
                key: key.clone(),
                padding: None,
            },
            MockEnv::new("admin", shd.instance.clone()),
        )
        .unwrap();

    ensemble
        .execute(
            &snip20::HandleMsg::SetViewingKey {
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
            assert_eq!(amount, Uint128::new(10000000000000))
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
            assert_eq!(amount, Uint128::new(1000000000000000))
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
            assert_eq!(amount, Uint128::new(10000000000000))
        }
        _ => {
            assert!(false)
        }
    }

    println!("Deploying admin contract!");

    let admin = ensemble
        .instantiate(
            reg_admin.id,
            &shade_admin::admin::InitMsg {},
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("admin".into()),
                code_hash: reg_admin.code_hash.clone(),
            }),
        )
        .unwrap();

    println!("Admin contract addr: {}", admin.instance.address);
    println!("Deploying sky contract");

    let sky = ensemble
        .instantiate(
            reg_sky.id,
            &sky::InitMsg {
                shade_admin: Contract {
                    address: admin.instance.address.clone(),
                    code_hash: admin.instance.code_hash.clone(),
                },
                shd_token: Contract {
                    address: shd.instance.address.clone(),
                    code_hash: shd.instance.code_hash.clone(),
                },
                silk_token: Contract {
                    address: silk.instance.address.clone(),
                    code_hash: silk.instance.code_hash.clone(),
                },
                sscrt_token: Contract {
                    address: sscrt.instance.address.clone(),
                    code_hash: sscrt.instance.code_hash.clone(),
                },
                treasury: Contract {
                    address: HumanAddr("admin".into()),
                    code_hash: "".to_string(),
                },
                viewing_key: "sky".to_string(),
                payback_rate: Decimal::percent(30),
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("sky".into()),
                code_hash: reg_sky.code_hash.clone(),
            }),
        )
        .unwrap();

    println!("Sky contract addr: {}", sky.instance.address);

    let mut handle_msg = shade_admin::admin::HandleMsg::AddContract {
        contract_address: sky.instance.address.clone().to_string(),
    };

    assert!(
        ensemble
            .execute(&handle_msg, MockEnv::new("admin", admin.instance.clone()))
            .is_ok()
    );

    println!("Testing GetConfig");

    let query_res = ensemble
        .query(sky.instance.address.clone(), &sky::QueryMsg::GetConfig {})
        .unwrap();
    match query_res {
        sky::QueryAnswer::Config { config } => assert_eq!(config, sky::Config {
            shade_admin: Contract {
                address: admin.instance.address.clone(),
                code_hash: admin.instance.code_hash.clone(),
            },
            shd_token: Contract {
                address: shd.instance.address.clone(),
                code_hash: shd.instance.code_hash.clone(),
            },
            silk_token: Contract {
                address: silk.instance.address.clone(),
                code_hash: silk.instance.code_hash.clone(),
            },
            sscrt_token: Contract {
                address: sscrt.instance.address.clone(),
                code_hash: sscrt.instance.code_hash.clone(),
            },
            treasury: Contract {
                address: HumanAddr("admin".into()),
                code_hash: "".to_string(),
            },
            payback_rate: Decimal::percent(30),
        }),
        _ => assert!(false),
    }

    println!("Sending sky some money");
    ensemble
        .execute(
            &snip20::HandleMsg::Transfer {
                recipient: sky.instance.address.clone(),
                amount: Uint128::new(100000000000000),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", shd.instance.clone()),
        )
        .unwrap();
    ensemble
        .execute(
            &snip20::HandleMsg::Transfer {
                recipient: sky.instance.address.clone(),
                amount: Uint128::new(1000000),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", silk.instance.clone()),
        )
        .unwrap();
    ensemble
        .execute(
            &snip20::HandleMsg::Transfer {
                recipient: sky.instance.address.clone(),
                amount: Uint128::new(1000000),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", sscrt.instance.clone()),
        )
        .unwrap();

    println!("testing sky balance query");
    let query_res = ensemble
        .query(sky.instance.address.clone(), &sky::QueryMsg::Balance {})
        .unwrap();
    match query_res {
        sky::QueryAnswer::Balance {
            shd_bal,
            silk_bal,
            sscrt_bal,
        } => {
            assert_eq!(shd_bal, Uint128::new(100000000000000));
            assert_eq!(silk_bal, Uint128::new(1000000));
            assert_eq!(sscrt_bal, Uint128::new(1000000));
        }
        _ => assert!(false),
    }

    println!("deploying shadeswap pair");
    let scrtswp = ensemble
        .instantiate(
            reg_mock_secretswap.id,
            &mock_secretswap_pair::contract::InitMsg {
                token_0: Contract {
                    address: shd.instance.address.clone(),
                    code_hash: shd.instance.code_hash.clone(),
                },
                token_1: Contract {
                    address: silk.instance.address.clone(),
                    code_hash: silk.instance.code_hash.clone(),
                },
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("scrtswp".into()),
                code_hash: reg_mock_secretswap.code_hash.clone(),
            }),
        )
        .unwrap();
    let scrtswp_pair_shd_silk = sky::cycles::ArbPair {
        pair_contract: Some(Contract {
            address: scrtswp.instance.address.clone(),
            code_hash: scrtswp.instance.code_hash.clone(),
        }),
        mint_info: None,
        token0: Contract {
            address: shd.instance.address.clone(),
            code_hash: shd.instance.code_hash.clone(),
        },
        token1: Contract {
            address: silk.instance.address.clone(),
            code_hash: silk.instance.code_hash.clone(),
        },
        dex: Dex::SecretSwap,
    };
    assert!(scrtswp_pair_shd_silk.clone().validate_pair().unwrap());
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
    pub factory_info: ContractLink<HumanAddr>,
    pub callback: Callback<HumanAddr>,
    pub prng_seed: Binary,
    pub entropy: Binary,
}
