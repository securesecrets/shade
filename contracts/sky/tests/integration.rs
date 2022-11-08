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
    mint::Mint,
    mock_band::MockBand,
    mock_secretswap_pair::MockSecretswapPair,
    mock_shadeswap_pair::MockShadeswapPair,
    oracle::Oracle,
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
        dex::{self, dex::Dex, secretswap, shadeswap},
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
    let reg_mock_shadeswap = ensemble.register(Box::new(MockShadeswapPair));
    let reg_oracle = ensemble.register(Box::new(Oracle));
    let reg_mint = ensemble.register(Box::new(Mint));
    let reg_band = ensemble.register(Box::new(MockBand));
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
                    amount: Uint128::new(10_000_000_000_000), // 10,000,000 SSCRT
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
                    amount: Uint128::new(1_000_000_000_000_000), // 10,000,000 SHD
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
                    amount: Uint128::new(10_000_000_000_000), // 10,000,000 SILK
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
            assert_eq!(amount, Uint128::new(10_000_000_000_000))
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
            assert_eq!(amount, Uint128::new(1_000_000_000_000_000))
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
            assert_eq!(amount, Uint128::new(10_000_000_000_000))
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
                min_amount: Uint128::new(1000000),
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
            min_amount: Uint128::new(1000000),
        }),
        _ => assert!(false),
    }

    println!("Sending sky some money");
    ensemble
        .execute(
            &snip20::HandleMsg::Transfer {
                recipient: sky.instance.address.clone(),
                amount: Uint128::new(100_000_000_000_000),
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
                amount: Uint128::new(1_000_000),
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
                amount: Uint128::new(1_000_000),
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
            assert_eq!(shd_bal, Uint128::new(100_000_000_000_000));
            assert_eq!(silk_bal, Uint128::new(1_000_000));
            assert_eq!(sscrt_bal, Uint128::new(1_000_000));
        }
        _ => assert!(false),
    }

    println!("deploying secretswap pair");
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
                fee_rate: Decimal::percent(1),
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

    println!("Sending secretswp silk shd pair some money");
    ensemble
        .execute(
            &snip20::HandleMsg::Transfer {
                recipient: scrtswp.instance.address.clone(),
                amount: Uint128::new(10_000_000_000_000),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", shd.instance.clone()),
        )
        .unwrap();
    ensemble
        .execute(
            &snip20::HandleMsg::Transfer {
                recipient: scrtswp.instance.address.clone(),
                amount: Uint128::new(1_000_000_000_000),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", silk.instance.clone()),
        )
        .unwrap();

    println!("test secretswap swapsimulation query");
    let query_res = ensemble
        .query(
            scrtswp.instance.address.clone(),
            &secretswap::PairQuery::Simulation {
                offer_asset: secretswap::Asset {
                    amount: Uint128::new(100_000_000),
                    info: secretswap::AssetInfo {
                        token: secretswap::Token {
                            contract_addr: shd.instance.address.clone(),
                            token_code_hash: shd.instance.code_hash.clone(),
                            viewing_key: "".to_string(),
                        },
                    },
                },
            },
        )
        .unwrap();
    let swp_result = match query_res {
        secretswap::SimulationResponse { return_amount, .. } => return_amount,
        _ => Uint128::zero(),
    };
    assert_eq!(
        swp_result.clone(),
        dex::dex::pool_take_amount(
            Uint128::new(100_000_000),
            Uint128::new(10_000_000_000_000),
            Uint128::new(1_000_000_000_000)
        )
        .checked_sub(
            dex::dex::pool_take_amount(
                Uint128::new(100_000_000),
                Uint128::new(10_000_000_000_000),
                Uint128::new(1_000_000_000_000)
            ) * Decimal::percent(1)
        )
        .unwrap()
    );

    let old_shd_bal =
        get_snip20_balance(&mut ensemble, shd.instance.address.clone(), "admin".into());
    let old_silk_bal =
        get_snip20_balance(&mut ensemble, silk.instance.address.clone(), "admin".into());
    println!(
        "Shd before swap: {}, Silk before swap: {}",
        old_shd_bal, old_silk_bal
    );

    println!("test secretswap swap");
    ensemble
        .execute(
            &snip20::HandleMsg::Send {
                recipient: scrtswp.instance.address.clone(),
                recipient_code_hash: Some(scrtswp.instance.code_hash.clone()),
                amount: Uint128::new(100_000_000),
                msg: Some(
                    to_binary(&secretswap::CallbackMsg {
                        swap: secretswap::CallbackSwap {
                            expected_return: Uint128::zero(),
                        },
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", shd.instance.clone()),
        )
        .unwrap();

    let new_shd_bal =
        get_snip20_balance(&mut ensemble, shd.instance.address.clone(), "admin".into());
    let new_silk_bal =
        get_snip20_balance(&mut ensemble, silk.instance.address.clone(), "admin".into());
    println!(
        "Shd after swap: {}, Silk after swap: {}",
        new_shd_bal, new_silk_bal
    );

    assert_eq!(
        new_shd_bal,
        old_shd_bal.checked_sub(Uint128::new(100_000_000)).unwrap()
    );
    assert_eq!(new_silk_bal, old_silk_bal.checked_add(swp_result).unwrap());

    println!("deploying shadeswap pair");
    let shdswp = ensemble
        .instantiate(
            reg_mock_shadeswap.id,
            &mock_shadeswap_pair::contract::InitMsg {
                token_0: Contract {
                    address: shd.instance.address.clone(),
                    code_hash: shd.instance.code_hash.clone(),
                },
                token_1: Contract {
                    address: silk.instance.address.clone(),
                    code_hash: silk.instance.code_hash.clone(),
                },
                fee_rate: Decimal::percent(1),
                whitelist: sky.instance.address.clone(),
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("shdswp".into()),
                code_hash: reg_mock_shadeswap.code_hash.clone(),
            }),
        )
        .unwrap();
    let shdswp_pair_shd_silk = sky::cycles::ArbPair {
        pair_contract: Some(Contract {
            address: shdswp.instance.address.clone(),
            code_hash: shdswp.instance.code_hash.clone(),
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
        dex: Dex::ShadeSwap,
    };
    assert!(shdswp_pair_shd_silk.clone().validate_pair().unwrap());

    println!("Sending shadeswp silk shd pair some money");
    ensemble
        .execute(
            &snip20::HandleMsg::Transfer {
                recipient: shdswp.instance.address.clone(),
                amount: Uint128::new(5_000_000_000_000),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", shd.instance.clone()),
        )
        .unwrap();
    ensemble
        .execute(
            &snip20::HandleMsg::Transfer {
                recipient: shdswp.instance.address.clone(),
                amount: Uint128::new(1_500_000_000_000),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", silk.instance.clone()),
        )
        .unwrap();

    println!("test shdswap swapsimulation query");
    let query_res = ensemble
        .query(
            shdswp.instance.address.clone(),
            &shadeswap::PairQuery::GetEstimatedPrice {
                offer: shadeswap::TokenAmount {
                    token: shadeswap::TokenType::CustomToken {
                        contract_addr: shd.instance.address.clone(),
                        token_code_hash: shd.instance.code_hash.clone(),
                    },
                    amount: Uint128::new(100_000_000),
                },
                exclude_fee: Some(true),
            },
        )
        .unwrap();
    let swp_result = match query_res {
        shadeswap::QueryMsgResponse::EstimatedPrice { estimated_price } => estimated_price,
        _ => Uint128::zero(),
    };
    println!("swap res = {}", swp_result);
    assert_eq!(
        swp_result.clone(),
        dex::dex::pool_take_amount(
            Uint128::new(100_000_000),
            Uint128::new(5_000_000_000_000),
            Uint128::new(1_500_000_000_000),
        )
    );

    let old_shd_bal =
        get_snip20_balance(&mut ensemble, shd.instance.address.clone(), "admin".into());
    let old_silk_bal =
        get_snip20_balance(&mut ensemble, silk.instance.address.clone(), "admin".into());
    println!(
        "Shd before swap: {}, Silk before swap: {}",
        old_shd_bal, old_silk_bal
    );

    println!("test shadeswap swap");
    ensemble
        .execute(
            &snip20::HandleMsg::Send {
                recipient: shdswp.instance.address.clone(),
                recipient_code_hash: Some(shdswp.instance.code_hash.clone()),
                amount: Uint128::new(100_000_000),
                msg: Some(
                    to_binary(&shadeswap::SwapTokens {
                        to: None,
                        expected_return: Some(Uint128::zero()),
                        router_link: None,
                        callback_signature: None,
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", shd.instance.clone()),
        )
        .unwrap();

    let new_shd_bal =
        get_snip20_balance(&mut ensemble, shd.instance.address.clone(), "admin".into());
    let new_silk_bal =
        get_snip20_balance(&mut ensemble, silk.instance.address.clone(), "admin".into());
    println!(
        "Shd after swap: {}, Silk after swap: {}",
        new_shd_bal, new_silk_bal
    );

    assert_eq!(
        new_shd_bal,
        old_shd_bal.checked_sub(Uint128::new(100_000_000)).unwrap()
    );
    //assert_eq!(new_silk_bal, old_silk_bal.checked_add(swp_result).unwrap());

    let old_shd_bal = new_shd_bal.clone();
    let old_silk_bal = new_silk_bal.clone();

    ensemble
        .execute(
            &sky::HandleMsg::SetCycles {
                cycles: vec![sky::cycles::Cycle {
                    pair_addrs: vec![shdswp_pair_shd_silk.clone(), scrtswp_pair_shd_silk.clone()],
                    start_addr: Contract {
                        address: shd.instance.address.clone(),
                        code_hash: shd.instance.code_hash.clone(),
                    },
                }],
                padding: None,
            },
            MockEnv::new("admin", sky.instance.clone()),
        )
        .unwrap();
    let res = ensemble
        .query(sky.instance.address.clone(), &sky::QueryMsg::GetCycles {})
        .unwrap();
    let cycles = match res {
        sky::QueryAnswer::GetCycles { cycles } => cycles,
        _ => vec![],
    };
    assert_eq!(cycles, vec![sky::cycles::Cycle {
        pair_addrs: vec![shdswp_pair_shd_silk.clone(), scrtswp_pair_shd_silk],
        start_addr: Contract {
            address: shd.instance.address.clone(),
            code_hash: shd.instance.code_hash.clone(),
        },
    }]);

    println!("Testing profitablitity query");
    let res = ensemble
        .query(
            sky.instance.address.clone(),
            &sky::QueryMsg::IsCycleProfitable {
                index: Uint128::new(0),
                amount: Uint128::new(100_000_000),
            },
        )
        .unwrap();
    let profit = match res {
        sky::QueryAnswer::IsCycleProfitable {
            is_profitable,
            direction,
            swap_amounts,
            profit,
        } => profit,

        _ => Uint128::zero(),
    };

    println!("Testing arb execution");
    ensemble
        .execute(
            &sky::HandleMsg::ArbCycle {
                amount: Uint128::new(100_000_000),
                index: Uint128::zero(),
                payback_addr: None,
                padding: None,
            },
            MockEnv::new("admin", sky.instance.clone()),
        )
        .unwrap();
    let new_shd_bal =
        get_snip20_balance(&mut ensemble, shd.instance.address.clone(), "admin".into());
    let new_silk_bal =
        get_snip20_balance(&mut ensemble, silk.instance.address.clone(), "admin".into());
    println!(
        "Shd after swap: {}, Silk after swap: {}",
        new_shd_bal, new_silk_bal
    );
    assert_eq!(
        new_shd_bal,
        old_shd_bal
            .checked_add(profit * Decimal::percent(30))
            .unwrap()
    );

    let scrtswp_new = ensemble
        .instantiate(
            reg_mock_secretswap.id,
            &mock_secretswap_pair::contract::InitMsg {
                token_0: Contract {
                    address: silk.instance.address.clone(),
                    code_hash: silk.instance.code_hash.clone(),
                },
                token_1: Contract {
                    address: sscrt.instance.address.clone(),
                    code_hash: sscrt.instance.code_hash.clone(),
                },
                fee_rate: Decimal::percent(1),
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("scrtswp_silk_sscrt".into()),
                code_hash: reg_mock_secretswap.code_hash.clone(),
            }),
        )
        .unwrap();
    let scrtswp_pair_silk_sscrt = sky::cycles::ArbPair {
        pair_contract: Some(Contract {
            address: scrtswp_new.instance.address.clone(),
            code_hash: scrtswp_new.instance.code_hash.clone(),
        }),
        mint_info: None,
        token0: Contract {
            address: silk.instance.address.clone(),
            code_hash: silk.instance.code_hash.clone(),
        },
        token1: Contract {
            address: sscrt.instance.address.clone(),
            code_hash: sscrt.instance.code_hash.clone(),
        },
        dex: Dex::SecretSwap,
    };
    assert!(scrtswp_pair_silk_sscrt.clone().validate_pair().unwrap());

    let shdswp_new = ensemble
        .instantiate(
            reg_mock_shadeswap.id,
            &mock_shadeswap_pair::contract::InitMsg {
                token_0: Contract {
                    address: shd.instance.address.clone(),
                    code_hash: shd.instance.code_hash.clone(),
                },
                token_1: Contract {
                    address: sscrt.instance.address.clone(),
                    code_hash: sscrt.instance.code_hash.clone(),
                },
                fee_rate: Decimal::percent(1),
                whitelist: sky.instance.address.clone(),
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("shdswp_sscrt_shd".into()),
                code_hash: reg_mock_shadeswap.code_hash.clone(),
            }),
        )
        .unwrap();
    let shdswp_pair_shd_sscrt = sky::cycles::ArbPair {
        pair_contract: Some(Contract {
            address: shdswp_new.instance.address.clone(),
            code_hash: shdswp_new.instance.code_hash.clone(),
        }),
        mint_info: None,
        token0: Contract {
            address: shd.instance.address.clone(),
            code_hash: shd.instance.code_hash.clone(),
        },
        token1: Contract {
            address: sscrt.instance.address.clone(),
            code_hash: sscrt.instance.code_hash.clone(),
        },
        dex: Dex::ShadeSwap,
    };
    assert!(shdswp_pair_shd_sscrt.clone().validate_pair().unwrap());

    ensemble
        .execute(
            &snip20::HandleMsg::Transfer {
                recipient: scrtswp_new.instance.address.clone(),
                amount: Uint128::new(50_000_000_000),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", silk.instance.clone()),
        )
        .unwrap();
    ensemble
        .execute(
            &snip20::HandleMsg::Transfer {
                recipient: scrtswp_new.instance.address.clone(),
                amount: Uint128::new(10_000_000_000),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", sscrt.instance.clone()),
        )
        .unwrap();
    ensemble
        .execute(
            &snip20::HandleMsg::Transfer {
                recipient: shdswp_new.instance.address.clone(),
                amount: Uint128::new(5_000_000_000_000),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", shd.instance.clone()),
        )
        .unwrap();
    ensemble
        .execute(
            &snip20::HandleMsg::Transfer {
                recipient: shdswp_new.instance.address.clone(),
                amount: Uint128::new(150_000_000_000),
                memo: None,
                padding: None,
            },
            MockEnv::new("admin", sscrt.instance.clone()),
        )
        .unwrap();

    let cycle_2 = sky::cycles::Cycle {
        pair_addrs: vec![
            shdswp_pair_shd_silk,
            scrtswp_pair_silk_sscrt,
            shdswp_pair_shd_sscrt,
        ],
        start_addr: Contract {
            address: shd.instance.address.clone(),
            code_hash: shd.instance.code_hash.clone(),
        },
    };

    assert!(cycle_2.validate_cycle().unwrap());

    ensemble
        .execute(
            &sky::HandleMsg::AppendCycles {
                cycle: vec![cycle_2],
                padding: None,
            },
            MockEnv::new("admin", sky.instance.clone()),
        )
        .unwrap();

    let res = ensemble
        .query(sky.instance.address.clone(), &sky::QueryMsg::GetCycles {})
        .unwrap();
    match res {
        sky::QueryAnswer::GetCycles { cycles } => assert_eq!(cycles.len(), 2),
        _ => {}
    }

    let res = ensemble
        .query(
            sky.instance.address.clone(),
            &sky::QueryMsg::IsCycleProfitable {
                amount: Uint128::new(1_000_000_000),
                index: Uint128::new(1),
            },
        )
        .unwrap();
    let profit = match res {
        sky::QueryAnswer::IsCycleProfitable {
            is_profitable,
            direction,
            swap_amounts,
            profit,
        } => profit,
        _ => Uint128::zero(),
    };

    let res = ensemble
        .query(
            sky.instance.address.clone(),
            &sky::QueryMsg::IsAnyCycleProfitable {
                amount: Uint128::new(1_000_000_000),
            },
        )
        .unwrap();
    let profit = match res {
        sky::QueryAnswer::IsAnyCycleProfitable {
            is_profitable,
            direction,
            swap_amounts,
            profit,
        } => profit,
        _ => vec![Uint128::zero()],
    };
    println!("profit: {:?}", profit);

    let query_res = ensemble
        .query(sky.instance.address.clone(), &sky::QueryMsg::Balance {})
        .unwrap();
    match query_res {
        sky::QueryAnswer::Balance {
            shd_bal,
            silk_bal,
            sscrt_bal,
        } => {
            println!("{}", shd_bal.u128());
        }
        _ => assert!(false),
    }

    let old_shd_bal =
        get_snip20_balance(&mut ensemble, shd.instance.address.clone(), "admin".into());
    let old_silk_bal =
        get_snip20_balance(&mut ensemble, silk.instance.address.clone(), "admin".into());
    println!(
        "Shd before swap: {}, Silk after swap: {}",
        old_shd_bal, old_silk_bal
    );

    let res = ensemble
        .execute(
            &sky::HandleMsg::ArbAllCycles {
                amount: Uint128::new(100_000_000),
                padding: None,
            },
            MockEnv::new("admin", sky.instance.clone()),
        )
        .unwrap();
    println!("{:?}", res);
    let new_shd_bal =
        get_snip20_balance(&mut ensemble, shd.instance.address.clone(), "admin".into());
    let new_silk_bal =
        get_snip20_balance(&mut ensemble, silk.instance.address.clone(), "admin".into());
    println!(
        "Shd after swap: {}, Silk after swap: {}",
        new_shd_bal, new_silk_bal
    );
    assert!(new_shd_bal > old_shd_bal);

    /*    println!("set up mint contracts");
    let band = ensemble
        .instantiate(
            reg_band.id,
            &shade_protocol::contract_interfaces::oracles::band::InitMsg {},
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("band".into()),
                code_hash: reg_band.code_hash.clone(),
            }),
        )
        .unwrap()
        .instance;

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
                    address: sscrt.instance.address.clone(),
                    code_hash: sscrt.instance.code_hash.clone(),
                },
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("oracle".into()),
                code_hash: reg_oracle.code_hash.clone(),
            }),
        )
        .unwrap()
        .instance;

    let silk_mint = ensemble
        .instantiate(
            reg_mint.id,
            &shade_protocol::contract_interfaces::mint::mint::InitMsg {
                admin: Some(HumanAddr("admin".into())),
                oracle: Contract {
                    address: oracle.address.clone(),
                    code_hash: oracle.code_hash.clone(),
                },
                native_asset: Contract {
                    address: shd.instance.address.clone(),
                    code_hash: shd.instance.code_hash.clone(),
                },
                peg: None,
                treasury: HumanAddr("admin".into()),
                secondary_burn: None,
                limit: None,
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("mint".into()),
                code_hash: reg_mint.code_hash.clone(),
            }),
        )
        .unwrap()
        .instance;
    let shd_mint = ensemble
        .instantiate(
            reg_mint.id,
            &shade_protocol::contract_interfaces::mint::mint::InitMsg {
                admin: Some(HumanAddr("admin".into())),
                oracle: Contract {
                    address: oracle.address.clone(),
                    code_hash: oracle.code_hash.clone(),
                },
                native_asset: Contract {
                    address: shd.instance.address.clone(),
                    code_hash: shd.instance.code_hash.clone(),
                },
                peg: None,
                treasury: HumanAddr("admin".into()),
                secondary_burn: None,
                limit: None,
            },
            MockEnv::new("admin", ContractLink {
                address: HumanAddr("mint".into()),
                code_hash: reg_mint.code_hash,
            }),
        )
        .unwrap()
        .instance;*/

    //assert!(false);
}

pub fn get_snip20_balance(
    ensemble: &mut ContractEnsemble,
    snip20_addr: HumanAddr,
    balance_owner: HumanAddr,
) -> Uint128 {
    match ensemble
        .query(snip20_addr, &snip20::QueryMsg::Balance {
            address: balance_owner,
            key: String::from("key"),
        })
        .unwrap()
    {
        snip20::QueryAnswer::Balance { amount } => amount,
        _ => Uint128::zero(),
    }
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
