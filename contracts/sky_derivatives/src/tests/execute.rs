use shade_protocol::c_std::{
    to_binary, from_binary,
    Addr, StdError, Uint128, Coin,
    Decimal,
};
use shade_protocol::contract_interfaces::{
    dao::adapter,
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
    generic_response::ResponseStatus,
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
use mock_stkd_temp::contract as mock_stkd;
use mock_sienna_temp::contract as mock_sienna;

use crate::tests::{init, init_with_pair, fill_dex_pairs, seeded_pair};

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
    
    let new_deriv = mock_stkd::InstantiateMsg {
        name: "derivative2".to_string(),
        symbol: "stkd-SCRT2".to_string(),
        decimals: 6,
        price: Uint128::new(2_000_000),
        unbonding_time: 21u32,
        unbonding_batch_interval: 3u32,
        staking_commission: Decimal::permille(2),
        unbond_commission: Decimal::from_ratio(5u32, 10_000u32),
    }.test_init(
        MockStkd::default(), 
        &mut chain, 
        Addr::unchecked("admin"), 
        "stkd-SCRT2", 
        &[]
    ).unwrap();

    let new_derivative = Derivative {
        contract: new_deriv.clone().into(),
        base_asset: base.clone().into(),
        staking_type: DerivativeType::StkdScrt,
        deriv_decimals: 6u32,
        base_decimals: 6u32,
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
        mock_stkd::QueryMsg::Balance {
            address: arb.address.clone(),
            key: "key2".into(),
        }.test_query::<mock_stkd::QueryAnswer>(&new_deriv, &chain).unwrap(),
        mock_stkd::QueryAnswer::Balance {
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

    let another_new_deriv = mock_stkd::InstantiateMsg {
        name: "derivative3".to_string(),
        symbol: "stkd-SCRT3".to_string(),
        decimals: 8,
        price: Uint128::new(3_000_000),
        unbonding_time: 21u32,
        unbonding_batch_interval: 3u32,
        staking_commission: Decimal::permille(2),
        unbond_commission: Decimal::from_ratio(5u32, 10_000u32),
    }.test_init(
        MockStkd::default(), 
        &mut chain, 
        Addr::unchecked("admin"), 
        "stkd-SCRT3", 
        &[]
    ).unwrap();

    let another_new_derivative = Derivative {
        contract: another_new_deriv.clone().into(),
        base_asset: new_snip20.clone().into(),
        staking_type: DerivativeType::StkdScrt,
        deriv_decimals: 6u32,
        base_decimals: 6u32,
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
        mock_stkd::QueryMsg::Balance {
            address: arb.address.clone(),
            key: "key2".to_string(),
        }.test_query::<mock_stkd::QueryAnswer>(&another_new_deriv, &chain).unwrap(),
        mock_stkd::QueryAnswer::Balance {
            amount: Uint128::zero(),
        },
    );
}

#[test]
fn set_pairs() {
    let (mut chain, admin, base, deriv, arb, config) = init();
    
    assert!(ExecuteMsg::SetPairs { // invalid dex pair
            pairs: fill_dex_pairs(1, deriv.clone().into(), base.clone().into()),
        }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).is_err()
    );

    assert!(
        ExecuteMsg::SetPairs {
            pairs: vec![ArbPair {
                pair_contract: Some(Contract {
                    address: Addr::unchecked("invalid decimals"),
                    code_hash: "hash".to_string(),
                }),
                mint_info: None,
                token0: base.clone().into(),
                token0_decimals: Uint128::new(7),
                token0_amount: None,
                token1: deriv.clone().into(),
                token1_decimals: Uint128::new(6),
                token1_amount: None,
                dex: Dex::ShadeSwap,
            }],
        }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).is_err()
    );

    let mut more_dex_pairs = fill_dex_pairs(4, base.clone().into(), deriv.clone().into());
    more_dex_pairs.remove(0);
    more_dex_pairs.remove(0); // just pairs #2 and #3
    ExecuteMsg::SetPairs {
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

#[test]
fn arb_pair() {
    let (mut chain, base, deriv, arb, pair) = init_with_pair();

    // Unprofitable
    assert_eq!(  // Pair: 1_000_000 deriv; 2_000_000 base
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: Uint128::new(1_000_000_000),
        },
    );

    let response = ExecuteMsg::Arbitrage {
        index: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).unwrap();

    let exp_profit = match from_binary(&response.data.unwrap()).unwrap() {
        ExecuteAnswer::Arbitrage { status, arb_amount, expected_profit } => {
            assert_eq!(
                (status, arb_amount, expected_profit),
                (ResponseStatus::Failure, Uint128::zero(), Uint128::zero()),
            );
            expected_profit
        },
        _ => Uint128::zero(),
    };
    let mut exp_balance = Uint128::new(1_000_000_000);

    assert_eq!(
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance { asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: exp_balance,
        },
    );

    // Profitable staking direction
    snip20::ExecuteMsg::Send { // Pair: 1_000_000 deriv; 2_025_000 base
        recipient: pair.address.to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(25_000),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    let response = ExecuteMsg::Arbitrage {
        index: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).unwrap();

    let exp_profit = match from_binary(&response.data.unwrap()).unwrap() {
        ExecuteAnswer::Arbitrage { status, arb_amount, expected_profit } => {
            assert_eq!(
                (status, arb_amount, expected_profit),
                (ResponseStatus::Success, Uint128::new(7444), Uint128::new(28)),
            );
            expected_profit
        },
        _ => Uint128::zero(),
    };
    exp_balance += exp_profit;

    assert_eq!(
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: exp_balance - Uint128::new(1), // off by one
        },
    );
    exp_balance -= Uint128::new(1);

    // Profitable unbond direction
    snip20::ExecuteMsg::Send { // Pair: 1_003_714 deriv; 1_967_529 base
        recipient: "admin".to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(50_000),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, pair.address.clone(), &[]).unwrap();

    let response = ExecuteMsg::Arbitrage {
        index: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).unwrap();

    let exp_profit = match from_binary(&response.data.unwrap()).unwrap() {
        ExecuteAnswer::Arbitrage { status, arb_amount, expected_profit } => {
            assert_eq!(
                (status, arb_amount, expected_profit),
                (ResponseStatus::Success, Uint128::new(16369), Uint128::new(137)),
            );
            expected_profit
        }, 
        _ => Uint128::zero(),
    };
    exp_balance += exp_profit;

    assert_eq!(
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: exp_balance,
        },
    );

    // Unprofitable because of fees
    snip20::ExecuteMsg::Send { // Pair: 995_457 deriv; 1_983_950 base
        recipient: pair.address.to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(52),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    let response = ExecuteMsg::Arbitrage {
        index: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).unwrap();

    let exp_profit = match from_binary(&response.data.unwrap()).unwrap() {
        ExecuteAnswer::Arbitrage { status, arb_amount, expected_profit } => {
            assert_eq!(
                (status, arb_amount, expected_profit),
                (ResponseStatus::Failure, Uint128::zero(), Uint128::zero()),
            );
            expected_profit
        }, 
        _ => Uint128::zero(),
    };

    assert_eq!(
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: exp_balance,
        },
    );

    // Profitable but barely, don't do
    snip20::ExecuteMsg::Send { // Pair: 995_457 deriv; 1_983_900 base
        recipient: "admin".to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(50),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, pair.address.clone(), &[]).unwrap();

    let response = ExecuteMsg::Arbitrage {
        index: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).unwrap();

    let exp_profit = match from_binary(&response.data.unwrap()).unwrap() {
        ExecuteAnswer::Arbitrage { status, arb_amount, expected_profit } => {
            assert_eq!(
                (status, arb_amount, expected_profit),
                (ResponseStatus::Failure, Uint128::new(24), Uint128::zero()),
            );
            expected_profit
        }, 
        _ => {
            assert!(false, "invalid return message");
            Uint128::zero()
        },
    };

    assert_eq!(
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: exp_balance,
        },
    );

    // Min profit amount - was profitable but not acceptably so
    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: None,
        derivative: None,
        trading_fees: None,
        max_arb_amount: None,
        min_profit_amount: Some(Uint128::new(10_000)),
        viewing_key: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    snip20::ExecuteMsg::Send { // Pair: 995_457 deriv; 2_025_000 base
        recipient: pair.address.to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(41_100),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    let response = ExecuteMsg::Arbitrage {
        index: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).unwrap();

    let exp_profit = match from_binary(&response.data.unwrap()).unwrap() {
        ExecuteAnswer::Arbitrage { status, arb_amount, expected_profit } => {
            assert_eq!(
                (status, arb_amount, expected_profit),
                (ResponseStatus::Failure, Uint128::new(11974), Uint128::new(72)),
            );
            expected_profit
        }, 
        _ => {
            assert!(false, "invalid return message");
            Uint128::zero()
        },
    };

    assert_eq!(
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: exp_balance,
        },
    );

    // Dex pair swap reversed
    let pair = seeded_pair(
        &mut chain, 
        deriv.clone(), // swapped 
        base.clone(), // swapped
        Uint128::new(1_000_000), 
        Uint128::new(2_025_000)
    );

    ExecuteMsg::AddPair {
        pair: ArbPair {
            pair_contract: Some(pair.clone().into()),
            mint_info: None,
            token0: base.clone().into(),
            token0_decimals: Uint128::new(6),
            token0_amount: None,
            token1: deriv.clone().into(),
            token1_decimals: Uint128::new(6),
            token1_amount: None,
            dex: Dex::SiennaSwap,
        }
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: None,
        derivative: None,
        trading_fees: None,
        max_arb_amount: None,
        min_profit_amount: Some(Uint128::new(1)),
        viewing_key: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    let response = ExecuteMsg::Arbitrage {
        index: Some(1),
    }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).unwrap();

    let exp_profit = match from_binary(&response.data.unwrap()).unwrap() {
        ExecuteAnswer::Arbitrage { status, arb_amount, expected_profit } => {
            assert_eq!(
                (status, arb_amount, expected_profit),
                (ResponseStatus::Success, Uint128::new(7444), Uint128::new(28)),
            );
            expected_profit
        }, 
        _ => {
            assert!(false, "invalid return message");
            Uint128::zero()
        },
    };
    exp_balance += exp_profit;

    assert_eq!(
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: exp_balance - Uint128::one(),  // Off by one
        },
    );
    exp_balance -= Uint128::one();

    // Low max swap
    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: None,
        derivative: None,
        trading_fees: None,
        max_arb_amount: Some(Uint128::new(1000)),
        min_profit_amount: None,
        viewing_key: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    let response = ExecuteMsg::Arbitrage {
        index: Some(0),
    }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).unwrap();

    let exp_profit = match from_binary(&response.data.unwrap()).unwrap() {
        ExecuteAnswer::Arbitrage { status, arb_amount, expected_profit } => {
            assert_eq!(
                (status, arb_amount, expected_profit),
                (ResponseStatus::Success, Uint128::new(1000), Uint128::new(11)),
            );
            expected_profit
        }, 
        _ => {
            assert!(false, "invalid return message");
            Uint128::zero()
        },
    };
    exp_balance += exp_profit;

    assert_eq!(
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: exp_balance + Uint128::new(2), // Off by two
        },
    );

    // Profitable different number of decimals - Stake
    let new_deriv = mock_stkd::InstantiateMsg {
        name: "10xstkd-SCRT".to_string(),
        symbol: "xstkd".to_string(),
        decimals: 5u8,
        price: Uint128::new(2_000_000),
        unbonding_time: 21u32,
        unbonding_batch_interval: 3u32,
        staking_commission: Decimal::permille(2),
        unbond_commission: Decimal::from_ratio(5u32, 10_000u32),
    }.test_init(MockStkd::default(), &mut chain, Addr::unchecked("admin"), "x-stkd-scrt", &[]).unwrap();

    mock_stkd::ExecuteMsg::Stake {
    }.test_exec(&new_deriv, &mut chain, Addr::unchecked("admin"), &[Coin {
        denom: "uscrt".to_string(),
        amount: Uint128::new(1_000_000_000),
    }]).unwrap();

    let pair = seeded_pair(
        &mut chain, 
        base.clone(),
        new_deriv.clone(),
        Uint128::new(2_025_000),
        Uint128::new(100_000),
    );

    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: None,
        derivative: Some(Derivative {
            contract: new_deriv.clone().into(),
            base_asset: base.clone().into(),
            staking_type: DerivativeType::StkdScrt,
            base_decimals: 6u32,
            deriv_decimals: 5u32,
        }),
        trading_fees: None,
        max_arb_amount: Some(Uint128::MAX),
        min_profit_amount: None,
        viewing_key: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    ExecuteMsg::AddPair {
        pair: ArbPair {
            pair_contract: Some(pair.clone().into()),
            mint_info: None,
            token0: base.clone().into(),
            token0_decimals: Uint128::new(6),
            token0_amount: None,
            token1: new_deriv.clone().into(),
            token1_decimals: Uint128::new(5),
            token1_amount: None,
            dex: Dex::SiennaSwap,
        },
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    let balance_query = QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: base.address.clone()
    }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap();
    let mut exp_balance = match balance_query {
        adapter::QueryAnswer::Balance { amount } => amount,
        _ => {
            assert!(false, "invalid return message");
            Uint128::zero()
        },
    };

    let response = ExecuteMsg::Arbitrage {
        index: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).unwrap();

    let exp_profit = match from_binary(&response.data.unwrap()).unwrap() {
        ExecuteAnswer::Arbitrage { status, arb_amount, expected_profit } => {
            assert_eq!(
                (status, arb_amount, expected_profit),
                (ResponseStatus::Success, Uint128::new(7444), Uint128::new(28)),
            );
            expected_profit
        }, 
        _ => {
            assert!(false, "invalid return message");
            Uint128::zero()
        },
    };
    exp_balance += exp_profit;

    assert_eq!(
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: exp_balance - Uint128::new(7), // off by 7 because of decimal cutoff
        },
    );
    exp_balance -= Uint128::new(7);
 
    // Profitable different number of decimals - Unbond
    snip20::ExecuteMsg::Send { // Pair: 100_371 deriv; 1_977_535 base
        recipient: "admin".to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(40_000),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, pair.address.clone(), &[]).unwrap();

    let response = ExecuteMsg::Arbitrage {
        index: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).unwrap();

    let exp_profit = match from_binary(&response.data.unwrap()).unwrap() {
        ExecuteAnswer::Arbitrage { status, arb_amount, expected_profit } => {
            assert_eq!(
                (status, arb_amount, expected_profit),
                (ResponseStatus::Success, Uint128::new(11398), Uint128::new(65)),
            );
            expected_profit
        }, 
        _ => {
            assert!(false, "invalid return message");
            Uint128::zero()
        },
    };
    exp_balance += exp_profit;

    assert_eq!(
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: exp_balance - Uint128::new(3), // Off by 3 because of decimal cutoff
        },
    );
    exp_balance -= Uint128::new(3);
 
    // Low available balance
    snip20::ExecuteMsg::Send {
        recipient: "admin".to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(999_970_000),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, arb.address.clone(), &[]).unwrap();

    snip20::ExecuteMsg::Send { // Pair: 100_371 deriv; 1_977_535 base
        recipient: pair.address.to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(40_000),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    let balance_query = QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
        asset: base.address.clone()
    }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap();
    let mut exp_balance = match balance_query {
        adapter::QueryAnswer::Balance { amount } => amount,
        _ => {
            assert!(false, "invalid return message");
            Uint128::zero()
        },
    };

    let response = ExecuteMsg::Arbitrage {
        index: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).unwrap();

    let exp_profit = match from_binary(&response.data.unwrap()).unwrap() {
        ExecuteAnswer::Arbitrage { status, arb_amount, expected_profit } => {
            assert_eq!(
                (status, arb_amount, expected_profit),
                (ResponseStatus::Success, Uint128::new(2321), Uint128::new(23)),
            );
            expected_profit
        }, 
        _ => {
            assert!(false, "invalid return message");
            Uint128::zero()
        },
    };
    exp_balance += exp_profit;

    assert_eq!(
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: exp_balance - Uint128::new(8), // Off by 8 because of rounding
        },
    );

    // Invalid Arbitrage Index
    assert!(
        ExecuteMsg::Arbitrage {
            index: Some(37),
        }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).is_err()
    );

    // Failed stake transaction - rounding means expected return is not enough
    let pair = seeded_pair(
        &mut chain, 
        base.clone(),
        new_deriv.clone(),
        Uint128::new(2_015_000),
        Uint128::new(100_000),
    );

    ExecuteMsg::AddPair {
        pair: ArbPair {
            pair_contract: Some(pair.clone().into()),
            mint_info: None,
            token0: base.clone().into(),
            token0_decimals: Uint128::new(6),
            token0_amount: None,
            token1: new_deriv.clone().into(),
            token1_decimals: Uint128::new(5),
            token1_amount: None,
            dex: Dex::SiennaSwap,
        },
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        mock_stkd::QueryMsg::Balance {
            address: arb.address.clone(),
            key: "key".to_string(),
        }.test_query::<mock_stkd::QueryAnswer>(&new_deriv, &chain).unwrap(),
        mock_stkd::QueryAnswer::Balance {
            amount: Uint128::new(1),
        },
    );

    assert!(
        ExecuteMsg::Arbitrage {
            index: Some(1),
        }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).is_err()
    );

    assert_eq!( // Assert no tokens are stuck in limbo, staked but not traded 
                // (assert whole transaction failed not half of the messages)
        mock_stkd::QueryMsg::Balance {
            address: arb.address.clone(),
            key: "key".to_string(),
        }.test_query::<mock_stkd::QueryAnswer>(&new_deriv, &chain).unwrap(),
        mock_stkd::QueryAnswer::Balance {
            amount: Uint128::new(1),
        },
    );

    // Failed unbond transaction - rounding means expected return is not enough 
    let pair = seeded_pair(
        &mut chain, 
        base.clone(),
        new_deriv.clone(),
        Uint128::new(2_015_000),
        Uint128::new(100_000),
    );

    ExecuteMsg::AddPair {
        pair: ArbPair {
            pair_contract: Some(pair.clone().into()),
            mint_info: None,
            token0: base.clone().into(),
            token0_decimals: Uint128::new(6),
            token0_amount: None,
            token1: new_deriv.clone().into(),
            token1_decimals: Uint128::new(5),
            token1_amount: None,
            dex: Dex::SiennaSwap,
        },
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        mock_stkd::QueryMsg::Balance {
            address: arb.address.clone(),
            key: "key".to_string(),
        }.test_query::<mock_stkd::QueryAnswer>(&new_deriv, &chain).unwrap(),
        mock_stkd::QueryAnswer::Balance {
            amount: Uint128::new(1),
        },
    );

    assert!(
        ExecuteMsg::Arbitrage {
            index: Some(1),
        }.test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).is_err(),
    );

    assert_eq!(
        mock_stkd::QueryMsg::Balance {
            address: arb.address.clone(),
            key: "key".to_string(),
        }.test_query::<mock_stkd::QueryAnswer>(&new_deriv, &chain).unwrap(),
        mock_stkd::QueryAnswer::Balance {
            amount: Uint128::new(1),
        },
    );

}

#[test]
fn arb_all_pairs() {
    let (mut chain, base, deriv, arb, pair) = init_with_pair();

    // derivative with 5 decimals to help create error transaction
    let deriv = mock_stkd::InstantiateMsg {
        name: "10xstkd-SCRT".to_string(),
        symbol: "xstkd".to_string(),
        decimals: 5u8,
        price: Uint128::new(2_000_000),
        unbonding_time: 21u32,
        unbonding_batch_interval: 3u32,
        staking_commission: Decimal::permille(2),
        unbond_commission: Decimal::from_ratio(5u32, 10_000u32),
    }.test_init(MockStkd::default(), &mut chain, Addr::unchecked("admin"), "x-stkd-scrt", &[]).unwrap();

    mock_stkd::ExecuteMsg::Stake {
    }.test_exec(&deriv, &mut chain, Addr::unchecked("admin"), &[Coin {
        denom: "uscrt".to_string(),
        amount: Uint128::new(1_000_000_000),
    }]).unwrap();

    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: None,
        derivative: Some(Derivative {
            contract: deriv.clone().into(),
            base_asset: base.clone().into(),
            staking_type: DerivativeType::StkdScrt,
            base_decimals: 6u32,
            deriv_decimals: 5u32,
        }),
        trading_fees: None,
        max_arb_amount: Some(Uint128::MAX),
        min_profit_amount: None,
        viewing_key: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    let pair_0 = seeded_pair(
        &mut chain, 
        base.clone(),
        deriv.clone(),
        Uint128::new(2_000_000),
        Uint128::new(100_000),
    );

    let pair_1 = seeded_pair(
        &mut chain, 
        base.clone(),
        deriv.clone(),
        Uint128::new(2_025_000),
        Uint128::new(100_000),
    );

    let pair_2 = seeded_pair(
        &mut chain, 
        base.clone(),
        deriv.clone(),
        Uint128::new(1_975_000),
        Uint128::new(100_000),
    );

    ExecuteMsg::SetPairs {
        pairs: vec![
            ArbPair {
                pair_contract: Some(pair_0.clone().into()),
                mint_info: None,
                token0: base.clone().into(),
                token0_decimals: Uint128::new(6),
                token0_amount: None,
                token1: deriv.clone().into(),
                token1_decimals: Uint128::new(5),
                token1_amount: None,
                dex: Dex::SiennaSwap,
            },
            ArbPair {
                pair_contract: Some(pair_1.clone().into()),
                mint_info: None,
                token0: base.clone().into(),
                token0_decimals: Uint128::new(6),
                token0_amount: None,
                token1: deriv.clone().into(),
                token1_decimals: Uint128::new(5),
                token1_amount: None,
                dex: Dex::SiennaSwap,
            },
            ArbPair {
                pair_contract: Some(pair_2.clone().into()),
                mint_info: None,
                token0: base.clone().into(),
                token0_decimals: Uint128::new(6),
                token0_amount: None,
                token1: deriv.clone().into(),
                token1_decimals: Uint128::new(5),
                token1_amount: None,
                dex: Dex::SiennaSwap,
            },
        ],
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    let response = ExecuteMsg::ArbAllPairs {}
        .test_exec(&arb, &mut chain, Addr::unchecked("user"), &[]).unwrap();

    let exp_profit = match from_binary(&response.data.unwrap()).unwrap() {
        ExecuteAnswer::ArbAllPairs { statuses, arb_amounts, expected_profits } => {
            assert_eq!(
                statuses,
                vec![ResponseStatus::Failure, ResponseStatus::Success, ResponseStatus::Success],
            );
            assert_eq!(
                arb_amounts,
                vec![Uint128::new(0), Uint128::new(7444), Uint128::new(8981)]
            );
            assert_eq!(
                expected_profits,
                vec![Uint128::new(0), Uint128::new(28), Uint128::new(40)]
            );
            expected_profits.into_iter().sum()
        },
        _ => {
            assert!(false, "invalid return message");
            Uint128::zero()
        },
    };
    
    let exp_balance = Uint128::new(1_000_000_000) + exp_profit;
    assert_eq!(
        QueryMsg::Adapter(adapter::SubQueryMsg::Balance {
            asset: base.address.clone(),
        }).test_query::<adapter::QueryAnswer>(&arb, &chain).unwrap(),
        adapter::QueryAnswer::Balance {
            amount: exp_balance - Uint128::new(8), // Off by 8 because of decimal cutoff
        },
    );
}

/*
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
