use shade_protocol::c_std::{
    to_binary, from_binary,
    Addr, StdError, Uint128, Coin,
    Decimal,
};
use shade_protocol::contract_interfaces::{
    dex::dex::Dex,
    sky::{
        cycles::{
            ArbPair, Derivative,
            DerivativeType,
        },
        sky_derivatives::{
            Config,
            ExecuteAnswer,
            ExecuteMsg,
            QueryAnswer,
            QueryMsg,
            TradingFees,
        },
    },
    admin,
    snip20,
};
use shade_protocol_temp::stkd;
use shade_protocol::utils::{
    asset::Contract,
    ExecuteCallback,
    InstantiateCallback,
    MultiTestable,
    Query,
};
use shade_protocol_temp::utils::{
    InstantiateCallback as OtherInstantiateCallback,
    MultiTestable as OtherMultiTestable,
    ExecuteCallback as OtherExecuteCallback,
    Query as OtherQuery,
};
use shade_protocol::multi_test::App;
use shade_multi_test::multi::{
    admin::init_admin_auth,
    snip20::Snip20,
    sky_derivatives::SkyDerivatives,
};
use shade_multi_test_temp::multi::mock_stkd::MockStkd;

use crate::tests::{init, fill_dex_pairs};

#[test]
fn update_config() {
    let (mut chain, admin, base, deriv, arb, config) = init();

    // Test no changes
    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: None,
        derivative: None,
        trading_fees: None,
        max_arb_amount: None,
        min_profit_amount: None,
        viewing_key: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::Config {}
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::Config {
            config: config.clone(),
        },
    );

    // Test change without admin priviledge
    assert!(
        ExecuteMsg::UpdateConfig {
            shade_admin_addr: None,
            treasury: None,
            derivative: None,
            trading_fees: None,
            max_arb_amount: None,
            min_profit_amount: None,
            viewing_key: None,
        }.test_exec(&arb, &mut chain, Addr::unchecked("not_admin"), &[]).is_err(),
    );
 
    // Test with changes
    let new_admin = init_admin_auth(&mut chain, &Addr::unchecked("admin"));
    
    let new_deriv = stkd::MockInstantiateMsg {
        name: "derivative2".to_string(),
        symbol: "stkd-SCRT2".to_string(),
        decimals: 6,
        price: Uint128::new(2),
    }.test_init(
        MockStkd::default(), 
        &mut chain, 
        Addr::unchecked("admin"), 
        "stkd-SCRT2", 
        &[]
    ).unwrap();

    let new_derivative = Derivative {
        contract: new_deriv.clone().into(),
        original_asset: base.clone().into(),
        staking_type: DerivativeType::StkdScrt,
    };

    let new_fees = TradingFees {
        dex_fee: Decimal::one(),
        stake_fee: Decimal::one(),
        unbond_fee: Decimal::one(),
    };

    ExecuteMsg::UpdateConfig {
        shade_admin_addr: Some(new_admin.clone().into()),
        treasury: Some(Addr::unchecked("treasury2")),
        derivative: Some(new_derivative.clone()),
        trading_fees: Some(new_fees.clone()),
        max_arb_amount: Some(Uint128::new(1_000_000_000)),
        min_profit_amount: Some(Uint128::new(1_000)),
        viewing_key: Some("key2".into()),
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::Config {}
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::Config {
            config: Config {
                shade_admin_addr: new_admin.clone().into(),
                treasury: Addr::unchecked("treasury2"),
                derivative: new_derivative,
                trading_fees: new_fees,
                max_arb_amount: Uint128::new(1_000_000_000),
                min_profit_amount: Uint128::new(1_000),
                viewing_key: "key2".into(),
            },
        },
    );

    // Test new viewing keys
    assert_eq!(
        snip20::QueryMsg::Balance {
            address: arb.address.to_string(),
            key: "key2".into(),
        }.test_query::<snip20::QueryAnswer>(&base, &chain).unwrap(),
        snip20::QueryAnswer::Balance {
            amount: Uint128::zero(),
        },
    );

    assert_eq!(
        stkd::QueryMsg::Balance {
            address: arb.address.clone(),
            key: "key2".into(),
        }.test_query::<stkd::QueryAnswer>(&new_deriv, &chain).unwrap(),
        stkd::QueryAnswer::Balance {
            amount: Uint128::zero(),
        },
    );

    // Test bad admin
    assert!(
        ExecuteMsg::UpdateConfig {
            shade_admin_addr: Some(Contract {
                address: Addr::unchecked("bad_admin"),
                code_hash: "does not exist".into(),
            }),
            treasury: None,
            derivative: None,
            trading_fees: None,
            max_arb_amount: None,
            min_profit_amount: None,
            viewing_key: None,
        }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).is_err(),
    );

    // Test bad trading fees
    let bad_fees = TradingFees {
        dex_fee: Decimal::raw(1_010_000_000_000_000_000),
        stake_fee: Decimal::one(),
        unbond_fee: Decimal::one(),
    };

    assert!(
        ExecuteMsg::UpdateConfig {
            shade_admin_addr: None,
            treasury: None,
            derivative: None,
            trading_fees: Some(bad_fees),
            max_arb_amount: None,
            min_profit_amount: None,
            viewing_key: None,
        }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).is_err(),
    );

    // Test new derivative changes vks
    let new_snip20 = snip20::InstantiateMsg {
        name: "snip20_2".into(),
        admin: Some("admin".into()),
        symbol: "NSNIP".into(),
        decimals: 7,
        initial_balances: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(true),
            enable_transfer: Some(true),
        }),
        query_auth: None,
    }.test_init(Snip20::default(), &mut chain, Addr::unchecked("admin"), "token2", &[]).unwrap();

    let another_new_deriv = stkd::MockInstantiateMsg {
        name: "derivative3".to_string(),
        symbol: "stkd-SCRT3".to_string(),
        decimals: 8,
        price: Uint128::new(3),
    }.test_init(
        MockStkd::default(), 
        &mut chain, 
        Addr::unchecked("admin"), 
        "stkd-SCRT3", 
        &[]
    ).unwrap();

    let another_new_derivative = Derivative {
        contract: another_new_deriv.clone().into(),
        original_asset: new_snip20.clone().into(),
        staking_type: DerivativeType::StkdScrt,
    };

    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: None,
        derivative: Some(another_new_derivative),
        trading_fees: None,
        max_arb_amount: None,
        min_profit_amount: None,
        viewing_key: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        snip20::QueryMsg::Balance {
            address: arb.address.to_string(),
            key: "key2".to_string(),
        }.test_query::<snip20::QueryAnswer>(&new_snip20, &chain).unwrap(),
        snip20::QueryAnswer::Balance {
            amount: Uint128::zero(),
        },
    );

    assert_eq!(
        stkd::QueryMsg::Balance {
            address: arb.address.clone(),
            key: "key2".to_string(),
        }.test_query::<stkd::QueryAnswer>(&another_new_deriv, &chain).unwrap(),
        stkd::QueryAnswer::Balance {
            amount: Uint128::zero(),
        },
    );
}

#[test]
fn set_dex_pairs() {
    let (mut chain, admin, base, deriv, arb, config) = init();
    
    assert!(ExecuteMsg::SetDexPairs { // invalid dex pair
            pairs: fill_dex_pairs(1, deriv.clone().into(), base.clone().into()),
        }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).is_err()
    );

    let mut more_dex_pairs = fill_dex_pairs(4, base.clone().into(), deriv.clone().into());
    more_dex_pairs.remove(0);
    more_dex_pairs.remove(0); // just pairs #2 and #3
    ExecuteMsg::SetDexPairs {
        pairs: more_dex_pairs,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::DexPairs {}
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::DexPairs {
            dex_pairs: vec![
                ArbPair {
                    pair_contract: Some(Contract {
                        address: Addr::unchecked("dex pair 2"),
                        code_hash: "hash".to_string(),
                    }),
                    mint_info: None,
                    token0: base.clone().into(),
                    token0_decimals: Uint128::new(6),
                    token0_amount: None,
                    token1: deriv.clone().into(),
                    token1_decimals: Uint128::new(6),
                    token1_amount: None,
                    dex: Dex::ShadeSwap,
                },
                ArbPair {
                    pair_contract: Some(Contract {
                        address: Addr::unchecked("dex pair 3"),
                        code_hash: "hash".to_string(),
                    }),
                    mint_info: None,
                    token0: base.clone().into(),
                    token0_decimals: Uint128::new(6),
                    token0_amount: None,
                    token1: deriv.clone().into(),
                    token1_decimals: Uint128::new(6),
                    token1_amount: None,
                    dex: Dex::ShadeSwap,
                },
            ],
        },
    ); 
}

#[test]
fn set_pair() {
    let (mut chain, admin, base, deriv, arb, config) = init();

    let more_dex_pairs = fill_dex_pairs(4, base.clone().into(), deriv.clone().into());
    ExecuteMsg::SetPair {
        pair: more_dex_pairs.get(3).unwrap().to_owned(),
        index: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::DexPairs {}
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::DexPairs {
            dex_pairs: vec![
                ArbPair {
                    pair_contract: Some(Contract {
                        address: Addr::unchecked("dex pair 3"),
                        code_hash: "hash".to_string(),
                    }),
                    mint_info: None,
                    token0: base.clone().into(),
                    token0_decimals: Uint128::new(6),
                    token0_amount: None,
                    token1: deriv.clone().into(),
                    token1_decimals: Uint128::new(6),
                    token1_amount: None,
                    dex: Dex::ShadeSwap,
                },
                ArbPair {
                    pair_contract: Some(Contract {
                        address: Addr::unchecked("dex pair 1"),
                        code_hash: "hash".to_string(),
                    }),
                    mint_info: None,
                    token0: base.clone().into(),
                    token0_decimals: Uint128::new(6),
                    token0_amount: None,
                    token1: deriv.clone().into(),
                    token1_decimals: Uint128::new(6),
                    token1_amount: None,
                    dex: Dex::ShadeSwap,
                },
            ],
        },
    ); 

    ExecuteMsg::SetPair {
        pair: more_dex_pairs.get(2).unwrap().to_owned(),
        index: Some(1),
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::DexPairs {}
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::DexPairs {
            dex_pairs: vec![
                ArbPair {
                    pair_contract: Some(Contract {
                        address: Addr::unchecked("dex pair 3"),
                        code_hash: "hash".to_string(),
                    }),
                    mint_info: None,
                    token0: base.clone().into(),
                    token0_decimals: Uint128::new(6),
                    token0_amount: None,
                    token1: deriv.clone().into(),
                    token1_decimals: Uint128::new(6),
                    token1_amount: None,
                    dex: Dex::ShadeSwap,
                },
                ArbPair {
                    pair_contract: Some(Contract {
                        address: Addr::unchecked("dex pair 2"),
                        code_hash: "hash".to_string(),
                    }),
                    mint_info: None,
                    token0: base.clone().into(),
                    token0_decimals: Uint128::new(6),
                    token0_amount: None,
                    token1: deriv.clone().into(),
                    token1_decimals: Uint128::new(6),
                    token1_amount: None,
                    dex: Dex::ShadeSwap,
                },
            ],
        },
    );
}

#[test]
fn add_pair() {
    let (mut chain, admin, base, deriv, arb, config) = init();

    let more_dex_pairs = fill_dex_pairs(1, base.clone().into(), deriv.clone().into());
    ExecuteMsg::AddPair {
        pair: more_dex_pairs.get(0).unwrap().to_owned(),
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::DexPairs {}
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::DexPairs {
            dex_pairs: vec![
                ArbPair {
                    pair_contract: Some(Contract {
                        address: Addr::unchecked("dex pair 0"),
                        code_hash: "hash".to_string(),
                    }),
                    mint_info: None,
                    token0: base.clone().into(),
                    token0_decimals: Uint128::new(6),
                    token0_amount: None,
                    token1: deriv.clone().into(),
                    token1_decimals: Uint128::new(6),
                    token1_amount: None,
                    dex: Dex::ShadeSwap,
                },
                ArbPair {
                    pair_contract: Some(Contract {
                        address: Addr::unchecked("dex pair 1"),
                        code_hash: "hash".to_string(),
                    }),
                    mint_info: None,
                    token0: base.clone().into(),
                    token0_decimals: Uint128::new(6),
                    token0_amount: None,
                    token1: deriv.clone().into(),
                    token1_decimals: Uint128::new(6),
                    token1_amount: None,
                    dex: Dex::ShadeSwap,
                },
                ArbPair {
                    pair_contract: Some(Contract {
                        address: Addr::unchecked("dex pair 0"),
                        code_hash: "hash".to_string(),
                    }),
                    mint_info: None,
                    token0: base.clone().into(),
                    token0_decimals: Uint128::new(6),
                    token0_amount: None,
                    token1: deriv.clone().into(),
                    token1_decimals: Uint128::new(6),
                    token1_amount: None,
                    dex: Dex::ShadeSwap,
                },
            ],
        },
    );
}

#[test]
fn remove_pair() {
    let (mut chain, admin, base, deriv, arb, config) = init();

    let more_dex_pairs = fill_dex_pairs(1, base.clone().into(), deriv.clone().into());
    ExecuteMsg::RemovePair {
        index: 1,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::DexPairs {}
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::DexPairs {
            dex_pairs: vec![
                ArbPair {
                    pair_contract: Some(Contract {
                        address: Addr::unchecked("dex pair 0"),
                        code_hash: "hash".to_string(),
                    }),
                    mint_info: None,
                    token0: base.clone().into(),
                    token0_decimals: Uint128::new(6),
                    token0_amount: None,
                    token1: deriv.clone().into(),
                    token1_decimals: Uint128::new(6),
                    token1_amount: None,
                    dex: Dex::ShadeSwap,
                },
            ],
        },
    );
}

/*
#[test]
fn arb_pair() {
    assert!(false);
}

#[test]
fn arb_all_pairs() {
    assert!(false);
}

#[test]
fn adapter_unbond() {
    assert!(false);
}

#[test]
fn adapter_claim() {
    assert!(false);
}

#[test]
fn adapter_update() {
    assert!(false);
}
*/
